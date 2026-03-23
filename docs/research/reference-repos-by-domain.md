# Reference Repos Organized by RUSVEL Domain

> Quick lookup: which repos to study when building each engine.

---

## Foundation (rusvel-core, adapters)

| Repo | Path | What to study |
|------|------|---------------|
| adk-rust | `/Users/bm/cod/dir6/adk-rust` | Agent/Llm/Tool traits, Content/Part types, session model, guardrails |
| windmill | `/Users/bm/cod/dir7/windmill` | Axum API crate splitting, job queue, worker patterns, trigger system |
| SpacetimeDB | `~/cod/trend/26-feb/SpacetimeDB` | Reactive DB queries, subscription model |
| cognee | `~/cod/trend/17-march/cognee` | Knowledge graph construction, chunking |
| OpenViking | `~/cod/trend/17-march/OpenViking` | Tiered memory (hot/warm/cold) |
| mem9 | `~/github-awesome/19-march/mem9` | Conversation-aware memory, forgetting curve |

## Forge Engine (Agent Orchestration)

| Repo | Path | What to study |
|------|------|---------------|
| forge-project | `/Users/bm/claude-parent/forge-project` | EventBus, safety controls, process spawning, WebSocket streaming |
| agentforge-hq | `~/cod/trend/10-march/agentforge-hq` | 100+ personas, org chart, governance, UnitOfWork |
| platform | `~/cod/trend/17-march/platform` | Merged workspace pattern, 23 crates coexisting |
| adk-rust | `/Users/bm/cod/dir6/adk-rust` | Workflow agents (Sequential/Parallel/Loop/Graph), A2A protocol |
| claude-flow | `~/cod/trend/26-feb/claude-flow` | Claude orchestration patterns |
| agency-agents | `~/cod/trend/10-march/agency-agents` | Multi-agent coordination |
| OpenDevin | `~/cod/dir7/OpenDevin` | Autonomous coding agent patterns |
| ghost-os | `~/github-awesome/19-march/ghost-os` | Agent OS concept |
| OpenJarvis | `~/github-awesome/19-march/OpenJarvis` | Personal assistant agent |
| deer-flow | `~/cod/trend/26-feb/deer-flow` | Agent workflow design |

## Code Engine (Code Intelligence)

| Repo | Path | What to study |
|------|------|---------------|
| codeilus | `/Users/bm/codeilus/codeilus` | Full pipeline: parse→graph→metrics→analyze→narrate→learn→export |
| c2rust | `/Users/bm/cod/dir6/c2rust` | AST transformation, C parser, transpilation rules |
| Immunant/c2rust | `~/cod/dir6/Immunant /c2rust` | Original C2Rust reference implementation |
| how-c-to-rust | `~/cod/dir6/how-c-to-rust` | C-to-Rust patterns |
| emerge | `~/codeilus/emerge` | Code structure analysis (Python, study the algorithms) |
| PocketFlow | `~/codeilus/PocketFlow-Tutorial-Codebase-Knowledge` | Turning code into tutorials |
| CodeVisualizer | `~/codeilus/CodeVisualizer` | Code visualization patterns |
| GitDiagram | `~/codeilus/gitdiagram` | Diagram generation from repos |

## Harvest Engine (Opportunity Discovery)

| Repo | Path | What to study |
|------|------|---------------|
| smart-standalone-harvestor | `/Users/bm/cod/dir7/smart-standalone-harvestor` | CDP scraping, job scoring, bid generation |
| freelancer | `~/smart-standalone-harvestor/freelancer` | Freelancer.com CDP interception |
| linkedin-captured | `~/cod/in-progress/linkedin-captured` | Chrome CDP tab capture |
| apify-upwork | `~/cod/dir4/apify-upwork` | Upwork scraping via Apify |
| linkedin-job | `~/cod/dir4/linkedin-job` | LinkedIn job scraping |
| job-hunt | `~/cod/dir5/job-hunt` | Job hunting automation |
| Scrapling | `~/cod/trend/26-feb/Scrapling` | Web scraping framework |
| chrome-cdp-skill | `~/github-awesome/19-march/chrome-cdp-skill` | CDP + AI agent integration |
| chrome-devtools-mcp | `~/cod/dir6/chrome-devtools-mcp` | Chrome DevTools as MCP |
| playwright-mcp | `~/cod/dir7/playwright-mcp` | Playwright as MCP |
| solance.ai | `~/cod/dir7/solance.ai` | Freelance AI platform design |

