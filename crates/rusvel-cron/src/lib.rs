//! Scheduled jobs: persist cron definitions in [`ObjectStore`], tick on a timer,
//! enqueue [`JobKind::ScheduledCron`] work on the central [`JobPort`].

use std::str::FromStr;
use std::sync::Arc;

use chrono::{DateTime, Utc};
use cron::Schedule;
use rusvel_core::domain::{JobKind, NewJob, ObjectFilter};
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;
use rusvel_core::ports::{JobPort, ObjectStore, StoragePort};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Object-store `kind` for persisted schedules (S-041 bucket `cron_schedules`).
pub const CRON_SCHEDULE_OBJECT_KIND: &str = "cron_schedules";

/// Persisted schedule row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronScheduleRecord {
    pub id: String,
    pub name: String,
    pub session_id: SessionId,
    /// `hourly` / `daily` / `weekly`, standard 5-field cron, or 6/7-field [`cron`] syntax.
    pub schedule: String,
    pub enabled: bool,
    #[serde(default)]
    pub payload: Value,
    /// Emitted as [`rusvel_core::domain::Event::kind`] when the job runs (ADR-005).
    #[serde(default)]
    pub event_kind: String,
    pub created_at: DateTime<Utc>,
    pub last_fired_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub metadata: Value,
}

/// List / summary (same fields as needed for UI).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronScheduleSummary {
    pub id: String,
    pub name: String,
    pub session_id: SessionId,
    pub schedule: String,
    pub enabled: bool,
    pub event_kind: String,
    pub created_at: DateTime<Utc>,
    pub last_fired_at: Option<DateTime<Utc>>,
}

#[derive(Clone)]
pub struct CronScheduler {
    storage: Arc<dyn StoragePort>,
    jobs: Arc<dyn JobPort>,
}

impl CronScheduler {
    pub fn new(storage: Arc<dyn StoragePort>, jobs: Arc<dyn JobPort>) -> Self {
        Self { storage, jobs }
    }

