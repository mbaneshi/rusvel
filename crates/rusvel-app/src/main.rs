//! Composition root for RUSVEL.
//!
//! Wires all adapter crates into engine ports and dispatches to the
//! appropriate surface: CLI (subcommand), MCP (--mcp flag), or API
//! (default — starts the web server).

use std::net::SocketAddr;
use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use forge_engine::ForgeEngine;
use rusvel_agent::AgentRuntime;
use rusvel_api::AppState;
use rusvel_auth::InMemoryAuthAdapter;
use rusvel_cli::Cli;
use rusvel_config::TomlConfig;
use rusvel_core::domain::*;
use rusvel_core::id::SessionId;
use rusvel_core::ports::{SessionPort, StoragePort};
use rusvel_db::Database;
use rusvel_event::EventBus;
use rusvel_jobs::JobQueue;
use rusvel_llm::ClaudeCliProvider;
use rusvel_core::id::AgentProfileId;
use rusvel_mcp::RusvelMcp;
use rusvel_memory::MemoryStore;
use rusvel_tool::ToolRegistry;

// ════════════════════════════════════════════════════════════════════
//  SessionPort adapter — bridges SessionStore -> SessionPort
// ════════════════════════════════════════════════════════════════════

struct SessionAdapter(Arc<dyn StoragePort>);

#[async_trait]
impl SessionPort for SessionAdapter {
    async fn create(&self, session: Session) -> rusvel_core::error::Result<SessionId> {
        let id = session.id;
        self.0.sessions().put_session(&session).await?;
        Ok(id)
    }
    async fn load(&self, id: &SessionId) -> rusvel_core::error::Result<Session> {
        self.0.sessions().get_session(id).await?.ok_or_else(|| {
            rusvel_core::error::RusvelError::NotFound {
                kind: "session".into(),
                id: id.to_string(),
            }
        })
    }
    async fn save(&self, session: &Session) -> rusvel_core::error::Result<()> {
        self.0.sessions().put_session(session).await
    }
    async fn list(&self) -> rusvel_core::error::Result<Vec<SessionSummary>> {
        self.0.sessions().list_sessions().await
    }
}

// ════════════════════════════════════════════════════════════════════
//  Data directory
// ════════════════════════════════════════════════════════════════════

fn rusvel_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".rusvel")
}

// ════════════════════════════════════════════════════════════════════
//  Seed data — populate ObjectStore with defaults on first run
// ════════════════════════════════════════════════════════════════════

