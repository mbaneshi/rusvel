# Claude Code — Community Enhancements & Next-Level Ideas

> A curated catalog of what the community has built to push Claude Code beyond its defaults.
> Use this as inspiration for RUSVEL's own AI-powered workflows.

---

## 1. Awesome Lists & Curated Collections

| Repo | What it offers |
|------|---------------|
| [hesreallyhim/awesome-claude-code](https://github.com/hesreallyhim/awesome-claude-code) | Skills, hooks, slash-commands, agent orchestrators, plugins |
| [jqueryscript/awesome-claude-code](https://github.com/jqueryscript/awesome-claude-code) | Tools, IDE integrations, frameworks, resources |
| [rohitg00/awesome-claude-code-toolkit](https://github.com/rohitg00/awesome-claude-code-toolkit) | 135 agents, 35 skills, 42 commands, 150+ plugins, 19 hooks, 15 rules, 7 templates, 8 MCP configs |
| [ComposioHQ/awesome-claude-plugins](https://github.com/ComposioHQ/awesome-claude-plugins) | Curated plugins: commands, agents, hooks, MCP servers |
| [FlorianBruniaux/claude-code-ultimate-guide](https://github.com/FlorianBruniaux/claude-code-ultimate-guide) | Beginner-to-power-user guide with production-ready templates |

---

## 2. Plugin & Skills Ecosystem

- **Official Plugin System** — Public beta Oct 2025, now stable. Browse at [claude.com/plugins](https://claude.com/plugins)
- **Top Plugins by installs**: Frontend Design (96K), Context7 (71K), Ralph Loop (57K), Code Review (50K), Playwright (28K)
- **[anthropics/skills](https://github.com/anthropics/skills)** — Official Agent Skills repo. Skills = folders with a `SKILL.md` that auto-activate based on context
- **[jeremylongshore/claude-code-plugins-plus-skills](https://github.com/jeremylongshore/claude-code-plugins-plus-skills)** — 340 plugins + 1,367 agent skills with CCPI package manager

---

## 3. Multi-Agent Orchestration

This is the biggest frontier — running multiple Claude Code instances in parallel.

| Tool | Stars | Description |
|------|-------|-------------|
| [claude-squad](https://github.com/smtg-ai/claude-squad) | 5.8k | Terminal app managing multiple Claude Code instances simultaneously |
| [Official Agent Teams](https://code.claude.com/docs/en/agent-teams) | — | One session as team lead, teammates in independent context windows (v2.1.32+) |
| [ruflo](https://github.com/ruvnet/ruflo) | — | Multi-agent swarms, distributed intelligence, RAG integration |
| [ccswarm](https://github.com/nwiizo/ccswarm) | — | Multi-agent orchestration with git worktree isolation |

**Key idea**: Spawn specialized agents (researcher, coder, reviewer, tester) that work in parallel on isolated git worktrees, then merge results.

---

## 4. GSD (Get Shit Done) — Spec-Driven Development

The leading community workflow framework with **23k stars**.

- **[gsd-build/get-shit-done](https://github.com/gsd-build/get-shit-done)** — ~50 Markdown files + Node.js CLI helper + hooks
- **How it works**:
  1. Interviews you about what you want to build
  2. Spawns parallel research agents
  3. Creates detailed specs
  4. Each plan = 2-3 tasks designed to fit in ~50% of a fresh context window
  5. Uses git worktree isolation per milestone
- **[gsd-2](https://github.com/gsd-build/gsd-2)** — V2 adds meta-prompting, context engineering, and long autonomous sessions

**Why it matters**: Solves the context window problem by breaking work into right-sized chunks with full specs.

---

## 5. Slash Commands & Hooks

- **[Claude-Command-Suite](https://github.com/qdhenry/Claude-Command-Suite)** — Professional commands for code review, security auditing, architectural analysis
- **[ChrisWiles/claude-code-showcase](https://github.com/ChrisWiles/claude-code-showcase)** — Full example project with hooks, skills, agents, commands, GitHub Actions
- **Hook types available**: `PreToolUse`, `PostToolUse`, `Notification`, `UserPromptSubmit`, `Stop`, `TaskCompleted`
- **Use case examples**:
  - Auto-lint on every file write
  - Block dangerous bash commands
  - Send Slack notifications on task completion
  - Auto-commit after each milestone

---

## 6. CI/CD & GitHub Automation

- **[claude-code-action](https://github.com/anthropics/claude-code-action)** — Official GitHub Action. Run Claude Code in CI/CD, analyze diffs, post findings to PRs
- Supports Anthropic API, AWS Bedrock, Google Vertex, Microsoft Foundry
- **Workflow recipes**:
  - Automated PR review with security focus (OWASP)
  - Scheduled maintenance and dependency updates
  - Issue triage and auto-labeling
  - Documentation sync on code changes

---

## 7. MCP Servers Worth Using

| Server | Purpose |
|--------|---------|
| **Context7** | Live, version-accurate docs for any framework |
| **Firecrawl** | Web scraping, crawling, structured data extraction |
| **Desktop Commander** | Background processes, output capture without context consumption |
| **Playwright** | Browser automation and testing |
| **GitHub MCP** | Repository management |
| **Supabase / PostgreSQL / SQLite** | Natural language database queries |
| **Brave Search / Perplexity** | Web research |
| **Memory MCP** | Knowledge graph-based persistent memory |

**Directories**: [mcp.so](https://mcp.so) (3,000+ servers), [Smithery](https://smithery.ai) (2,200+ servers)

---

## 8. IDE Integrations

- **VS Code** — Official extension with chat panel, checkpoint undo, @-mentions, parallel conversations
- **JetBrains** — Official plugin for IntelliJ, WebStorm, PyCharm
- **[claudecode.nvim](https://github.com/coder/claudecode.nvim)** — Community Neovim plugin (reverse-engineered WebSocket MCP protocol)
- **Xcode** — Native Claude Agent SDK support
- Any terminal-equipped editor works via the CLI directly

---

## 9. Monitoring, Analytics & Cost Tracking

| Tool | What it does |
|------|-------------|
| [Official Analytics Dashboard](https://claude.ai/analytics/claude-code) | Lines accepted, accept rate, PRs created, per-user costs |
| [ccusage](https://ccusage.com/) | Token consumption patterns, expensive conversation identification |
| [Claude-Code-Usage-Monitor](https://github.com/Maciek-roboblog/Claude-Code-Usage-Monitor) | Real-time usage with predictions and warnings |
| [SigNoz Dashboard](https://signoz.io/docs/dashboards/dashboard-templates/claude-code-dashboard/) | OpenTelemetry-based dashboard template |
| OpenTelemetry integration | Pipe to Prometheus + Grafana for custom dashboards |

---

## 10. Power-User Tips & Context Management

From [ykdojo/claude-code-tips](https://github.com/ykdojo/claude-code-tips) (45 tips) and community best practices:

- **Context window discipline**: At 70% context, precision drops. At 85%, hallucinations increase. Use subagents to offload research
- **CLAUDE.md**: Keep under 300 lines. Focus on what a linter can't enforce. Include bash commands, code style, workflow rules
- **Git worktrees** (`--worktree` flag): Isolated worktrees for parallel development without file conflicts
- **Headless mode** (`-p` flag): Run non-interactively from scripts, CI/CD, or cron jobs
- **Compaction**: Customize compaction behavior in CLAUDE.md to preserve critical context
- **Use Gemini CLI as Claude Code's minion**: Delegate cheap research tasks to free-tier models

---

## 11. Agent SDK — Programmatic Claude Code

- **[Agent SDK](https://platform.claude.com/docs/en/agent-sdk/overview)** — Same tools, agent loop, and context management as Claude Code, but programmable in Python and TypeScript
- Powers both Claude Code (terminal) and Claude Cowork (desktop)
- Build custom agents with the exact same capabilities as Claude Code

---

## 12. Wild & Creative Use Cases

- **Film production**: VC produced a 30-second spec ad entirely via Claude Code (scripting, ElevenLabs voiceover, Veo 3 visuals, ffmpeg)
- **Medical software**: CEO built medical imaging software with zero coding background
- **Genome analysis**: Designer reverse-engineered their own genome
- **Sound packs**: Warcraft III Peon notification sounds plugin (3,600 likes in 24 hours, 40+ community sound packs)
- **Overnight agents**: Autonomous agents that self-modify their own behavior while you sleep
- **Non-technical founders**: Launching full startups entirely with Claude Code

---

## Ideas for RUSVEL

Based on this research, high-impact enhancements for RUSVEL's Claude Code workflow:

1. **Multi-agent orchestration** — Use Agent Teams or claude-squad for parallel engine development
2. **GSD-style specs** — Break RUSVEL work into context-window-sized chunks with full specs
3. **Custom slash commands** — `/forge-plan`, `/engine-test`, `/deploy-check` tailored to RUSVEL's workflow
4. **Hooks for safety** — Pre-commit hooks that verify engine isolation (engines never import adapters)
5. **MCP server for RUSVEL** — Expose RUSVEL's own APIs as MCP tools so Claude Code can self-operate the system
6. **Context7 for docs** — Always-current Axum, SvelteKit 5, Tailwind 4 documentation in context
7. **Cost monitoring** — Track token usage across sessions to optimize prompting
8. **Self-building loop** — Claude Code developing RUSVEL while RUSVEL manages Claude Code's workflow (the meta-goal)
