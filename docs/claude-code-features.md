# Claude Code — Complete Feature Reference

> Every capability of the CLI tool, organized by category.
> Last updated: 2026-03-23

---

## 1. Core Modes

| Mode | Invocation | Description |
|------|-----------|-------------|
| **Interactive REPL** | `claude` | Multi-turn conversation with streaming, history, context preservation |
| **One-Shot (Print)** | `claude -p "query"` | Non-interactive, outputs result and exits — ideal for scripting |
| **Pipe** | `cat file \| claude -p` | Accept stdin, process, write stdout — Unix-friendly |
| **Continue** | `claude -c` | Resume most recent conversation |
| **Resume** | `claude -r "session"` | Resume a specific named session |
| **Bash escape** | `! command` | Run shell directly inside session, output added to context |

---

## 2. Slash Commands (~60+)

### Session

| Command | Description |
|---------|-------------|
| `/resume`, `/continue` | Resume previous conversations |
| `/branch`, `/fork` | Create conversation branches |
| `/rename` | Name the current session |
| `/clear`, `/reset` | Clear history, start fresh |
| `/rewind`, `/checkpoint` | Restore to a previous conversation+code state |
| `/exit`, `/quit` | Exit Claude Code |

### Context & Output

| Command | Description |
|---------|-------------|
| `/context` | Visualize context window usage |
| `/compact` | Compress conversation to reclaim context |
| `/copy` | Copy last response to clipboard |
| `/diff` | Interactive diff viewer |
| `/export` | Export conversation as text |

### Configuration

| Command | Description |
|---------|-------------|
| `/config`, `/settings` | Open settings interface |
| `/model` | Switch model (opus/sonnet/haiku) |
| `/effort` | Set effort level (low/medium/high/max) |
| `/theme` | Color themes (dark/light/colorblind) |
| `/keybindings` | Customize keyboard shortcuts |
| `/terminal-setup` | Configure terminal keybindings |
| `/vim` | Toggle Vim editing mode |
| `/color` | Set session prompt bar color |
| `/permissions` | View/update permission rules |
| `/sandbox` | Toggle sandbox mode |

### AI & Agents

| Command | Description |
|---------|-------------|
| `/plan` | Enter plan mode (read-only analysis) |
| `/memory` | View/edit CLAUDE.md and auto-memory |
| `/agents` | Manage subagents |
| `/skills` | List available skills |
| `/mcp` | Manage MCP server connections |
| `/plugin` | Manage plugins |

### Development & Integration

| Command | Description |
|---------|-------------|
| `/help` | Show available commands |
| `/doctor` | Diagnose and verify installation |
| `/hooks` | View hook configurations |
| `/ide` | Manage IDE integrations |
| `/pr-comments` | Fetch PR comments from GitHub |
| `/install-github-app` | Set up GitHub Actions integration |
| `/install-slack-app` | Install Slack integration |
| `/security-review` | Analyze code for vulnerabilities |
| `/add-dir` | Add working directories to session |

### Account & Usage

| Command | Description |
|---------|-------------|
| `/cost` | Show token usage statistics |
| `/usage` | Show plan usage limits |
| `/stats` | Visualize usage and streaks |
| `/status` | Show version, model, account info |
| `/login` / `/logout` | Sign in/out |
| `/feedback`, `/bug` | Submit feedback/bug reports |
| `/release-notes` | View changelog |
| `/insights` | Generate session analysis report |
| `/upgrade` | Switch to higher plan |

### Tasks & Automation

| Command | Description |
|---------|-------------|
| `/tasks` | List and manage background tasks |
| `/loop` | Schedule recurring prompts (e.g. `/loop 5m /check`) |
| `/btw` | Ask side questions without polluting history |
| `/fast` | Toggle fast mode (cost/speed tradeoff) |

### Misc

| Command | Description |
|---------|-------------|
| `/voice` | Toggle voice dictation |
| `/chrome` | Configure Chrome browser integration |
| `/remote-control`, `/rc` | Enable remote control from desktop |
| `/desktop`, `/app` | Continue session in desktop app |
| `/mobile` | Download mobile app |
| `/passes` | Share free week with friends |
| `/stickers` | Order Claude Code stickers |

