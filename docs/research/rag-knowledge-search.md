# RAG, Knowledge Graph, Search, Workflow & Memory — Research

> Cataloged: 2026-03-23
> Focus: Embeddable, single-binary, Rust-native tools for RUSVEL

---

## 1. RAG Frameworks

### rig (Rust-native) — TOP PICK
- **Repo:** `github.com/0xPlaygrounds/rig` | ~6,400 stars
- **What:** Modular LLM application framework. Agents, RAG pipelines, structured output, tool use.
- **Key:** Built-in in-memory vector store in `rig-core`; companion crates for LanceDB, Qdrant, Neo4j.
- **Hex fit:** Defines `CompletionModel`, `EmbeddingModel`, `VectorStore` traits — maps to our ports. Lean core, few deps.
- **Crate:** `cargo add rig-core`, `cargo add rig-lancedb`
- **Value:** Saves building RAG plumbing (chunking, embedding, retrieval, context assembly).
- **Clone:** `gh repo clone 0xPlaygrounds/rig -- --depth 1`

### swiftide (Rust-native)
- **Repo:** `github.com/bosun-ai/swiftide` | ~630 stars
- **What:** Streaming indexing/query pipelines with tree-sitter code chunking, LanceDB/Qdrant integration.
- **Hex fit:** Pipeline-oriented, trait-based. Plug our own loaders/transformers.
- **Crate:** `cargo add swiftide`
- **Value:** Tree-sitter code chunking directly useful for `code-engine`. Streaming > batch.
- **Clone:** `gh repo clone bosun-ai/swiftide -- --depth 1`

### graphrag-rs (Rust-native)
- **Repo:** `github.com/automataIA/graphrag-rs`
- **What:** Full GraphRAG pipeline — entity extraction, knowledge graph construction, hierarchical clustering, multi-hop retrieval. 220+ tests, 50k+ lines, 4 crates.
- **Key:** Includes 75KB pure-Rust vector search (zero external deps).
- **Hex fit:** Trait-based, 15+ core abstractions. Wrap as `KnowledgePort` adapter.
- **Value:** Combines knowledge graph + vector search + RAG in one Rust package.
- **Clone:** `gh repo clone automataIA/graphrag-rs -- --depth 1`

### edgequake (Rust-native)
- **Repo:** `github.com/raphaelmansuy/edgequake`
- **What:** LightRAG algorithm in Rust — extracts knowledge graph during indexing, traverses during query. Up to 6000x token reduction vs traditional RAG.
- **Clone:** `gh repo clone raphaelmansuy/edgequake -- --depth 1`

### Reference (Python — steal patterns only)
- **nano-graphrag** (`github.com/gusye1234/nano-graphrag`) — Minimal GraphRAG, good for building our own in Rust
- **Microsoft GraphRAG** (`github.com/microsoft/graphrag`) — The original paper implementation
- **LlamaIndex** — Node/index abstraction, composable query engines, "query pipeline" pattern
- **Haystack** — Pipeline-as-DAG with typed components, "retrievers" as first-class abstraction

---

## 2. Knowledge Graph

### cozo (Rust-native) — TOP PICK
- **Repo:** `github.com/cozodb/cozo` | ~3,300 stars
- **What:** Relational-graph-vector database. Datalog query language. HNSW vector indices built in. Graph algorithms (PageRank, community detection, shortest path) built in.
- **Backend:** RocksDB or SQLite.
- **Hex fit:** Perfect for embedded single-binary. One engine for graph + vector + relational. Create `GraphPort` adapter wrapping CozoDB.
- **Crate:** `cargo add cozo` with `storage-sqlite` feature
- **Value:** Replaces need for separate graph DB + vector DB. Vector search via HNSW built into Datalog queries.
- **Clone:** `gh repo clone cozodb/cozo -- --depth 1`

### oxigraph (Rust-native)
- **Repo:** `github.com/oxigraph/oxigraph` | ~1,550 stars
- **What:** Full SPARQL 1.1 RDF store on RocksDB. Embedded library mode.
- **Value:** Best if you want standards-based knowledge representation (RDF triples, SPARQL queries).
- **Crate:** `cargo add oxigraph`

### indradb (Rust-native)
- **Repo:** `github.com/indradb/indradb` | ~1,500 stars
- **What:** Property graph database. Embeddable. Simpler than CozoDB (no vector search, no Datalog).
- **Crate:** `cargo add indradb-lib`

### Recommendation
**CozoDB** is the clear winner — graph + vector + relational in one embedded Rust library.

---

## 3. Vector / Semantic Search + Embeddings

### lancedb (Rust-native) — TOP PICK for vectors
- **Repo:** `github.com/lancedb/lancedb` | ~5,500+ stars | **Already cloned**
- **What:** Embedded vector database on Lance columnar format. Zero-copy, auto-versioning. Vector similarity + full-text search + SQL filtering.
- **Hex fit:** Embeds as library. Create `VectorStorePort` adapter. 40-60ms queries.
- **Crate:** `cargo add lancedb`
- **Value:** Best pure vector DB for single-binary. Pairs with rig via `rig-lancedb`.

