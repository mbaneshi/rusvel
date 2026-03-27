# Proposal: Minimalist Entrepreneur Starter Kit + Department Tool Wiring

> Import Sahil Lavingia's 10 business skills into Rusvel as a first-class starter kit —
> but first, wire the 5 target departments so agents can actually act on the advice.

# The Problem

he state report (Chapter 7) identifies the single biggest leverage gap:

> **Only 3 of 13 departments (forge, code, content) have their tools wired into
> the agent system. The other 10 have working engines that agents simply can't invoke.**

The ME kit targets 7 departments. Two are already wired (forge, content). Five are
skeleton: **harvest, product, gtm, finance, growth**. Without tool wiring, installing
the kit gives each department an advisor prompt but no ability to act — the agent can
tell you to "track your first 100 customers" but can't actually create a deal in CRM.

**The fix:** Wire tools first, then layer the kit on top. The combined result is
departments that both know *what* to do (skills/rules) and *can* do it (tools).

---

## Phase 0: Wire 5 Skeleton Departments (prerequisite)

Each department follows the established pattern from dept-forge and dept-code:
define `ToolDefinition`, write handler closure, register in `register()`.

### Harvest (dept-harvest) — 5 tools

| Tool | Engine Method | Args | Effect |
|------|-------------|------|--------|
| `harvest.scan` | `scan()` | session_id, source_url | Scan source, score, store opportunities |
| `harvest.score` | `score_opportunity()` | session_id, opportunity_id | Re-score a stored opportunity |
| `harvest.propose` | `generate_proposal()` | session_id, opportunity_id, profile | Generate proposal for opportunity |
| `harvest.pipeline` | `pipeline()` | session_id | Get pipeline stats (counts per stage) |
| `harvest.list` | `list_opportunities()` | session_id, stage? | List opportunities, optionally by stage |

### GTM (dept-gtm) — 6 tools

| Tool | Engine Method | Args | Effect |
|------|-------------|------|--------|
| `gtm.add_contact` | `crm().add_contact()` | name, email, source | Create CRM contact |
| `gtm.add_deal` | `crm().add_deal()` | contact_id, title, value | Create deal in pipeline |
| `gtm.advance_deal` | `crm().advance_deal()` | deal_id, stage | Move deal to next stage |
| `gtm.list_deals` | `crm().list_deals()` | stage? | List deals, optionally by stage |
| `gtm.create_invoice` | `invoices().create_invoice()` | deal_id, items | Create invoice |
| `gtm.total_revenue` | `invoices().total_revenue()` | — | Sum of paid invoices |

### Finance (dept-finance) — 4 tools

| Tool | Engine Method | Args | Effect |
|------|-------------|------|--------|
| `finance.record` | `ledger().record()` | kind (income/expense), amount, description | Add ledger entry |
| `finance.balance` | `ledger().balance()` | — | Current balance (income - expenses) |
| `finance.add_tax_estimate` | `tax().add_estimate()` | category, amount | Record tax estimate |
| `finance.tax_liability` | `tax().total_liability()` | — | Sum of estimated tax liability |

### Product (dept-product) — 4 tools

| Tool | Engine Method | Args | Effect |
|------|-------------|------|--------|
| `product.add_feature` | `roadmap().add_feature()` | name, priority, status | Add feature to roadmap |
| `product.list_features` | `roadmap().list_features()` | status? | List roadmap features |
| `product.create_tier` | `pricing().create_tier()` | name, monthly, annual, features | Create pricing tier |
| `product.list_tiers` | `pricing().list_tiers()` | — | List pricing tiers |

### Growth (dept-growth) — 4 tools

| Tool | Engine Method | Args | Effect |
|------|-------------|------|--------|
| `growth.funnel` | `funnel().list_stages()` | — | Get funnel stage counts |
| `growth.update_funnel` | `funnel().add_stage()` | stage, count | Update users at funnel stage |
| `growth.record_kpi` | `kpi().record_kpi()` | name, value, unit | Record a KPI data point |
| `growth.list_kpis` | `kpi().list_kpis()` | — | List all tracked KPIs |

