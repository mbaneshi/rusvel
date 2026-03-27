# RUSVEL UI/UX Research — Perplexity

> Competitive analysis & UI patterns. Copy everything below the line into Perplexity.

---

I'm designing the web UI for a solo-founder tool called RUSVEL. It's a single Rust+SvelteKit binary that replaces an entire agency — 13 AI-powered departments (Forge/orchestration, Code, Content, Harvest/leads, GTM/sales, Finance, Product, Growth, Distribution, Legal, Support, Infra, Flow/workflows), each with its own agents, skills, rules, workflows, and chat.

The current UI has a flat 3-column layout: sidebar listing all 13 departments, a 288px tabbed panel for department config (agents, skills, rules, etc.), and a chat pane. It doesn't scale.

I need to redesign this as a proper workspace UI — think GitHub (top tabs + left sidebar + right sidebar + scrolling main), Linear (project switcher + view tabs + list/board), or Notion (workspace switcher + sidebar + breadcrumb + content).

Please research and compare:

1. **GitHub's layout pattern**: How does GitHub handle the hierarchy (org → repo → tab → content)? What are the left sidebar, right sidebar, and main area responsibilities? How do they handle context switching between repos?

2. **Linear's layout**: How does Linear handle project switching + different views (list, board, timeline)? How do they handle the sidebar + main content relationship?

3. **Retool / Airplane / Windmill**: Internal tool builders — how do they organize multiple apps/workflows/resources in one workspace?

4. **n8n / Langflow / Flowise**: AI workflow tools — how do they organize nodes, workflows, credentials, and execution in their UI?

5. **Cursor / Windsurf / Claude Code (web)**: AI coding tools — how do they combine chat, file tree, editor, terminal, and tool output?

For each, identify:
- Navigation hierarchy (how many levels? horizontal vs vertical?)
- How they handle 10+ different "workspaces" or "projects"
- How they balance configuration (settings, rules) vs execution (chat, run)
- Left sidebar vs right sidebar responsibilities
- What goes in the main scrolling area

I'm looking for the best pattern for: "pick a department → configure its capabilities → execute via chat/tools → monitor results" — all in one UI.
