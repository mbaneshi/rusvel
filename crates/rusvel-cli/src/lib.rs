//! CLI surface for RUSVEL using Clap 4.
//!
//! Exposes a [`Cli`] struct with subcommands and a [`run`] function that
//! dispatches to the appropriate engine method.

use std::sync::Arc;
use chrono::Utc;
use clap::{Parser, Subcommand, ValueEnum};
use uuid::Uuid;

use forge_engine::ForgeEngine;
use rusvel_core::domain::*;
use rusvel_core::error::{Result, RusvelError};
use rusvel_core::id::SessionId;
use rusvel_core::ports::SessionPort;

// ── Active session config ────────────────────────────────────────

fn rusvel_dir() -> std::path::PathBuf {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    std::path::PathBuf::from(home).join(".rusvel")
}

fn load_active_session() -> Result<SessionId> {
    let path = rusvel_dir().join("active_session");
    let raw = std::fs::read_to_string(&path).map_err(|_| {
        RusvelError::Config("No active session. Run `rusvel session create <name>` first.".into())
    })?;
    let uuid: Uuid = raw.trim().parse().map_err(|e| {
        RusvelError::Config(format!("Invalid session ID in {}: {e}", path.display()))
    })?;
    Ok(SessionId::from_uuid(uuid))
}

fn save_active_session(id: &SessionId) -> Result<()> {
    let dir = rusvel_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| RusvelError::Config(format!("Cannot create {}: {e}", dir.display())))?;
    std::fs::write(dir.join("active_session"), id.to_string())
        .map_err(|e| RusvelError::Config(format!("Cannot write active session: {e}")))?;
    Ok(())
}

// ── CLI structure (Clap derive) ──────────────────────────────────

