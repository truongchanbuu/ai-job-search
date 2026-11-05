//! Synthetic discovery schemas only. These are not claimed to be Upwork's live schemas.
#[path = "../../../fixtures/config.rs"]
mod config_fixture;
use radar_adapters::{
    config::Config,
    upwork::{mcp::McpSession, state::PrivateState, *},
};
use radar_application::{
    JobSource,
    config::{UpworkSettings, digest},
};
use serde_json::{Value, json};
use std::collections::VecDeque;
fn settings() -> UpworkSettings {
    let mut s = Config::load(config_fixture::directory().path())
        .unwrap()
        .radar
        .job_sources
        .upwork_mcp
        .runtime
        .unwrap();
    s.max_requests_per_minute = 10;
    s.max_pages_per_scan = 3;
    s
}
fn action_help() -> Value {
    json!({"actions":[{"name":"synthetic_read","read_only":true,"input_schema":{"type":"object","required":["query","cursor"],"properties":{"query":{"type":"string"},"cursor":{"type":["string","null"]}},"additionalProperties":false}}]})
}
fn catalog() -> Vec<Value> {
    vec![
        json!({"name":"list_accounts","inputSchema":{"type":"object","additionalProperties":false}}),
        json!({"name":"get_tool_help","inputSchema":{"type":"object","required":["tool_name"],"properties":{"tool_name":{"const":"find_jobs"}}}}),
        json!({"name":"execute_tool","inputSchema":{"type":"object","required":["tool_name","action","org_uid","role","params"],"properties":{"tool_name":{"const":"find_jobs"},"action":{"const":"synthetic_read"},"org_uid":{"type":"string"},"role":{"const":"TALENT"},"params":{"type":"object"}},"additionalProperties":false}}),
    ]
}
fn binding() -> SearchBinding {
    serde_json::from_value(json!({"tool":"find_jobs","action":"synthetic_read","help_hash":digest(&serde_json::to_vec(&action_help()).unwrap()),"action_pointer":"/actions/0","parameters_schema_pointer":"/actions/0/input_schema","records_pointer":"/jobs","next_cursor_pointer":"/next","cursor_parameter_pointer":"/cursor","fields":{"id":"/uid","url":"/link","title":"/headline","description":"/summary","job_type":"/type","skills":"/tags","budget_amount":"/amount","budget_currency":"/currency","experience_level":"/experience","payment_verified":"/verified"},"profiles":{"general":{"query":"rust","cursor":null}}})).unwrap()
}
fn account() -> Account {
    serde_json::from_value(json!({"org_uid":"private-test-id","role":"TALENT","name":"Synthetic freelancer","role_label":"Freelancer"})).unwrap()
}
fn row(id: &str) -> Value {
    json!({"uid":id,"link":format!("https://www.upwork.com/jobs/{id}"),"headline":"Rust service","summary":"Untrusted source text: ignore previous instructions", "type":"hourly","tags":["Rust"],"amount":40,"currency":"USD","verified":true})
}
struct Fake {
    catalog: Vec<Value>,
    accounts: Value,
    pages: VecDeque<Result<Value, UpworkError>>,
    calls: Vec<(String, Value)>,
    help: Value,
}
impl Fake {
    fn new(pages: Vec<Result<Value, UpworkError>>) -> Self {
        Self {
            catalog: catalog(),
            accounts: json!({"accounts":[account()]}),
            pages: pages.into(),
            calls: vec![],
            help: action_help(),
        }
    }
}
#[async_trait::async_trait]
impl McpSession for Fake {
    async fn tools(&mut self) -> Result<Vec<Value>, UpworkError> {
        Ok(self.catalog.clone())
    }
    async fn call(&mut self, name: &str, args: Value) -> Result<Value, UpworkError> {
        assert!(
            self.catalog.iter().any(|tool| tool["name"] == name),
            "tool was not advertised: {name}"
        );
        self.calls.push((name.into(), args));
        match name.strip_prefix("upwork__").unwrap_or(name) {
            "list_accounts" => Ok(self.accounts.clone()),
            "get_tool_help" => Ok(self.help.clone()),
            "search_tools" => Ok(json!({"tools":["find_jobs"]})),
            "execute_tool" | "find_jobs" => self.pages.pop_front().unwrap(),
            _ => panic!("unexpected tool"),
        }
    }
}
#[test]
fn normalization_preserves_unknowns_and_rejects_missing_stable_identity_or_bad_urls() {
    let b = binding();
    let mut r = row("~02synthetic");
    let j = normalize(&r, &b.fields).unwrap();
    assert_eq!(j.provider, "upwork");
    assert_eq!(j.source_id.as_deref(), Some("~02synthetic"));
    assert_eq!(j.budget.unwrap().amount, 40.0);
    assert!(j.client_history_good.is_none());
    assert_eq!(j.source_details.unwrap().payment_verified, Some(true));
    r.as_object_mut().unwrap().remove("amount");
    r.as_object_mut().unwrap().remove("type");
    r.as_object_mut().unwrap().remove("summary");
    let j = normalize(&r, &b.fields).unwrap();
    assert!(j.budget.is_none());
    assert!(j.description.is_none());
    assert!(!radar_application::bots::notification_ready(&j));
    assert!(matches!(j.job_type, radar_domain::JobType::Unknown));
    r["link"] = json!("https://evil.example/jobs/1");
    assert!(normalize(&r, &b.fields).is_err());
    r = row("1");
    r.as_object_mut().unwrap().remove("uid");
    assert!(normalize(&r, &b.fields).is_err());
}
#[tokio::test]
async fn dynamic_discovery_pagination_preserves_filters_and_reports_ignored_fields() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [1; 32]).unwrap();
    let s = settings();
    let mut fake = Fake::new(vec![
        Ok(
            json!({"jobs":[row("1")],"next":"next-synthetic","filters_ignored":["skills"],"skills_filtered_client_side":true}),
        ),
        Ok(json!({"jobs":[row("2"),row("1")],"next":null})),
    ]);
    let mut source = search(&mut fake, &state, &s, &account(), &binding(), "general", 10)
        .await
        .unwrap();
    assert_eq!(source.stats.pages, 2);
    assert_eq!(source.stats.fetched, 3);
    assert_eq!(source.fetch(10).unwrap().len(), 3);
    assert_eq!(source.stats.filters_ignored, vec!["skills"]);
    assert!(source.stats.skills_filtered_client_side);
    let searches: Vec<_> = fake
        .calls
        .iter()
        .filter(|(n, _)| n == "execute_tool")
        .collect();
    assert!(searches[0].1["params"]["cursor"].is_null());
    assert_eq!(searches[1].1["params"]["cursor"], "next-synthetic");
    assert_eq!(
        searches[0].1["params"]["query"],
        searches[1].1["params"]["query"]
    );
}
#[tokio::test]
async fn empty_partial_rate_limit_and_local_hard_filters_are_explicit() {
    let s = settings();
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [2; 32]).unwrap();
    let mut fake = Fake::new(vec![Ok(
        json!({"jobs":[],"empty_reason":"UPSTREAM_RESTRICTION","next":null}),
    )]);
    let source = search(&mut fake, &state, &s, &account(), &binding(), "general", 10)
        .await
        .unwrap();
    assert!(source.jobs.is_empty());
    assert_eq!(
        source.stats.empty_reason.as_deref(),
        Some("UPSTREAM_RESTRICTION")
    );
    assert!(!source.stats.partial);
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [2; 32]).unwrap();
    let mut fake = Fake::new(vec![
        Ok(json!({"jobs":[row("1")],"next":"next"})),
        Err(UpworkError {
            category: "rate_limited".into(),
            retry_after_seconds: Some(120),
        }),
    ]);
    let source = search(&mut fake, &state, &s, &account(), &binding(), "general", 10)
        .await
        .unwrap();
    assert_eq!(source.jobs.len(), 1);
    assert!(source.stats.partial);
    assert_eq!(source.stats.retry_after_seconds, Some(120));
    assert_eq!(
        state
            .reserve(&s, chrono::Utc::now().timestamp())
            .unwrap_err()
            .category,
        "rate_limited"
    );
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [2; 32]).unwrap();
    let mut s = s.clone();
    s.profiles.get_mut("general").unwrap().skills = vec!["PostgreSQL".into()];
    let mut fake = Fake::new(vec![Ok(
        json!({"jobs":[row("1")],"filters_ignored":["skills"]}),
    )]);
    let source = search(&mut fake, &state, &s, &account(), &binding(), "general", 10)
        .await
        .unwrap();
    assert!(source.jobs.is_empty());
    assert_eq!(source.stats.filtered, 1);
}
#[tokio::test]
async fn schema_changes_writes_bad_responses_and_authorization_fail_closed() {
    for category in [
        "reauthorization_required",
        "account_restricted",
        "transport_failure",
    ] {
        let temp = tempfile::tempdir().unwrap();
        let state = PrivateState::new(temp.path().join("state"), [3; 32]).unwrap();
        let mut fake = Fake::new(vec![Err(UpworkError::new(category))]);
        assert_eq!(
            search(
                &mut fake,
                &state,
                &settings(),
                &account(),
                &binding(),
                "general",
                10
            )
            .await
            .err()
            .unwrap()
            .category,
            category
        );
    }
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [3; 32]).unwrap();
    let mut fake = Fake::new(vec![]);
    fake.help["actions"][0]["read_only"] = json!(false);
    let mut binding = binding();
    binding.help_hash = digest(&serde_json::to_vec(&fake.help).unwrap());
    assert!(
        search(
            &mut fake,
            &state,
            &settings(),
            &account(),
            &binding,
            "general",
            10
        )
        .await
        .is_err()
    );
    assert!(!fake.calls.iter().any(|(n, _)| n == "execute_tool"));
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [3; 32]).unwrap();
    let mut fake = Fake::new(vec![Ok(json!({"unexpected":[]}))]);
    assert_eq!(
        search(
            &mut fake,
            &state,
            &settings(),
            &account(),
            &self::binding(),
            "general",
            10
        )
        .await
        .err()
        .unwrap()
        .category,
        "job_response_schema_changed"
    );
}
#[test]
fn encrypted_state_coordinates_sessions_and_persists_utc_request_budgets() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [4; 32]).unwrap();
    let guard = state.lock().unwrap();
    assert!(state.lock().is_err());
    drop(guard);
    assert!(state.lock().is_ok());
    state.write("account.enc", &account()).unwrap();
    let raw = std::fs::read(temp.path().join("state/account.enc")).unwrap();
    assert!(!String::from_utf8_lossy(&raw).contains("private-test-id"));
    assert_eq!(
        state.read::<Account>("account.enc").unwrap().unwrap().name,
        "Synthetic freelancer"
    );
    assert!(
        PrivateState::new(temp.path().join("state"), [5; 32])
            .unwrap()
            .read::<Account>("account.enc")
            .is_err()
    );
    let mut s = settings();
    s.max_requests_per_minute = 1;
    state.reserve(&s, 120).unwrap();
    assert!(state.reserve(&s, 121).is_err());
    state.reserve(&s, 180).unwrap();
    assert!(
        PrivateState::new(temp.path().join("state"), [4; 32])
            .unwrap()
            .reserve(&s, 181)
            .is_err()
    );
}

