//! Boot sequence for the Department-as-App architecture (ADR-014).
//!
//! Implements `installed_departments()` + `boot_departments()`:
//!
//! 1. Read all manifests (no side effects, fast)
//! 2. Validate IDs and dependencies
//! 3. Register departments in dependency order
//! 4. Finalize [`RegistrationContext`] → registry + tools + event subs + job handlers
//!
//! The composition root (`main.rs`) creates ports, then calls
//! `boot_departments()` which registers all departments. The host wires
//! event subscriptions to [`rusvel_event::EventBus`] and job handlers to
//! the queue worker.

use std::sync::Arc;

use rusvel_core::DepartmentApp;
use rusvel_core::department::{
    DepartmentsBootArtifacts, EventSubscription, RegistrationContext, resolve_dependency_order,
    validate_unique_ids,
};
use rusvel_core::domain::JobKind;
use rusvel_core::ports::*;
use rusvel_event::EventBus;
use tokio::sync::broadcast;

/// Map a queued [`JobKind`] to the string key departments use in [`JobHandlerRegistrar`].
pub fn department_job_registry_key(kind: &JobKind) -> String {
    match kind {
        JobKind::ContentPublish => "content.publish".into(),
        JobKind::CodeAnalyze => "code.analyze".into(),
        JobKind::HarvestScan => "harvest.scan".into(),
        JobKind::Custom(s) => s.clone(),
        _ => String::new(),
    }
}

/// Subscribe to live [`EventBus`] broadcasts and invoke matching department handlers.
pub fn spawn_department_event_dispatch(
    event_bus: Arc<EventBus>,
    subscriptions: Vec<EventSubscription>,
) {
    if subscriptions.is_empty() {
        return;
    }
    tracing::info!(
        count = subscriptions.len(),
        "Department event dispatch: bridging EventBus → department handlers"
    );
    tokio::spawn(async move {
        let mut rx = event_bus.subscribe();
        loop {
            match rx.recv().await {
                Ok(event) => {
                    for sub in &subscriptions {
                        if sub.event_kind == event.kind {
                            let h = sub.handler.clone();
                            let kind = sub.event_kind.clone();
                            let ev = event.clone();
                            tokio::spawn(async move {
                                if let Err(e) = h(ev).await {
                                    tracing::warn!(
                                        error = %e,
                                        kind = %kind,
                                        "department event handler failed"
                                    );
                                }
                            });
                        }
                    }
                }
                Err(broadcast::RecvError::Lagged(_)) => {}
                Err(broadcast::RecvError::Closed) => break,
            }
        }
    });
}

/// Ordered list of installed departments.
///
/// Like Django's `INSTALLED_APPS` — order matters for dependencies.
/// Departments with no `depends_on` are listed first.
pub fn installed_departments() -> Vec<Box<dyn DepartmentApp>> {
    vec![
        // Core departments (no deps on other depts)
        Box::new(dept_forge::ForgeDepartment::new()),
        Box::new(dept_code::CodeDepartment::new()),

        // Departments with cross-dept event subscriptions
        Box::new(dept_content::ContentDepartment::new()),
        Box::new(dept_harvest::HarvestDepartment::new()),
        Box::new(dept_flow::FlowDepartment::new()),

        // Departments with minimal logic (progressive enhancement)
        Box::new(dept_gtm::GtmDepartment::new()),
        Box::new(dept_finance::FinanceDepartment::new()),
        Box::new(dept_product::ProductDepartment::new()),
        Box::new(dept_growth::GrowthDepartment::new()),
        Box::new(dept_distro::DistroDepartment::new()),
        Box::new(dept_legal::LegalDepartment::new()),
        Box::new(dept_support::SupportDepartment::new()),
        Box::new(dept_infra::InfraDepartment::new()),
    ]
}

/// Boot all departments: validate manifests, register in dependency order,
/// return the generated registry.
///
/// # Arguments
///
/// * `departments` — from [`installed_departments()`]
/// * Ports — shared infrastructure created by the composition root
#[allow(clippy::too_many_arguments)]
pub async fn boot_departments(
    departments: &[Box<dyn DepartmentApp>],
    agent: Arc<dyn AgentPort>,
    events: Arc<dyn EventPort>,
    storage: Arc<dyn StoragePort>,
    jobs: Arc<dyn JobPort>,
    memory: Arc<dyn MemoryPort>,
    sessions: Arc<dyn SessionPort>,
    config: Arc<dyn ConfigPort>,
    auth: Arc<dyn AuthPort>,
    embedding: Option<Arc<dyn EmbeddingPort>>,
    vector_store: Option<Arc<dyn VectorStorePort>>,
) -> anyhow::Result<DepartmentsBootArtifacts> {
    // Phase 1: Read all manifests (no side effects)
    let manifests: Vec<_> = departments.iter().map(|d| d.manifest()).collect();

    // Phase 2: Validate
    validate_unique_ids(&manifests)?;
    let order = resolve_dependency_order(&manifests)?;

    tracing::info!(
        "Department boot: {} departments, dependency order: {}",
        manifests.len(),
        order
            .iter()
            .map(|&i| manifests[i].id.as_str())
            .collect::<Vec<_>>()
            .join(" → ")
    );

    // Phase 3: Register in dependency order
    let mut ctx = RegistrationContext::new(
        agent,
        events,
        storage,
        jobs,
        memory,
        sessions,
        config,
        auth,
        embedding,
        vector_store,
    );

    for &idx in &order {
        let dept = &departments[idx];
        let manifest = dept.manifest();
        ctx.add_manifest(manifest.clone());

        if let Err(e) = dept.register(&mut ctx).await {
            tracing::error!(
                "Failed to register department '{}': {e}",
                manifest.id
            );
            // Continue — don't let one failed department block the rest
        }
    }

    let tools_n = ctx.tools.len();
    let ev_n = ctx.event_handlers.len();
    let job_n = ctx.job_handlers.len();

    let artifacts = ctx.finalize();
    tracing::info!(
        "Department boot complete: {} departments registered, {} tools, {} event handlers, {} job handlers",
        artifacts.registry.departments.len(),
        tools_n,
        ev_n,
        job_n,
    );

    Ok(artifacts)
}