/// RUSVEL — the AI-native solo-founder workbench.
#[derive(Parser, Debug)]
#[command(name = "rusvel", version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Manage workspace sessions.
    Session {
        #[command(subcommand)]
        action: SessionCmd,
    },
    /// Forge engine: mission planning and goals.
    Forge {
        #[command(subcommand)]
        action: ForgeCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum SessionCmd {
    /// Create a new session.
    Create { name: String },
    /// List all sessions.
    List,
    /// Switch the active session.
    Switch { id: String },
}

#[derive(Subcommand, Debug)]
pub enum ForgeCmd {
    /// Mission planning commands.
    Mission {
        #[command(subcommand)]
        action: MissionCmd,
    },
}

#[derive(Subcommand, Debug)]
pub enum MissionCmd {
    /// Generate a prioritized daily plan.
    Today,
    /// List all goals for the active session.
    Goals,
    /// Add a new goal.
    Goal {
        #[command(subcommand)]
        action: GoalCmd,
    },
    /// Generate a periodic review.
    Review {
        #[arg(long, default_value = "week")]
        period: TimeframeArg,
    },
}

#[derive(Subcommand, Debug)]
pub enum GoalCmd {
    /// Add a new goal to the active session.
    Add {
        title: String,
        #[arg(long, default_value = "")]
        description: String,
        #[arg(long, default_value = "month")]
        timeframe: TimeframeArg,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TimeframeArg { Day, Week, Month, Quarter }

impl From<TimeframeArg> for Timeframe {
    fn from(t: TimeframeArg) -> Self {
        match t {
            TimeframeArg::Day => Timeframe::Day,
            TimeframeArg::Week => Timeframe::Week,
            TimeframeArg::Month => Timeframe::Month,
            TimeframeArg::Quarter => Timeframe::Quarter,
        }
    }
}

// ── run — dispatch CLI commands ──────────────────────────────────

/// Execute the parsed CLI. The caller (`rusvel-app`) constructs the
/// `ForgeEngine` and `SessionPort` with real adapters.
pub async fn run(
    cli: Cli, engine: Arc<ForgeEngine>, session_port: Arc<dyn SessionPort>,
) -> Result<()> {
    match cli.command {
        None => {
            println!("RUSVEL — starting web server...");
            println!("(not yet implemented in rusvel-cli; use rusvel-app)");
            Ok(())
        }
        Some(Commands::Session { action }) => handle_session(action, session_port).await,
        Some(Commands::Forge { action }) => handle_forge(action, engine).await,
    }
}

async fn handle_session(cmd: SessionCmd, port: Arc<dyn SessionPort>) -> Result<()> {
    match cmd {
        SessionCmd::Create { name } => {
            let now = Utc::now();
            let session = Session {
                id: SessionId::new(), name: name.clone(), kind: SessionKind::General,
                tags: vec![], config: SessionConfig::default(),
                created_at: now, updated_at: now, metadata: serde_json::json!({}),
            };
            let id = port.create(session).await?;
            save_active_session(&id)?;
            println!("Session created: {name}");
            println!("  ID: {id}  (set as active session)");
            Ok(())
        }
        SessionCmd::List => {
            let sessions = port.list().await?;
            if sessions.is_empty() {
                println!("No sessions. Create one with: rusvel session create <name>");
                return Ok(());
            }
            let active = load_active_session().ok();
            println!("{:<38}  {:<20}  {:<10}  {}", "ID", "NAME", "KIND", "UPDATED");
            println!("{}", "-".repeat(90));
            for s in &sessions {
                let marker = if active.as_ref() == Some(&s.id) { " *" } else { "" };
                println!("{:<38}  {:<20}  {:<10}  {}{}", s.id,
                    truncate(&s.name, 20), format!("{:?}", s.kind),
                    s.updated_at.format("%Y-%m-%d %H:%M"), marker);
            }
            Ok(())
        }
        SessionCmd::Switch { id } => {
            let uuid: Uuid = id.parse()
                .map_err(|e| RusvelError::Validation(format!("Invalid UUID: {e}")))?;
            let sid = SessionId::from_uuid(uuid);
            let _ = port.load(&sid).await?; // verify exists
            save_active_session(&sid)?;
            println!("Active session set to: {sid}");
            Ok(())
        }
    }
}

async fn handle_forge(cmd: ForgeCmd, engine: Arc<ForgeEngine>) -> Result<()> {
    match cmd {
        ForgeCmd::Mission { action } => handle_mission(action, engine).await,
    }
}

async fn handle_mission(cmd: MissionCmd, engine: Arc<ForgeEngine>) -> Result<()> {
    let session_id = load_active_session()?;
    match cmd {
        MissionCmd::Today => {
            println!("Generating daily plan...\n");
            let plan = engine.mission_today(&session_id).await?;
            println!("Daily Plan -- {}\n{}", plan.date, "=".repeat(50));
            for (i, task) in plan.tasks.iter().enumerate() {
                println!("  {}. [{:?}] {}", i + 1, task.priority, task.title);
            }
            if !plan.focus_areas.is_empty() {
                println!("\nFocus areas:");
                for a in &plan.focus_areas { println!("  - {a}"); }
            }
            if !plan.notes.is_empty() { println!("\nNotes: {}", plan.notes); }
            Ok(())
        }
        MissionCmd::Goals => {
            let goals = engine.list_goals(&session_id).await?;
            if goals.is_empty() {
                println!("No goals. Add one: rusvel forge mission goal add <title>");
                return Ok(());
            }
            println!("{:<38}  {:<25}  {:<10}  {:<10}  {}", "ID", "TITLE", "TIMEFRAME", "STATUS", "PROGRESS");
            println!("{}", "-".repeat(100));
            for g in &goals {
                println!("{:<38}  {:<25}  {:<10}  {:<10}  {:.0}%", g.id,
                    truncate(&g.title, 25), format!("{:?}", g.timeframe),
                    format!("{:?}", g.status), g.progress * 100.0);
            }
            Ok(())
        }
        MissionCmd::Goal { action } => match action {
            GoalCmd::Add { title, description, timeframe } => {
                let goal = engine.set_goal(&session_id, title, description, timeframe.into()).await?;
                println!("Goal created:\n  ID:        {}\n  Title:     {}\n  Timeframe: {:?}",
                    goal.id, goal.title, goal.timeframe);
                Ok(())
            }
        },
        MissionCmd::Review { period } => {
            let tf: Timeframe = period.into();
            println!("Generating {:?} review...\n", tf);
            let review = engine.review(&session_id, tf).await?;
            println!("Review ({:?})\n{}", review.period, "=".repeat(50));
            print_list("Accomplishments", &review.accomplishments);
            print_list("Blockers", &review.blockers);
            print_list("Insights", &review.insights);
            print_list("Next actions", &review.next_actions);
            Ok(())
        }
    }
}

// ── Helpers ──────────────────────────────────────────────────────

fn print_list(heading: &str, items: &[String]) {
    if !items.is_empty() {
        println!("\n{heading}:");
        for item in items { println!("  - {item}"); }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max { s.to_string() } else { format!("{}...", &s[..max.saturating_sub(3)]) }
}
