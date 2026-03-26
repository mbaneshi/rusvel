//! In-memory event bus implementing [`EventPort`] from `rusvel-core`.
//!
//! Events are both broadcast to live subscribers (via `tokio::sync::broadcast`)
//! and persisted to an [`EventStore`] for later retrieval.

pub mod triggers;

pub use triggers::TriggerManager;

use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::broadcast;

use rusvel_core::domain::{Event, EventFilter};
use rusvel_core::error::Result;
use rusvel_core::id::EventId;
use rusvel_core::ports::{EventPort, EventStore};

/// Default broadcast channel capacity.
const DEFAULT_CAPACITY: usize = 256;

/// In-memory event bus that broadcasts events to subscribers and
/// persists them via an [`EventStore`] backend.
pub struct EventBus {
    store: Arc<dyn EventStore>,
    sender: broadcast::Sender<Event>,
}

impl EventBus {
    /// Create a new `EventBus` backed by the given store.
    pub fn new(store: Arc<dyn EventStore>) -> Self {
        Self::with_capacity(store, DEFAULT_CAPACITY)
    }

    /// Create a new `EventBus` with a custom broadcast channel capacity.
    pub fn with_capacity(store: Arc<dyn EventStore>, capacity: usize) -> Self {
        let (sender, _) = broadcast::channel(capacity);
        Self { store, sender }
    }

    /// Subscribe to live events. Returns a broadcast receiver.
    ///
    /// This is **not** part of `EventPort` but available on the concrete type
    /// for components that need real-time event streaming.
    pub fn subscribe(&self) -> broadcast::Receiver<Event> {
        self.sender.subscribe()
    }
}

#[async_trait]
impl EventPort for EventBus {
    async fn emit(&self, event: Event) -> Result<EventId> {
        let id = event.id;
        // Persist first so the event is durable even if no subscribers exist.
        self.store.append(&event).await?;
        // Broadcast to live subscribers. Ignore send errors (no active receivers).
        let _ = self.sender.send(event);
        Ok(id)
    }

    async fn get(&self, id: &EventId) -> Result<Option<Event>> {
        self.store.get(id).await
    }

    async fn query(&self, filter: EventFilter) -> Result<Vec<Event>> {
        self.store.query(filter).await
    }
}

// ════════════════════════════════════════════════════════════════════
//  Tests
// ════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::Utc;
    use std::sync::Mutex;

    /// Minimal in-memory EventStore for testing.
    struct MemStore {
        events: Mutex<Vec<Event>>,
    }

    impl MemStore {
        fn new() -> Self {
            Self {
                events: Mutex::new(Vec::new()),
            }
        }
    }

    #[async_trait]
    impl EventStore for MemStore {
        async fn append(&self, event: &Event) -> Result<()> {
            self.events.lock().unwrap().push(event.clone());
            Ok(())
        }

        async fn get(&self, id: &EventId) -> Result<Option<Event>> {
            let events = self.events.lock().unwrap();
            Ok(events.iter().find(|e| e.id == *id).cloned())
        }

        async fn query(&self, filter: EventFilter) -> Result<Vec<Event>> {
            let events = self.events.lock().unwrap();
            let mut result: Vec<Event> = events
                .iter()
                .filter(|e| {
                    filter.kind.as_ref().is_none_or(|k| &e.kind == k)
                        && filter.source.as_ref().is_none_or(|s| &e.source == s)
                        && filter
                            .session_id
                            .is_none_or(|sid| e.session_id == Some(sid))
                        && filter.since.is_none_or(|t| e.created_at >= t)
                })
                .cloned()
                .collect();
            if let Some(limit) = filter.limit {
                result.truncate(limit as usize);
            }
            Ok(result)
        }
    }

    fn make_event(kind: &str) -> Event {
        Event {
            id: EventId::new(),
            session_id: None,
            run_id: None,
            source: "forge".into(),
            kind: kind.into(),
            payload: serde_json::json!({}),
            created_at: Utc::now(),
            metadata: serde_json::json!({}),
        }
    }

    #[tokio::test]
    async fn emit_persists_and_returns_id() {
        let store = Arc::new(MemStore::new());
        let bus = EventBus::new(store.clone());

        let event = make_event("test.created");
        let id = bus.emit(event).await.unwrap();

        let fetched = bus.get(&id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().kind, "test.created");
    }

    #[tokio::test]
    async fn subscriber_receives_events() {
        let store = Arc::new(MemStore::new());
        let bus = EventBus::new(store);

        let mut rx = bus.subscribe();
        let event = make_event("test.broadcast");
        bus.emit(event).await.unwrap();

        let received = rx.recv().await.unwrap();
        assert_eq!(received.kind, "test.broadcast");
    }

    #[tokio::test]
    async fn query_filters_by_kind() {
        let store = Arc::new(MemStore::new());
        let bus = EventBus::new(store);

        bus.emit(make_event("a.one")).await.unwrap();
        bus.emit(make_event("b.two")).await.unwrap();
        bus.emit(make_event("a.one")).await.unwrap();

        let filter = EventFilter {
            kind: Some("a.one".into()),
            ..Default::default()
        };
        let results = bus.query(filter).await.unwrap();
        assert_eq!(results.len(), 2);
    }
}
