# Chapter 6 — Data Flows

## Agent Execution Loop

The core runtime (`rusvel-agent/src/lib.rs`) implements a tool-use loop with streaming:

```
User Input
    │
    ▼
┌──────────────────────────────────────┐
│ 1. Build LlmRequest                  │
│    • system instructions             │
│    • conversation history            │
│    • tool definitions (filtered)     │
│    • model + effort config           │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ 2. Call LlmPort.stream() or         │
│    LlmPort.generate()               │
│    → emits AgentEvent::TextDelta    │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ 3. Check finish_reason               │
│    ├─ Stop/Length/ContentFilter → 6  │
│    └─ ToolUse → 4                   │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ 4. Execute tool                      │
│    a. Run pre-hooks (Allow/Modify/   │
│       Deny)                          │
│    b. ToolPort.call(name, args)      │
│    c. Run post-hooks                 │
│    → emits AgentEvent::ToolCall     │
│    → emits AgentEvent::ToolResult   │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ 5. Append tool result to messages    │
│    Loop back to step 1               │
│    (max 10 iterations)               │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│ 6. Return AgentOutput                │
│    • response text                   │
│    • tool calls made                 │
│    • usage stats (tokens, cost)      │
└──────────────────────────────────────┘
```

### Context Compaction

When conversation exceeds 30 turns, older messages are summarized using a `ModelTier::Fast` call. This keeps the context window manageable while preserving key information.

### Deferred Tool Loading

```
Initial prompt:  only non-searchable tools included
Agent calls:     tool_search("file operations")
System injects:  read_file, write_file, edit_file into next turn
Result:          ~85% token savings on tool definitions
```

---

## Department Chat Flow

```
User types message in /dept/[id] chat
    │
    ▼
Frontend: POST /api/dept/{dept}/chat (SSE)
    │
    ▼
┌──────────────────────────────────────┐
│ 1. Validate dept (registry lookup)   │
│ 2. Load config (3-layer cascade)     │
│    Registry → Stored → Session       │
│ 3. Interceptors:                     │
│    !build X  → capability builder    │
│    /skill    → expand template       │
│    @agent    → override prompt+model │
│ 4. Load enabled rules → append to    │
│    system prompt                     │
│ 5. RAG: embed query → vector search  │
│    → inject relevant knowledge       │
│ 6. Build AgentConfig                 │
│ 7. AgentRuntime.run_streaming()      │
│    → SSE events to frontend          │
│ 8. Post: store msg, emit event,      │
│    dispatch hooks                    │
└──────────────────────────────────────┘
    │
    ▼
Frontend receives SSE:
  text_delta      → append to message bubble
  tool_call_start → show ToolCallCard (pending)
  tool_call_end   → update ToolCallCard (result)
  run_completed   → finalize, show cost
  run_failed      → show error
```

---

## Tool Dispatch Flow

```
AgentRuntime extracts ToolCall from LLM response
    │
    ▼
ToolPort.call(name, args)
    │
    ▼
┌──────────────────────────────────────┐
│ 1. Lookup tool by name in registry   │
│ 2. Validate args vs JSON Schema      │
│    (check required fields)           │
│ 3. Check permission:                 │
│    dept exact > dept prefix >        │
│    dept wildcard > global exact >    │
│    global prefix > global wildcard > │
│    default (Auto)                    │
│ 4. Permission result:               │
│    Auto → execute handler            │
│    Supervised → AWAITING_APPROVAL    │
│    Locked → return failure           │
│ 5. Handler receives JSON args        │
│ 6. Returns ToolResult                │
└──────────────────────────────────────┘
```

---

## MCP Integration Flow

```
MCP Server config (ObjectStore)
    │
    ▼
McpClientManager.connect(config)
    │
    ▼
McpClient.connect()
    │
    ├─ Spawn external process (stdio)
    ├─ JSON-RPC handshake: initialize
    └─ Request: tools/list
           │
           ▼
    For each discovered tool:
    ┌──────────────────────────────────────┐
    │ 1. Create ToolDefinition             │
    │    name: "{server}__{tool}"           │
    │ 2. Create handler closure that:      │
    │    a. Sends JSON-RPC tools/call      │
    │    b. Reads response content blocks  │
    │    c. Returns ToolResult             │
    │ 3. Register in ToolRegistry          │
    └──────────────────────────────────────┘

Agent can now call MCP tools by namespaced name
```

---

## RAG / Knowledge Flow

