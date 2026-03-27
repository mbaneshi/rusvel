use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use code_engine::CodeEngine;
use rusvel_core::domain::{Event, EventFilter, ObjectFilter};
use rusvel_core::engine::Engine;
use rusvel_core::error::Result;
use rusvel_core::id::EventId;
use rusvel_core::ports::*;

struct FakeObjectStore;
#[async_trait]
impl ObjectStore for FakeObjectStore {
    async fn put(&self, _: &str, _: &str, _: serde_json::Value) -> Result<()> {
        Ok(())
    }
    async fn get(&self, _: &str, _: &str) -> Result<Option<serde_json::Value>> {
        Ok(None)
    }
    async fn delete(&self, _: &str, _: &str) -> Result<()> {
        Ok(())
    }
    async fn list(&self, _: &str, _: ObjectFilter) -> Result<Vec<serde_json::Value>> {
        Ok(vec![])
    }
}

struct FakeStorage;
impl StoragePort for FakeStorage {
    fn events(&self) -> &dyn EventStore {
        panic!("not used")
    }
    fn objects(&self) -> &dyn ObjectStore {
        &FakeObjectStore
    }
    fn sessions(&self) -> &dyn SessionStore {
        panic!("not used")
    }
    fn jobs(&self) -> &dyn JobStore {
        panic!("not used")
    }
    fn metrics(&self) -> &dyn MetricStore {
        panic!("not used")
    }
}

struct RecordingEvents(Mutex<Vec<Event>>);
#[async_trait]
impl EventPort for RecordingEvents {
    async fn emit(&self, event: Event) -> Result<EventId> {
        let id = event.id;
        self.0.lock().unwrap().push(event);
        Ok(id)
    }
    async fn get(&self, _: &EventId) -> Result<Option<Event>> {
        Ok(None)
    }
    async fn query(&self, _: EventFilter) -> Result<Vec<Event>> {
        Ok(vec![])
    }
}

fn make_engine(events: Arc<RecordingEvents>) -> CodeEngine {
    CodeEngine::new(Arc::new(FakeStorage), events)
}

#[tokio::test]
async fn analyze_discovers_rust_symbols_and_emits_event() {
    let events = Arc::new(RecordingEvents(Mutex::new(Vec::new())));
    let engine = make_engine(events.clone());

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("lib.rs"),
        "pub fn greet() {}\nfn helper() {}\npub fn farewell() {}",
    )
    .unwrap();

    let analysis = engine.analyze(dir.path()).await.unwrap();
    assert_eq!(analysis.symbols.len(), 3);
    assert_eq!(analysis.metrics.total_symbols, 3);
    assert_eq!(analysis.metrics.total_files, 1);

    let emitted = events.0.lock().unwrap();
    assert_eq!(emitted.len(), 1);
    assert_eq!(emitted[0].kind, "code.analyzed");
    assert_eq!(emitted[0].payload["total_symbols"], 3);
}

#[tokio::test]
async fn search_returns_relevant_results_after_analysis() {
    let events = Arc::new(RecordingEvents(Mutex::new(Vec::new())));
    let engine = make_engine(events);

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(
        dir.path().join("main.rs"),
        "fn compute_score() {}\nfn render_chart() {}",
    )
    .unwrap();

    engine.analyze(dir.path()).await.unwrap();

    let results = engine.search("compute_score", 10).unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].symbol_name, "compute_score");
}

#[tokio::test]
async fn search_before_analyze_returns_error() {
    let events = Arc::new(RecordingEvents(Mutex::new(Vec::new())));
    let engine = make_engine(events);

    let err = engine.search("anything", 5);
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(msg.contains("no analysis index"));
}

#[tokio::test]
async fn analyze_empty_directory_produces_zero_symbols() {
    let events = Arc::new(RecordingEvents(Mutex::new(Vec::new())));
    let engine = make_engine(events);

    let dir = tempfile::tempdir().unwrap();
    let analysis = engine.analyze(dir.path()).await.unwrap();
    assert_eq!(analysis.symbols.len(), 0);
    assert_eq!(analysis.metrics.total_files, 0);
}

#[tokio::test]
async fn engine_trait_kind_and_health() {
    let events = Arc::new(RecordingEvents(Mutex::new(Vec::new())));
    let engine = make_engine(events);

    assert_eq!(engine.kind(), "code");
    assert_eq!(engine.name(), "Code Engine");

    let health = engine.health().await.unwrap();
    assert!(health.healthy);
    assert_eq!(health.metadata["indexed"], false);

    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("a.rs"), "fn x() {}").unwrap();
    engine.analyze(dir.path()).await.unwrap();

    let health = engine.health().await.unwrap();
    assert_eq!(health.metadata["indexed"], true);
}
