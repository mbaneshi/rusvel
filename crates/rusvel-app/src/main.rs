//! Composition root for RUSVEL.
//!
//! Wires all adapter crates into engine ports and dispatches to the
//! appropriate surface: CLI (subcommand), MCP (--mcp flag), or API
//! (default — starts the web server).

use std::io::IsTerminal;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use clap::Parser;
use tracing_subscriber::EnvFilter;

use forge_engine::ForgeEngine;
use rusvel_agent::AgentRuntime;
use rusvel_api::AppState;
use rusvel_core::registry::DepartmentRegistry;
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
//  Embedded frontend assets (built from frontend/build/)
// ════════════════════════════════════════════════════════════════════

#[derive(rust_embed::Embed)]
#[folder = "../../frontend/build/"]
#[prefix = ""]
struct FrontendAssets;

/// Extract embedded frontend assets to a temporary directory.
/// Returns the path to the directory containing the extracted files.
fn extract_embedded_frontend() -> Option<PathBuf> {
    let dir = std::env::temp_dir().join("rusvel-frontend");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        tracing::warn!("Failed to create temp frontend dir: {e}");
        return None;
    }

    let mut count = 0usize;
    for path in FrontendAssets::iter() {
        if let Some(content) = FrontendAssets::get(&path) {
            let out = dir.join(path.as_ref());
            if let Some(parent) = out.parent() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    tracing::warn!("Failed to create dir {}: {e}", parent.display());
                    continue;
                }
            }
            if let Err(e) = std::fs::write(&out, content.data.as_ref()) {
                tracing::warn!("Failed to write {}: {e}", out.display());
                continue;
            }
            count += 1;
        }
    }

    if count > 0 && dir.join("index.html").exists() {
        tracing::info!("Extracted {count} embedded frontend files to {}", dir.display());
        Some(dir)
    } else {
        tracing::debug!("No embedded frontend assets found");
        None
    }
}

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
//  First-run wizard — interactive onboarding via cliclack
// ════════════════════════════════════════════════════════════════════

