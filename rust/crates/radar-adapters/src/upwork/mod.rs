pub mod mcp;
mod normalize;
pub mod state;
pub use normalize::normalize;
use radar_application::{
    Capability, JobSource,
    config::{UpworkSearch, UpworkSettings, digest},
};
use radar_domain::Job;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

pub const ENDPOINT: &str = "https://mcp.upwork.com/mcp";
#[derive(Debug, Clone, Serialize, Deserialize, thiserror::Error)]
#[error("Upwork {category}; manual mode only")]
pub struct UpworkError {
    pub category: String,
    pub retry_after_seconds: Option<u64>,
}
impl UpworkError {
    pub fn new(category: &str) -> Self {
        Self {
            category: category.into(),
            retry_after_seconds: None,
        }
    }
}
#[derive(Clone, Serialize, Deserialize)]
pub struct Account {
    pub org_uid: String,
    pub role: String,
    pub name: String,
    pub role_label: String,
    #[serde(default)]
    pub suspended: bool,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SearchBinding {
    pub tool: String,
    pub action: String,
    pub help_hash: String,
    /// JSON pointers into live get_tool_help response, chosen after inspecting discovery.
    pub action_pointer: String,
    pub parameters_schema_pointer: String,
    pub records_pointer: String,
    pub next_cursor_pointer: Option<String>,
    pub cursor_parameter_pointer: Option<String>,
    /// JSON pointers relative to each returned record; absent mappings mean unknown.
    pub fields: BTreeMap<String, String>,
    pub profiles: BTreeMap<String, Value>,
}
#[derive(Clone, Default, Serialize, Deserialize)]
pub struct SearchStats {
    pub fetched: usize,
    pub accepted: usize,
    pub invalid: usize,
    pub filtered: usize,
    pub pages: u32,
    pub partial: bool,
    pub error_category: Option<String>,
    pub retry_after_seconds: Option<u64>,
    pub empty_reason: Option<String>,
    pub filters_ignored: Vec<String>,
    pub skills_filtered_client_side: bool,
}
pub struct UpworkJobSource {
    pub jobs: Vec<Job>,
    pub stats: SearchStats,
}
impl JobSource for UpworkJobSource {
    fn name(&self) -> &str {
        "upwork"
    }
    fn capability(&self) -> Capability {
        Capability {
            available: true,
            diagnostic: "authorized manual MCP result batch".into(),
        }
    }
    fn fetch(&mut self, limit: usize) -> anyhow::Result<Vec<Job>> {
        Ok(self.jobs.iter().take(limit).cloned().collect())
    }
}
#[async_trait::async_trait]
pub trait UpworkGateway: Send + Sync {
    async fn search(
        &self,
        settings: &UpworkSettings,
        profile: &str,
        limit: usize,
    ) -> Result<UpworkJobSource, UpworkError>;
}
pub struct OfficialUpwork;
#[async_trait::async_trait]
impl UpworkGateway for OfficialUpwork {
    async fn search(
        &self,
        settings: &UpworkSettings,
        profile: &str,
        limit: usize,
    ) -> Result<UpworkJobSource, UpworkError> {
        let state = Arc::new(state::PrivateState::from_settings(settings)?);
        let _guard = state.lock()?;
        let account: Account = state
            .read("account.enc")?
            .ok_or_else(|| UpworkError::new("account_selection_required"))?;
        let binding: SearchBinding = state
            .read("binding.enc")?
            .ok_or_else(|| UpworkError::new("search_binding_required"))?;
        state.reserve(settings, chrono::Utc::now().timestamp())?;
        let mut session = match mcp::OfficialSession::connect(settings, state.clone()).await {
            Ok(s) => s,
            Err(e) => {
                if matches!(
                    e.category.as_str(),
                    "rate_limited" | "transport_failure" | "timeout"
                ) {
                    state.defer(settings, &e)?
                }
                return Err(e);
            }
        };
        let result = search(
            &mut session,
            &state,
            settings,
            &account,
            &binding,
            profile,
            limit,
        )
        .await;
        session.close().await;
        if result
            .as_ref()
            .is_ok_and(|s| s.stats.error_category.is_none())
        {
            state.reset_backoff()?;
        }
        result
    }
}
fn schema(tool: &Value) -> Result<&Value, UpworkError> {
    tool.get("inputSchema")
        .ok_or_else(|| UpworkError::new("invalid_schema"))
}
fn validate(schema: &Value, args: &Value) -> Result<(), UpworkError> {
    let validator =
        jsonschema::validator_for(schema).map_err(|_| UpworkError::new("invalid_schema"))?;
    if !validator.is_valid(args) {
        return Err(UpworkError::new("parameters_do_not_match_live_schema"));
    }
    Ok(())
}
fn logical_tool_name(name: &str) -> &str {
    name.strip_prefix("upwork__").unwrap_or(name)
}
fn tool<'a>(tools: &'a [Value], name: &str) -> Result<&'a Value, UpworkError> {
    tools
        .iter()
        .find(|t| t.get("name").and_then(Value::as_str) == Some(name))
        .or_else(|| {
            tools.iter().find(|t| {
                t.get("name")
                    .and_then(Value::as_str)
                    .is_some_and(|advertised| {
                        logical_tool_name(advertised) == logical_tool_name(name)
                    })
            })
        })
        .ok_or_else(|| UpworkError::new("required_tool_unavailable"))
}
async fn call(
    session: &mut dyn mcp::McpSession,
    state: &state::PrivateState,
    settings: &UpworkSettings,
    tools: &[Value],
    name: &str,
    args: Value,
) -> Result<Value, UpworkError> {
    let selected = tool(tools, name)?;
    validate(schema(selected)?, &args)?;
    // MCP calls must use the exact advertised name, including its namespace.
    let advertised = selected["name"].as_str().expect("resolved tool name");
    state.reserve(settings, chrono::Utc::now().timestamp())?;
    let result = session.call(advertised, args).await.and_then(|v| {
        mcp::check_result(&v, false)?;
        Ok(v)
    });
    if let Err(e) = &result
        && (e.category == "rate_limited"
            || e.category == "transport_failure"
            || e.category == "timeout")
    {
        state.defer(settings, e)?
    }
    result
}
async fn accounts(
    session: &mut dyn mcp::McpSession,
    state: &state::PrivateState,
    settings: &UpworkSettings,
    tools: &[Value],
) -> Result<Vec<Account>, UpworkError> {
    let value = call(session, state, settings, tools, "list_accounts", json!({})).await?;
    let list = value
        .get("accounts")
        .or_else(|| value.pointer("/data/accounts"))
        .or_else(|| value.as_array().map(|_| &value))
        .ok_or_else(|| UpworkError::new("account_schema_changed"))?;
    serde_json::from_value(list.clone()).map_err(|_| UpworkError::new("account_schema_changed"))
}
async fn help(
    session: &mut dyn mcp::McpSession,
    state: &state::PrivateState,
    settings: &UpworkSettings,
    tools: &[Value],
    account: &Account,
) -> Result<Value, UpworkError> {
    if tool(tools, "search_tools").is_ok() {
        let mut args = json!({"role":account.role,"query":"find_jobs"});
        // Only send account context if it exists in the advertised discovery schema.
        if schema(tool(tools, "search_tools")?)?
            .pointer("/properties/org_uid")
            .is_some()
        {
            args["org_uid"] = json!(account.org_uid)
        }
        call(session, state, settings, tools, "search_tools", args).await?;
    }
    let mut args = json!({"tool_name":"find_jobs"});
    for (key, value) in [("org_uid", &account.org_uid), ("role", &account.role)] {
        if schema(tool(tools, "get_tool_help")?)?
            .pointer(&format!("/properties/{key}"))
            .is_some()
        {
            args[key] = json!(value)
        }
    }
    call(session, state, settings, tools, "get_tool_help", args).await
}
#[derive(Serialize, Deserialize)]
pub struct Discovery {
    pub tools: Vec<Value>,
    pub help: Value,
    pub help_hash: String,
    pub account_label: String,
}
fn help_hash(help: &Value) -> Result<String, UpworkError> {
    let mut schema = help.clone();
    // Upwork returns a new request trace on each help call; it is not schema content.
    if let Some(object) = schema.as_object_mut() {
        object.remove("trace_id");
    }
    Ok(digest(
        &serde_json::to_vec(&schema).map_err(|_| UpworkError::new("invalid_schema"))?,
    ))
}
pub async fn discover(
    session: &mut dyn mcp::McpSession,
    state: &state::PrivateState,
    settings: &UpworkSettings,
    index: Option<usize>,
) -> Result<Discovery, UpworkError> {
    state.reserve(settings, chrono::Utc::now().timestamp())?;
    let tools = session.tools().await?;
    let list = accounts(session, state, settings, &tools).await?;
    let selected = if let Some(index) = index {
        list.get(index)
    } else if list.len() == 1 {
        list.first()
    } else {
        for (i, a) in list.iter().enumerate() {
            println!("{i}: {} ({})", safe(&a.name), safe(&a.role_label))
        }
        return Err(UpworkError::new("select_account_index_explicitly"));
    }
    .ok_or_else(|| UpworkError::new("account_selection_required"))?;
    if selected.role != "TALENT" || selected.suspended {
        return Err(UpworkError::new("freelancer_account_unavailable"));
    }
    let help = help(session, state, settings, &tools, selected).await?;
    let help_hash = help_hash(&help)?;
    state.write("account.enc", selected)?;
    Ok(Discovery {
        tools,
        help,
        help_hash,
        account_label: format!("{} ({})", safe(&selected.name), safe(&selected.role_label)),
    })
}
fn safe(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(200).collect()
}
pub fn verify_binding(
    tools: &[Value],
    help: &Value,
    binding: &SearchBinding,
) -> Result<(), UpworkError> {
    if logical_tool_name(&binding.tool) != "find_jobs" || help_hash(help)? != binding.help_hash {
        return Err(UpworkError::new("search_schema_changed"));
    }
    let action = help
        .pointer(&binding.action_pointer)
        .ok_or_else(|| UpworkError::new("search_action_unavailable"))?;
    let action_name = action
        .get("name")
        .or_else(|| action.get("action"))
        .and_then(Value::as_str)
        .or_else(|| action.as_str());
    if action_name != Some(&binding.action) {
        return Err(UpworkError::new("search_action_unavailable"));
    }
    if action.get("read_only").and_then(Value::as_bool) == Some(false) {
        return Err(UpworkError::new("search_read_only_not_verified"));
    }
    let readonly = action.get("read_only").and_then(Value::as_bool) == Some(true)
        || help.get("read_only").and_then(Value::as_bool) == Some(true)
        || tool(tools, &binding.tool)
            .ok()
            .and_then(|t| t.pointer("/annotations/readOnlyHint"))
            .and_then(Value::as_bool)
            == Some(true);
    if !readonly {
        return Err(UpworkError::new("search_read_only_not_verified"));
    }
    let parameter_schema = help
        .pointer(&binding.parameters_schema_pointer)
        .ok_or_else(|| UpworkError::new("search_schema_changed"))?;
    for params in binding.profiles.values() {
        validate(parameter_schema, params)?;
    }
    for required in ["id", "url", "title"] {
        if !binding.fields.contains_key(required) {
            return Err(UpworkError::new("required_job_mapping_missing"));
        }
    }
    Ok(())
}
pub async fn search(
    session: &mut dyn mcp::McpSession,
    state: &state::PrivateState,
    settings: &UpworkSettings,
    account: &Account,
    binding: &SearchBinding,
    profile: &str,
    limit: usize,
) -> Result<UpworkJobSource, UpworkError> {
    let criteria = settings
        .profiles
        .get(profile)
        .filter(|p| p.enabled)
        .ok_or_else(|| UpworkError::new("search_profile_disabled"))?;
    state.reserve(settings, chrono::Utc::now().timestamp())?;
    let tools = session.tools().await?;
    let current = accounts(session, state, settings, &tools).await?;
    if !current
        .iter()
        .any(|a| a.org_uid == account.org_uid && a.role == "TALENT" && !a.suspended)
    {
        return Err(UpworkError::new("account_restricted"));
    }
    let help = help(session, state, settings, &tools, account).await?;
    verify_binding(&tools, &help, binding)?;
    let mut params = binding
        .profiles
        .get(profile)
        .cloned()
        .ok_or_else(|| UpworkError::new("search_parameters_required"))?;
    let params_schema = help
        .pointer(&binding.parameters_schema_pointer)
        .expect("validated pointer");
    let limit = limit
        .min(criteria.result_limit as usize)
        .min(settings.max_results_per_scan as usize);
    let mut jobs = vec![];
    let mut stats = SearchStats::default();
    let mut cursors = BTreeSet::new();
    let compact = tool(&tools, "execute_tool").is_ok() && tool(&tools, &binding.tool).is_err();
    for _ in 0..settings.max_pages_per_scan {
        validate(params_schema, &params)?;
        let args = if compact {
            json!({"tool_name":logical_tool_name(&binding.tool),"action":binding.action,"org_uid":account.org_uid,"role":account.role,"params":params})
        } else if schema(tool(&tools, &binding.tool)?)?
            .pointer("/properties/params")
            .is_some()
        {
            json!({"action":binding.action,"org_uid":account.org_uid,"params":params})
        } else {
            let mut args = params.clone();
            let object = args
                .as_object_mut()
                .ok_or_else(|| UpworkError::new("invalid_parameters"))?;
            object.insert("action".into(), json!(binding.action));
            object.insert("org_uid".into(), json!(account.org_uid));
            if schema(tool(&tools, &binding.tool)?)?
                .pointer("/properties/role")
                .is_some()
            {
                object.insert("role".into(), json!(account.role));
            }
            args
        };
        let result = call(
            session,
            state,
            settings,
            &tools,
            if compact {
                "execute_tool"
            } else {
                &binding.tool
            },
            args,
        )
        .await;
        let page = match result {
            Ok(v) => v,
            Err(e) if stats.pages > 0 => {
                stats.partial = true;
                stats.error_category = Some(e.category);
                stats.retry_after_seconds = e.retry_after_seconds;
                break;
            }
            Err(e) => return Err(e),
        };
        stats.pages += 1;
        let metadata = |name: &str| {
            page.get(name)
                .or_else(|| page.get("metadata").and_then(|m| m.get(name)))
        };
        stats.empty_reason = metadata("empty_reason").map(|value| {
            if let Some(text) = value.as_str() {
                return safe(text);
            }
            let parts: Vec<_> = ["code", "reason", "message", "recovery"]
                .iter()
                .filter_map(|key| value.get(key).and_then(Value::as_str))
                .map(safe)
                .collect();
            if parts.is_empty() {
                "upstream empty_reason reported in an unrecognized format".into()
            } else {
                parts.join(": ")
            }
        });
        if let Some(ignored) = metadata("filters_ignored") {
            if let Some(values) = ignored.as_array() {
                stats.filters_ignored.extend(values.iter().map(|v| {
                    v.as_str()
                        .or_else(|| {
                            v.get("filter")
                                .or_else(|| v.get("name"))
                                .and_then(Value::as_str)
                        })
                        .map(safe)
                        .unwrap_or_else(|| {
                            "upstream ignored filter in an unrecognized format".into()
                        })
                }));
            } else {
                stats
                    .filters_ignored
                    .push("upstream reported ignored filters".into())
            }
        }
        stats.skills_filtered_client_side |=
            metadata("skills_filtered_client_side").is_some_and(|v| {
                v.as_bool()
                    .unwrap_or_else(|| v.as_array().is_some_and(|values| !values.is_empty()))
            });
        let rows = page
            .pointer(&binding.records_pointer)
            .and_then(Value::as_array)
            .ok_or_else(|| UpworkError::new("job_response_schema_changed"))?;
        if rows.len() > 1000 {
            return Err(UpworkError::new("job_response_too_large"));
        }
        for row in rows {
            stats.fetched += 1;
            match normalize(row, &binding.fields) {
                Ok(job) if normalize::matches(&job, criteria) => {
                    if jobs.len() < limit {
                        jobs.push(job)
                    } else {
                        stats.partial = true
                    }
                }
                Ok(_) => stats.filtered += 1,
                Err(_) => stats.invalid += 1,
            }
        }
        let next = binding
            .next_cursor_pointer
            .as_ref()
            .and_then(|p| page.pointer(p))
            .filter(|v| !v.is_null() && **v != json!(""));
        let Some(next) = next else { break };
        if jobs.len() >= limit || stats.pages >= settings.max_pages_per_scan {
            stats.partial = true;
            break;
        }
        let pointer = binding
            .cursor_parameter_pointer
            .as_ref()
            .ok_or_else(|| UpworkError::new("pagination_mapping_required"))?;
        if !cursors.insert(next.to_string()) {
            return Err(UpworkError::new("pagination_cycle"));
        }
        // Live search requires the optional string cursor to be absent on the first page.
        let (parent, key) = pointer
            .rsplit_once('/')
            .filter(|_| pointer.starts_with('/'))
            .ok_or_else(|| UpworkError::new("pagination_mapping_required"))?;
        let object = params
            .pointer_mut(parent)
            .and_then(Value::as_object_mut)
            .ok_or_else(|| UpworkError::new("pagination_mapping_required"))?;
        object.insert(key.replace("~1", "/").replace("~0", "~"), next.clone());
    }
    stats.accepted = jobs.len();
    // Local filters always enforce the configured constraints, even when ignored upstream.
    Ok(UpworkJobSource { jobs, stats })
}
