//! Rust source parser using tree-sitter.

use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CodeError, Result};

/// Kind of symbol extracted from source code.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum SymbolKind {
    Function,
    Struct,
    Enum,
    Trait,
    Impl,
    Const,
    Module,
}

/// Visibility of a symbol.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Visibility {
    Public,
    Private,
}

/// A symbol extracted from a Rust source file.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub file_path: PathBuf,
    pub line: usize,
    pub end_line: usize,
    pub visibility: Visibility,
    pub body: String,
}

/// Parse a single Rust file into a list of symbols.
pub fn parse_file(path: &Path) -> Result<Vec<Symbol>> {
    let source = fs::read_to_string(path)
        .map_err(|e| CodeError::Io(format!("{}: {}", path.display(), e)))?;
    parse_source(&source, path)
}

/// Parse Rust source code (given as a string) into symbols.
pub fn parse_source(source: &str, path: &Path) -> Result<Vec<Symbol>> {
    let mut parser = tree_sitter::Parser::new();
    let language = tree_sitter_rust::LANGUAGE;
    parser
        .set_language(&language.into())
        .map_err(|e| CodeError::Parse(e.to_string()))?;

    let tree = parser
        .parse(source, None)
        .ok_or_else(|| CodeError::Parse("tree-sitter parse failed".into()))?;

    let mut symbols = Vec::new();
    collect_symbols(tree.root_node(), source.as_bytes(), path, &mut symbols);
    Ok(symbols)
}

fn collect_symbols(node: tree_sitter::Node<'_>, source: &[u8], path: &Path, out: &mut Vec<Symbol>) {
    let kind_map: &[(&str, SymbolKind)] = &[
        ("function_item", SymbolKind::Function),
        ("struct_item", SymbolKind::Struct),
        ("enum_item", SymbolKind::Enum),
        ("trait_item", SymbolKind::Trait),
        ("impl_item", SymbolKind::Impl),
        ("const_item", SymbolKind::Const),
        ("mod_item", SymbolKind::Module),
    ];

    for (ts_kind, sym_kind) in kind_map {
        if node.kind() == *ts_kind {
            let name = extract_name(&node, source, sym_kind);
            let vis = if has_visibility_modifier(&node, source) {
                Visibility::Public
            } else {
                Visibility::Private
            };
            let body = node_text(&node, source);
            out.push(Symbol {
                name,
                kind: sym_kind.clone(),
                file_path: path.to_path_buf(),
                line: node.start_position().row + 1,
                end_line: node.end_position().row + 1,
                visibility: vis,
                body,
            });
            // Don't recurse into matched items (except impl blocks)
            if *sym_kind != SymbolKind::Impl {
                return;
            }
        }
    }

    let count = node.child_count();
    for i in 0..count {
        if let Some(child) = node.child(i) {
            collect_symbols(child, source, path, out);
        }
    }
}

fn extract_name(node: &tree_sitter::Node<'_>, source: &[u8], kind: &SymbolKind) -> String {
    // For impl items, build "impl Type" or "impl Trait for Type"
    if *kind == SymbolKind::Impl {
        return build_impl_name(node, source);
    }
    // Most items have a `name` child
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i)
            && (child.kind() == "identifier" || child.kind() == "type_identifier")
        {
            return node_text(&child, source);
        }
    }
    "<anonymous>".into()
}

fn build_impl_name(node: &tree_sitter::Node<'_>, source: &[u8]) -> String {
    let text = node_text(node, source);
    // Take first line, trim body
    let first_line = text.lines().next().unwrap_or("impl");
    let trimmed = first_line.trim_end_matches('{').trim();
    trimmed.to_string()
}

fn has_visibility_modifier(node: &tree_sitter::Node<'_>, source: &[u8]) -> bool {
    for i in 0..node.child_count() {
        if let Some(child) = node.child(i)
            && child.kind() == "visibility_modifier"
        {
            let text = node_text(&child, source);
            return text.starts_with("pub");
        }
    }
    false
}

fn node_text(node: &tree_sitter::Node<'_>, source: &[u8]) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    String::from_utf8_lossy(&source[start..end]).to_string()
}

/// Recursively parse all `.rs` files in a directory.
pub fn parse_directory(dir: &Path) -> Result<Vec<Symbol>> {
    let mut symbols = Vec::new();
    walk_rs_files(dir, &mut symbols)?;
    Ok(symbols)
}

fn walk_rs_files(dir: &Path, symbols: &mut Vec<Symbol>) -> Result<()> {
    let entries =
        fs::read_dir(dir).map_err(|e| CodeError::Io(format!("{}: {}", dir.display(), e)))?;
    for entry in entries {
        let entry = entry.map_err(|e| CodeError::Io(e.to_string()))?;
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, symbols)?;
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            match parse_file(&path) {
                Ok(mut syms) => symbols.append(&mut syms),
                Err(e) => {
                    eprintln!("warning: skipping {}: {}", path.display(), e);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_rust_snippet() {
        let src = r#"
pub fn greet(name: &str) -> String {
    format!("Hello, {name}")
}

struct Point { x: f64, y: f64 }

pub enum Color { Red, Green, Blue }
"#;
        let syms = parse_source(src, Path::new("test.rs")).unwrap();
        assert_eq!(syms.len(), 3);
        assert_eq!(syms[0].name, "greet");
        assert_eq!(syms[0].kind, SymbolKind::Function);
        assert_eq!(syms[0].visibility, Visibility::Public);
        assert_eq!(syms[1].name, "Point");
        assert_eq!(syms[1].kind, SymbolKind::Struct);
        assert_eq!(syms[1].visibility, Visibility::Private);
        assert_eq!(syms[2].name, "Color");
        assert_eq!(syms[2].kind, SymbolKind::Enum);
    }
}
