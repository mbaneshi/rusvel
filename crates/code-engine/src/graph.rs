//! Symbol dependency graph built from parsed symbols.

use crate::parser::Symbol;

/// A graph of symbol dependencies.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolGraph {
    pub nodes: Vec<Symbol>,
    pub edges: Vec<(usize, usize)>,
}

impl SymbolGraph {
    /// Build a dependency graph from a list of symbols.
    ///
    /// An edge A -> B is added when symbol A's body contains symbol B's
    /// name as a word (simple heuristic).
    pub fn build(symbols: Vec<Symbol>) -> Self {
        let mut edges = Vec::new();
        for (i, a) in symbols.iter().enumerate() {
            for (j, b) in symbols.iter().enumerate() {
                if i == j || b.name.len() < 2 {
                    continue;
                }
                if contains_word(&a.body, &b.name) {
                    edges.push((i, j));
                }
            }
        }
        Self {
            nodes: symbols,
            edges,
        }
    }

    /// Return indices of symbols that `symbol_index` depends on.
    pub fn dependencies(&self, symbol_index: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(from, _)| *from == symbol_index)
            .map(|(_, to)| *to)
            .collect()
    }

    /// Return indices of symbols that depend on `symbol_index`.
    pub fn dependents(&self, symbol_index: usize) -> Vec<usize> {
        self.edges
            .iter()
            .filter(|(_, to)| *to == symbol_index)
            .map(|(from, _)| *from)
            .collect()
    }

    /// Compute connected components using union-find.
    pub fn connected_components(&self) -> Vec<Vec<usize>> {
        let n = self.nodes.len();
        let mut parent: Vec<usize> = (0..n).collect();
        let mut rank = vec![0u32; n];

        for &(a, b) in &self.edges {
            union(&mut parent, &mut rank, a, b);
        }

        let mut components: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            components.entry(root).or_default().push(i);
        }
        components.into_values().collect()
    }
}

fn find(parent: &mut [usize], i: usize) -> usize {
    if parent[i] != i {
        parent[i] = find(parent, parent[i]);
    }
    parent[i]
}

fn union(parent: &mut [usize], rank: &mut [u32], a: usize, b: usize) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra == rb {
        return;
    }
    if rank[ra] < rank[rb] {
        parent[ra] = rb;
    } else if rank[ra] > rank[rb] {
        parent[rb] = ra;
    } else {
        parent[rb] = ra;
        rank[ra] += 1;
    }
}

/// Check if `haystack` contains `needle` as a whole word.
fn contains_word(haystack: &str, needle: &str) -> bool {
    for (idx, _) in haystack.match_indices(needle) {
        let before_ok = idx == 0
            || !haystack.as_bytes()[idx - 1].is_ascii_alphanumeric()
                && haystack.as_bytes()[idx - 1] != b'_';
        let after_pos = idx + needle.len();
        let after_ok = after_pos >= haystack.len()
            || !haystack.as_bytes()[after_pos].is_ascii_alphanumeric()
                && haystack.as_bytes()[after_pos] != b'_';
        if before_ok && after_ok {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::{SymbolKind, Visibility};
    use std::path::PathBuf;

    fn sym(name: &str, body: &str) -> Symbol {
        Symbol {
            name: name.into(),
            kind: SymbolKind::Function,
            file_path: PathBuf::from("test.rs"),
            line: 1,
            end_line: 1,
            visibility: Visibility::Private,
            body: body.into(),
        }
    }

    #[test]
    fn graph_edges_and_components() {
        let symbols = vec![
            sym("foo", "fn foo() { bar() }"),
            sym("bar", "fn bar() { }"),
            sym("baz", "fn baz() { }"),
        ];
        let g = SymbolGraph::build(symbols);
        assert!(g.edges.contains(&(0, 1))); // foo -> bar
        assert_eq!(g.dependencies(0), vec![1]);
        assert_eq!(g.dependents(1), vec![0]);
        let cc = g.connected_components();
        // foo+bar connected, baz isolated
        assert_eq!(cc.len(), 2);
    }
}
