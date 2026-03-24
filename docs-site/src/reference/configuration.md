
## Directory Structure

RUSVEL stores its configuration and data in `~/.rusvel/`:

```
~/.rusvel/
├── active_session       # UUID of the currently active session
├── config.toml          # Global configuration
├── profile.toml         # User profile (name, email, bio)
├── departments.toml     # Department registry overrides (optional)
└── rusvel.db            # SQLite database (WAL mode)
```

## Config Hierarchy

Configuration follows a three-layer cascade. More specific layers override less specific ones:

```
Global config  →  Department config  →  Session config
```

For example, you can set a default model globally, override it for the Code department to use a more capable model, and further override it for a specific session.

## Global Configuration (config.toml)

```toml
# Default LLM model for all departments
model = "claude-sonnet-4-20250514"

# Default effort level: "low", "medium", "high"
effort = "medium"

# API server settings
[server]
host = "0.0.0.0"
port = 3000

# LLM provider configuration
[llm]
default_provider = "ollama"

[llm.ollama]
base_url = "http://localhost:11434"

[llm.openai]
# Key loaded from OPENAI_API_KEY env var
model = "gpt-4o"

[llm.claude]
# Key loaded from ANTHROPIC_API_KEY env var
model = "claude-sonnet-4-20250514"
```

## User Profile (profile.toml)

```toml
name = "Your Name"
email = "you@example.com"
bio = "Solo founder building products with AI"
company = "Your Company"
```

The profile is used by agents for context -- the Content department knows your name for bylines, the GTM department uses your company name in outreach, etc.

## Department Configuration

Department-level config is set via the API or the Settings page in the UI:

```bash
# Set Code department to use high effort and a specific model
curl -X PUT http://localhost:3000/api/dept/code/config \
  -H "Content-Type: application/json" \
  -d '{
    "model": "claude-sonnet-4-20250514",
    "effort": "high",
    "permission_mode": "default",
    "add_dirs": ["."]
  }'
```

### Department Config Options

| Key | Type | Description |
|-----|------|------------|
| `model` | string | LLM model to use for this department |
| `effort` | string | Response depth: `low`, `medium`, `high` |
| `permission_mode` | string | Tool permissions: `default`, `restricted` |
| `add_dirs` | string[] | Working directories (for Code department) |
| `system_prompt` | string | Override the department system prompt |
| `max_turns` | number | Max conversation turns per request |

## Environment Variables

| Variable | Purpose |
|----------|---------|
| `ANTHROPIC_API_KEY` | Claude API authentication |
| `OPENAI_API_KEY` | OpenAI API authentication |
| `RUSVEL_DB_PATH` | Override database file path |
| `RUSVEL_CONFIG_DIR` | Override config directory (default: `~/.rusvel`) |
| `RUST_LOG` | Logging level (e.g., `info`, `debug`, `rusvel_api=debug`) |

## Department Registry (departments.toml)

The department registry can be customized by placing a `departments.toml` file in `~/.rusvel/`. This is optional -- RUSVEL has built-in defaults for all 12 departments.

Each department block defines:

```toml
[[department]]
id = "custom"
name = "Custom"
title = "Custom Department"
engine_kind = "Forge"
icon = "+"
color = "blue"
system_prompt = "You are a custom department agent..."
capabilities = ["custom_capability"]
tabs = ["actions", "agents", "events"]

[[department.quick_actions]]
label = "Do Something"
prompt = "Execute the custom task..."
```

## Database

RUSVEL uses SQLite in WAL (Write-Ahead Logging) mode for concurrent reads. The database file is at `~/.rusvel/rusvel.db` by default.

The database stores: sessions, goals, events, conversations, agents, skills, rules, workflows, hooks, MCP server configs, and analytics data. Migrations run automatically on startup.
