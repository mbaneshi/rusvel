---
title: Workflows
description: Chain agents together for multi-step automated tasks.
---

## What Are Workflows?

Workflows chain multiple agents together to accomplish multi-step tasks. Instead of manually prompting one agent at a time, you define a flow that routes work between agents automatically.

## Workflow Patterns

RUSVEL supports four execution patterns:

### Sequential

Agents run one after another. Each agent's output feeds into the next as input.

```
Agent A → Agent B → Agent C → Result
```

**Example:** Research a topic (Agent A) > write a blog post (Agent B) > adapt for Twitter (Agent C).

### Parallel

Multiple agents run simultaneously. Results are collected and merged.

```
Agent A ─┐
Agent B ─┤→ Merge → Result
Agent C ─┘
```

**Example:** Analyze code quality (Agent A), run security scan (Agent B), check dependencies (Agent C) -- all at once.

### Loop

An agent runs repeatedly until a condition is met or a maximum iteration count is reached.

```
Agent A → Check → (repeat or done)
```

**Example:** Iterate on a draft until it meets quality criteria.

### Graph

A directed acyclic graph of agents with conditional branching. The most flexible pattern.

```
Agent A → [if pass] → Agent B → Agent D
       → [if fail] → Agent C → Agent D
```

**Example:** Analyze an opportunity, route to different proposal strategies based on the score.

## Creating Workflows

### Via the Web UI

1. Navigate to a department that supports workflows (Forge, Code, GTM)
2. Open the **Workflows** tab (labeled "Flows")
3. Click **"Add Workflow"**
4. Define the steps, agents, and pattern
5. Save

### Via the API

```bash
# Create a workflow
curl -X POST http://localhost:3000/api/workflows \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Blog Pipeline",
    "department": "content",
    "pattern": "sequential",
    "steps": [
      {"agent": "content-writer", "prompt": "Research and outline: {topic}"},
      {"agent": "content-writer", "prompt": "Write full draft from outline"},
      {"agent": "content-writer", "prompt": "Adapt for Twitter thread"}
    ]
  }'

# List workflows
curl http://localhost:3000/api/workflows

# Get a specific workflow
curl http://localhost:3000/api/workflows/<id>
```

## Running Workflows

### Via the API

```bash
curl -X POST http://localhost:3000/api/workflows/<id>/run \
  -H "Content-Type: application/json" \
  -d '{"variables": {"topic": "Hexagonal Architecture in Rust"}}'
```

The workflow executes each step in order, passing context between agents. Results are streamed back as events.

## Workflow Variables

Workflows accept variables at execution time. Use `{variable_name}` in step prompts:

```json
{
  "steps": [
    {"prompt": "Analyze the {language} codebase in {directory}"},
    {"prompt": "Write tests for any uncovered functions"}
  ]
}
```

When running, provide the variables:

```json
{"variables": {"language": "Rust", "directory": "crates/rusvel-core"}}
```

## Managing Workflows

```bash
# Update a workflow
curl -X PUT http://localhost:3000/api/workflows/<id> \
  -H "Content-Type: application/json" \
  -d '{"name": "Updated Pipeline", "steps": [...]}'

# Delete a workflow
curl -X DELETE http://localhost:3000/api/workflows/<id>
```
