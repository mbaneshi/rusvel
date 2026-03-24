
## What Are Agents?

Agents are AI personas with specific roles, instructions, tools, and budgets. Each department has a default agent, and you can create custom agents for specialized tasks.

Agents are powered by the `AgentPort` runtime, which wraps LLM access, tool execution, and memory into a single orchestration layer.

## Agent Profile Structure

Every agent has:

```
name            — Display name (e.g., "rust-engine")
role            — One-line role description
instructions    — System prompt guiding behavior
model           — Which LLM to use (provider + model name)
allowed_tools   — List of tools the agent can invoke
capabilities    — What the agent can do (code_analysis, content_creation, etc.)
budget_limit    — Optional spending cap per run
department      — Which department this agent belongs to
```

## Seeded Agents

RUSVEL ships with 5 pre-configured agents:

| Agent | Department | Role |
|-------|-----------|------|
| **rust-engine** | Code | Rust development -- writes, tests, and refactors Rust code |
| **svelte-ui** | Code | Frontend development -- SvelteKit components and styling |
| **test-writer** | Code | Writes unit and integration tests for existing code |
| **content-writer** | Content | Drafts blog posts, social media content, and documentation |
| **proposal-writer** | Harvest | Writes proposals for freelance opportunities |

## Creating Custom Agents

### Via the Web UI

1. Navigate to any department
2. Open the **Agents** tab in the department panel
3. Click **"Add Agent"**
4. Fill in name, role, instructions, and model
5. Save

### Via the API

```bash
curl -X POST http://localhost:3000/api/agents \
  -H "Content-Type: application/json" \
  -d '{
    "name": "seo-analyst",
    "role": "SEO and keyword research specialist",
    "instructions": "You analyze websites for SEO performance...",
    "model": "claude-sonnet-4-20250514",
    "department": "distro",
    "capabilities": ["seo"]
  }'
```

### Managing Agents

```bash
# List all agents
curl http://localhost:3000/api/agents

# Get a specific agent
curl http://localhost:3000/api/agents/<id>

# Update an agent
curl -X PUT http://localhost:3000/api/agents/<id> \
  -H "Content-Type: application/json" \
  -d '{"instructions": "Updated instructions..."}'

# Delete an agent
curl -X DELETE http://localhost:3000/api/agents/<id>
```

## Model Selection

Agents can use any configured LLM provider:

| Provider | Example Models |
|----------|---------------|
| Ollama | `llama3.1`, `mistral`, `codellama` |
| Claude API | `claude-sonnet-4-20250514`, `claude-opus-4-20250514` |
| OpenAI | `gpt-4o`, `gpt-4o-mini` |

The model is set per agent and falls back to the department default, then the global default.

## Agent Budgets

Set a `budget_limit` to cap how much an agent can spend per run. This is useful for expensive operations like long code generation sessions.

When the budget is exceeded, the agent pauses and requests approval to continue.

## How Agents Execute

When you chat with a department, here is what happens:

1. Your message goes to the department's agent
2. The agent runtime loads the system prompt (department prompt + active rules)
3. The LLM generates a response, optionally calling tools
4. Tool results are fed back to the LLM for follow-up
5. The final response is returned to the chat
6. An event is logged with tokens used and tools called

Engines never call the LLM directly (ADR-009). They always go through the AgentPort, which handles prompt construction, tool routing, retries, and memory injection.