async fn seed_defaults(storage: &Arc<dyn StoragePort>) -> Result<()> {
    let objects = storage.objects();
    let empty_filter = ObjectFilter::default();

    // ── Seed agents ──────────────────────────────────────────────
    let existing_agents = objects.list("agents", empty_filter.clone()).await?;
    if existing_agents.is_empty() {
        tracing::info!("Seeding default agents...");
        let agents = vec![
            AgentProfile {
                id: AgentProfileId::new(),
                name: "rust-engine".into(),
                role: "Rust backend engineer".into(),
                instructions: "Build RUSVEL engine crates. Follow hexagonal architecture.".into(),
                default_model: ModelRef { provider: ModelProvider::Claude, model: "opus".into() },
                allowed_tools: vec!["read_file".into(), "write_file".into(), "bash".into()],
                capabilities: vec![Capability::CodeAnalysis, Capability::ToolUse],
                budget_limit: None,
                metadata: serde_json::json!({"engine": "code", "department": "Code"}),
            },
            AgentProfile {
                id: AgentProfileId::new(),
                name: "svelte-ui".into(),
                role: "Frontend engineer".into(),
                instructions: "Build SvelteKit 5 pages using RUSVEL design system.".into(),
                default_model: ModelRef { provider: ModelProvider::Claude, model: "sonnet".into() },
                allowed_tools: vec!["read_file".into(), "write_file".into(), "bash".into()],
                capabilities: vec![Capability::CodeAnalysis, Capability::ToolUse],
                budget_limit: None,
                metadata: serde_json::json!({"engine": "code", "department": "Code"}),
            },
            AgentProfile {
                id: AgentProfileId::new(),
                name: "test-writer".into(),
                role: "QA and test engineer".into(),
                instructions: "Write tests for RUSVEL crates and frontend.".into(),
                default_model: ModelRef { provider: ModelProvider::Claude, model: "sonnet".into() },
                allowed_tools: vec!["read_file".into(), "write_file".into(), "bash".into()],
                capabilities: vec![Capability::CodeAnalysis, Capability::ToolUse],
                budget_limit: None,
                metadata: serde_json::json!({"engine": "code", "department": "Code"}),
            },
            AgentProfile {
                id: AgentProfileId::new(),
                name: "content-writer".into(),
                role: "Content strategist and writer".into(),
                instructions: "Draft blog posts, articles, and long-form content.".into(),
                default_model: ModelRef { provider: ModelProvider::Claude, model: "sonnet".into() },
                allowed_tools: vec!["read_file".into(), "web_search".into()],
                capabilities: vec![Capability::ContentCreation],
                budget_limit: None,
                metadata: serde_json::json!({"engine": "content", "department": "Content"}),
            },
            AgentProfile {
                id: AgentProfileId::new(),
                name: "proposal-writer".into(),
                role: "Proposal and business development specialist".into(),
                instructions: "Draft compelling proposals tailored to each gig.".into(),
                default_model: ModelRef { provider: ModelProvider::Claude, model: "opus".into() },
                allowed_tools: vec!["read_file".into(), "web_search".into()],
                capabilities: vec![Capability::ContentCreation, Capability::OpportunityDiscovery],
                budget_limit: None,
                metadata: serde_json::json!({"engine": "harvest", "department": "Harvest"}),
            },
        ];
        for agent in &agents {
            let val = serde_json::to_value(agent)?;
            objects.put("agents", &agent.id.to_string(), val).await?;
        }
        tracing::info!("Seeded {} default agents", agents.len());
    }

    // ── Seed skills ──────────────────────────────────────────────
    let existing_skills = objects.list("skills", empty_filter.clone()).await?;
    if existing_skills.is_empty() {
        tracing::info!("Seeding default skills...");
        let skills = vec![
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Code Review",
                "description": "Analyze code for bugs, patterns, and improvements",
                "prompt_template": "Review the following code. Identify bugs, anti-patterns, and suggest improvements:\n\n{input}",
                "metadata": {"engine": "code"}
            }),
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Blog Draft",
                "description": "Draft a blog post from a topic and key points",
                "prompt_template": "Write a blog post about: {topic}\n\nKey points to cover:\n{points}\n\nTarget audience: {audience}",
                "metadata": {"engine": "content"}
            }),
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Proposal Draft",
                "description": "Draft a proposal for a freelance opportunity",
                "prompt_template": "Draft a compelling proposal for this opportunity:\n\nTitle: {title}\nDescription: {description}\nBudget: {budget}\n\nHighlight relevant experience and propose a clear plan.",
                "metadata": {"engine": "harvest"}
            }),
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Test Generator",
                "description": "Generate tests for a given Rust module or SvelteKit component",
                "prompt_template": "Generate comprehensive tests for:\n\n{input}\n\nInclude unit tests, edge cases, and integration tests where appropriate.",
                "metadata": {"engine": "code"}
            }),
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Daily Standup",
                "description": "Summarize progress and plan for the day",
                "prompt_template": "Based on recent activity, generate a standup summary:\n- What was accomplished yesterday\n- What is planned for today\n- Any blockers",
                "metadata": {"engine": "forge"}
            }),
        ];
        for skill in &skills {
            let id = skill["id"].as_str().unwrap();
            objects.put("skills", id, skill.clone()).await?;
        }
        tracing::info!("Seeded {} default skills", skills.len());
    }

    // ── Seed rules ───────────────────────────────────────────────
    let existing_rules = objects.list("rules", empty_filter.clone()).await?;
    if existing_rules.is_empty() {
        tracing::info!("Seeding default rules...");
        let rules = vec![
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Hexagonal Architecture",
                "content": "Engines must never import adapter crates. They depend only on rusvel-core port traits. All domain logic flows through ports.",
                "enabled": true,
                "metadata": {"engine": "code"}
            }),
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Human Approval Gate",
                "content": "All content publishing and outreach sending requires human approval before execution. Never auto-publish or auto-send without explicit approval.",
                "enabled": true,
                "metadata": {}
            }),
            serde_json::json!({
                "id": uuid::Uuid::now_v7().to_string(),
                "name": "Crate Size Limit",
                "content": "Each crate must stay under 2000 lines. If a crate exceeds this, refactor it into smaller, focused crates with single responsibility.",
                "enabled": true,
                "metadata": {"engine": "code"}
            }),
        ];
        for rule in &rules {
            let id = rule["id"].as_str().unwrap();
            objects.put("rules", id, rule.clone()).await?;
        }
        tracing::info!("Seeded {} default rules", rules.len());
    }

    Ok(())
}