### fastembed-rs (Rust-native) — TOP PICK for local embeddings
- **Repo:** `github.com/Anush008/fastembed-rs` | ~300+ stars
- **What:** Local embedding inference via ONNX Runtime. Supports BGE, all-MiniLM, quantized variants. No Python, no server.
- **Hex fit:** Implement `EmbeddingPort` adapter. Downloads models on first use, then runs locally.
- **Crate:** `cargo add fastembed`
- **Value:** Embeddings inside binary. 3-5x faster than Python, 60-80% less memory.
- **Clone:** `gh repo clone Anush008/fastembed-rs -- --depth 1`

### embed-anything (Rust-native)
- **Repo:** `github.com/StarlightSearch/EmbedAnything` | ~1,200+ stars
- **What:** Multimodal embedding (text, images, audio, PDF). Candle + ONNX backends.
- **Crate:** `cargo add embed_anything`
- **Value:** Pick over fastembed-rs if you need to embed images/audio/PDFs.

### ort (Rust-native)
- **Repo:** `github.com/pykeio/ort` | ~1,000+ stars
- **What:** ONNX Runtime Rust bindings. Foundation layer used by fastembed-rs.
- **Crate:** `cargo add ort`

### Patterns from cloned repos
- **Meilisearch** — Hybrid full-text + vector search with federated indexing and composable ranking rules
- **Qdrant** — Payload filtering on vectors (filter by session_id, engine, date range). Mirrors ADR-007 metadata approach
- **Sonic** — Ultra-lightweight keyword search with FST-based autocomplete and typo correction

### Note on Qdrant
Excellent but runs as separate server. For single-binary, LanceDB is the better choice.

---

## 4. Workflow / Planning

### petgraph (Rust-native) — TOP PICK
- **Repo:** `github.com/petgraph/petgraph` | ~3,000+ stars | 9.2M downloads/month
- **What:** Standard Rust graph library. DAG, toposort, Dijkstra, DFS/BFS, cycle detection, SCC, MST.
- **Hex fit:** Model tasks as DAG, toposort for execution order. Use in `forge-engine` mission planning.
- **Crate:** `cargo add petgraph`

### daggy (Rust-native)
- **Repo:** `github.com/mitchmindtree/daggy`
- **What:** Thin petgraph wrapper enforcing DAG invariants at insertion time.
- **Crate:** `cargo add daggy`

### orka (Rust-native)
- **Repo:** `github.com/excsn/orka`
- **What:** Async, pluggable, type-safe workflow engine. Step definitions, shared context, conditional logic.
- **Value:** Higher-level than petgraph. Good for multi-step agent workflows (research → draft → review → publish).

### sayiir (Rust-native)
- **Repo:** `github.com/sayiir/sayiir`
- **What:** Embeddable durable workflow engine. Checkpoint-based recovery. Branching, loops, fork/join, signals, cancel, pause, resume, retries, timeouts.
- **Value:** Durability — survives crashes mid-workflow. No determinism constraints on tasks.

### Patterns from cloned repos
- **N8N** — Node-based workflow DAG with graph traversal utilities. Execution model: trigger → process → output with parallel branches.
- **Dify** — ReAct/function-call routers, multi-strategy routing.

### Recommendation
Start with `petgraph` for plan DAGs. Add `sayiir` later for durable multi-day pipelines.

---

## 5. Context / Memory Management

No Rust-native memory frameworks exist. Steal patterns, implement in Rust.

### Patterns

| Pattern | Source | What it does |
|---------|--------|-------------|
| **Mem0** | Python | Extract discrete facts from conversations, store, retrieve by similarity |
| **Letta/MemGPT** | Python | OS-like memory: RAM (working context, bounded) + disk (archival, unlimited). Agent pages in/out |
| **Zep** | Python | Temporal knowledge graph — facts with timestamps, supersedable |
| **Khoj** | Python (25k stars) | Self-hosted AI assistant with document indexing + search |

### RUSVEL implementation — compose existing pieces

1. **Working memory** — In-memory bounded buffer (recent conversation turns)
2. **Episodic memory** — `rusvel-memory` FTS5 (already exists)
3. **Semantic memory** — LanceDB vector search (add this)
4. **Graph memory** — CozoDB entity relationships (add this)

New port:
```rust
pub trait ContextPort: Send + Sync {
    async fn load(&self, session_id: &SessionId, limit: usize) -> Result<Vec<ContextItem>>;
    async fn save(&self, item: ContextItem) -> Result<()>;
    async fn search(&self, query: &str, limit: usize) -> Result<Vec<ContextItem>>;
    async fn summarize(&self, items: &[ContextItem]) -> Result<String>;
}
```

