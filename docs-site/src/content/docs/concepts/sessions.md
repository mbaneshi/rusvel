---
title: Sessions
description: Understand RUSVEL's session-based workspace model.
---

## What Are Sessions?

Sessions are RUSVEL's top-level organizational unit. Think of them as workspaces or projects. All data in RUSVEL -- goals, events, conversations, agent runs, content items, opportunities -- is scoped to a session.

You always have one **active session**. All CLI commands and API calls operate on the active session by default.

## Session Kinds

Each session has a kind that hints at its purpose:

| Kind | Use Case | Example |
|------|----------|---------|
| `Project` | A codebase or product you are building | "RUSVEL", "My SaaS App" |
| `Lead` | A potential client or deal | "Acme Corp Engagement" |
| `ContentCampaign` | A content series or marketing push | "Q1 Blog Series" |
| `General` | Catch-all for anything else | "Scratch Pad" |

The session kind is informational. It does not restrict which departments or features you can use.

## Creating Sessions

### CLI

```bash
cargo run -- session create "My Startup"
```

This creates a new session and sets it as active. The session ID is stored in `~/.rusvel/active_session`.

### API

```bash
curl -X POST http://localhost:3000/api/sessions \
  -H "Content-Type: application/json" \
  -d '{"name": "My Startup", "kind": "Project"}'
```

## Listing Sessions

### CLI

```bash
cargo run -- session list
```

Output shows all sessions with the active one marked:

```
ID                                      NAME                  KIND        UPDATED
------------------------------------------------------------------------------------------
a1b2c3d4-e5f6-...                       My Startup            Project     2026-03-23 14:30 *
b2c3d4e5-f6g7-...                       Blog Series           Content..   2026-03-22 09:15
```

### API

```bash
curl http://localhost:3000/api/sessions
```

## Switching Sessions

### CLI

```bash
cargo run -- session switch <session-id>
```

### API

Load a specific session by ID:

```bash
curl http://localhost:3000/api/sessions/<session-id>
```

## Data Stored Per Session

Each session contains:

- **Goals** -- strategic objectives with timeframes and progress tracking
- **Events** -- an immutable, append-only log of everything that happens
- **Conversations** -- chat history with each department agent
- **Agent runs** -- records of every agent execution (input, output, tokens used)
- **Content items** -- blog posts, tweets, proposals in various states
- **Opportunities** -- leads, gigs, and deals in the pipeline
- **Contacts** -- CRM entries tied to the session
- **Tasks** -- generated from daily plans, linked to goals

## Session Configuration

Sessions can override global configuration. The config cascade is:

```
Global config  →  Department config  →  Session config
```

This lets you use different LLM models, approval policies, or tool permissions per session.