    /// Run [`Self::tick`] on a fixed interval (missed ticks skipped). Used by the API binary.
    ///
    /// Accepts a shutdown receiver so the ticker stops when the app shuts down.
    pub fn spawn_interval_ticker(
        self: Arc<Self>,
        period: std::time::Duration,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) -> tokio::task::JoinHandle<()> {
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(period);
            interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
            loop {
                tokio::select! {
                    _ = interval.tick() => {
                        if let Err(e) = self.tick().await {
                            tracing::warn!(error = %e, "Cron tick failed");
                        }
                    }
                    _ = shutdown_rx.changed() => {
                        tracing::info!("Cron ticker shutting down");
                        break;
                    }
                }
            }
        })
    }

    fn objects(&self) -> &dyn ObjectStore {
        self.storage.objects()
    }

    pub async fn create(
        &self,
        name: String,
        session_id: SessionId,
        schedule: String,
        payload: Value,
        event_kind: String,
        enabled: bool,
    ) -> Result<CronScheduleRecord> {
        if name.trim().is_empty() {
            return Err(RusvelError::Validation("name must not be empty".into()));
        }
        parse_schedule(&schedule).map_err(RusvelError::Validation)?;

        let id = uuid::Uuid::now_v7().to_string();
        let now = Utc::now();
        let ek = if event_kind.trim().is_empty() {
            "cron.fired".to_string()
        } else {
            event_kind
        };
        let record = CronScheduleRecord {
            id: id.clone(),
            name: name.clone(),
            session_id,
            schedule: schedule.clone(),
            enabled,
            payload,
            event_kind: ek,
            created_at: now,
            last_fired_at: None,
            metadata: json!({}),
        };

        self.objects()
            .put(
                CRON_SCHEDULE_OBJECT_KIND,
                &id,
                serde_json::to_value(&record)
                    .map_err(|e| RusvelError::Serialization(e.to_string()))?,
            )
            .await?;

        Ok(record)
    }

    pub async fn list(&self) -> Result<Vec<CronScheduleSummary>> {
        let rows = self
            .objects()
            .list(CRON_SCHEDULE_OBJECT_KIND, ObjectFilter::default())
            .await?;

        let mut out: Vec<CronScheduleSummary> = rows
            .into_iter()
            .filter_map(|v| serde_json::from_value::<CronScheduleRecord>(v).ok())
            .map(|r| CronScheduleSummary {
                id: r.id,
                name: r.name,
                session_id: r.session_id,
                schedule: r.schedule,
                enabled: r.enabled,
                event_kind: r.event_kind,
                created_at: r.created_at,
                last_fired_at: r.last_fired_at,
            })
            .collect();
        out.sort_by_key(|s| s.created_at);
        Ok(out)
    }

    pub async fn get(&self, id: &str) -> Result<Option<CronScheduleRecord>> {
        let raw = self.objects().get(CRON_SCHEDULE_OBJECT_KIND, id).await?;
        Ok(match raw {
            Some(v) => Some(
                serde_json::from_value(v).map_err(|e| RusvelError::Serialization(e.to_string()))?,
            ),
            None => None,
        })
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.objects().delete(CRON_SCHEDULE_OBJECT_KIND, id).await
    }

    pub async fn update(
        &self,
        id: &str,
        name: Option<String>,
        schedule: Option<String>,
        enabled: Option<bool>,
        payload: Option<Value>,
        event_kind: Option<String>,
    ) -> Result<CronScheduleRecord> {
        let mut record = self.get(id).await?.ok_or_else(|| RusvelError::NotFound {
            kind: CRON_SCHEDULE_OBJECT_KIND.into(),
            id: id.into(),
        })?;

        if let Some(n) = name {
            if n.trim().is_empty() {
                return Err(RusvelError::Validation("name must not be empty".into()));
            }
            record.name = n;
        }
        if let Some(s) = schedule {
            parse_schedule(&s).map_err(RusvelError::Validation)?;
            record.schedule = s;
        }
        if let Some(e) = enabled {
            record.enabled = e;
        }
        if let Some(p) = payload {
            record.payload = p;
        }
        if let Some(ek) = event_kind {
            record.event_kind = if ek.trim().is_empty() {
                "cron.fired".into()
            } else {
                ek
            };
        }

        self.objects()
            .put(
                CRON_SCHEDULE_OBJECT_KIND,
                id,
                serde_json::to_value(&record)
                    .map_err(|e| RusvelError::Serialization(e.to_string()))?,
            )
            .await?;

        Ok(record)
    }

    /// Evaluate all enabled schedules; enqueue at most one [`ScheduledCron`] job per schedule
    /// when the next fire time is due (best-effort ~tick interval).
    pub async fn tick(&self) -> Result<()> {
        let rows = self
            .objects()
            .list(CRON_SCHEDULE_OBJECT_KIND, ObjectFilter::default())
            .await?;

        let now = Utc::now();

        for v in rows {
            let mut record: CronScheduleRecord = match serde_json::from_value(v) {
                Ok(r) => r,
                Err(_) => continue,
            };

            if !record.enabled {
                continue;
            }

            let schedule = match parse_schedule(&record.schedule) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(id = %record.id, error = %e, "Invalid cron schedule in store");
                    continue;
                }
            };

            let anchor = record
                .last_fired_at
                .unwrap_or_else(|| now - chrono::Duration::seconds(1));

            let Some(next) = next_fire_after(&schedule, anchor) else {
                continue;
            };

            if next > now {
                continue;
            }

            let nj = NewJob {
                session_id: record.session_id,
                kind: JobKind::ScheduledCron,
                payload: json!({
                    "schedule_id": record.id,
                    "payload": record.payload,
                    "event_kind": record.event_kind,
                }),
                max_retries: 1,
                metadata: json!({ "cron_schedule_id": record.id }),
                scheduled_at: None,
            };

            self.jobs.enqueue(nj).await?;
            record.last_fired_at = Some(now);
            if let Err(e) = self
                .objects()
                .put(
                    CRON_SCHEDULE_OBJECT_KIND,
                    &record.id,
                    serde_json::to_value(&record)
                        .map_err(|e| RusvelError::Serialization(e.to_string()))?,
                )
                .await
            {
                tracing::warn!(id = %record.id, error = %e, "Failed to persist last_fired_at for cron");
            }
        }

        Ok(())
    }
}

fn next_fire_after(schedule: &Schedule, after: DateTime<Utc>) -> Option<DateTime<Utc>> {
    schedule.after(&after).next()
}

/// Normalize presets and 5-field cron to the 7-field format expected by [`cron`] 0.15.
pub fn parse_schedule(expr: &str) -> std::result::Result<Schedule, String> {
    let normalized = normalize_schedule_expr(expr)?;
    Schedule::from_str(&normalized).map_err(|e| e.to_string())
}

fn normalize_schedule_expr(expr: &str) -> std::result::Result<String, String> {
    let e = expr.trim();
    if e.is_empty() {
        return Err("schedule expression is empty".into());
    }
    let lower = e.to_lowercase();
    match lower.as_str() {
        "hourly" => return Ok("0 0 * * * * *".into()),
        "daily" => return Ok("0 0 9 * * * *".into()),
        "weekly" => return Ok("0 0 9 * * 1 *".into()),
        _ => {}
    }

    let parts: Vec<&str> = e.split_whitespace().collect();
    match parts.len() {
        5 => Ok(format!(
            "0 {} {} {} {} {} *",
            parts[0], parts[1], parts[2], parts[3], parts[4]
        )),
        6 => Ok(format!("{e} *")),
        _ => Ok(e.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn presets_parse() {
        parse_schedule("hourly").unwrap();
        parse_schedule("daily").unwrap();
        parse_schedule("weekly").unwrap();
    }

    #[test]
    fn five_field_unix_wraps() {
        let s = parse_schedule("0 * * * *").unwrap();
        let next = next_fire_after(&s, Utc::now() - chrono::Duration::hours(1));
        assert!(next.is_some());
    }
}
