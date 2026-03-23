//! Burn rate and runway forecasting.

use chrono::{DateTime, Utc};
use rusvel_core::error::Result;
use rusvel_core::id::SessionId;
use rusvel_core::ports::StoragePort;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ledger::{LedgerManager, TransactionKind};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunwaySnapshot {
    pub monthly_burn: f64,
    pub monthly_income: f64,
    pub cash_on_hand: f64,
    pub months_remaining: f64,
    pub calculated_at: DateTime<Utc>,
    pub metadata: serde_json::Value,
}

const KIND: &str = "finance_runway";

pub struct RunwayManager {
    storage: Arc<dyn StoragePort>,
}

impl RunwayManager {
    pub fn new(storage: Arc<dyn StoragePort>) -> Self {
        Self { storage }
    }

    pub async fn calculate(
        &self,
        session_id: SessionId,
        cash_on_hand: f64,
        ledger: &LedgerManager,
    ) -> Result<RunwaySnapshot> {
        let txs = ledger.list_transactions(session_id).await?;
        let monthly_income: f64 = txs
            .iter()
            .filter(|t| t.kind == TransactionKind::Income)
            .map(|t| t.amount)
            .sum();
        let monthly_burn: f64 = txs
            .iter()
            .filter(|t| t.kind == TransactionKind::Expense)
            .map(|t| t.amount)
            .sum();
        let net_burn = monthly_burn - monthly_income;
        let months_remaining = if net_burn > 0.0 {
            cash_on_hand / net_burn
        } else {
            f64::INFINITY
        };
        let snapshot = RunwaySnapshot {
            monthly_burn,
            monthly_income,
            cash_on_hand,
            months_remaining,
            calculated_at: Utc::now(),
            metadata: serde_json::json!({}),
        };
        let key = format!("{session_id}_latest");
        self.storage
            .objects()
            .put(KIND, &key, serde_json::to_value(&snapshot)?)
            .await?;
        Ok(snapshot)
    }

    pub async fn latest(&self, session_id: SessionId) -> Result<Option<RunwaySnapshot>> {
        let key = format!("{session_id}_latest");
        let val = self.storage.objects().get(KIND, &key).await?;
        Ok(val.and_then(|v| serde_json::from_value(v).ok()))
    }
}