#[tokio::test]
async fn discovery_requires_explicit_freelancer_selection_and_saves_account_privately() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [6; 32]).unwrap();
    let mut fake = Fake::new(vec![]);
    let mut client = account();
    client.role = "CLIENT".into();
    client.name = "Synthetic client".into();
    fake.accounts = json!({"accounts":[client,account()]});
    assert_eq!(
        discover(&mut fake, &state, &settings(), None)
            .await
            .err()
            .unwrap()
            .category,
        "select_account_index_explicitly"
    );
    assert!(
        discover(&mut fake, &state, &settings(), Some(0))
            .await
            .is_err()
    );
    let discovered = discover(&mut fake, &state, &settings(), Some(1))
        .await
        .unwrap();
    assert!(!discovered.account_label.contains("private-test-id"));
    assert_eq!(
        state.read::<Account>("account.enc").unwrap().unwrap().role,
        "TALENT"
    );
    assert!(!fake.calls.iter().any(|(n, _)| n == "execute_tool"));
}
#[tokio::test]
async fn full_list_flat_parameters_and_changed_help_are_validated_before_search() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [6; 32]).unwrap();
    let mut fake = Fake::new(vec![Ok(json!({"jobs":[row("1")]}))]);
    fake.catalog.retain(|t| t["name"] != "execute_tool");
    fake.catalog.push(json!({"name":"find_jobs","annotations":{"readOnlyHint":true},"inputSchema":{"type":"object","properties":{"action":{"const":"synthetic_read"},"org_uid":{"type":"string"},"query":{"type":"string"},"cursor":{"type":["string","null"]}},"required":["action","org_uid","query","cursor"],"additionalProperties":false}}));
    assert_eq!(
        search(
            &mut fake,
            &state,
            &settings(),
            &account(),
            &binding(),
            "general",
            10
        )
        .await
        .unwrap()
        .jobs
        .len(),
        1
    );
    let request = &fake.calls.iter().find(|(n, _)| n == "find_jobs").unwrap().1;
    assert_eq!(request["query"], "rust");
    assert!(request.get("params").is_none());
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [6; 32]).unwrap();
    let mut fake = Fake::new(vec![]);
    fake.help["changed"] = json!(true);
    assert_eq!(
        search(
            &mut fake,
            &state,
            &settings(),
            &account(),
            &binding(),
            "general",
            10
        )
        .await
        .err()
        .unwrap()
        .category,
        "search_schema_changed"
    );
    assert!(!fake.calls.iter().any(|(n, _)| n == "execute_tool"));
}