// ════════════════════════════════════════════════════════════════════
//  Main
// ════════════════════════════════════════════════════════════════════

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    // 2. Data directory
    let data_dir = rusvel_dir();
    std::fs::create_dir_all(&data_dir)?;

    // 3. Concrete adapters
    let db: Arc<Database> = Arc::new(Database::open(data_dir.join("rusvel.db"))?);
    let config = Arc::new(TomlConfig::load(data_dir.join("config.toml"))?);
    let llm: Arc<dyn rusvel_core::ports::LlmPort> = Arc::new(ClaudeCliProvider::max_subscription());
    let events = Arc::new(EventBus::new(db.clone() as Arc<dyn rusvel_core::ports::EventStore>));
    let memory: Arc<dyn rusvel_core::ports::MemoryPort> = Arc::new(
        MemoryStore::open(data_dir.join("memory.db").to_str().unwrap_or("memory.db"))?,
    );
    let tools: Arc<dyn rusvel_core::ports::ToolPort> = Arc::new(ToolRegistry::new());
    let jobs: Arc<dyn rusvel_core::ports::JobPort> = Arc::new(JobQueue::new());
    let _auth = Arc::new(InMemoryAuthAdapter::from_env());
    let sessions: Arc<dyn SessionPort> =
        Arc::new(SessionAdapter(db.clone() as Arc<dyn StoragePort>));
    let agent: Arc<dyn rusvel_core::ports::AgentPort> =
        Arc::new(AgentRuntime::new(llm.clone(), tools.clone(), memory.clone()));

    // 4. Build Forge Engine
    let forge = Arc::new(ForgeEngine::new(
        agent,
        events.clone(),
        memory,
        db.clone() as Arc<dyn StoragePort>,
        jobs,
        sessions.clone(),
        config,
    ));

    // 5. Seed default data (agents, skills, rules) on first run
    let storage_ref: Arc<dyn StoragePort> = db.clone() as Arc<dyn StoragePort>;
    seed_defaults(&storage_ref).await?;

    // 6. Load user profile
    let profile_path = data_dir.join("profile.toml");
    let profile = match UserProfile::load(&profile_path) {
        Ok(p) => {
            tracing::info!("Loaded profile for {}", p.identity.name);
            Some(p)
        }
        Err(e) => {
            tracing::warn!("No profile loaded ({}): {e}", profile_path.display());
            None
        }
    };

    // 7. Parse CLI
    let cli = Cli::parse();

    // 8. Dispatch
    if cli.command.is_some() {
        // Subcommand present -> CLI handler
        rusvel_cli::run(cli, forge.clone(), sessions.clone()).await?;
    } else if cli.mcp {
        // --mcp flag: start MCP server over stdio (JSON-RPC)
        tracing::info!("Starting MCP server on stdio...");
        let mcp = Arc::new(RusvelMcp::new(forge.clone(), sessions.clone()));
        rusvel_mcp::run_stdio(mcp).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    } else {
        // Default: start the API web server with graceful shutdown
        let state = AppState {
            forge: forge.clone(),
            sessions: sessions.clone(),
            events: events.clone(),
            storage: db.clone() as Arc<dyn StoragePort>,
            profile,
        };

        // Look for frontend build in known locations
        let frontend_dir = [
            std::path::PathBuf::from("frontend/build"),           // dev: run from repo root
            data_dir.join("frontend"),                             // installed: ~/.rusvel/frontend
            std::env::current_exe()                                // next to binary
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("frontend"),
        ]
        .into_iter()
        .find(|p| p.join("index.html").exists());

        if let Some(ref dir) = frontend_dir {
            tracing::info!("Serving frontend from {}", dir.display());
        }
        let router = rusvel_api::build_router_with_frontend(state, frontend_dir);
        let addr: SocketAddr = "127.0.0.1:3000".parse()?;
        tracing::info!("RUSVEL starting on http://{addr}");

        let server = rusvel_api::start_server(router, addr);
        let shutdown = tokio::signal::ctrl_c();

        tokio::select! {
            result = server => result.map_err(|e| anyhow::anyhow!("{e}"))?,
            _ = shutdown => tracing::info!("Shutdown signal received, exiting"),
        }
    }

    Ok(())
}
