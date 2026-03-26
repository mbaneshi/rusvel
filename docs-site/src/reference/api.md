
## Overview

RUSVEL exposes a JSON REST API on port 3000 via Axum. 124 handler functions across 23 modules. All endpoints are prefixed with `/api/`. CORS is enabled for all origins.

## Core

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/health` | Health check, returns server status |
| `GET` | `/api/config` | Get global configuration |
| `PUT` | `/api/config` | Update global configuration |
| `GET` | `/api/config/models` | List available LLM models |
| `GET` | `/api/config/tools` | List available tools |
| `GET` | `/api/analytics` | Get analytics dashboard data |
| `GET` | `/api/profile` | Get user profile |
| `PUT` | `/api/profile` | Update user profile |

## Sessions

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/sessions` | List all sessions |
| `POST` | `/api/sessions` | Create a new session |
| `GET` | `/api/sessions/{id}` | Get a session by ID |
| `GET` | `/api/sessions/{id}/mission/today` | Generate daily plan for session |
| `GET` | `/api/sessions/{id}/mission/goals` | List goals for session |
| `POST` | `/api/sessions/{id}/mission/goals` | Create a goal in session |
| `GET` | `/api/sessions/{id}/events` | Query events for session |

## Chat (God Agent)

| Method | Path | Description |
|--------|------|------------|
| `POST` | `/api/chat` | Send a message to the God Agent |
| `GET` | `/api/chat/conversations` | List all God Agent conversations |
| `GET` | `/api/chat/conversations/{id}` | Get conversation history by ID |

## Departments

Departments use parameterized routes. Replace `{dept}` with the department ID: `forge`, `code`, `content`, `harvest`, `gtm`, `finance`, `product`, `growth`, `distro`, `legal`, `support`, `infra`.

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/departments` | List all department definitions |
| `POST` | `/api/dept/{dept}/chat` | Send a message to a department agent |
| `GET` | `/api/dept/{dept}/chat/conversations` | List conversations for department |
| `GET` | `/api/dept/{dept}/chat/conversations/{id}` | Get department conversation history |
| `GET` | `/api/dept/{dept}/config` | Get department configuration |
| `PUT` | `/api/dept/{dept}/config` | Update department configuration |
| `GET` | `/api/dept/{dept}/events` | Query events for department |

## Engine-Specific Routes

Wired engines expose additional endpoints under the department namespace:

| Method | Path | Description |
|--------|------|------------|
| `POST` | `/api/dept/code/analyze` | Run code analysis (parser, dependency graph, metrics) |
| `POST` | `/api/dept/code/search` | BM25 search across codebase |
| `POST` | `/api/dept/content/draft` | Draft content on a topic |
| `POST` | `/api/dept/content/from-code` | Generate content from code analysis |
| `POST` | `/api/dept/content/adapt` | Adapt content for a platform |
| `POST` | `/api/dept/content/publish` | Publish content (requires approval) |
| `GET` | `/api/dept/content/calendar` | Get content calendar |
| `GET` | `/api/dept/content/analytics` | Get content analytics |
| `POST` | `/api/dept/harvest/scan` | Scan sources for opportunities |
| `POST` | `/api/dept/harvest/score` | Score an opportunity |
| `POST` | `/api/dept/harvest/propose` | Generate a proposal |
| `GET` | `/api/dept/harvest/pipeline` | Get opportunity pipeline |
| `GET` | `/api/dept/harvest/sources` | List configured sources |

## Flow Engine

DAG workflow engine with petgraph. Supports code, condition, and agent node types.

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/flows` | List all flows |
| `POST` | `/api/flows` | Create a flow |
| `GET` | `/api/flows/{id}` | Get a flow by ID |
| `PUT` | `/api/flows/{id}` | Update a flow |
| `DELETE` | `/api/flows/{id}` | Delete a flow |
| `POST` | `/api/flows/{id}/run` | Execute a flow |
| `GET` | `/api/flows/{id}/status` | Get flow execution status |

## Knowledge / RAG

Vector-backed knowledge base for semantic search.

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/knowledge` | List knowledge entries |
| `POST` | `/api/knowledge` | Add a knowledge entry |
| `GET` | `/api/knowledge/{id}` | Get a knowledge entry |
| `POST` | `/api/knowledge/search` | Semantic search across knowledge base |
| `DELETE` | `/api/knowledge/{id}` | Delete a knowledge entry |

## Database Browser (RusvelBase)

Schema introspection, table viewer, and SQL runner.

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/db/tables` | List all database tables |
| `GET` | `/api/db/tables/{table}` | Get table schema |
| `GET` | `/api/db/tables/{table}/rows` | Query rows from a table |
| `POST` | `/api/db/query` | Execute a SQL query |

## Approvals

Human-in-the-loop approval queue for content publishing and outreach.

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/approvals` | List pending approvals |
| `POST` | `/api/approvals/{id}/approve` | Approve a pending item |
| `POST` | `/api/approvals/{id}/reject` | Reject a pending item |

## Agents CRUD

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/agents` | List all agents |
| `POST` | `/api/agents` | Create an agent |
| `GET` | `/api/agents/{id}` | Get an agent by ID |
| `PUT` | `/api/agents/{id}` | Update an agent |
| `DELETE` | `/api/agents/{id}` | Delete an agent |

## Skills CRUD

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/skills` | List all skills |
| `POST` | `/api/skills` | Create a skill |
| `GET` | `/api/skills/{id}` | Get a skill by ID |
| `PUT` | `/api/skills/{id}` | Update a skill |
| `DELETE` | `/api/skills/{id}` | Delete a skill |

## Rules CRUD

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/rules` | List all rules |
| `POST` | `/api/rules` | Create a rule |
| `GET` | `/api/rules/{id}` | Get a rule by ID |
| `PUT` | `/api/rules/{id}` | Update a rule |
| `DELETE` | `/api/rules/{id}` | Delete a rule |

## Workflows

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/workflows` | List all workflows |
| `POST` | `/api/workflows` | Create a workflow |
| `GET` | `/api/workflows/{id}` | Get a workflow by ID |
| `PUT` | `/api/workflows/{id}` | Update a workflow |
| `DELETE` | `/api/workflows/{id}` | Delete a workflow |
| `POST` | `/api/workflows/{id}/run` | Execute a workflow with variables |

## MCP Servers

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/mcp-servers` | List configured MCP servers |
| `POST` | `/api/mcp-servers` | Add an MCP server |
| `PUT` | `/api/mcp-servers/{id}` | Update an MCP server |
| `DELETE` | `/api/mcp-servers/{id}` | Remove an MCP server |

## Hooks

| Method | Path | Description |
|--------|------|------------|
| `GET` | `/api/hooks` | List all hooks |
| `POST` | `/api/hooks` | Create a hook |
| `PUT` | `/api/hooks/{id}` | Update a hook |
| `DELETE` | `/api/hooks/{id}` | Delete a hook |
| `GET` | `/api/hooks/events` | List available hook event types |

## System

| Method | Path | Description |
|--------|------|------------|
| `POST` | `/api/capability/build` | Build a new capability from natural language (`!build` command) |
| `POST` | `/api/help` | AI-powered help -- ask questions about RUSVEL |
| `POST` | `/api/system/visual-test` | Run visual regression tests |
| `GET` | `/api/system/visual-report` | Get visual test report |
| `POST` | `/api/system/visual-report/self-correct` | Auto-generate fix skills/rules from visual diffs |
