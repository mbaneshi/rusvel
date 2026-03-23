---
title: API Reference
description: Complete REST API endpoint reference for the RUSVEL HTTP server.
---

## Overview

RUSVEL exposes a JSON REST API on port 3000 via Axum. All endpoints are prefixed with `/api/`. CORS is enabled for all origins.

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

## Other

| Method | Path | Description |
|--------|------|------------|
| `POST` | `/api/capability/build` | Build a new capability (engine extension) |
| `POST` | `/api/help` | AI-powered help -- ask questions about RUSVEL |
