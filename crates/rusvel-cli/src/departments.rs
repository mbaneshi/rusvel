//! Department subcommands — Tier 1 (one-shot CLI).
//!
//! Each department gets a subcommand group with basic operations:
//! list, status, and department-specific actions.

use std::sync::Arc;

use clap::Subcommand;
use rusvel_core::domain::ObjectFilter;
use rusvel_core::error::Result;
use rusvel_core::ports::StoragePort;

// ── Department command enum ──────────────────────────────────────

#[derive(Subcommand, Debug)]
pub enum DeptCmd {
    /// Finance: ledger, runway, tax.
    Finance {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Growth: funnels, cohorts, KPIs.
    Growth {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Distribution: marketplace, SEO, affiliates.
    Distro {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Legal: contracts, compliance, IP.
    Legal {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Support: tickets, knowledge base, NPS.
    Support {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Infrastructure: deploys, monitoring, incidents.
    Infra {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Product: roadmap, pricing, feedback.
    Product {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Code intelligence: analysis, search.
    Code {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Harvest: opportunities, proposals, pipeline.
    Harvest {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Content: drafts, calendar, publishing.
    Content {
        #[command(subcommand)]
        action: DeptAction,
    },
    /// Go-to-market: CRM, outreach, invoices.
    Gtm {
        #[command(subcommand)]
        action: DeptAction,
    },
}

#[derive(Subcommand, Debug)]
pub enum DeptAction {
    /// List all items in this department.
    List {
        /// Object kind to list (e.g. "transactions", "tickets", "contracts").
        #[arg(short, long)]
        kind: Option<String>,
        /// Maximum items to show.
        #[arg(short = 'n', long, default_value = "20")]
        limit: usize,
    },
    /// Show department status summary.
    Status,
    /// Show recent events for this department.
    Events {
        /// Maximum events to show.
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
    },
}

// ── Department metadata ──────────────────────────────────────────

struct DeptMeta {
    name: &'static str,
    collections: &'static [&'static str],
    event_prefix: &'static str,
}

fn dept_meta(dept: &str) -> DeptMeta {
    match dept {
        "finance" => DeptMeta {
            name: "Finance",
            collections: &["transactions", "tax_estimates", "runway_snapshots"],
            event_prefix: "finance.",
        },
        "growth" => DeptMeta {
            name: "Growth",
            collections: &["funnel_stages", "cohorts", "kpi_entries"],
            event_prefix: "growth.",
        },
        "distro" => DeptMeta {
            name: "Distribution",
            collections: &["listings", "keywords", "partners"],
            event_prefix: "distro.",
        },
        "legal" => DeptMeta {
            name: "Legal",
            collections: &["contracts", "compliance_checks", "ip_assets"],
            event_prefix: "legal.",
        },
        "support" => DeptMeta {
            name: "Support",
            collections: &["tickets", "articles", "nps_responses"],
            event_prefix: "support.",
        },
        "infra" => DeptMeta {
            name: "Infrastructure",
            collections: &["deployments", "health_checks", "incidents"],
            event_prefix: "infra.",
        },
        "product" => DeptMeta {
            name: "Product",
            collections: &["features", "pricing_tiers", "feedback"],
            event_prefix: "product.",
        },
        "code" => DeptMeta {
            name: "Code",
            collections: &["analyses", "symbols"],
            event_prefix: "code.",
        },
        "harvest" => DeptMeta {
            name: "Harvest",
            collections: &["opportunities", "proposals"],
            event_prefix: "harvest.",
        },
        "content" => DeptMeta {
            name: "Content",
            collections: &["content", "scheduled_posts"],
            event_prefix: "content.",
        },
        "gtm" => DeptMeta {
            name: "Go-to-Market",
            collections: &["deals", "sequences", "invoices", "contacts"],
            event_prefix: "gtm.",
        },
        _ => DeptMeta {
            name: "Unknown",
            collections: &[],
            event_prefix: "",
        },
    }
}

// ── Dispatch ─────────────────────────────────────────────────────

pub async fn handle_dept(cmd: DeptCmd, storage: Arc<dyn StoragePort>) -> Result<()> {
    let (key, action) = match cmd {
        DeptCmd::Finance { action } => ("finance", action),
        DeptCmd::Growth { action } => ("growth", action),
        DeptCmd::Distro { action } => ("distro", action),
        DeptCmd::Legal { action } => ("legal", action),
        DeptCmd::Support { action } => ("support", action),
        DeptCmd::Infra { action } => ("infra", action),
        DeptCmd::Product { action } => ("product", action),
        DeptCmd::Code { action } => ("code", action),
        DeptCmd::Harvest { action } => ("harvest", action),
        DeptCmd::Content { action } => ("content", action),
        DeptCmd::Gtm { action } => ("gtm", action),
    };

    let meta = dept_meta(key);

    match action {
        DeptAction::List { kind, limit } => dept_list(meta, storage, kind, limit).await,
        DeptAction::Status => dept_status(meta, storage).await,
        DeptAction::Events { limit } => dept_events(meta, storage, limit).await,
    }
}

// ── Handlers ─────────────────────────────────────────────────────

async fn dept_list(
    meta: DeptMeta,
    storage: Arc<dyn StoragePort>,
    kind: Option<String>,
    limit: usize,
) -> Result<()> {
    let objects = storage.objects();
    let filter = ObjectFilter { limit: Some(limit as u32), ..Default::default() };

    // If a specific kind was given, list that; otherwise list all collections
    let collections: Vec<&str> = match &kind {
        Some(k) => vec![k.as_str()],
        None => meta.collections.to_vec(),
    };

    println!("┌─ {} Department", meta.name);
    println!("│");

    for collection in collections {
        let items = objects.list(collection, filter.clone()).await.unwrap_or_default();
        println!("├─ {} ({} items)", collection, items.len());
        for item in items.iter().take(limit) {
            let name = item.get("name")
                .or_else(|| item.get("title"))
                .and_then(|v| v.as_str())
                .unwrap_or("(unnamed)");
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("?");
            let short_id = if id.len() > 8 { &id[..8] } else { id };
            println!("│  {short_id} {name}");
        }
        println!("│");
    }
    println!("└─");
    Ok(())
}

async fn dept_status(meta: DeptMeta, storage: Arc<dyn StoragePort>) -> Result<()> {
    let objects = storage.objects();
    let filter = ObjectFilter::default();

    println!("┌─ {} Department — Status", meta.name);
    println!("│");

    let mut total = 0usize;
    for collection in meta.collections {
        let count = objects.list(collection, filter.clone()).await
            .map(|v| v.len())
            .unwrap_or(0);
        total += count;
        println!("│  {collection:<20} {count:>5} items");
    }
    println!("│");
    println!("│  Total: {total} items");
    println!("└─");
    Ok(())
}

async fn dept_events(meta: DeptMeta, storage: Arc<dyn StoragePort>, limit: usize) -> Result<()> {
    let objects = storage.objects();
    // Events are stored in the "events" collection; filter by prefix
    let filter = ObjectFilter { limit: Some((limit * 5) as u32), ..Default::default() };
    let all_events = objects.list("events", filter).await.unwrap_or_default();

    let prefix = meta.event_prefix;
    let matching: Vec<_> = all_events.iter()
        .filter(|e| {
            e.get("kind").and_then(|v| v.as_str()).is_some_and(|k| k.starts_with(prefix))
        })
        .take(limit)
        .collect();

    println!("┌─ {} Department — Recent Events", meta.name);
    println!("│");

    if matching.is_empty() {
        println!("│  (no events)");
    } else {
        for event in &matching {
            let kind = event.get("kind").and_then(|v| v.as_str()).unwrap_or("?");
            let time = event.get("created_at").and_then(|v| v.as_str()).unwrap_or("?");
            let short_time = if time.len() > 16 { &time[..16] } else { time };
            println!("│  {short_time} {kind}");
        }
    }
    println!("│");
    println!("└─");
    Ok(())
}

/// List all available department names (used by REPL completer).
pub fn department_names() -> &'static [&'static str] {
    &[
        "finance", "growth", "distro", "legal", "support",
        "infra", "product", "code", "harvest", "content", "gtm",
    ]
}