async fn first_run_wizard(
    data_dir: &std::path::Path,
    sessions: &Arc<dyn SessionPort>,
) -> Result<Option<UserProfile>> {
    use cliclack::{intro, input, confirm, select, outro_note};

    intro("Welcome to RUSVEL — Your AI-Powered Virtual Agency")?;

    // Detect LLM availability
    let ollama_ok = reqwest::Client::new()
        .get("http://localhost:11434/api/tags")
        .timeout(std::time::Duration::from_secs(2))
        .send()
        .await
        .is_ok();
    if ollama_ok {
        cliclack::log::success("Ollama detected at localhost:11434")?;
    } else {
        cliclack::log::warning("Ollama not running — will use Claude CLI as fallback")?;
    }

    // User identity
    let name: String = input("What's your name?")
        .placeholder("e.g. Mehdi")
        .required(true)
        .interact()?;

    let role: String = input("What's your role?")
        .placeholder("e.g. Solo founder, Developer")
        .default_input("Solo founder")
        .interact()?;

    // Create profile.toml
    let profile_content = format!(
        r#"[identity]
name = "{name}"
role = "{role}"
tagline = ""

[skills]
primary = []
secondary = []

[products]
names = []
description = ""

[mission]
vision = ""
values = []

[preferences]
style = "direct"
"#
    );
    std::fs::write(data_dir.join("profile.toml"), &profile_content)?;
    cliclack::log::success(format!("Profile saved for {name}"))?;

    // Create first session
    let create_session: bool = confirm("Create your first session?")
        .initial_value(true)
        .interact()?;

    if create_session {
        let session_name: String = input("Session name")
            .placeholder("e.g. my-startup")
            .default_input("default")
            .interact()?;

        let kind_idx: &str = select("Session type")
            .item("Project", "Project", "For building a product or app")
            .item("Lead", "Lead", "For tracking a freelance opportunity")
            .item("ContentCampaign", "Content Campaign", "For a content initiative")
            .item("General", "General", "For everything else")
            .interact()?;

        let kind = match kind_idx {
            "Project" => SessionKind::Project,
            "Lead" => SessionKind::Lead,
            "ContentCampaign" => SessionKind::ContentCampaign,
            _ => SessionKind::General,
        };

        let now = Utc::now();
        let session = Session {
            id: SessionId::new(),
            name: session_name.clone(),
            kind,
            tags: vec![],
            config: SessionConfig::default(),
            created_at: now,
            updated_at: now,
            metadata: serde_json::json!({}),
        };
        let id = sessions.create(session).await?;

        // Save as active session
        let active_path = data_dir.join("active_session");
        std::fs::write(&active_path, id.to_string())?;
        cliclack::log::success(format!("Session \"{session_name}\" created"))?;
    }

    outro_note(
        "Ready!",
        "Next steps:\n  → Open http://localhost:3000 in your browser\n  → Or run: rusvel forge mission today",
    )?;

    // Load and return the profile we just created
    Ok(UserProfile::load(&data_dir.join("profile.toml")).ok())
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
    let jobs_for_worker = jobs.clone();
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

    // 6. Spawn background job queue worker
    let _job_worker = tokio::spawn(async move {
        let job_port = jobs_for_worker;
        loop {
            match job_port.dequeue(&[]).await {
                Ok(Some(job)) => {
                    // Skip jobs awaiting human approval (ADR-008)
                    if job.status == JobStatus::AwaitingApproval {
                        tracing::debug!(job_id = %job.id, "Skipping job awaiting approval");
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        continue;
                    }

                    let job_id = job.id;
                    tracing::info!(job_id = %job_id, kind = ?job.kind, "Processing job");

                    let result = match job.kind {
                        JobKind::ContentPublish => {
                            tracing::info!(job_id = %job_id, "Would publish content (placeholder)");
                            Ok(JobResult {
                                output: serde_json::json!({"action": "content_publish", "status": "placeholder"}),
                                metadata: serde_json::json!({}),
                            })
                        }
                        JobKind::HarvestScan => {
                            tracing::info!(job_id = %job_id, "Would scan opportunities (placeholder)");
                            Ok(JobResult {
                                output: serde_json::json!({"action": "harvest_scan", "status": "placeholder"}),
                                metadata: serde_json::json!({}),
                            })
                        }
                        JobKind::OutreachSend => {
                            tracing::info!(job_id = %job_id, "Would send outreach (placeholder)");
                            Ok(JobResult {
                                output: serde_json::json!({"action": "outreach_send", "status": "placeholder"}),
                                metadata: serde_json::json!({}),
                            })
                        }
                        JobKind::CodeAnalyze => {
                            tracing::info!(job_id = %job_id, "Would analyze code (placeholder)");
                            Ok(JobResult {
                                output: serde_json::json!({"action": "code_analyze", "status": "placeholder"}),
                                metadata: serde_json::json!({}),
                            })
                        }
                        _ => {
                            tracing::warn!(job_id = %job_id, kind = ?job.kind, "Unknown job kind");
                            Ok(JobResult {
                                output: serde_json::json!({"action": "unknown", "kind": format!("{:?}", job.kind)}),
                                metadata: serde_json::json!({}),
                            })
                        }
                    };

                    match result {
                        Ok(job_result) => {
                            if let Err(e) = job_port.complete(&job_id, job_result).await {
                                tracing::error!(job_id = %job_id, error = %e, "Failed to mark job complete");
                            }
                        }
                        Err(error) => {
                            if let Err(e) = job_port.fail(&job_id, error).await {
                                tracing::error!(job_id = %job_id, error = %e, "Failed to mark job failed");
                            }
                        }
                    }
                }
                Ok(None) => {
                    // No jobs available, sleep before polling again
                }
                Err(e) => {
                    tracing::error!(error = %e, "Error dequeuing job");
                }
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    });
    tracing::info!("Job queue worker started");

    // 7. Parse CLI early so we can skip wizard for subcommands
    let cli = Cli::parse();

    // 8. Load user profile — run first-run wizard if no profile exists
    let profile_path = data_dir.join("profile.toml");
    let profile = if profile_path.exists() {
        match UserProfile::load(&profile_path) {
            Ok(p) => {
                tracing::info!("Loaded profile for {}", p.identity.name);
                Some(p)
            }
            Err(e) => {
                tracing::warn!("No profile loaded ({}): {e}", profile_path.display());
                None
            }
        }
    } else if cli.command.is_none() && !cli.mcp && std::io::stdin().is_terminal() {
        // First run + interactive terminal + no subcommand → run wizard
        match first_run_wizard(&data_dir, &sessions).await {
            Ok(p) => p,
            Err(e) => {
                tracing::warn!("Wizard skipped: {e}");
                None
            }
        }
    } else {
        None
    };

    // 9. Dispatch
    let storage_port: Arc<dyn StoragePort> = db.clone() as Arc<dyn StoragePort>;
    if cli.command.is_some() {
        // Subcommand present -> CLI handler (includes `shell` command)
        rusvel_cli::run(cli, forge.clone(), sessions.clone(), storage_port).await?;
    } else if cli.tui {
        // --tui flag: launch TUI dashboard
        tracing::info!("Launching TUI dashboard...");
        let session_name = crate::rusvel_dir()
            .join("active_session")
            .exists()
            .then(|| {
                std::fs::read_to_string(rusvel_dir().join("active_session"))
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            })
            .unwrap_or_else(|| "(no session)".into());

        // Load data from storage for the dashboard
        let objects = storage_port.objects();
        let filter = rusvel_core::domain::ObjectFilter::default();

        let goals_json = objects.list("goals", filter.clone()).await.unwrap_or_default();
        let goals: Vec<rusvel_core::domain::Goal> = goals_json
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        let tasks_json = objects.list("tasks", filter.clone()).await.unwrap_or_default();
        let tasks: Vec<rusvel_core::domain::Task> = tasks_json
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        let opps_json = objects.list("opportunities", filter.clone()).await.unwrap_or_default();
        let opportunities: Vec<rusvel_core::domain::Opportunity> = opps_json
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        let events_json = objects.list("events", rusvel_core::domain::ObjectFilter {
            limit: Some(50), ..Default::default()
        }).await.unwrap_or_default();
        let recent_events: Vec<rusvel_core::domain::Event> = events_json
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect();

        let tui_data = rusvel_tui::TuiData {
            session_name,
            goals,
            tasks,
            opportunities,
            recent_events,
        };
        rusvel_tui::run_tui(tui_data).await?;
    } else if cli.mcp {
        // --mcp flag: start MCP server over stdio (JSON-RPC)
        tracing::info!("Starting MCP server on stdio...");
        let mcp = Arc::new(RusvelMcp::new(forge.clone(), sessions.clone()));
        rusvel_mcp::run_stdio(mcp).await.map_err(|e| anyhow::anyhow!("{e}"))?;
    } else {
        // Default: start the API web server with graceful shutdown
        // Load department registry
        let registry = DepartmentRegistry::load(&data_dir.join("departments.toml"));
        tracing::info!("Loaded {} departments from registry", registry.departments.len());

        let state = AppState {
            forge: forge.clone(),
            sessions: sessions.clone(),
            events: events.clone(),
            storage: db.clone() as Arc<dyn StoragePort>,
            profile,
            registry,
        };

        // Look for frontend build in known locations (filesystem first)
        let frontend_dir = [
            PathBuf::from("frontend/build"),                       // dev: run from repo root
            data_dir.join("frontend"),                             // installed: ~/.rusvel/frontend
            std::env::current_exe()                                // next to binary
                .unwrap_or_default()
                .parent()
                .unwrap_or(std::path::Path::new("."))
                .join("frontend"),
        ]
        .into_iter()
        .find(|p| p.join("index.html").exists())
        // Fallback: extract embedded assets (rust-embed) to temp dir
        .or_else(|| {
            tracing::info!("No frontend build on disk, trying embedded assets...");
            extract_embedded_frontend()
        });

        if let Some(ref dir) = frontend_dir {
            tracing::info!("Serving frontend from {}", dir.display());
        } else {
            tracing::warn!("No frontend found — UI will not be available");
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
