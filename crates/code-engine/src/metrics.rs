//! Basic code metrics: line counts and project-level statistics.

use std::fs;
use std::path::Path;

use crate::error::{CodeError, Result};
use crate::parser::Symbol;

/// Line-level metrics for a single file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FileMetrics {
    pub path: String,
    pub lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub code_lines: usize,
}

/// Project-level aggregate metrics.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProjectMetrics {
    pub total_files: usize,
    pub total_lines: usize,
    pub total_symbols: usize,
    pub avg_function_length: f64,
    pub largest_function: Option<String>,
}

/// Count lines in a single file, categorizing blank, comment, and code.
pub fn count_lines(path: &Path) -> Result<FileMetrics> {
    let content = fs::read_to_string(path)
        .map_err(|e| CodeError::Io(format!("{}: {}", path.display(), e)))?;
    Ok(count_lines_str(&content, &path.display().to_string()))
}

/// Count lines from a string (useful for testing).
pub fn count_lines_str(content: &str, path_label: &str) -> FileMetrics {
    let mut lines = 0usize;
    let mut blank = 0usize;
    let mut comment = 0usize;
    let mut in_block_comment = false;

    for line in content.lines() {
        lines += 1;
        let trimmed = line.trim();

        if in_block_comment {
            comment += 1;
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.is_empty() {
            blank += 1;
        } else if trimmed.starts_with("//") {
            comment += 1;
        } else if trimmed.starts_with("/*") {
            comment += 1;
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
        }
    }

    FileMetrics {
        path: path_label.to_string(),
        lines,
        blank_lines: blank,
        comment_lines: comment,
        code_lines: lines.saturating_sub(blank + comment),
    }
}

/// Compute project-level metrics from symbols and file metrics.
pub fn compute_project_metrics(symbols: &[Symbol], file_metrics: &[FileMetrics]) -> ProjectMetrics {
    use crate::parser::SymbolKind;

    let functions: Vec<&Symbol> = symbols
        .iter()
        .filter(|s| s.kind == SymbolKind::Function)
        .collect();

    let avg_fn_len = if functions.is_empty() {
        0.0
    } else {
        let total: usize = functions
            .iter()
            .map(|f| f.end_line.saturating_sub(f.line) + 1)
            .sum();
        total as f64 / functions.len() as f64
    };

    let largest = functions
        .iter()
        .max_by_key(|f| f.end_line.saturating_sub(f.line))
        .map(|f| f.name.clone());

    ProjectMetrics {
        total_files: file_metrics.len(),
        total_lines: file_metrics.iter().map(|f| f.lines).sum(),
        total_symbols: symbols.len(),
        avg_function_length: avg_fn_len,
        largest_function: largest,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn count_lines_correctly() {
        let src = "// comment\n\nfn main() {\n    println!(\"hi\");\n}\n";
        let m = count_lines_str(src, "test.rs");
        assert_eq!(m.lines, 5);
        assert_eq!(m.blank_lines, 1);
        assert_eq!(m.comment_lines, 1);
        assert_eq!(m.code_lines, 3);
    }
}
