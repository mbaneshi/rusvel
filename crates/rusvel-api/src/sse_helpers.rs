use std::convert::Infallible;

use axum::response::sse::Event;
use chrono::Utc;
use futures::stream::{self, Stream};

use rusvel_agent::{AgUiEvent, AgentEvent, ag_ui_json_with_conversation, agent_event_to_ag_ui};
use rusvel_core::domain::{ModelProvider, ModelRef, Part};

pub(crate) fn run_started_event(run_id: &str, conversation_id: &str) -> Result<Event, Infallible> {
    let ev = AgUiEvent::RunStarted {
        run_id: run_id.to_string(),
        timestamp: Utc::now().to_rfc3339(),
    };
    Ok(Event::default()
        .event(ev.sse_name())
        .data(ag_ui_json_with_conversation(&ev, conversation_id)))
}

pub(crate) fn prelude_stream(
    run_id: String,
    conversation_id: String,
) -> impl Stream<Item = Result<Event, Infallible>> {
    stream::once(async move { run_started_event(&run_id, &conversation_id) })
}

pub(crate) fn extract_done_text(output: &rusvel_core::domain::AgentOutput) -> String {
    output
        .content
        .parts
        .iter()
        .filter_map(|p| match p {
            Part::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

pub(crate) fn run_completed_sse(
    run_id: &str,
    full_text: String,
    cost: f64,
    conversation_id: &str,
) -> Event {
    let ev = AgUiEvent::RunCompleted {
        run_id: run_id.to_string(),
        output: full_text,
    };
    let mut v = serde_json::to_value(&ev).unwrap_or(serde_json::json!({}));
    if let Some(obj) = v.as_object_mut() {
        obj.insert("cost_usd".into(), serde_json::json!(cost));
        obj.insert(
            "conversation_id".into(),
            serde_json::Value::String(conversation_id.to_string()),
        );
    }
    Event::default().event(ev.sse_name()).data(v.to_string())
}

pub(crate) fn other_event_sse(run_id: &str, event: AgentEvent, conversation_id: &str) -> Event {
    let ag = agent_event_to_ag_ui(run_id, event);
    Event::default()
        .event(ag.sse_name())
        .data(ag_ui_json_with_conversation(&ag, conversation_id))
}

pub(crate) fn parse_model_ref(model: &str) -> ModelRef {
    if let Some((provider, name)) = model.split_once('/') {
        let provider = match provider {
            "ollama" => ModelProvider::Ollama,
            "openai" => ModelProvider::OpenAI,
            "claude" => ModelProvider::Claude,
            "gemini" => ModelProvider::Gemini,
            other => ModelProvider::Other(other.into()),
        };
        ModelRef {
            provider,
            model: name.into(),
        }
    } else {
        ModelRef {
            provider: ModelProvider::Claude,
            model: model.into(),
        }
    }
}