**Total: 23 new tools across 5 departments.**

Each tool follows the same pattern:

```rust
// In dept-harvest/src/lib.rs register()
ctx.tools.register(
    ToolDefinition {
        name: "harvest.pipeline".into(),
        description: "Get pipeline stats — opportunity counts by stage".into(),
        parameters: json!({
            "type": "object",
            "properties": {
                "session_id": { "type": "string" }
            },
            "required": ["session_id"]
        }),
        searchable: true,  // discoverable via tool_search
    },
    |args| {
        let engine = ENGINE.get().unwrap();
        let session_id = SessionId(args["session_id"].as_str().unwrap().into());
        Box::pin(async move {
            let stats = engine.pipeline(&session_id).await?;
            Ok(serde_json::to_value(stats)?)
        })
    },
);
```

---

## Phase 1: Skill Import (the ME kit)

With tools wired, skills become actionable. Each skill's advisor prompt now has
engine tools available — the agent can both advise and execute.

### 1a. Parse & convert

Source: `slavingia/skills` — 10 SKILL.md files with YAML frontmatter
(`name`, `description`, optional `argument-hint`) and markdown body.

```rust
SkillDefinition {
    id: uuid_v7(),
    name: frontmatter.name,               // "validate-idea"
    description: frontmatter.description,
    prompt_template: body_markdown,        // Full advisor prompt + {{input}} suffix
    metadata: json!({
        "engine": target_department,
        "source": "minimalist-entrepreneur",
        "author": "sahil-lavingia",
    }),
}
```

| # | Skill | Target Dept | Now Can Use Tools |
|---|-------|-------------|-------------------|
| 1 | `find-community` | harvest | `harvest.scan`, `harvest.list` |
| 2 | `validate-idea` | harvest | `harvest.score`, `harvest.pipeline` |
| 3 | `processize` | product | `product.add_feature`, `product.list_features` |
| 4 | `mvp` | product | `product.add_feature`, `product.create_tier` |
| 5 | `first-customers` | gtm | `gtm.add_contact`, `gtm.add_deal`, `gtm.list_deals` |
| 6 | `pricing` | finance | `finance.record`, `finance.balance` |
| 7 | `marketing-plan` | content + growth | `content.draft`, `growth.funnel`, `growth.record_kpi` |
| 8 | `grow-sustainably` | finance + growth | `finance.balance`, `growth.list_kpis` |
| 9 | `company-values` | forge | `mission_today`, `set_goal` |
| 10 | `minimalist-review` | forge (global) | all tools (no engine scope) |

For skills spanning two departments, create one skill per department with the same
prompt but different `metadata.engine`.

### 1b. Extract rules

Six skills contain prescriptive principles that become `RuleDefinition` entries
injected into department system prompts via `load_rules_for_engine()`:

| Rule | Department | Extracted From |
|------|-----------|----------------|
| "Start with community, not product ideas" | harvest | find-community |
| "Validate: can you solve it manually? Will people pay?" | harvest | validate-idea |
| "Ship in a weekend. Build forms and lists, not features" | product | mvp |
| "Sell to concentric circles: friends → community → cold" | gtm | first-customers |
| "Profitability from day one. Default alive, not default dead" | finance | grow-sustainably |
| "Apply minimalist review to every major decision" | forge | minimalist-review (global) |

### 1c. Register as starter kit

4th built-in kit in `kits.rs`:

```rust
StarterKit {
    id: "minimalist-entrepreneur".into(),
    name: "Minimalist Entrepreneur".into(),
    description: "Business methodology from Sahil Lavingia — \
                  10 skills + 6 rules across 7 departments".into(),
    target_audience: "Solo founders, indie hackers, bootstrappers".into(),
    departments: vec!["harvest", "product", "gtm", "finance",
                      "content", "growth", "forge"],
    entities: vec![/* 10 skills + 6 rules = 16 KitEntity items */],
}
```

