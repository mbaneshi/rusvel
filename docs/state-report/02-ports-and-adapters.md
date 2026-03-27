# Chapter 2 — Ports & Adapters

## All 16 Port Traits

Defined in `crates/rusvel-core/src/ports.rs`:

### 1. LlmPort
```
generate(request) → LlmResponse
stream(request) → Receiver<LlmStreamEvent>
embed(texts) → Vec<Vec<f32>>
list_models() → Vec<ModelInfo>
submit_batch(requests) → BatchId
poll_batch(id) → BatchStatus
```

### 2. AgentPort
```
create(config) → AgentId
run(id, input) → AgentOutput
stop(id) → ()
status(id) → AgentStatus
```

### 3. ToolPort
```
register(definition, handler) → ()
call(name, args) → ToolResult
list() → Vec<ToolDefinition>
search(query) → Vec<ToolDefinition>
schema(name) → JsonSchema
```

### 4. EventPort
```
emit(event) → ()
get(id) → Event
query(filter) → Vec<Event>
```

### 5. StoragePort (facade for 5 sub-stores)
```
events() → &dyn EventStore
objects() → &dyn ObjectStore
sessions() → &dyn SessionStore
jobs() → &dyn JobStore
metrics() → &dyn MetricStore
```

#### 5a. EventStore
```
append(event) → ()
get(id) → Event
query(filter) → Vec<Event>
```

#### 5b. ObjectStore
```
put(kind, id, value) → ()
get(kind, id) → Value
delete(kind, id) → ()
list(kind, filter) → Vec<Value>
```

#### 5c. SessionStore
```
put_session(session) → ()
get_session(id) → Session
list_sessions() → Vec<Session>
put_run(run) → ()
get_run(id) → Run
list_runs(session_id) → Vec<Run>
put_thread(thread) → ()
get_thread(id) → Thread
list_threads(run_id) → Vec<Thread>
```

#### 5d. JobStore
```
enqueue(job) → ()
dequeue(kinds) → Option<Job>
update(job) → ()
get(id) → Job
list(filter) → Vec<Job>
```

#### 5e. MetricStore
```
record(metric) → ()
query(filter) → Vec<Metric>
```

### 6. MemoryPort
```
store(entry) → ()
recall(id) → MemoryEntry
search(query, session_id) → Vec<MemoryEntry>
forget(id) → ()
```

### 7. JobPort
```
enqueue(job) → ()
dequeue(kinds) → Option<Job>
complete(id, result) → ()
hold_for_approval(id) → ()
fail(id, error) → ()
schedule(id, cron) → ()
cancel(id) → ()
approve(id) → ()
list(filter) → Vec<Job>
```

### 8. SessionPort
```
create(session) → SessionId
load(id) → Session
save(session) → ()
list() → Vec<Session>
```

### 9. AuthPort
```
store_credential(key, value) → ()
get_credential(key) → String
refresh(key) → ()
delete_credential(key) → ()
```

### 10. ConfigPort
```
get_value(key) → String
set_value(key, value) → ()
get_typed<T>(key) → T
set_typed<T>(key, value) → ()
```

### 11. EmbeddingPort
```
embed(texts) → Vec<Vec<f32>>
embed_one(text) → Vec<f32>
model_name() → String
dimensions() → usize
```

### 12. VectorStorePort
```
upsert(id, content, embedding, metadata) → ()
search(query_embedding, limit) → Vec<VectorSearchResult>
delete(id) → ()
list(limit) → Vec<VectorEntry>
count() → usize
```

### 13. DeployPort
```
deploy(artifact) → DeployResult
status(id) → DeployStatus
```

### 14. TerminalPort
```
create_window(name) → WindowId
list_windows() → Vec<Window>
list_panes_for_session(session_id) → Vec<Pane>
close_window(id) → ()
create_pane(window_id, command) → PaneId
write_pane(id, data) → ()
inject_pane_output(id, output) → ()
resize_pane(id, rows, cols) → ()
close_pane(id) → ()
subscribe_pane(id) → Receiver<PaneEvent>
get_layout(window_id) → Layout
set_layout(window_id, layout) → ()
panes_for_run(run_id) → Vec<Pane>
panes_for_flow(flow_id) → Vec<Pane>
```

### 15. BrowserPort
```
connect(endpoint) → ()
disconnect() → ()
tabs() → Vec<Tab>
observe(tab_id) → Receiver<BrowserEvent>
evaluate_js(tab_id, script) → Value
navigate(tab_id, url) → ()
```