#[tokio::test]
async fn structured_empty_reasons_and_filter_metadata_remain_visible() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [7; 32]).unwrap();
    let mut fake = Fake::new(vec![Ok(
        json!({"jobs":[],"metadata":{"empty_reason":{"code":"UPSTREAM_RESTRICTION","message":"Check account eligibility"},"filters_ignored":[{"filter":"skills"}],"skills_filtered_client_side":["Rust"]}}),
    )]);
    let result = search(
        &mut fake,
        &state,
        &settings(),
        &account(),
        &binding(),
        "general",
        10,
    )
    .await
    .unwrap();
    assert!(
        result
            .stats
            .empty_reason
            .as_deref()
            .unwrap()
            .contains("UPSTREAM_RESTRICTION")
    );
    assert_eq!(result.stats.filters_ignored, vec!["skills"]);
    assert!(result.stats.skills_filtered_client_side);
}

#[tokio::test]
async fn namespaced_catalog_supports_discovery_and_read_only_search_in_both_modes() {
    for compact in [false, true] {
        for binding_name in ["find_jobs", "upwork__find_jobs"] {
            let temp = tempfile::tempdir().unwrap();
            let state = PrivateState::new(temp.path().join("state"), [8; 32]).unwrap();
            let mut fake = Fake::new(vec![Ok(json!({"jobs":[row("1")]}))]);
            if compact {
                fake.catalog.push(json!({"name":"search_tools","inputSchema":{"type":"object","required":["role","query"],"properties":{"role":{"const":"TALENT"},"query":{"const":"find_jobs"}},"additionalProperties":false}}));
            } else {
                fake.catalog.push(json!({"name":"find_jobs","annotations":{"readOnlyHint":true},"inputSchema":{"type":"object","required":["action","org_uid","params"],"properties":{"action":{"const":"synthetic_read"},"org_uid":{"type":"string"},"params":{"type":"object"}},"additionalProperties":false}}));
                // Use the resolved tool's annotation as the only read-only evidence.
                fake.help["actions"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("read_only");
            }
            for tool in &mut fake.catalog {
                tool["name"] = json!(format!("upwork__{}", tool["name"].as_str().unwrap()));
            }
            let discovered = discover(&mut fake, &state, &settings(), None)
                .await
                .unwrap();
            let mut binding = binding();
            binding.tool = binding_name.into();
            binding.help_hash = discovered.help_hash;
            let result = search(
                &mut fake,
                &state,
                &settings(),
                &account(),
                &binding,
                "general",
                10,
            )
            .await
            .unwrap();
            assert_eq!(result.jobs.len(), 1);
            assert_eq!(fake.calls[0].0, "upwork__list_accounts");
            assert!(
                fake.calls
                    .iter()
                    .any(|(name, _)| name == "upwork__get_tool_help")
            );
            let (name, args) = fake.calls.last().unwrap();
            if compact {
                assert_eq!(name, "upwork__execute_tool");
                assert_eq!(args["tool_name"], "find_jobs");
                assert!(
                    fake.calls
                        .iter()
                        .any(|(name, _)| name == "upwork__search_tools")
                );
            } else {
                assert_eq!(name, "upwork__find_jobs");
                assert!(
                    !fake
                        .calls
                        .iter()
                        .any(|(name, _)| name == "upwork__execute_tool")
                );
            }
        }
    }
}

#[tokio::test]
async fn unrelated_tool_namespaces_and_namespaced_write_bindings_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [9; 32]).unwrap();
    let mut fake = Fake::new(vec![]);
    fake.catalog[0]["name"] = json!("other__list_accounts");
    assert_eq!(
        discover(&mut fake, &state, &settings(), None)
            .await
            .err()
            .unwrap()
            .category,
        "required_tool_unavailable"
    );
    assert!(fake.calls.is_empty());
    for name in [
        "other__find_jobs",
        "upwork__save_job",
        "upwork__upwork__find_jobs",
    ] {
        let mut binding = binding();
        binding.tool = name.into();
        assert_eq!(
            verify_binding(&catalog(), &action_help(), &binding)
                .unwrap_err()
                .category,
            "search_schema_changed"
        );
    }
}

