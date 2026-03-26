//! Flatten [`LlmRequest`] messages into a single string for CLIs that take one prompt.

use rusvel_core::domain::*;

pub(crate) fn extract_text(content: &Content) -> String {
    content
        .parts
        .iter()
        .filter_map(|p| match p {
            Part::Text(t) => Some(t.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Build one prompt string from an [`LlmRequest`] (system/user/assistant/tool tags).
pub(crate) fn flat_prompt(request: &LlmRequest) -> String {
    let mut parts = Vec::new();

    for msg in &request.messages {
        let text = extract_text(&msg.content);
        if text.is_empty() {
            continue;
        }
        match msg.role {
            LlmRole::System => parts.push(format!("<system>\n{text}\n</system>")),
            LlmRole::User => parts.push(text),
            LlmRole::Assistant => parts.push(format!("<assistant>\n{text}\n</assistant>")),
            LlmRole::Tool => parts.push(format!("<tool-result>\n{text}\n</tool-result>")),
        }
    }

    parts.join("\n\n")
}
