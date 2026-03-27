# RUSVEL UI/UX Design — Gemini

> Product design & information architecture. Copy everything below the line into Gemini.

---

I need help designing the information architecture and UI layout for RUSVEL, an AI-powered virtual agency for solo founders. Here's the full product context:

## What RUSVEL Is
A single binary (Rust backend + SvelteKit frontend) that runs locally. It provides 13 AI departments, each acting as a specialized team member:

- **Forge** — Agent orchestration, mission planning, daily briefs, goal tracking
- **Code** — Code analysis, symbol search, dependency graphs, metrics
- **Content** — AI writing, platform publishing (LinkedIn, Twitter, DEV.to), content calendar
- **Harvest** — Lead discovery, opportunity scoring, proposal generation
- **GTM** — CRM, outreach sequences, deal pipeline, invoicing
- **Finance** — Ledger, runway, tax estimation, P&L
- **Product** — Roadmap, pricing, feedback tracking
- **Growth** — Funnel analysis, cohort tracking, KPIs
- **Distribution** — SEO, marketplace, affiliates
- **Legal** — Contracts, IP, compliance
- **Support** — Tickets, knowledge base, NPS
- **Infra** — CI/CD, deployments, monitoring, incidents
- **Flow** — Visual DAG workflow builder connecting departments

Each department has:
- **Chat** — AI conversation with SSE streaming, tool calls visible, approval gates
- **Agents** — Named AI agents with roles, models, instructions (CRUD)
- **Skills** — Reusable prompt templates invoked by name (CRUD)
- **Rules** — System prompt injections, toggleable (CRUD)
- **Workflows** — Multi-step agent sequences (CRUD + run)
- **MCP Servers** — External tool connections (CRUD)
- **Hooks** — Event-triggered automations (CRUD)
- **Engine** — Department-specific tools (code analyze, content draft, harvest scan, etc.)
- **Actions** — Quick actions + AI capability builder
- **Terminal** — Department-scoped terminal
- **Events** — Activity timeline
- **Settings** — Model, effort level, budget, allowed tools, system prompt

Plus cross-cutting pages: Dashboard, Global Chat (god agent), Approval Queue, Database Browser, Knowledge Base, Flow Editor, Settings.

## The User
One person. Solo founder. Uses this daily as their virtual team. They context-switch between departments frequently. Sometimes they're in "configure mode" (setting up agents, skills, rules). Sometimes they're in "execute mode" (chatting, running workflows, reviewing results). Sometimes they're in "monitor mode" (checking events, dashboards, approvals).

## Current Problem
The current layout puts everything in a flat sidebar (20+ items) with a cramped 288px tabbed panel. No URL hierarchy. No room for rich content. Chat is always visible even when configuring.

## What I Need From You

Design the information architecture for this app. Specifically:

1. **Navigation hierarchy**: How should the user move through Department → Section → Action? How many navigation levels? What's persistent vs contextual?

2. **Layout zones**: Where should each type of content live? Consider:
   - A top bar / header area
   - A department selector mechanism
   - A section/feature selector
   - A left sidebar (contextual navigation, filters, tree views?)
   - A right sidebar (info panel, quick stats, recent activity?)
   - A main content area (scrollable)
   - Where does chat live? Always visible? Toggle? Drawer? Full page?

3. **Mode handling**: How should the UI adapt between:
   - Configure mode (editing agents, skills, rules — lots of forms)
   - Execute mode (chatting, running workflows — streaming content)
   - Monitor mode (events, dashboards, approvals — read-heavy)

4. **Department identity**: Each department has its own color, icon, and personality. How should this manifest in the UI without being overwhelming with 13 departments?

5. **The chat question**: Chat is core to every department, but users also need to work with structured data (CRUD entities). Should chat be:
   - A permanent side panel (current)?
   - A full-page section you navigate to?
   - A drawer/overlay you can summon from anywhere?
   - An inline element within each section?

Please provide your recommended layout as an ASCII diagram, explain the reasoning for each zone, and describe how the user flows through a typical session: "I want to set up a new content skill, test it in chat, then create a workflow that uses it."