---

## Phase 2: Playbook — The Founder Journey

A built-in playbook chains skills in book order. Now each step has **both** an
advisor prompt and access to department tools:

```
"builtin-founder-journey"
  Step 1:  Agent(find-community)      → harvest   [can scan sources]
  Step 2:  Agent(validate-idea)       → harvest   [can score opportunities]
  Step 3:  Approval("Idea validated? Proceed to build?")
  Step 4:  Agent(processize)          → product   [can add features to roadmap]
  Step 5:  Agent(mvp)                 → product   [can create pricing tiers]
  Step 6:  Agent(first-customers)     → gtm       [can create contacts, deals]
  Step 7:  Agent(pricing)             → finance   [can record revenue, check balance]
  Step 8:  Approval("Revenue flowing? Ready to scale?")
  Step 9:  Agent(marketing-plan)      → content   [can draft content]
  Step 10: Agent(grow-sustainably)    → growth    [can track KPIs, funnel]
  Step 11: Agent(company-values)      → forge     [can set goals]
```

Each step threads `{{last_output}}` forward. Approval gates at validation and
first revenue create natural human checkpoints.

---

## Phase 3: Flow Template — Visual DAG

Same journey as a `FlowDef` for the `/flows` UI:

- 10 agent nodes (one per skill, each parameterized with dept tools)
- 2 condition nodes (idea validated? revenue achieved?)
- Connections with "true"/"false" branches from condition nodes
- Canvas positions arranged left-to-right in book progression order
- Editable: users can skip steps, add parallel branches, rewire

---

## Phase 4: Forge Persona — Minimalist Advisor

11th persona in `PersonaManager`:

```rust
AgentProfile {
    name: "MinimalistAdvisor".into(),
    role: "minimalist_advisor".into(),
    instructions: "You are a business advisor channeling The Minimalist Entrepreneur. \
                   You help solo founders build sustainable, profitable businesses. \
                   You always apply the 8 core principles before recommending action. \
                   When advising, use available department tools to ground advice in \
                   real data — check the pipeline, review the balance, look at KPIs.",
    default_model: ModelRef { provider: "ollama", model: "llama3.2" },
    allowed_tools: vec!["web_search", "file_write", "harvest.*", "gtm.*",
                         "finance.*", "product.*", "growth.*"],
    capabilities: vec![Capability::Planning, Capability::ContentCreation],
    budget_limit: Some(0.50),
}
```

Key difference from v1: this persona has **cross-department tool access**, so it
can pull real data when advising (check revenue before recommending hiring, review
pipeline before suggesting marketing spend).

---

## Implementation Plan

| Phase | Step | Scope | Crate(s) | Effort |
|-------|------|-------|----------|--------|
| **0** | 0.1 | Wire harvest tools (5 tools) | `dept-harvest` | Medium |
| | 0.2 | Wire gtm tools (6 tools) | `dept-gtm` | Medium |
| | 0.3 | Wire finance tools (4 tools) | `dept-finance` | Small |
| | 0.4 | Wire product tools (4 tools) | `dept-product` | Small |
| | 0.5 | Wire growth tools (4 tools) | `dept-growth` | Small |
| | 0.6 | Tests: tool registration + round-trip calls | All 5 dept-* | Medium |
| **1** | 1.1 | Vendor 10 SKILL.md as string constants | `rusvel-api/kits.rs` | Small |
| | 1.2 | Map skills → SkillDefinition with dept scoping | `rusvel-api/kits.rs` | Small |
| | 1.3 | Extract 6 rules → RuleDefinition | `rusvel-api/kits.rs` | Small |
| | 1.4 | Register ME starter kit (4th built-in) | `rusvel-api/kits.rs` | Small |
| | 1.5 | Tests: kit install round-trip | `rusvel-api` | Small |
| **2** | 2.1 | Add builtin-founder-journey playbook | `rusvel-api/playbooks.rs` | Medium |
| **3** | 3.1 | Add founder journey flow template | `rusvel-api/flow_routes.rs` | Medium |
| **4** | 4.1 | Add MinimalistAdvisor persona | `forge-engine/personas.rs` | Small |
| **+** | +.1 | Frontend: kit install UI + journey progress | `frontend/` | Medium |

