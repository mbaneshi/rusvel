//! Safety controls for the Forge Engine.
//!
//! Provides budget enforcement, concurrency limiting, and a simple
//! circuit breaker to protect against runaway agent costs.

use rusvel_core::error::{Result, RusvelError};
use std::sync::Mutex;

/// RAII guard that releases a concurrency slot on drop.
pub struct SafetySlot<'a> {
    guard: &'a SafetyGuard,
}

impl Drop for SafetySlot<'_> {
    fn drop(&mut self) {
        let mut active = self.guard.active_runs.lock().unwrap();
        *active = active.saturating_sub(1);
    }
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircuitState {
    Closed,
    Open,
}

/// Safety guard enforcing budget, concurrency, and circuit-breaker policies.
pub struct SafetyGuard {
    cost_limit: f64,
    total_spent: Mutex<f64>,
    max_concurrent: usize,
    active_runs: Mutex<usize>,
    consecutive_failures: Mutex<u32>,
    failure_threshold: u32,
    budget_warning_emitted: Mutex<bool>,
}

impl SafetyGuard {
    /// Create a new guard with the given limits.
    pub fn new(cost_limit: f64, max_concurrent: usize, failure_threshold: u32) -> Self {
        Self {
            cost_limit,
            total_spent: Mutex::new(0.0),
            max_concurrent,
            active_runs: Mutex::new(0),
            consecutive_failures: Mutex::new(0),
            failure_threshold,
            budget_warning_emitted: Mutex::new(false),
        }
    }

    /// Configured aggregate spend ceiling.
    pub fn cost_limit(&self) -> f64 {
        self.cost_limit
    }

    /// After [`record_spend`](Self::record_spend), call to emit at most one budget warning when
    /// cumulative spend reaches **≥ 80%** of [`cost_limit`](Self::cost_limit).
    pub fn take_budget_warning_if_needed(&self) -> Option<(f64, f64)> {
        let spent = *self.total_spent.lock().unwrap();
        let threshold = self.cost_limit * 0.8;
        if spent < threshold {
            return None;
        }
        let mut w = self.budget_warning_emitted.lock().unwrap();
        if *w {
            return None;
        }
        *w = true;
        Some((spent, self.cost_limit))
    }

    /// Check whether the estimated cost would exceed the budget.
    pub fn check_budget(&self, estimated_cost: f64) -> Result<()> {
        let spent = *self.total_spent.lock().unwrap();
        if spent + estimated_cost > self.cost_limit {
            return Err(RusvelError::BudgetExceeded {
                spent,
                limit: self.cost_limit,
            });
        }
        Ok(())
    }

    /// Record an actual spend amount.
    pub fn record_spend(&self, cost: f64) {
        let mut spent = self.total_spent.lock().unwrap();
        *spent += cost;
    }

    /// Return the total amount spent so far.
    pub fn total_spent(&self) -> f64 {
        *self.total_spent.lock().unwrap()
    }

    /// Check whether a new concurrent run is allowed.
    pub fn check_concurrency(&self) -> Result<()> {
        let active = *self.active_runs.lock().unwrap();
        if active >= self.max_concurrent {
            return Err(RusvelError::Validation(format!(
                "concurrency limit reached: {active}/{}",
                self.max_concurrent
            )));
        }
        Ok(())
    }

    /// Acquire a concurrency slot. Returns an RAII guard that releases on drop.
    pub fn acquire_slot(&self) -> Result<SafetySlot<'_>> {
        let mut active = self.active_runs.lock().unwrap();
        if *active >= self.max_concurrent {
            return Err(RusvelError::Validation(format!(
                "concurrency limit reached: {active}/{}",
                self.max_concurrent
            )));
        }
        *active += 1;
        Ok(SafetySlot { guard: self })
    }

    /// Return the number of currently active runs.
    pub fn active_runs(&self) -> usize {
        *self.active_runs.lock().unwrap()
    }

    // ── Circuit breaker ───────────────────────────────────────────

    /// Return the current circuit state.
    pub fn circuit_state(&self) -> CircuitState {
        let failures = *self.consecutive_failures.lock().unwrap();
        if failures >= self.failure_threshold {
            CircuitState::Open
        } else {
            CircuitState::Closed
        }
    }

    /// Record a successful run (resets failure counter).
    pub fn record_success(&self) {
        *self.consecutive_failures.lock().unwrap() = 0;
    }

    /// Record a failed run. Returns the new circuit state.
    pub fn record_failure(&self) -> CircuitState {
        let mut failures = self.consecutive_failures.lock().unwrap();
        *failures += 1;
        if *failures >= self.failure_threshold {
            CircuitState::Open
        } else {
            CircuitState::Closed
        }
    }

    /// Manually reset the circuit breaker to closed.
    pub fn reset_circuit(&self) {
        *self.consecutive_failures.lock().unwrap() = 0;
    }

    /// Check that the circuit is closed (i.e. runs are allowed).
    pub fn check_circuit(&self) -> Result<()> {
        if self.circuit_state() == CircuitState::Open {
            return Err(RusvelError::Validation(
                "circuit breaker is open — too many consecutive failures".into(),
            ));
        }
        Ok(())
    }
}

impl Default for SafetyGuard {
    fn default() -> Self {
        Self::new(10.0, 4, 5)
    }
}

// ── Tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_enforcement() {
        let g = SafetyGuard::new(1.0, 4, 5);
        assert!(g.check_budget(0.50).is_ok());
        g.record_spend(0.80);
        assert!(g.check_budget(0.30).is_err());
        assert!(g.check_budget(0.20).is_ok());
    }

    #[test]
    fn concurrency_slot_raii() {
        let g = SafetyGuard::new(10.0, 2, 5);
        let s1 = g.acquire_slot().unwrap();
        let _s2 = g.acquire_slot().unwrap();
        assert_eq!(g.active_runs(), 2);
        assert!(g.acquire_slot().is_err());
        drop(s1);
        assert_eq!(g.active_runs(), 1);
        assert!(g.acquire_slot().is_ok());
    }

    #[test]
    fn circuit_breaker_opens_after_threshold() {
        let g = SafetyGuard::new(10.0, 4, 3);
        assert_eq!(g.circuit_state(), CircuitState::Closed);
        g.record_failure();
        g.record_failure();
        assert_eq!(g.circuit_state(), CircuitState::Closed);
        g.record_failure();
        assert_eq!(g.circuit_state(), CircuitState::Open);
        assert!(g.check_circuit().is_err());
        g.record_success();
        assert_eq!(g.circuit_state(), CircuitState::Closed);
    }

    #[test]
    fn budget_warning_fires_once_at_eighty_percent() {
        let g = SafetyGuard::new(10.0, 4, 5);
        assert!(g.take_budget_warning_if_needed().is_none());
        g.record_spend(7.0);
        assert!(g.take_budget_warning_if_needed().is_none());
        g.record_spend(1.0);
        let w = g.take_budget_warning_if_needed();
        assert_eq!(w, Some((8.0, 10.0)));
        assert!(g.take_budget_warning_if_needed().is_none());
    }
}
