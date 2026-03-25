//! Reciprocal rank fusion (RRF) for merging ranked retrieval lists.

use std::collections::HashMap;
use std::hash::Hash;

/// Default RRF constant `k` (common in literature, e.g. 60).
pub const RRF_K_DEFAULT: usize = 60;

/// Merge multiple best-first ranked lists into a single score per distinct item.
///
/// `ranked_lists[i]` is ordered best-to-worst. Each `T` must be a stable fusion key
/// (e.g. `memory:<uuid>` vs `vector:<id>`).
pub fn reciprocal_rank_fusion<T: Clone + Eq + Hash>(
    ranked_lists: &[Vec<T>],
    k: usize,
) -> Vec<(T, f64)> {
    let mut scores: HashMap<T, f64> = HashMap::new();
    for list in ranked_lists {
        for (rank_idx, id) in list.iter().enumerate() {
            let rank = rank_idx + 1;
            *scores.entry(id.clone()).or_insert(0.0) += 1.0 / (k as f64 + rank as f64);
        }
    }
    let mut out: Vec<(T, f64)> = scores.into_iter().collect();
    out.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rrf_orders_by_fused_score() {
        let a = vec!["x".to_string(), "y".to_string()];
        let b = vec!["y".to_string(), "z".to_string()];
        let fused = reciprocal_rank_fusion(&[a, b], 60);
        assert_eq!(fused.len(), 3);
        assert_eq!(fused[0].0, "y");
    }
}