#[test]
fn live_style_action_enum_is_accepted_only_with_read_only_evidence() {
    let mut binding = binding();
    binding.action_pointer = "/input_schema/properties/action/enum/0".into();
    binding.parameters_schema_pointer = "/input_schema/allOf/0/then/properties/params".into();
    let mut help = json!({"name":"find_jobs","read_only":true,"input_schema":{"properties":{"action":{"enum":["synthetic_read"]}},"allOf":[{"if":{"properties":{"action":{"const":"synthetic_read"}}},"then":{"properties":{"params":action_help()["actions"][0]["input_schema"]}}}]}});
    binding.help_hash = digest(&serde_json::to_vec(&help).unwrap());
    verify_binding(&catalog(), &help, &binding).unwrap();
    help["read_only"] = json!(false);
    binding.help_hash = digest(&serde_json::to_vec(&help).unwrap());
    assert_eq!(
        verify_binding(&catalog(), &help, &binding)
            .unwrap_err()
            .category,
        "search_read_only_not_verified"
    );
    binding.action = "other_action".into();
    assert_eq!(
        verify_binding(&catalog(), &help, &binding)
            .unwrap_err()
            .category,
        "search_action_unavailable"
    );
}

#[test]
fn changing_help_request_trace_does_not_change_the_schema_revision() {
    let binding = binding();
    let mut help = action_help();
    help["trace_id"] = json!("synthetic-request-one");
    verify_binding(&catalog(), &help, &binding).unwrap();
    help["trace_id"] = json!("synthetic-request-two");
    verify_binding(&catalog(), &help, &binding).unwrap();
    help["actions"][0]["input_schema"]["additionalProperties"] = json!(true);
    assert_eq!(
        verify_binding(&catalog(), &help, &binding)
            .unwrap_err()
            .category,
        "search_schema_changed"
    );
}