## Content Engine (Creation & Publishing)

| Repo | Path | What to study |
|------|------|---------------|
| contentforge | `/Users/bm/contentforge` | Full pipeline: draft→adapt→schedule→publish, platform adapters |
| content-mbaneshi | `/Users/bm/content-mbaneshi` | Brand templates, editorial calendar, content workflow |
| content-auto-machine | `~/cod/dir5/content-auto-machine` | Content automation pipeline |
| mana | `/Users/bm/cod/dir7/mana` | Multi-agent creative pipeline (Director + 8 sub-agents) |
| cine-combined | `~/cod/in-progress/cine-combined` | Tool registry, workspace management, credit model |
| cine/cineclip/cinefilm | `~/cod/dir5/cine-related/` | Earlier cine iterations |
| MoneyPrinterV2 | `~/cod/trend/5-march/MoneyPrinterV2` | Automated video content |
| tome | `~/github-awesome/19-march/tome` | Documentation generation |
| seomachine | `~/cod/trend/5-march/seomachine` | SEO content automation |
| remotion | `~/cod/dir4/remotion-stuff/` | Programmatic video generation |

## Ops Engine (Business Operations)

| Repo | Path | What to study |
|------|------|---------------|
| solo-os (dir4) | `~/cod/dir4/claude-sale/solo-os` | 5 agent domains, CopilotKit chat, business models |
| solo-os (dir8) | `~/cod/dir8/solo-os` | Layered arch, Docker orchestration, schema-per-domain |
| mclic (Throne) | `~/cod/dir7/mclic` | INBOUND→PROCESS→OUTBOUND taxonomy, 23 source categories |
| next-level-mcli | `~/cod/dir7/next-level-mcli` | Refined Throne iteration |
| tradesage-mvp | `~/cod/dir4/tradesage-mvp` | Trading AI patterns |
| evershop | `~/cod/trend/20-feb/evershop` | E-commerce patterns |
| stripe-play | `~/cod/in-progress/stripe-play` | Stripe integration patterns |
| founders-kit | `~/github-awesome/19-march/founders-kit` | Startup resources |

## Mission Engine (Goals & Planning)

| Repo | Path | What to study |
|------|------|---------------|
| mclic (Throne) | `~/cod/dir7/mclic` | Executive agent patterns, goal-setting workflow |
| daily-log-plan | `~/cod/in-progress/daily-log-plan` | Daily planning system |
| gsd-2 | `~/github-awesome/19-march/gsd-2` | "Get stuff done" patterns |
| strategy-decision-planning | `~/cod/dir7/not/strategy-decision-planning` | Strategy/decision frameworks |
| superpowers | Multiple locations | Developer productivity patterns |

## Connect Engine (Networking & Outreach)

| Repo | Path | What to study |
|------|------|---------------|
| openclaw ecosystem | `~/cod/dir7/openclaw-play/` | Messaging gateway, skill discovery, agent networking |
| linkedin-captured | `~/cod/in-progress/linkedin-captured` | LinkedIn data capture |
| smart-tool-chat | `~/cod/dir7/smart-tool-chat` | Tool-augmented chat patterns |
| voicebox | `~/cod/dir7/voicebox` | Voice communication patterns |
| moltbot | `~/cod/dir7/moltbot` | Bot interaction patterns |

## RAG, Knowledge Graph, Search & Memory

> See full analysis: `docs/research/rag-knowledge-search.md`

### RAG Frameworks (Rust-native)

