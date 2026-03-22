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
#[allow(unused_imports)] // TODO: wire --mcp flag dispatch
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

    // 5. Load user profile
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

    // 6. Parse CLI
    let cli = Cli::parse();

    // 7. Dispatch
    if cli.command.is_some() {
        // Subcommand present -> CLI handler
        rusvel_cli::run(cli, forge.clone(), sessions.clone()).await?;
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
