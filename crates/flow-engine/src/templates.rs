//! Built-in [`FlowDef`] templates (cross-engine handoff, etc.).

use rusvel_core::domain::{FlowConnectionDef, FlowDef, FlowNodeDef};
use rusvel_core::id::{FlowId, FlowNodeId};

/// Stable template id (S-042 demo — two agent nodes chained; save via `POST /api/flows`).
pub fn cross_engine_handoff_template() -> FlowDef {
    let flow_id =
        FlowId::from_uuid(uuid::Uuid::parse_str("00000000-0000-4000-8000-000000000042").unwrap());
    let n1 = FlowNodeId::new();
    let n2 = FlowNodeId::new();
    FlowDef {
        id: flow_id,
        name: "Cross-engine handoff (demo)".into(),
        description: "Agent → agent: first node reads {{trigger}}; second consumes {{n1}}. \
                       Use as a starting point for harvest→content / code→GTM pipelines."
            .into(),
        nodes: vec![
            FlowNodeDef {
                id: n1,
                node_type: "agent".into(),
                name: "Synthesize signals".into(),
                parameters: serde_json::json!({
                    "prompt": "You are a cross-department synthesis step. Input context: {{trigger}}. \
                                Output exactly 3 bullets: (1) signals (2) risks (3) next check.",
                    "model": "sonnet"
                }),
                position: (0.0, 0.0),
                on_error: Default::default(),
                metadata: serde_json::json!({}),
            },
            FlowNodeDef {
                id: n2,
                node_type: "agent".into(),
                name: "Downstream handoff".into(),
                parameters: serde_json::json!({
                    "prompt": "You shape the next step for content/GTM. Prior output: {{n1}}. \
                                Write one short paragraph plus one suggested CTA line.",
                    "model": "sonnet"
                }),
                position: (240.0, 0.0),
                on_error: Default::default(),
                metadata: serde_json::json!({}),
            },
        ],
        connections: vec![FlowConnectionDef {
            source_node: n1,
            source_output: "main".into(),
            target_node: n2,
            target_input: "main".into(),
            metadata: Default::default(),
        }],
        variables: std::collections::HashMap::new(),
        metadata: serde_json::json!({
            "template": "cross_engine_handoff",
            "sprint": "S-042"
        }),
    }
}
