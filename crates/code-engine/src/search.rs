//! BM25 full-text search over symbols.

use std::collections::HashMap;

/// A search result with relevance score.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub symbol_name: String,
    pub file_path: String,
    pub line: usize,
    pub score: f64,
}

/// A BM25 search index built from (name, content) document pairs.
pub struct SearchIndex {
    docs: Vec<DocEntry>,
    /// term -> list of (`doc_index`, `term_freq`)
    inverted: HashMap<String, Vec<(usize, u32)>>,
    avg_dl: f64,
}

struct DocEntry {
    symbol_name: String,
    file_path: String,
    line: usize,
    doc_len: u32,
}

const K1: f64 = 1.2;
const B: f64 = 0.75;

impl SearchIndex {
    /// Build an index from parsed symbols.
    pub fn build(
        items: &[(String, String, String, usize)], // (name, file_path, content, line)
    ) -> Self {
        let mut docs = Vec::with_capacity(items.len());
        let mut inverted: HashMap<String, Vec<(usize, u32)>> = HashMap::new();
        let mut total_len: u64 = 0;

        for (idx, (name, file_path, content, line)) in items.iter().enumerate() {
            let tokens = tokenize(content);
            let doc_len = tokens.len() as u32;
            total_len += u64::from(doc_len);

            // Count term frequencies
            let mut tf_map: HashMap<&str, u32> = HashMap::new();
            for t in &tokens {
                *tf_map.entry(t.as_str()).or_default() += 1;
            }
            for (term, freq) in tf_map {
                inverted
                    .entry(term.to_string())
                    .or_default()
                    .push((idx, freq));
            }

            docs.push(DocEntry {
                symbol_name: name.clone(),
                file_path: file_path.clone(),
                line: *line,
                doc_len,
            });
        }

        let avg_dl = if docs.is_empty() {
            1.0
        } else {
            total_len as f64 / docs.len() as f64
        };

        Self {
            docs,
            inverted,
            avg_dl,
        }
    }

    /// Search the index, returning up to `limit` results sorted by score.
    pub fn search(&self, query: &str, limit: usize) -> Vec<SearchResult> {
        let query_terms = tokenize(query);
        let n = self.docs.len() as f64;
        let mut scores = vec![0.0f64; self.docs.len()];

        for term in &query_terms {
            let Some(postings) = self.inverted.get(term.as_str()) else {
                continue;
            };
            let df = postings.len() as f64;
            let idf = ((n - df + 0.5) / (df + 0.5) + 1.0).ln();

            for &(doc_idx, tf) in postings {
                let dl = f64::from(self.docs[doc_idx].doc_len);
                let tf_f = f64::from(tf);
                let num = tf_f * (K1 + 1.0);
                let denom = tf_f + K1 * (1.0 - B + B * dl / self.avg_dl);
                scores[doc_idx] += idf * num / denom;
            }
        }

        let mut ranked: Vec<(usize, f64)> = scores
            .iter()
            .enumerate()
            .filter(|t| *t.1 > 0.0)
            .map(|t| (t.0, *t.1))
            .collect();
        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        ranked.truncate(limit);

        ranked
            .into_iter()
            .map(|(idx, score)| {
                let doc = &self.docs[idx];
                SearchResult {
                    symbol_name: doc.symbol_name.clone(),
                    file_path: doc.file_path.clone(),
                    line: doc.line,
                    score,
                }
            })
            .collect()
    }
}

/// Tokenize text by splitting on non-alphanumeric chars and lowercasing.
fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|s| !s.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bm25_returns_relevant_results() {
        let items = vec![
            (
                "greet".into(),
                "lib.rs".into(),
                "fn greet name hello world".into(),
                1,
            ),
            (
                "add".into(),
                "math.rs".into(),
                "fn add numbers sum".into(),
                5,
            ),
            (
                "parse".into(),
                "parser.rs".into(),
                "fn parse source tree".into(),
                10,
            ),
        ];
        let idx = SearchIndex::build(&items);
        let results = idx.search("greet hello", 10);
        assert!(!results.is_empty());
        assert_eq!(results[0].symbol_name, "greet");
        assert!(results[0].score > 0.0);
    }
}
