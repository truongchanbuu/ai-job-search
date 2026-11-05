use anyhow::{Result, bail, ensure};
use radar_application::{AiProvider, ai::*, config::AiRuntime};
use radar_domain::Verdict;
use serde_json::{Value, json};

pub fn mock(input: &AiInput) -> Result<ProviderReply> {
    let mut refs: Vec<String> = input
        .evidence
        .facts
        .iter()
        .filter(|f| f.verified && (input.kind == TaskKind::Evaluate || f.shareable))
        .map(|f| f.id.clone())
        .take(2)
        .collect();
    let missing_skills: Vec<String> = input
        .job
        .job
        .skills
        .iter()
        .filter(|skill| {
            !input.evidence.facts.iter().any(|fact| {
                fact.statement
                    .to_lowercase()
                    .contains(&skill.to_lowercase())
            })
        })
        .cloned()
        .collect();
    let below_minimum = input.job.job.budget.as_ref().is_some_and(|budget| {
        matches!(input.job.job.job_type, radar_domain::JobType::Hourly)
            && budget.currency == "USD"
            && budget.amount < input.evidence.minimum_hourly_usd
    });
    let pricing=match &input.job.job.budget {
        Some(budget) if matches!(input.job.job.job_type,radar_domain::JobType::Hourly) && budget.currency=="USD"=>format!("Posted hourly value USD {}; configured minimum USD {}. {}. Confirm the posted range and terms before quoting.",budget.amount,input.evidence.minimum_hourly_usd,if below_minimum {"Below the configured minimum"}else{"At or above the configured minimum"}),
        Some(_)=>"A posted budget exists; scope and estimated effort must be clarified before pricing a fixed project.".into(),
        None=>"Compensation is unknown; ask for budget and payment terms before quoting.".into(),
    };
    let output = match input.kind {
        TaskKind::Evaluate => serde_json::to_value(Analysis {
            schema_version: 1,
            job_id: input.job.id,
            recommendation: if input.job.evaluation.verdict == Verdict::Skip {
                Recommendation::Skip
            } else if refs.is_empty() || input.job.job.description.is_none() {
                Recommendation::RequiresClarification
            } else if !missing_skills.is_empty() || below_minimum {
                Recommendation::LowPriority
            } else if input.job.evaluation.verdict == Verdict::ApplyCandidate {
                Recommendation::StrongCandidate
            } else {
                Recommendation::WorthInvestigating
            },
            technical_fit:
                "Only supplied evidence supports this assessment; review gaps before committing."
                    .into(),
            strengths: input
                .evidence
                .facts
                .iter()
                .filter(|f| refs.contains(&f.id))
                .map(|f| f.statement.clone())
                .collect(),
            gaps: missing_skills,
            risks: input.job.evaluation.vetoes.clone(),
            assumptions: vec!["Scope needs a reviewed acceptance checklist.".into()],
            evidence_references: refs,
            estimated_effort: Effort {
                best_case_hours: None,
                conservative_hours: None,
                assumptions: vec!["No reliable effort estimate before scope clarification.".into()],
                dependencies: vec![
                    "Access, deliverables and acceptance criteria need confirmation.".into(),
                ],
                confidence: Certainty::Low,
            },
            pricing_considerations: vec![pricing],
            confidence: Certainty::Low,
            unknowns: vec![
                "Client reliability is unknown unless verified source fields support it.".into(),
            ],
            suggested_next_action:
                "Review the deterministic verdict, evidence and scope with the client.".into(),
        })?,
        TaskKind::Proposal => {
            if let Some(template) = &input.proposal_template {
                refs = radar_application::proposal_templates::relevant_evidence(
                    &input.job.job,
                    &input.evidence.facts,
                    template.definition.max_evidence,
                );
            }
            ensure!(!refs.is_empty(), "shareable verified evidence required");
            let mut proposal=Proposal {screening_answers:vec![],schema_version:1,job_id:input.job.id,proposal_text:String::new(),evidence_references:refs,
                questions_for_client:vec!["Which acceptance criteria define a successful first milestone?".into()],assumptions:vec!["Scope will be confirmed before a commitment.".into()],warnings:vec!["Review all claims and availability before using this draft.".into()],
                opening:format!("For {}, a useful starting point is a clear acceptance checklist.",input.job.job.title),
                approach:"The proposed approach is to clarify the inputs, implement a small vertical slice, and verify the expected behavior with focused tests.".into(),next_step:"A short scope review can establish dependencies and a realistic first milestone.".into()};
            if let Some(template) = &input.proposal_template {
                proposal.opening = format!(
                    "For {}, the goal is a focused change with a clear acceptance check.",
                    input.job.job.title
                );
                proposal.approach = template.definition.mock_approach.clone();
                proposal.next_step =
                    "The first deliverable can be agreed after confirming the required behavior."
                        .into();
                proposal.questions_for_client = if template.definition.max_questions == 0 {
                    vec![]
                } else {
                    vec![template.definition.question.clone()]
                };
            }
            proposal.proposal_text = render_proposal(&proposal, input)?;
            serde_json::to_value(proposal)?
        }
    };
    Ok(ProviderReply {
        output,
        usage: Usage {
            estimated_cost_usd: Some(0.0),
            ..Default::default()
        },
    })
}
#[derive(Debug, thiserror::Error)]
#[error("{category}")]
pub struct ProviderFailure {
    pub category: &'static str,
    pub usage: Usage,
}
pub struct ResponsesProvider;
#[async_trait::async_trait]
impl AiProvider for ResponsesProvider {
    fn draft(
        &self,
        _: &radar_domain::Job,
        _: &radar_domain::Profile,
    ) -> Result<radar_application::AiDraft> {
        bail!("use the reviewed manual AI workflow")
    }
    async fn infer(&self, settings: &AiRuntime, input: &AiInput) -> Result<ProviderReply> {
        let key = std::env::var(&settings.api_key_env).map_err(|_| ProviderFailure {
            category: "provider_credentials_missing",
            usage: Usage::default(),
        })?;
        if key.trim().is_empty() {
            return Err(ProviderFailure {
                category: "provider_credentials_missing",
                usage: Usage::default(),
            }
            .into());
        }
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let body = responses_request(settings, input)?;
        let response = client
            .post("https://api.openai.com/v1/responses")
            .bearer_auth(key)
            .json(&body)
            .send()
            .await
            .map_err(|_| anyhow::anyhow!("provider_connection_failure"))?;
        if !response.status().is_success() {
            let category = match response.status().as_u16() {
                401 | 403 => "provider_authentication_failed",
                429 => "provider_rate_limited",
                400..=499 => "provider_request_rejected",
                _ => "provider_unavailable",
            };
            return Err(ProviderFailure {
                category,
                usage: Usage::default(),
            }
            .into());
        }
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| anyhow::anyhow!("provider_transport_failure"))?
        {
            ensure!(
                bytes.len() + chunk.len() <= settings.max_output_bytes + 32768,
                "provider_response_too_large"
            );
            bytes.extend_from_slice(&chunk);
        }
        let value: Value = serde_json::from_slice(&bytes)
            .map_err(|_| anyhow::anyhow!("invalid_provider_response"))?;
        responses_reply(settings, value)
    }
}
/// Builds a tool-free request; credentials and raw identity proofs are excluded.
pub fn responses_request(settings: &AiRuntime, input: &AiInput) -> Result<Value> {
    let mut input = input.clone();
    input.job.evaluation.evidence.clear();
    let body = json!({"model":settings.model,"store":false,"tools":[],"tool_choice":"none","max_output_tokens":settings.max_output_tokens,
        "instructions":INSTRUCTIONS,"input":serde_json::to_string(&input)?,
        "text":{"format":{"type":"json_schema","name":"radar_output","strict":true,"schema":output_schema(input.kind)}}});
    ensure!(
        serde_json::to_vec(&body)?.len() <= settings.max_input_bytes,
        "provider_input_too_large"
    );
    Ok(body)
}
/// Decodes only completed structured text and safe usage counters.
pub fn responses_reply(settings: &AiRuntime, value: Value) -> Result<ProviderReply> {
    let input_tokens = value.pointer("/usage/input_tokens").and_then(Value::as_u64);
    let output_tokens = value
        .pointer("/usage/output_tokens")
        .and_then(Value::as_u64);
    let cost = match (
        input_tokens,
        output_tokens,
        settings.input_usd_per_million_tokens,
        settings.output_usd_per_million_tokens,
    ) {
        (Some(i), Some(o), Some(a), Some(b)) => Some((i as f64 * a + o as f64 * b) / 1_000_000.0),
        _ => None,
    };
    let usage = Usage {
        input_tokens,
        output_tokens,
        estimated_cost_usd: cost.filter(|cost| cost.is_finite()),
    };
    if value["status"] != "completed" {
        return Err(ProviderFailure {
            category: "provider_incomplete_or_refused",
            usage,
        }
        .into());
    }
    let text = value["output"]
        .as_array()
        .into_iter()
        .flatten()
        .flat_map(|message| message["content"].as_array().into_iter().flatten())
        .find(|content| content["type"] == "output_text")
        .and_then(|content| content["text"].as_str())
        .ok_or_else(|| ProviderFailure {
            category: "provider_output_missing",
            usage: usage.clone(),
        })?;
    if text.len() > settings.max_output_bytes {
        return Err(ProviderFailure {
            category: "provider_output_too_large",
            usage,
        }
        .into());
    }
    let output = serde_json::from_str(text).map_err(|_| ProviderFailure {
        category: "invalid_model_json",
        usage: usage.clone(),
    })?;
    Ok(ProviderReply { output, usage })
}

pub fn provider(name: &str) -> Result<std::sync::Arc<dyn AiProvider>> {
    match name {
        "mock" => Ok(std::sync::Arc::new(crate::MockAiProvider)),
        "openai_responses" => Ok(std::sync::Arc::new(ResponsesProvider)),
        _ => bail!("unsupported_ai_provider"),
    }
}