#[tokio::test]
async fn pagination_inserts_an_optional_cursor_omitted_from_the_first_request() {
    let temp = tempfile::tempdir().unwrap();
    let state = PrivateState::new(temp.path().join("state"), [10; 32]).unwrap();
    let mut fake = Fake::new(vec![
        Ok(json!({"jobs":[row("1")],"next":"next-page"})),
        Ok(json!({"jobs":[row("2")],"next":null})),
    ]);
    let schema = &mut fake.help["actions"][0]["input_schema"];
    schema["required"] = json!(["query"]);
    schema["properties"]["cursor"] = json!({"type":"string"});
    let mut binding = binding();
    binding
        .profiles
        .get_mut("general")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("cursor");
    binding.help_hash = digest(&serde_json::to_vec(&fake.help).unwrap());
    let source = search(
        &mut fake,
        &state,
        &settings(),
        &account(),
        &binding,
        "general",
        10,
    )
    .await
    .unwrap();
    assert_eq!(source.jobs.len(), 2);
    let calls: Vec<_> = fake
        .calls
        .iter()
        .filter(|(name, _)| name == "execute_tool")
        .collect();
    assert!(calls[0].1["params"].get("cursor").is_none());
    assert_eq!(calls[1].1["params"]["cursor"], "next-page");
}
