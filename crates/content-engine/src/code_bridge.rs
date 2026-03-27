//! Build [`CodeAnalysisSummary`](rusvel_core::domain::CodeAnalysisSummary) from persisted
//! `code_analysis` JSON (same shape as `code_engine::CodeAnalysis`) without depending on
//! `code-engine`.

use rusvel_core::domain::CodeAnalysisSummary;
use rusvel_core::error::{Result, RusvelError};

/// Parse a stored code analysis document from the object store into a summary for prompts.
pub fn summary_from_stored_code_analysis(v: &serde_json::Value) -> Result<CodeAnalysisSummary> {
    let snapshot_id = v
        .get("snapshot")
        .and_then(|s| s.get("id"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .ok_or_else(|| RusvelError::Validation("code_analysis.snapshot.id missing".into()))?;

    let repo_path = v
        .get("snapshot")
        .and_then(|s| s.get("repo"))
        .and_then(|r| r.get("local_path"))
        .and_then(|v| v.as_str())
        .unwrap_or(".")
        .to_string();

    let total_files = v
        .get("metrics")
        .and_then(|m| m.get("total_files"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let total_symbols = v
        .get("metrics")
        .and_then(|m| m.get("total_symbols"))
        .and_then(|v| v.as_u64())
        .unwrap_or(0) as usize;

    let largest_function = v
        .get("metrics")
        .and_then(|m| m.get("largest_function"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let mut top_symbols: Vec<String> = v
        .get("symbols")
        .and_then(|s| s.as_array())
        .map(|arr| {
            arr.iter()
                .filter(|sym| {
                    sym.get("kind")
                        .and_then(|k| k.as_str())
                        .is_some_and(|k| k == "Function")
                })
                .take(10)
                .filter_map(|sym| {
                    sym.get("name")
                        .and_then(|n| n.as_str())
                        .map(|s| s.to_string())
                })
                .collect()
        })
        .unwrap_or_default();

    if top_symbols.is_empty() {
        top_symbols = v
            .get("symbols")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .take(10)
                    .filter_map(|sym| {
                        sym.get("name")
                            .and_then(|n| n.as_str())
                            .map(|s| s.to_string())
                    })
                    .collect()
            })
            .unwrap_or_default();
    }

    Ok(CodeAnalysisSummary {
        snapshot_id,
        repo_path,
        total_files,
        total_symbols,
        top_symbols,
        largest_function,
        metadata: Default::default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minimal_stored_analysis() {
        let v = serde_json::json!({
            "snapshot": {
                "id": "550e8400-e29b-41d4-a716-446655440000",
                "repo": { "local_path": "/tmp/repo", "remote_url": null },
                "analyzed_at": "2025-01-01T00:00:00Z"
            },
            "symbols": [
                { "name": "main", "kind": "Function", "file_path": "/tmp/repo/src/lib.rs", "line": 1, "end_line": 2, "visibility": "Public", "body": "" }
            ],
            "metrics": {
                "total_files": 3,
                "total_symbols": 12,
                "largest_function": "run"
            }
        });
        let s = summary_from_stored_code_analysis(&v).unwrap();
        assert_eq!(s.snapshot_id, "550e8400-e29b-41d4-a716-446655440000");
        assert_eq!(s.repo_path, "/tmp/repo");
        assert_eq!(s.total_files, 3);
        assert_eq!(s.total_symbols, 12);
        assert_eq!(s.largest_function.as_deref(), Some("run"));
        assert_eq!(s.top_symbols, vec!["main"]);
    }
}
