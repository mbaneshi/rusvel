//! Code → content → approve → publish (mock platform), no real LLM or external HTTP.

use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::http::{Request, StatusCode};
use chrono::Utc;
use code_engine::CodeEngine;
use content_engine::{ContentEngine, MockPlatformAdapter};
use forge_engine::ForgeEngine;
use rusvel_api::{AppState, build_router};
use rusvel_config::TomlConfig;
use rusvel_agent::AgentRuntime;
use rusvel_core::domain::{
    AgentOutput, AgentStatus, ApprovalStatus, Content, ContentItem, ContentKind, FinishReason,
    LlmRequest, LlmResponse, LlmUsage, ModelRef, Session, SessionConfig, SessionKind,
    ToolDefinition, ToolResult,
};
use rusvel_core::error::Result;
use rusvel_core::id::{RunId, SessionId};
use rusvel_core::ports::{
    AgentPort, ConfigPort, EventPort, JobPort, LlmPort, MemoryPort, SessionPort, StoragePort,
    ToolPort,
};
use rusvel_core::registry::DepartmentRegistry;
use rusvel_db::Database;
use rusvel_event::EventBus;
use rusvel_memory::MemoryStore;
use serde_json::{Value, json};
use tower::ServiceExt;

struct SessionAdapter(Arc<dyn StoragePort>);

#[async_trait]
impl SessionPort for SessionAdapter {
    async fn create(&self, session: Session) -> Result<SessionId> {
        let id = session.id;
        self.0.sessions().put_session(&session).await?;
        Ok(id)
    }
    async fn load(&self, id: &SessionId) -> Result<Session> {
        self.0.sessions().get_session(id).await?.ok_or_else(|| {
            rusvel_core::error::RusvelError::NotFound {
                kind: "session".into(),
                id: id.to_string(),
            }
        })
    }
    async fn save(&self, session: &Session) -> Result<()> {
        self.0.sessions().put_session(session).await
    }
    async fn list(&self) -> Result<Vec<rusvel_core::domain::SessionSummary>> {
        self.0.sessions().list_sessions().await
    }
}

struct StubLlm;

#[async_trait]
impl LlmPort for StubLlm {
    async fn generate(&self, _: LlmRequest) -> Result<LlmResponse> {
        Ok(LlmResponse {
            content: Content::text("stub"),
            finish_reason: FinishReason::Stop,
            usage: LlmUsage::default(),
            metadata: json!({}),
        })
    }
    async fn embed(&self, _: &ModelRef, _: &str) -> Result<Vec<f32>> {
        Ok(vec![])
    }
    async fn list_models(&self) -> Result<Vec<ModelRef>> {
        Ok(vec![])
    }
}

struct StubTool;

#[async_trait]
impl ToolPort for StubTool {
    async fn register(&self, _: ToolDefinition) -> Result<()> {
        Ok(())
    }
    async fn call(&self, _: &str, _: Value) -> Result<ToolResult> {
        Ok(ToolResult {
            success: true,
            output: Content::text("ok"),
            metadata: json!({}),
        })
    }
    fn list(&self) -> Vec<ToolDefinition> {
        vec![]
    }
    fn search(&self, _: &str, _: usize) -> Vec<ToolDefinition> {
        vec![]
    }
    fn schema(&self, _: &str) -> Option<Value> {
        None
    }
}

struct StaticDraftAgent;

#[async_trait]
impl AgentPort for StaticDraftAgent {
    async fn create(&self, _: rusvel_core::domain::AgentConfig) -> Result<RunId> {
        Ok(RunId::new())
    }
    async fn run(&self, _: &RunId, _: Content) -> Result<AgentOutput> {
        Ok(AgentOutput {
            run_id: RunId::new(),
            content: Content::text("# Generated Title\n\nBody from mock agent."),
            tool_calls: 0,
            usage: rusvel_core::domain::LlmUsage::default(),
            cost_estimate: 0.0,
            metadata: json!({}),
        })
    }
    async fn stop(&self, _: &RunId) -> Result<()> {
        Ok(())
    }
    async fn status(&self, _: &RunId) -> Result<AgentStatus> {
        Ok(AgentStatus::Idle)
    }
}

async fn json_request(
    router: &mut Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Vec<u8>) {
    let builder = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let req = if let Some(b) = body {
        builder
            .body(Body::from(serde_json::to_vec(&b).unwrap()))
            .unwrap()
    } else {
        builder.body(Body::empty()).unwrap()
    };
    let res = router.oneshot(req).await.unwrap();
    let status = res.status();
    let bytes = to_bytes(res.into_body(), usize::MAX).await.unwrap();
    (status, bytes.to_vec())
}