---

## 3. Keyboard Shortcuts

### General

| Shortcut | Action |
|----------|--------|
| `Ctrl+C` | Cancel current operation |
| `Ctrl+D` | Exit Claude Code |
| `Ctrl+G` | Open prompt in external editor |
| `Ctrl+L` | Clear terminal screen |
| `Ctrl+O` | Toggle verbose output |
| `Ctrl+V` / `Cmd+V` | Paste image from clipboard |
| `Ctrl+B` | Background running tasks |
| `Ctrl+T` | Toggle task list |
| `Ctrl+R` | Reverse search command history |
| `Esc+Esc` | Rewind/summarize |
| `Shift+Tab` / `Alt+M` | Toggle permission modes |
| `Option+P` / `Alt+P` | Switch model |
| `Option+T` / `Alt+T` | Toggle extended thinking |

### Text Editing

| Shortcut | Action |
|----------|--------|
| `Ctrl+K` | Delete to end of line |
| `Ctrl+U` | Delete entire line |
| `Ctrl+Y` | Paste deleted text |
| `Alt+Y` | Cycle paste history |
| `Alt+B` | Move cursor back one word |
| `Alt+F` | Move cursor forward one word |

### Multiline Input

| Shortcut | Terminal |
|----------|---------|
| `\` + Enter | All terminals |
| `Option+Enter` | macOS default |
| `Shift+Enter` | iTerm2, WezTerm, Ghostty, Kitty |
| `Ctrl+J` | Line feed character |

---

## 4. Built-in Tools

### File Operations

| Tool | Description |
|------|-------------|
| **Read** | Read files (code, images, PDFs up to 20 pages, Jupyter notebooks) |
| **Write** | Create or overwrite files |
| **Edit** | Targeted string-replacement edits |
| **Glob** | Find files by glob pattern |
| **Grep** | Regex content search (ripgrep-based) |

### Code Execution

| Tool | Description |
|------|-------------|
| **Bash** | Execute shell commands with timeout, background support |
| **NotebookEdit** | Modify Jupyter notebook cells |
| **LSP** | Code intelligence via language servers |

### Web & External

| Tool | Description |
|------|-------------|
| **WebFetch** | Fetch content from URLs |
| **WebSearch** | Perform web searches |
| **ListMcpResourcesTool** | List MCP server resources |
| **ReadMcpResourceTool** | Read MCP resource content |

### Task Management

| Tool | Description |
|------|-------------|
| **TaskCreate** | Create background tasks |
| **TaskGet/TaskList** | Retrieve task details or list |
| **TaskOutput** | Get background task output |
| **TaskStop** | Kill running tasks |
| **TaskUpdate** | Update task status |

### Agents & Planning

| Tool | Description |
|------|-------------|
| **Agent** | Spawn specialized subagents |
| **EnterPlanMode / ExitPlanMode** | Switch to/from plan mode |
| **EnterWorktree / ExitWorktree** | Create/exit isolated git worktrees |
| **Skill** | Execute skills |
| **ToolSearch** | Search and load deferred tools on demand |
| **AskUserQuestion** | Prompt user for input |

### Scheduling

| Tool | Description |
|------|-------------|
| **CronCreate** | Schedule recurring tasks |
| **CronDelete** | Cancel scheduled tasks |
| **CronList** | List scheduled tasks |

---

## 5. CLI Flags & Options

### Session Control

```
claude                            # Interactive session
claude "query"                    # Interactive with initial prompt
claude -p "query"                 # One-shot print mode
claude -c                         # Continue most recent
claude -r "session"               # Resume specific session
claude --resume                   # Open session picker
claude --fork-session             # Branch a session
claude --from-pr <number>         # Resume PR-linked session
--name, -n                        # Name the session
--session-id                      # Use specific UUID
--worktree, -w                    # Create isolated git worktree
--remote                          # Create web session on claude.ai
--remote-control, --rc            # Enable remote control
--teleport                        # Resume web session in terminal
```

### Model & Performance

```
--model                           # Select model (opus/sonnet/haiku or full ID)
--effort                          # Effort level (low/medium/high/max)
--fallback-model                  # Auto-fallback when overloaded
--max-turns                       # Limit agentic turns
--max-budget-usd                  # Spending cap
--chrome / --no-chrome            # Enable/disable Chrome automation
```

### Configuration & Context

```
--add-dir                         # Add additional working directories
--agent                           # Specify a subagent
--agents                          # Define custom subagents (JSON)
--tools                           # Restrict available tools
--allowedTools                    # Tools that don't prompt
--disallowedTools                 # Tools to remove
--permission-mode                 # plan/default/acceptEdits/dontAsk
--dangerously-skip-permissions    # Skip all permission prompts
--settings                        # Load settings JSON file
--append-system-prompt            # Append to system prompt
--system-prompt                   # Replace entire system prompt
--mcp-config                      # Load MCP servers from files
--strict-mcp-config               # Use only specified MCP config
--plugin-dir                      # Load plugins from directory
```

### Input/Output

```
--output-format                   # text | json | stream-json
--input-format                    # text | stream-json
--json-schema                     # Get validated JSON output
--include-partial-messages        # Include streaming events
--no-session-persistence          # Don't save sessions
--verbose                         # Full turn-by-turn output
--bare                            # Minimal mode, skip auto-discovery
--disable-slash-commands          # Disable skills and commands
```

### Other

```
--init / --init-only              # Run initialization hooks
--maintenance                     # Run maintenance hooks
--ide                             # Auto-connect to IDE
--betas                           # Include beta headers
--debug                           # Enable debug with category filtering
--version, -v                     # Show version
--update                          # Update to latest version
```

---

## 6. Memory System

### CLAUDE.md (Persistent Instructions)

- Project instructions checked into repo
- Multiple scope levels: managed policy → project → user
- Auto-discovered in directory hierarchy (walks up to root)
- Nested CLAUDE.md in subdirectories
- File imports with `@path/to/file` syntax (up to 5 hops)
- Organization-wide managed CLAUDE.md

### Rules System (`.claude/rules/`)

- Modular topic-specific rule files
- Path-specific rules with glob patterns (e.g. `*.test.ts` → "always use vitest")
- User-level rules in `~/.claude/rules/`
- Symlink support for sharing across repos

### Auto Memory (`~/.claude/projects/`)

- Claude takes notes on preferences across sessions
- 200-line `MEMORY.md` index
- Topic-specific memory files
- Per-project memory directory
- Machine-local, not cloud-synced
- `/memory` to view and edit

---

## 7. Subagents (Specialized AI Workers)

### Built-in Subagents

| Agent | Description |
|-------|-------------|
| **Explore** | Fast read-only codebase search |
| **Plan** | Research agent for plan mode |
| **General-purpose** | Complex multi-step tasks |
| **Bash** | Separate shell execution context |
| **statusline-setup** | Status line configuration |
| **claude-code-guide** | Feature documentation queries |

### Custom Subagents

- Create via `/agents` interactive interface
- File-based definition (YAML frontmatter + markdown prompt)
- CLI-defined via `--agents` JSON flag
- Plugin-provided subagents
- Scoped per user, project, or session

### Subagent Features

- Custom system prompts and tool restrictions
- MCP server scoping per subagent
- Background execution with `run_in_background`
- Worktree isolation (`isolation: "worktree"`)
- Model selection per subagent
- Max turn limits
- Auto-delegation based on description matching
- `@agent-name` mention invocation
- Hook support (SubagentStart/SubagentStop)
- Context isolation from parent

---

## 8. Hooks System (15+ Lifecycle Events)

### Hook Events

| Event | When |
|-------|------|
| `SessionStart` | Session begins |
| `InstructionsLoaded` | After CLAUDE.md loads |
| `UserPromptSubmit` | Before processing user input |
| `PreToolUse` | Before tool execution |
| `PostToolUse` | After successful tool use |
| `PostToolUseFailure` | After tool failure |
| `PermissionRequest` | Before permission prompt |
| `Notification` | When notification fires |
| `SubagentStart` | Subagent launches |
| `SubagentStop` | Subagent finishes |
| `Stop` | Claude stops responding |
| `PreCompact` | Before context compaction |
| `PostCompact` | After context compaction |
| `SessionEnd` | Session ends |
| `WorktreeCreate/Remove` | Worktree lifecycle |
| `ConfigChange` | Settings change |
| `TaskCompleted` | Background task finishes |

### Hook Types

| Type | Description |
|------|-------------|
| **Command** | Execute shell scripts |
| **HTTP** | Call external webhooks |
| **Prompt** | Use Claude as hook logic |
| **Agent** | Use subagent as hook logic |

### Common Uses

- Auto-format code after edits (prettier, rustfmt)
- Run linters/tests automatically after changes
- Block edits to protected files
- Desktop notifications on completion
- Audit configuration changes
- Re-inject context after compaction
- Auto-approve specific permissions

---

## 9. Skills (Reusable Workflows)

### Bundled Skills

| Skill | Description |
|-------|-------------|
| `/simplify` | Code review for reuse, quality, efficiency |
| `/batch` | Research and execute large changes |
| `/debug` | Enable debug logging |
| `/loop` | Run prompts on recurring intervals |
| `/claude-api` | Build with Claude API/SDKs |
| `/init` | Initialize CLAUDE.md for a project |
| `/pr-comments` | Fetch PR comments |
| `/security-review` | Security vulnerability analysis |
| `/insights` | Session analysis report |

### Custom Skills

- Markdown files with YAML frontmatter
- Invoke with `/skillname`
- Auto-discovered in `.claude/skills/` or `~/.claude/skills/`
- Context-specific and namespace-isolated in plugins
- Tool restrictions and user access control

---

## 10. MCP (Model Context Protocol)

### Transport Types

| Type | Description |
|------|-------------|
| **stdio** | Local command execution |
| **HTTP** | HTTP/HTTPS endpoints |
| **SSE** | Server-sent events |
| **WebSocket** | WS/WSS connections |

### Configuration

- `.mcp.json` files at project, user, or local scope
- Environment variable expansion
- OAuth authentication support
- Tool search with deferred loading (10% context limit)
- Managed MCP policies for organizations

### Popular Servers

GitHub, Slack, PostgreSQL, Sentry, Gmail, Google Calendar, Notion, Canva, Playwright (browser), and 100+ more.

---

## 11. Settings & Configuration

### Settings Hierarchy (highest → lowest priority)

1. **Managed Policy** — organization-enforced
2. **User** — `~/.claude/settings.json`
3. **Project** — `.claude/settings.json`
4. **Local** — `.claude/settings.local.json` (machine-only, gitignored)

### Key Settings

- Model selection and restrictions
- Permission rules (allow/deny with glob patterns)
- Default effort level
- Sandbox configuration
- Environment variables
- Hook definitions
- Plugin enablement
- System prompt overrides
- Extended thinking defaults
- Auto-memory on/off

### Permission Modes

| Mode | Description |
|------|-------------|
| `plan` | Read-only, no changes |
| `default` | Prompt for risky operations |
| `acceptEdits` | Auto-approve file edits, prompt for bash |
| `dontAsk` | Auto-approve everything (use with caution) |
| `bypassPermissions` | Skip all prompts (requires flag) |

---

## 12. IDE Integrations

### VS Code Extension

- Prompt box in editor sidebar
- File/folder references via `@`
- Session resumption and multiple conversations
- Terminal integration
- Git operations and Chrome automation
- Plugin management and configuration UI
- Built-in IDE MCP server

### JetBrains Plugin

- IDE detection and remote development support
- WSL configuration
- ESC key binding options
- External terminal access

---

## 13. Extended Thinking

- Enabled by default on Opus 4.6 and Sonnet 4.6
- Configurable thinking budget via effort level
- Toggle with `Option/Alt+T`
- Verbose mode (`Ctrl+O`) to see reasoning
- Effort levels: low (skip thinking) → max (deep reasoning)

---

## 14. Git Integration

- Git status awareness at session start
- Automatic branch detection
- Worktree creation (`--worktree` or `-w`)
- PR status display in session picker (green/yellow/red/gray)
- PR linking for session resumption (`--from-pr`)
- Commit creation with Co-Authored-By
- PR comment fetching
- Non-git VCS support via custom hooks

---

## 15. File & Media Handling

| Format | Support |
|--------|---------|
| **Code** | All major languages |
| **Images** | PNG, JPG, WEBP — paste from clipboard (`Ctrl+V`) or path reference |
| **PDFs** | Read with page range selection (max 20 pages per request) |
| **Notebooks** | Jupyter `.ipynb` — read all cells + edit via NotebookEdit |
| **Config** | JSON, YAML, TOML, INI, etc. |
| **Screenshots** | Full visual analysis |

---

## 16. Cloud & Deployment

### Cloud Execution

- Claude Code on the Web (claude.ai)
- Remote Control from desktop
- Default Docker image for cloud environments
- Setup scripts and environment persistence

### Third-Party Providers

| Provider | Description |
|----------|-------------|
| **Amazon Bedrock** | AWS-hosted Claude |
| **Google Vertex AI** | GCP-hosted Claude |
| **Microsoft Foundry** | Azure-hosted Claude |
| **LiteLLM** | Gateway proxy |
| **Custom gateways** | Proxy, mTLS, custom CA certs |

### CI/CD Integration

- GitHub Actions (via GitHub App)
- GitLab CI/CD
- Claude comments on PRs automatically
- Code review in CI pipelines

---

## 17. Security & Privacy

### Built-in Security

- Permission-based architecture (nothing runs without approval by default)
- Sandbox isolation (filesystem + network)
- Prompt injection detection and flagging
- Secure development practices (OWASP-aware)

### Enterprise Features

- Zero Data Retention (ZDR)
- BAA (Business Associate Agreement)
- Data governance controls
- Authentication policies and SSO
- Audit logging
- Managed policy enforcement

### Privacy Controls

- `/privacy-settings` — view and control data usage
- Feedback opt-out
- Development Partner Program opt-out
- Telemetry controls

---

## 18. Performance & Cost

| Feature | Description |
|---------|-------------|
| `/cost` | Token usage and cost tracking |
| `--max-budget-usd` | Spending cap per session |
| `/fast` | Toggle fast mode (lower cost) |
| `--effort low` | Minimal thinking for speed |
| `--bare` | Skip auto-discovery for fast startup |
| `--fallback-model` | Auto-switch on rate limits |
| Prompt caching | Automatic for repeated context |
| Auto-compaction | At ~95% context capacity |
| Subagent isolation | Separate context windows |

---

## 19. Agent Teams (Experimental)

- Multiple independent Claude sessions working in parallel
- Shared task lists for coordination
- Peer-to-peer messaging between teammates
- Team display modes (split view)
- Quality gate enforcement
- Isolated context per teammate

---

## 20. Authentication

| Method | Description |
|--------|-------------|
| Email/password | `claude auth login` |
| SSO | Enterprise single sign-on |
| API key | Via `ANTHROPIC_API_KEY` env var |
| OAuth | For MCP servers |
| Cloud provider | Bedrock/Vertex/Foundry credentials |

### Subscriptions

Free → Claude Pro → Claude Teams → Claude Enterprise → API billing

---

## 21. Debugging & Diagnostics

| Feature | Description |
|---------|-------------|
| `--debug` | Enable debug mode with category filters |
| `--verbose` / `Ctrl+O` | Full turn-by-turn output |
| `/doctor` | Verify installation and diagnose issues |
| `/status` | Session and account information |
| Hook debugging | Inspect hook inputs/outputs |
| Permission debugging | See why tools are blocked |
