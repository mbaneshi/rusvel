//! Finance Department — DepartmentApp wrapping `finance-engine` (ADR-014).

mod manifest;

use std::sync::{Arc, OnceLock};

use async_trait::async_trait;
use finance_engine::FinanceEngine;
use rusvel_core::Engine;
use rusvel_core::department::*;
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;

pub struct FinanceDepartment {
    engine: OnceLock<Arc<FinanceEngine>>,
}

impl FinanceDepartment {
    pub fn new() -> Self {
        Self {
            engine: OnceLock::new(),
        }
    }

    pub fn engine(&self) -> Option<&Arc<FinanceEngine>> {
        self.engine.get()
    }
}

impl Default for FinanceDepartment {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_session_id(args: &serde_json::Value) -> rusvel_core::error::Result<SessionId> {
    args.get("session_id")
        .and_then(|v| v.as_str())
        .and_then(|s| s.parse().ok())
        .map(SessionId::from_uuid)
        .ok_or_else(|| {
            rusvel_core::error::RusvelError::Validation("session_id required or invalid".into())
        })
}

#[async_trait]
impl DepartmentApp for FinanceDepartment {
    fn manifest(&self) -> DepartmentManifest {
        manifest::finance_manifest()
    }

    async fn register(&self, ctx: &mut RegistrationContext) -> Result<()> {
        let engine = Arc::new(FinanceEngine::new(
            ctx.storage.clone(),
            ctx.events.clone(),
            ctx.agent.clone(),
            ctx.jobs.clone(),
        ));
        let _ = self.engine.set(engine.clone());

        let eng = engine.clone();
        ctx.tools.add(
            "finance",
            "finance.ledger.record",
            "Record an income or expense transaction in the ledger",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "kind": { "type": "string", "enum": ["Income", "Expense"], "description": "Transaction type" },
                    "amount": { "type": "number", "description": "Amount in dollars" },
                    "description": { "type": "string", "description": "What the transaction is for" },
                    "category": { "type": "string", "description": "Budget category (e.g. sales, infra)" }
                },
                "required": ["session_id", "kind", "amount", "description", "category"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let kind: finance_engine::TransactionKind = serde_json::from_value(
                        args.get("kind").cloned().unwrap_or_default(),
                    )
                    .map_err(|_| {
                        rusvel_core::error::RusvelError::Validation(
                            "kind must be Income or Expense".into(),
                        )
                    })?;
                    let amount = args.get("amount").and_then(|v| v.as_f64()).ok_or_else(|| {
                        rusvel_core::error::RusvelError::Validation("amount required".into())
                    })?;
                    let description = args
                        .get("description")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let category = args
                        .get("category")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let tx_id = eng
                        .ledger()
                        .record(sid, kind, amount, description, category)
                        .await?;
                    Ok(ToolOutput {
                        content: format!("Transaction recorded: {tx_id}"),
                        is_error: false,
                        metadata: serde_json::json!({ "transaction_id": tx_id.to_string() }),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "finance",
            "finance.ledger.balance",
            "Get the current ledger balance (income minus expenses) for a session",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" }
                },
                "required": ["session_id"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let balance = eng.ledger().balance(sid).await?;
                    Ok(ToolOutput {
                        content: format!("{balance:.2}"),
                        is_error: false,
                        metadata: serde_json::json!({ "balance": balance }),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "finance",
            "finance.tax.add_estimate",
            "Add a tax estimate for a category and period",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" },
                    "category": { "type": "string", "enum": ["Income", "SelfEmployment", "Sales", "Deduction"], "description": "Tax category" },
                    "amount": { "type": "number", "description": "Estimated tax amount" },
                    "period": { "type": "string", "description": "Tax period (e.g. Q1, Q2)" }
                },
                "required": ["session_id", "category", "amount", "period"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let category: finance_engine::TaxCategory = serde_json::from_value(
                        args.get("category").cloned().unwrap_or_default(),
                    )
                    .map_err(|_| {
                        rusvel_core::error::RusvelError::Validation(
                            "category must be Income, SelfEmployment, Sales, or Deduction".into(),
                        )
                    })?;
                    let amount = args.get("amount").and_then(|v| v.as_f64()).ok_or_else(|| {
                        rusvel_core::error::RusvelError::Validation("amount required".into())
                    })?;
                    let period = args
                        .get("period")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| {
                            rusvel_core::error::RusvelError::Validation("period required".into())
                        })?
                        .to_string();
                    let est_id = eng
                        .tax()
                        .add_estimate(sid, category, amount, period)
                        .await?;
                    Ok(ToolOutput {
                        content: format!("Tax estimate added: {est_id}"),
                        is_error: false,
                        metadata: serde_json::json!({ "estimate_id": est_id.to_string() }),
                    })
                })
            }),
        );

        let eng = engine.clone();
        ctx.tools.add(
            "finance",
            "finance.tax.total_liability",
            "Calculate total tax liability for a session (estimates minus deductions)",
            serde_json::json!({
                "type": "object",
                "properties": {
                    "session_id": { "type": "string", "description": "Session UUID" }
                },
                "required": ["session_id"]
            }),
            Arc::new(move |args| {
                let eng = eng.clone();
                Box::pin(async move {
                    let sid = parse_session_id(&args)?;
                    let total = eng.tax().total_liability(sid).await?;
                    Ok(ToolOutput {
                        content: format!("{total:.2}"),
                        is_error: false,
                        metadata: serde_json::json!({ "total_liability": total }),
                    })
                })
            }),
        );

        tracing::info!("Finance department registered");
        Ok(())
    }

    async fn shutdown(&self) -> Result<()> {
        if let Some(engine) = self.engine.get() {
            engine.shutdown().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn department_creates() {
        let dept = FinanceDepartment::new();
        let m = dept.manifest();
        assert_eq!(m.id, "finance");
        assert_eq!(m.requires_ports.len(), 4);
        assert!(dept.engine().is_none());
    }

    #[test]
    fn manifest_is_pure() {
        let dept = FinanceDepartment::new();
        let m1 = dept.manifest();
        let m2 = dept.manifest();
        assert_eq!(m1.id, m2.id);
        assert_eq!(m1.quick_actions.len(), m2.quick_actions.len());
    }
}