async fn test_router() -> (Router, Arc<MockPlatformAdapter>, SessionId) {
    let base = std::env::temp_dir().join(format!("rusvel-api-ict-{}", uuid::Uuid::now_v7()));
    std::fs::create_dir_all(&base).expect("temp dir");
    let db_path = base.join("rusvel.db");
    let db: Arc<Database> = Arc::new(Database::open(&db_path).expect("db"));
    let storage: Arc<dyn StoragePort> = db.clone();
    let config: Arc<dyn ConfigPort> = Arc::new(
        TomlConfig::load(base.join("config.toml")).expect("config"),
    );
    let events: Arc<dyn EventPort> = Arc::new(EventBus::new(
        db.clone() as Arc<dyn rusvel_core::ports::EventStore>,
    ));
    let memory: Arc<dyn MemoryPort> = Arc::new(
        MemoryStore::open(base.join("memory.db").to_str().unwrap()).expect("memory"),
    );
    let jobs: Arc<dyn JobPort> = db.clone() as Arc<dyn JobPort>;
    let sessions: Arc<dyn SessionPort> = Arc::new(SessionAdapter(storage.clone()));
    let agent: Arc<dyn AgentPort> = Arc::new(StaticDraftAgent);
    let tools: Arc<dyn ToolPort> = Arc::new(StubTool);
    let agent_runtime = Arc::new(AgentRuntime::new(
        Arc::new(StubLlm),
        tools.clone(),
        memory.clone(),
    ));

    let forge = Arc::new(ForgeEngine::new(
        agent.clone(),
        events.clone(),
        memory.clone(),
        storage.clone(),
        jobs.clone(),
        sessions.clone(),
        config.clone(),
    ));

    let code_engine = Arc::new(CodeEngine::new(storage.clone(), events.clone()));
    let content_engine = Arc::new(ContentEngine::new(
        storage.clone(),
        events.clone(),
        agent,
        jobs.clone(),
    ));
    let mock_tw = Arc::new(MockPlatformAdapter::new(rusvel_core::domain::Platform::Twitter));
    content_engine.register_platform(mock_tw.clone());

    let now = Utc::now();
    let session_id = SessionId::new();
    let session = Session {
        id: session_id,
        name: "test".into(),
        kind: SessionKind::General,
        tags: vec![],
        config: SessionConfig::default(),
        created_at: now,
        updated_at: now,
        metadata: json!({}),
    };
    sessions.create(session).await.expect("session");

    let state = AppState {
        forge,
        code_engine: Some(code_engine),
        content_engine: Some(content_engine),
        harvest_engine: None,
        flow_engine: None,
        sessions,
        events,
        jobs,
        database: db.clone(),
        storage,
        profile: None,
        registry: DepartmentRegistry::load(PathBuf::from("/__no_such__/departments.toml").as_path()),
        embedding: None,
        vector_store: None,
        memory: memory.clone(),
        deploy: None,
        agent_runtime,
        tools,
    };

    let router = build_router(state);
    (router, mock_tw, session_id)
}

#[tokio::test]
async fn post_from_code_creates_items_for_kinds() {
    let (mut router, _, session_id) = test_router().await;
    let code_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../code-engine/src");

    let (status, body) = json_request(
        &mut router,
        "POST",
        "/api/dept/content/from-code",
        Some(json!({
            "session_id": session_id.to_string(),
            "path": code_src.to_string_lossy(),
            "kinds": ["LinkedInPost", "Thread"]
        })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let items: Vec<ContentItem> = serde_json::from_slice(&body).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].kind, ContentKind::LinkedInPost);
    assert_eq!(items[1].kind, ContentKind::Thread);
}

#[tokio::test]
async fn patch_approve_sets_approval_approved() {
    let (mut router, _, session_id) = test_router().await;
    let code_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../code-engine/src");

    let (_st, b) = json_request(
        &mut router,
        "POST",
        "/api/dept/content/from-code",
        Some(json!({
            "session_id": session_id.to_string(),
            "path": code_src.to_string_lossy(),
            "kinds": ["Blog"]
        })),
    )
    .await;
    let items: Vec<ContentItem> = serde_json::from_slice(&b).unwrap();
    let id = items[0].id.to_string();

    let (status, body) = json_request(
        &mut router,
        "PATCH",
        &format!("/api/dept/content/{id}/approve"),
        None,
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    let item: ContentItem = serde_json::from_slice(&body).unwrap();
    assert_eq!(item.approval, ApprovalStatus::Approved);
}

#[tokio::test]
async fn publish_uses_mock_platform_after_approve() {
    let (mut router, mock_tw, session_id) = test_router().await;
    let code_src: PathBuf = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../code-engine/src");

    let (_st, b) = json_request(
        &mut router,
        "POST",
        "/api/dept/content/from-code",
        Some(json!({
            "session_id": session_id.to_string(),
            "path": code_src.to_string_lossy(),
            "kinds": ["Tweet"]
        })),
    )
    .await;
    let items: Vec<ContentItem> = serde_json::from_slice(&b).unwrap();
    let id = items[0].id.to_string();

    let (st, _) = json_request(
        &mut router,
        "PATCH",
        &format!("/api/dept/content/{id}/approve"),
        None,
    )
    .await;
    assert_eq!(st, StatusCode::OK);

    let (pub_st, _) = json_request(
        &mut router,
        "POST",
        "/api/dept/content/publish",
        Some(json!({
            "session_id": session_id.to_string(),
            "content_id": id,
            "platform": "Twitter"
        })),
    )
    .await;
    assert_eq!(pub_st, StatusCode::OK);
    assert_eq!(mock_tw.published_items().len(), 1);
}

#[tokio::test]
async fn get_db_tables_lists_user_tables() {
    let (mut router, _, _) = test_router().await;
    let (status, body) = json_request(&mut router, "GET", "/api/db/tables", None).await;
    assert_eq!(status, StatusCode::OK);
    let tables: Vec<Value> = serde_json::from_slice(&body).expect("json array");
    let names: Vec<&str> = tables
        .iter()
        .filter_map(|t| t.get("name").and_then(|n| n.as_str()))
        .collect();
    assert!(
        names.iter().any(|n| *n == "sessions" || *n == "events"),
        "expected core tables, got {names:?}"
    );
}