### PR Strategy

| PR | Contents | Depends On |
|----|----------|-----------|
| **PR 1** | Phase 0: Wire 5 departments (23 tools) | Nothing |
| **PR 2** | Phase 1: ME starter kit (10 skills + 6 rules) | PR 1 |
| **PR 3** | Phase 2-4: Playbook + flow template + persona | PR 2 |
| **PR 4** | Frontend polish: kit install UI, journey tracker | PR 2 |

PR 1 stands alone as the #1 priority gap fix — it benefits all future kits, not
just the ME kit. PRs 3 and 4 can ship in parallel.

---

## Before vs. After

### Before (current state)

```
User: /validate-idea I want to build a project management tool
Agent: [advisor prompt activates, gives business advice]
       "Have you found 10 people who would pay? Can you solve it manually?"
User: Great advice. Now track this as an opportunity.
Agent: Sorry, I don't have tools to do that in this department.
```

### After (with Phase 0 + Phase 1)

```
User: /validate-idea I want to build a project management tool
Agent: [advisor prompt activates + harvest tools available]
       "Let me check your current pipeline first..."
       [calls harvest.pipeline] → "You have 3 opportunities in Cold stage"
       "Based on the framework: have you found 10 people who would pay?"
User: Yes, I've talked to 12 freelancers who all need this.
Agent: "Green flag — real manual validation. Let me score this."
       [calls harvest.score] → Score: 0.82
       "Strong score. I'll advance this to Qualified."
```

---

## What This Does NOT Include

- **No new crates.** All changes fit existing crate boundaries.
- **No schema changes.** Tools, skills, rules, flows all use ObjectStore.
- **No external dependencies.** SKILL.md content vendored as string constants.
- **Only 5 of 10 skeleton departments wired.** The remaining 5 (flow, distro,
  legal, support, infra) aren't targeted by ME skills — wire them separately.

## Wiring Matrix After Completion

| Department | Tools Before | Tools After | ME Skills | ME Rules |
|:-----------|:-----------:|:----------:|:---------:|:--------:|
| forge | 5 | 5 | 2 | 1 |
| code | 2 | 2 | — | — |
| content | 5+ | 5+ | 1 | — |
| **harvest** | **0** | **5** | **2** | **2** |
| **gtm** | **0** | **6** | **1** | **1** |
| **finance** | **0** | **4** | **2** | **1** |
| **product** | **0** | **4** | **2** | **1** |
| **growth** | **0** | **4** | **2** | — |
| flow | 0 | 0 | — | — |
| distro | 0 | 0 | — | — |
| legal | 0 | 0 | — | — |
| support | 0 | 0 | — | — |
| infra | 0 | 0 | — | — |

**Result: 8 of 13 departments fully operational (up from 3). 23 new tools. 10 skills. 6 rules. 1 playbook. 1 flow template. 1 persona.**

---

## Open Questions

1. **Vendored vs. fetched** — Embed SKILL.md as Rust string constants (like existing
   kits), or add a `POST /api/kits/import` endpoint for git repos? Recommendation:
   vendor now, add import endpoint later when more skill packs exist.

2. **Cross-department tool access** — Should the MinimalistAdvisor persona (Phase 4)
   have wildcard access to all 5 department tool namespaces? Or should cross-dept
   access go through `delegate_agent`? Recommendation: wildcard for advisor persona,
   delegate_agent for playbook steps.

3. **Remaining 5 departments** — Wire flow, distro, legal, support, infra in a
   separate effort? Or include them in PR 1 for completeness? Recommendation:
   separate — they have no ME skill coverage and can be wired independently.
