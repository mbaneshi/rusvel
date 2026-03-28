# Engine-Specific Testing

Each wired engine has dedicated API endpoints and CLI commands beyond the generic department CRUD.

## Code Engine

```bash
# Analyze codebase
curl -s -X POST http://localhost:3000/api/dept/code/analyze \
  -H 'Content-Type: application/json' \
  -d '{"path": "."}' | jq .
```

**Expected:** Returns analysis with file count, LOC, complexity metrics.

```bash
# Search symbols
curl -s "http://localhost:3000/api/dept/code/search?query=DepartmentApp" | jq .
```

**Expected:** BM25 ranked results with file paths and line numbers.

```bash
# CLI equivalents
cargo run -- code analyze .
cargo run -- code search "DepartmentApp"
```

## Content Engine

```bash
# Draft content (requires Ollama)
curl -s -X POST http://localhost:3000/api/dept/content/draft \
  -H 'Content-Type: application/json' \
  -d '{"topic": "Building CLI tools in Rust"}' | jq .
```

**Expected:** Returns markdown draft with title, body, metadata.

```bash
# Code-to-content pipeline
curl -s -X POST http://localhost:3000/api/dept/content/from-code \
  -H 'Content-Type: application/json' \
  -d '{"path": ".", "topic": "architecture"}' | jq .
```

**Expected:** Analyzes code first, then generates content based on analysis.

```bash
# List content
curl -s http://localhost:3000/api/dept/content/list | jq .
```

**Expected:** Array of content items with status (Draft, Adapted, Scheduled, Published).

```bash
# Approve content
curl -s -X PATCH http://localhost:3000/api/dept/content/<id>/approve | jq .
```

**Expected:** Content status changes. Approval status updated.

```bash
# Schedule content
curl -s -X POST http://localhost:3000/api/dept/content/schedule \
  -H 'Content-Type: application/json' \
  -d '{"content_id": "<id>", "scheduled_at": "2026-04-01T09:00:00Z"}' | jq .
```

**Expected:** Content marked as Scheduled with timestamp.

```bash
# Publish content
curl -s -X POST http://localhost:3000/api/dept/content/publish \
  -H 'Content-Type: application/json' \
  -d '{"content_id": "<id>"}' | jq .
```

**Expected:** Content published (or job created with approval gate).

```bash
# List scheduled content
curl -s http://localhost:3000/api/dept/content/scheduled | jq .
```

```bash
# CLI
cargo run -- content draft "test topic"
```

## Harvest Engine

```bash
# Scan for opportunities (requires Ollama)
curl -s -X POST http://localhost:3000/api/dept/harvest/scan | jq .
```

**Expected:** Scans configured sources, returns discovered opportunities.

```bash
# Score an opportunity
curl -s -X POST http://localhost:3000/api/dept/harvest/score \
  -H 'Content-Type: application/json' \
  -d '{"opportunity_id": "<id>"}' | jq .
```

**Expected:** Returns score (0.0-1.0) with scoring rationale.

```bash
# Generate proposal
curl -s -X POST http://localhost:3000/api/dept/harvest/proposal \
  -H 'Content-Type: application/json' \
  -d '{"opportunity_id": "<id>"}' | jq .
```

**Expected:** AI-generated proposal draft (approval-gated).

```bash
# Pipeline stats
curl -s http://localhost:3000/api/dept/harvest/pipeline | jq .
```

**Expected:** Counts by stage (Cold, Contacted, Qualified, ProposalSent, Won, Lost).

```bash
# List opportunities
curl -s http://localhost:3000/api/dept/harvest/list | jq .
```

```bash
# Advance opportunity
curl -s -X POST http://localhost:3000/api/dept/harvest/advance \
  -H 'Content-Type: application/json' \
  -d '{"opportunity_id": "<id>", "stage": "Contacted"}' | jq .
```

**Expected:** Opportunity stage updated.

```bash
# Record outcome
curl -s -X POST http://localhost:3000/api/dept/harvest/outcome \
  -H 'Content-Type: application/json' \
  -d '{"opportunity_id": "<id>", "outcome": "Won", "value": 5000.0}' | jq .
```

**Expected:** Outcome recorded with value.

```bash
# List outcomes
curl -s http://localhost:3000/api/dept/harvest/outcomes | jq .
```

```bash
# CLI
cargo run -- harvest pipeline
```

## GTM Engine

```bash
# Create contact
curl -s -X POST http://localhost:3000/api/dept/gtm/contacts \
  -H 'Content-Type: application/json' \
  -d '{"name": "Alice", "emails": ["alice@example.com"], "company": "Acme"}' | jq .
```

**Expected:** Contact created with ID.