| Repo | Stars | What to study |
|------|-------|---------------|
| `0xPlaygrounds/rig` | 6.4k | RAG pipelines, agent abstractions, VectorStore trait, LanceDB companion |
| `bosun-ai/swiftide` | 630 | Streaming indexing, tree-sitter code chunking, query pipelines |
| `automataIA/graphrag-rs` | new | Full GraphRAG: entity extraction → knowledge graph → multi-hop retrieval |
| `raphaelmansuy/edgequake` | new | LightRAG in Rust — 6000x token reduction |

### Knowledge Graph (Rust-native)

| Repo | Stars | What to study |
|------|-------|---------------|
| `cozodb/cozo` | 3.3k | **Top pick.** Graph + vector + relational. Datalog, HNSW, PageRank. Embedded like SQLite |
| `oxigraph/oxigraph` | 1.5k | SPARQL 1.1 RDF store. Standards-based knowledge representation |
| `indradb/indradb` | 1.5k | Simple property graph DB. Embeddable |

### Vector Search & Embeddings (Rust-native)

| Repo | Stars | What to study |
|------|-------|---------------|
| `lancedb/lancedb` | 5.5k | **Already cloned.** Embedded vector DB, auto-indexing, SQL filtering |
| `Anush008/fastembed-rs` | 300+ | Local embeddings via ONNX. BGE/MiniLM. No Python |
| `StarlightSearch/EmbedAnything` | 1.2k | Multimodal embedding (text, images, audio, PDF) |
| `pykeio/ort` | 1k | ONNX Runtime Rust bindings. Foundation for local inference |

### Workflow / Planning (Rust-native)

| Repo | Stars | What to study |
|------|-------|---------------|
| `petgraph/petgraph` | 3k+ | Standard Rust graph lib. DAG, toposort, DFS/BFS. Use for mission planning |
| `mitchmindtree/daggy` | small | Thin petgraph wrapper enforcing DAG invariants |
| `excsn/orka` | new | Async workflow engine with typed steps, shared context |
| `sayiir/sayiir` | new | Durable workflow engine with checkpoint recovery |

### Memory Patterns (Python — steal patterns only)

| Repo | Stars | What to study |
|------|-------|---------------|
| `khoj-ai/khoj` | 25k | Self-hosted AI assistant, document indexing, search |
| `gusye1234/nano-graphrag` | — | Minimal GraphRAG impl, good reference for Rust port |
| `microsoft/graphrag` | — | Original GraphRAG paper implementation |

### Already Cloned (good-repo/)

| Repo | Relevant for |
|------|-------------|
| `meilisearch` | Hybrid full-text + vector search, composable ranking rules |
| `qdrant` | Payload-filtered vector search, segment architecture |
| `lancedb` | Embedded vector DB, SQL filtering, auto-indexing |
| `sonic` | Lightweight keyword search, FST autocomplete |
| `ast-grep` | Tree-sitter pattern matching, code search |
| `tree-sitter` | Incremental parsing, error-tolerant AST |
| `dify` | RAG pipeline (extract → chunk → embed → retrieve → rerank) |
| `n8n` | Workflow DAG, node execution, approval flows |

---

## Infrastructure & Tooling

| Repo | Path | What to study |
|------|------|---------------|
| supabase-self-official | `~/cod/dir7/subabase-self-official` | Auth, realtime, storage API design |
| tailscale repos | `~/cod/in-progress/tailscaling/` | Mesh networking, secure connectivity |
| carokia-brain | `/Users/bm/carokia-brain` | Rust AI decision engine |
| carokia-web | `/Users/bm/carokia-web` | SvelteKit + Rust patterns |
| CLI-Anything | `~/github-awesome/19-march/CLI-Anything` | Agent-native CLI design |
| mcp2cli | `~/github-awesome/19-march/mcp2cli` | MCP → CLI adapter patterns |
| Auto-claude-code-research | `~/github-awesome/19-march/Auto-claude-code-research-in-sleep` | Automated research runner |
