
## Overview

The Flow department manages DAG-based workflows -- directed acyclic graphs of code, condition, and agent nodes executed via petgraph. It lets you compose multi-step automations that chain department actions, conditional logic, and AI agent calls into repeatable pipelines.

## Quick Actions

| Action | What It Does |
|--------|-------------|
| **Create flow** | Define a new DAG workflow with nodes and edges |
| **Run flow** | Execute a workflow and track node-by-node progress |
| **List executions** | Show past and in-progress workflow runs |

## Example Prompts

- "Create a workflow that analyzes code then drafts a blog post from the results."
- "Show me all running flow executions."
- "Retry the failed node in my last workflow run."
- "List available node types for building flows."

## Node Types

- **Code** -- execute a code snippet or shell command
- **Condition** -- branch based on a boolean expression
- **Agent** -- invoke an AI agent with a prompt

The `parallel_evaluate` node type is available when `RUSVEL_FLOW_PARALLEL_EVALUATE=1` is set.

## API Routes

See the [API reference](/reference/api/) for full details under the **Flow engine (DAG)** section, including create, run, resume, retry, and checkpoint endpoints.

## Tabs

Actions, Agents, Rules, Events