### Reference repos to clone
- `gh repo clone khoj-ai/khoj -- --depth 1`
- `gh repo clone gusye1234/nano-graphrag -- --depth 1`
- `gh repo clone microsoft/graphrag -- --depth 1`

---

## 6. Integration Plan for RUSVEL

### Phase 1 — Immediate (crate dependencies)

```toml
# forge-engine/Cargo.toml
petgraph = "0.7"

# rusvel-llm/Cargo.toml
fastembed = "4"

# rusvel-memory/Cargo.toml (or rusvel-db)
lancedb = "0.15"
```

New ports in `rusvel-core`:
```rust
pub trait EmbeddingPort: Send + Sync {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
}

pub trait VectorStorePort: Send + Sync {
    async fn upsert(&self, id: &str, embedding: Vec<f32>, metadata: Value) -> Result<()>;
    async fn search(&self, query: Vec<f32>, limit: usize, filter: Option<Value>) -> Result<Vec<SearchResult>>;
}
```

### Phase 2 — Medium-term

```toml
# rusvel-agent/Cargo.toml (or new rusvel-rag)
rig-core = "0.31"

# new crate: rusvel-graph/Cargo.toml
cozo = { version = "0.7", features = ["storage-sqlite"] }

# code-engine/Cargo.toml
swiftide = "0.16"
```

New port:
```rust
pub trait GraphPort: Send + Sync {
    async fn add_entity(&self, entity: Entity) -> Result<()>;
    async fn add_relation(&self, from: &str, relation: &str, to: &str) -> Result<()>;
    async fn query(&self, query: &str) -> Result<Vec<Value>>;
    async fn neighbors(&self, entity_id: &str, depth: usize) -> Result<SubGraph>;
}
```

### Phase 3 — Knowledge engine

New `knowledge-engine` or enhanced `rusvel-memory`:
- Index conversations, code, content, opportunities into knowledge graph
- Hybrid retrieval: FTS5 + vector similarity + graph traversal
- Context assembly: pull relevant memories before each agent call
- Temporal awareness: facts decay, get superseded, are time-scoped

### Architecture

```
                    ┌─────────────────────────────────┐
                    │       ENGINES (consumers)        │
                    │  forge / code / harvest / etc.   │
                    └────────────┬────────────────────┘
                                 │ (uses traits only)
                    ┌────────────┴────────────────────┐
                    │       rusvel-core (ports)        │
                    │  EmbeddingPort                   │
                    │  VectorStorePort                 │
                    │  GraphPort                       │
                    │  ContextPort                     │
                    │  MemoryPort (existing)           │
                    └────────────┬────────────────────┘
                                 │ (implemented by)
          ┌──────────────────────┼─────────────────────────┐
          │                      │                         │
  ┌───────┴───────┐   ┌─────────┴────────┐   ┌───────────┴──────┐
  │ rusvel-llm    │   │ rusvel-memory     │   │ rusvel-graph     │
  │ + fastembed   │   │ + FTS5 (existing) │   │ + CozoDB         │
  │ (EmbeddingPort│   │ + LanceDB vectors │   │ (GraphPort)      │
  │  adapter)     │   │ (VectorStorePort  │   │                  │
  └───────────────┘   │  + MemoryPort)    │   └──────────────────┘
                      └──────────────────┘
```

---

## 7. Repos to Clone (when ready)

```bash
# RAG frameworks
gh repo clone 0xPlaygrounds/rig -- --depth 1
gh repo clone bosun-ai/swiftide -- --depth 1
gh repo clone automataIA/graphrag-rs -- --depth 1
gh repo clone raphaelmansuy/edgequake -- --depth 1

# Knowledge graph
gh repo clone cozodb/cozo -- --depth 1

# Embeddings
gh repo clone Anush008/fastembed-rs -- --depth 1

# Reference (patterns only)
gh repo clone gusye1234/nano-graphrag -- --depth 1
gh repo clone microsoft/graphrag -- --depth 1
gh repo clone khoj-ai/khoj -- --depth 1
```

---

## 8. Key Insights

1. **CozoDB is the most underrated tool here.** Graph + vector + relational in one embedded Rust library. Could replace both a separate graph DB and vector DB.
2. **fastembed + lancedb** gives local embeddings + vector search with zero external services. The minimum viable RAG stack.
3. **rig-core** saves months of RAG plumbing work. Its trait system aligns with hexagonal architecture.
4. **petgraph** is the obvious choice for workflow DAGs — 9.2M downloads/month, battle-tested.
5. **Memory management is a composition problem**, not a library problem. Compose FTS5 + vectors + graph + bounded buffers.
6. **Dify's RAG pipeline** (extraction → chunking → embedding → retrieval → reranking) is the production-proven pattern to implement.
7. **graphrag-rs** is the most ambitious Rust-native option — if it matures, it could be the single dependency for knowledge + RAG.