```bash
# List contacts
curl -s http://localhost:3000/api/dept/gtm/contacts | jq .

# List deals
curl -s http://localhost:3000/api/dept/gtm/deals | jq .

# Advance deal
curl -s -X POST http://localhost:3000/api/dept/gtm/deals/advance \
  -H 'Content-Type: application/json' \
  -d '{"deal_id": "<id>", "stage": "Qualified"}' | jq .
```

### Outreach Sequences

```bash
# Create sequence
curl -s -X POST http://localhost:3000/api/dept/gtm/outreach/sequences \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "cold-intro",
    "steps": [
      {"kind": "email", "template": "Hi {{name}}, ..."},
      {"kind": "follow_up", "delay_days": 3, "template": "Following up..."}
    ]
  }' | jq .

# List sequences
curl -s http://localhost:3000/api/dept/gtm/outreach/sequences | jq .

# Activate sequence
curl -s -X POST http://localhost:3000/api/dept/gtm/outreach/sequences/<id>/activate | jq .

# Execute outreach (approval-gated)
curl -s -X POST http://localhost:3000/api/dept/gtm/outreach/execute | jq .
```

### Invoicing

```bash
# Create invoice
curl -s -X POST http://localhost:3000/api/dept/gtm/invoices \
  -H 'Content-Type: application/json' \
  -d '{
    "contact_id": "<id>",
    "amount": 5000.00,
    "description": "Consulting services",
    "due_date": "2026-04-15"
  }' | jq .

# Get invoice
curl -s http://localhost:3000/api/dept/gtm/invoices/<id> | jq .

# List invoices
curl -s http://localhost:3000/api/dept/gtm/invoices | jq .

# Update status
curl -s -X POST http://localhost:3000/api/dept/gtm/invoices/<id>/status \
  -H 'Content-Type: application/json' \
  -d '{"status": "sent"}' | jq .
```

## Flow Engine (DAG Workflows)

### CRUD

```bash
# Create a flow
curl -s -X POST http://localhost:3000/api/flows \
  -H 'Content-Type: application/json' \
  -d '{
    "name": "test-flow",
    "nodes": [
      {"id": "start", "type": "code", "config": {"command": "echo start"}},
      {"id": "check", "type": "condition", "config": {"expression": "true"}},
      {"id": "end", "type": "code", "config": {"command": "echo done"}}
    ],
    "edges": [
      {"from": "start", "to": "check"},
      {"from": "check", "to": "end"}
    ]
  }' | jq .
```

**Expected:** Flow created with ID. DAG is validated (no cycles).

```bash
curl -s http://localhost:3000/api/flows | jq .
curl -s http://localhost:3000/api/flows/<id> | jq .
curl -s -X PUT http://localhost:3000/api/flows/<id> \
  -H 'Content-Type: application/json' \
  -d '{"name": "updated-flow"}' | jq .
curl -s -X DELETE http://localhost:3000/api/flows/<id>
```

### Execution

```bash
# Run a flow
curl -s -X POST http://localhost:3000/api/flows/<id>/run | jq .
```

**Expected:** Returns execution ID. Nodes execute in topological order.

```bash
# List executions
curl -s http://localhost:3000/api/flows/<id>/executions | jq .

# Get execution details
curl -s http://localhost:3000/api/flows/executions/<exec-id> | jq .

# Get checkpoint
curl -s http://localhost:3000/api/flows/executions/<exec-id>/checkpoint | jq .

# Resume suspended execution
curl -s -X POST http://localhost:3000/api/flows/executions/<exec-id>/resume | jq .

# Retry a failed node
curl -s -X POST http://localhost:3000/api/flows/executions/<exec-id>/retry/<node-id> | jq .
```

### Node Types

```bash
curl -s http://localhost:3000/api/flows/node-types | jq .
```

**Expected:** `["code", "condition", "agent"]`. If `RUSVEL_FLOW_PARALLEL_EVALUATE=1`, also includes `parallel_evaluate`.

### Templates

```bash
curl -s http://localhost:3000/api/flows/templates/cross-engine-handoff | jq .
```

**Expected:** Pre-built flow template showing cross-engine data handoff.

## Capability Engine (!build)

### Via API

```bash
curl -s -X POST http://localhost:3000/api/capability/build \
  -H 'Content-Type: application/json' \
  -d '{"description": "A GitHub PR review agent"}' | jq .
```

**Expected:** Returns bundle of created entities (agent, skills, rules, hooks, MCP config).

### Via Chat

```bash
curl -N http://localhost:3000/api/dept/forge/chat \
  -H 'Content-Type: application/json' \
  -d '{"message": "!build a daily standup summary agent"}'
```

**Expected:** Agent detects `!build` prefix, invokes capability engine, creates entities, reports what was installed.
