# ADR — Auth phase 2 (session-scoped API keys)

## Status

Proposed (stub). Complements env bearer token (`RUSVEL_API_TOKEN`) and in-memory `rusvel-auth`.

## Context

- Today: optional single shared bearer token; no per-session or per-client keys.
- Goal: allow automation (CI, webhooks, integrations) to call the API with revocable credentials scoped to a session or role.

## Decision (stub)

1. **Port**: Introduce `ApiKeyPort` in `rusvel-core` — validate `Authorization: Bearer <key>`, return `SessionId` + optional scope (read-only vs full).
2. **Storage**: SQLite table `api_keys` (hash of secret, `session_id`, `created_at`, `label`) via `rusvel-db`; never store plaintext secrets.
3. **Middleware**: Axum layer after CORS, before handlers — if `Authorization` matches an API key, attach `Extension<ValidatedApiKey>`; else fall through to existing env bearer check.
4. **UI**: Settings later — create/revoke keys; out of scope for this stub.

## Consequences

- Engines remain unaware of HTTP auth; only `rusvel-api` applies middleware.
- Migration: additive schema; default install has zero keys (no behavior change).

## Implementation stub

No code path is enabled by default until `ApiKeyPort` is implemented and wired in `rusvel-app` composition root.