---

## Concrete Adapter Map

| Port | Adapter Crate | Concrete Type | Backend |
|------|---------------|---------------|---------|
| StoragePort | rusvel-db | `Database` | SQLite WAL (rusqlite) |
| ConfigPort | rusvel-config | `TomlConfig` | TOML file + in-memory session overlays |
| LlmPort | rusvel-llm | `MultiProvider` | ClaudeCliProvider, CursorAgentProvider, OllamaProvider, OpenAiProvider |
| LlmPort (decorator) | rusvel-llm | `CostTrackingLlm` | Wraps base LLM, records spend to MetricStore |
| EmbeddingPort | rusvel-embed | `FastEmbedAdapter` | fastembed (local model) |
| MemoryPort | rusvel-memory | `MemoryStore` | SQLite + FTS5 |
| EventPort | rusvel-event | `EventBus` | In-memory broadcast + SQLite EventStore |
| ToolPort | rusvel-tool | `ToolRegistry` | Arc HashMap |
| VectorStorePort | rusvel-vector | `LanceVectorStore` | LanceDB (Arrow columnar) |
| AuthPort | rusvel-auth | `InMemoryAuthAdapter` | In-memory from env vars |
| TerminalPort | rusvel-terminal | `TerminalManager` | PTY management |
| BrowserPort | rusvel-cdp | `CdpClient` | Chrome DevTools Protocol (WebSocket) |
| DeployPort | rusvel-deploy | `FlyDeployPort` | Fly.io API |
| AgentPort | rusvel-agent | `AgentRuntime` | Orchestrates LLM + Tool + Memory |
| SessionPort | rusvel-app | `SessionAdapter` | Wraps StoragePort::sessions() |
| JobPort | rusvel-jobs | `InMemoryJobQueue` | Vec<Job> behind Mutex |

## LLM Provider Chain

```
CostTrackingLlm (decorator — records spend per request)
  └─ MultiProvider (routes by ModelProvider enum)
       ├─ ClaudeCliProvider  → Claude CLI subprocess (real streaming)
       ├─ CursorAgentProvider → Cursor Agent Mode integration
       ├─ OllamaProvider     → Local Ollama HTTP API
       └─ OpenAiProvider     → OpenAI-compatible HTTP API
```

### Streaming

- `ClaudeCliProvider` emits `LlmStreamEvent::Delta(text)` line-by-line via async reader
- Other providers use batch `generate()` with simulated stream fallback
- `CostTrackingLlm` passes through streaming transparently

### ModelTier Routing

```
ModelTier::Fast   → haiku-class models (cheap, fast)
ModelTier::Medium → sonnet-class models (balanced)
ModelTier::Full   → opus-class models (max capability)
```

Applied via `apply_model_tier()` which maps tier to provider-specific model strings.

## Tool System

### ToolRegistry Internals

```
HashMap<String, (ToolDefinition, ToolHandler)>
  where ToolHandler = Arc<dyn Fn(Value) → BoxFuture<Result<ToolResult>>>
```

### ScopedToolRegistry

Filtered view of ToolRegistry:
- Prefix-based: `["code_"]` → only tools starting with `code_`
- Exact: `["read_file", "write_file"]` → only named tools
- Used to give each department access to only relevant tools

### Permission Modes

```
ToolPermissionMode::Auto       → execute immediately
ToolPermissionMode::Supervised → return AWAITING_APPROVAL, require human
ToolPermissionMode::Locked     → return failure
```

Resolution order: dept-specific exact > dept prefix > dept wildcard > global exact > global prefix > global wildcard > default (Auto)

### Registered Tools (22+)

**Built-in (10):** read_file, write_file, edit_file, glob, grep, bash, git_status, git_diff, git_log, tool_search

**Memory (4):** memory_write, memory_read, memory_search, memory_delete

**Terminal (2):** terminal_open, terminal_watch

**Delegation (1):** delegate_agent

**Browser (3):** browser_observe, browser_search, browser_act

**Flow (1):** invoke_flow

**Engine (12):** harvest (5), content (5), code (2) — registered via `rusvel-engine-tools`

### Deferred Tool Loading

To save tokens, not all tools are sent in the initial LLM prompt. Only non-searchable tools are included. When the agent calls `tool_search(query)`, matching tools are injected into subsequent turns. This achieves ~85% token savings on tool definitions.