```
Ingest:
  POST /api/knowledge/ingest { content, source }
      │
      ├─ Chunk into 500-char segments
      ├─ Store in MemoryPort (SQLite FTS5)
      └─ Embed → VectorStorePort.upsert() (LanceDB)

Search (vector):
  POST /api/knowledge/search { query, limit }
      │
      ├─ EmbeddingPort.embed_one(query)
      └─ VectorStorePort.search(embedding, limit)
          → VectorSearchResult { id, content, score }

Hybrid Search:
  POST /api/knowledge/hybrid-search { query, limit, k }
      │
      ├─ FTS leg: MemoryPort.search(query, session)
      ├─ Vector leg: VectorStorePort.search(embedding, limit)
      └─ Reciprocal Rank Fusion (RRF, configurable K)
          → HybridSearchHit { source, score, content }

Auto-indexing (background):
  EventBus subscription on:
    code.analyzed, content.drafted, content.published,
    harvest.opportunity.*, flow.execution.completed
      │
      └─ Extract text → MemoryPort + VectorStorePort
```

---

## Event System Flow

```
Any component:
  EventPort.emit(Event { session_id, source, kind, payload })
      │
      ▼
EventBus:
  1. Persist to EventStore (SQLite)
  2. Broadcast to live subscribers (tokio::broadcast)
      │
      ├─ TriggerManager checks registered EventTriggers
      │   └─ If match: spawn configured action
      │
      ├─ Knowledge indexer (background)
      │   └─ Auto-ingest event content into RAG
      │
      └─ Department event handlers (from register())
          └─ e.g., content subscribes to code.analyzed
```

### Event Naming Convention
```
{engine}.{domain}.{action}

Examples:
  code.analyzed
  content.drafted
  content.published
  harvest.opportunity.discovered
  gtm.deal.updated
  finance.income.recorded
  flow.execution.completed
  mission.goal.created
```

---

## Job Queue Flow

```
Enqueue:
  JobPort.enqueue(Job { kind, payload, session_id })
      │
      ▼
Job stored in Vec<Job> with status: Queued

Worker (background loop):
  JobPort.dequeue(supported_kinds)
      │
      ▼
  Route by JobKind:
    CodeAnalyze    → CodeEngine.analyze(path)
    ContentPublish → ContentEngine.publish(session, id, platform)
    HarvestScan    → HarvestEngine.scan(session, sources)
    ProposalDraft  → HarvestEngine.generate_proposal(session, opp_id)
    OutreachSend   → (not yet wired)
      │
      ▼
  On success: JobPort.complete(id, result)
  On failure: JobPort.fail(id, error)
  On approval needed: JobPort.hold_for_approval(id)
```

---

## Frontend ↔ Backend Communication

```
┌─────────────────┐         ┌──────────────────┐
│   SvelteKit     │  REST   │   Axum API       │
│   Frontend      │◄───────►│   :3000          │
│                 │         │                  │
│   /dept/[id]    │  SSE    │   /api/dept/     │
│   chat panel    │◄────────│   {dept}/chat    │
│                 │         │                  │
│   Terminal      │   WS    │   /api/terminal/ │
│   (xterm.js)    │◄───────►│   ws             │
└─────────────────┘         └──────────────────┘

REST:  CRUD operations, config, analytics
SSE:   Chat streaming (text_delta, tool_call, run_completed)
WS:    Terminal binary frames (PTY I/O)
```

---

## Cross-System Data Flow Diagram

```
┌──────────────────────────────────────────────────────┐
│                    User / Web UI                      │
└────────────────────────┬─────────────────────────────┘
                         │
                    ┌────▼─────┐
                    │ AgentPort │ (rusvel-agent)
                    └────┬──────┘
                         │
        ┌────────────────┼────────────────┐
        │                │                │
        ▼                ▼                ▼
   ┌─────────┐     ┌─────────┐     ┌──────────┐
   │ LlmPort │     │ToolPort │     │MemoryPort│
   └────┬────┘     └────┬────┘     └─────┬────┘
        │               │                │
   ┌────▼──────┐   ┌────▼──────┐   ┌─────▼────┐
   │Providers  │   │Tool       │   │FTS5      │
   │Claude CLI │   │Registry   │   │+ embed   │
   │Ollama     │   │22+ tools  │   │SQLite    │
   │OpenAI     │   │+ MCP      │   └──────────┘
   │Cursor     │   │+ perms    │
   └───────────┘   └───────────┘
                        │
       ┌────────────────┼────────────────┬───────────┐
       │                │                │           │
       ▼                ▼                ▼           ▼
  ┌─────────┐     ┌─────────┐     ┌─────────┐ ┌─────────┐
  │ Storage │     │ Vector  │     │  Jobs   │ │ Events  │
  │ Port    │     │ Store   │     │  Port   │ │  Port   │
  └────┬────┘     └────┬────┘     └────┬────┘ └────┬────┘
       │               │              │            │
  ┌────▼────┐     ┌────▼────┐    ┌────▼────┐ ┌────▼────┐
  │ SQLite  │     │ LanceDB │    │In-memory│ │EventBus │
  │ 5 stores│     │ Arrow   │    │Vec<Job> │ │+persist │
  └─────────┘     └─────────┘    └─────────┘ └─────────┘
```
