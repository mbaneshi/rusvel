
## Prerequisites

RUSVEL is a Rust + SvelteKit application. You need the following installed:

| Dependency | Version | Required | Purpose |
|-----------|---------|----------|---------|
| **Rust** | Edition 2024 (nightly or stable 1.85+) | Yes | Backend, all engines |
| **Node.js** | 20+ | Yes | Frontend build |
| **pnpm** | 9+ | Yes | Frontend package manager |
| **SQLite** | 3.35+ | Bundled | Database (WAL mode) |
| **Ollama** | Latest | Optional | Local LLM inference |

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Install Node.js + pnpm

Use your preferred method (nvm, fnm, or direct install):

```bash
# With nvm
nvm install 22
nvm use 22

# Install pnpm
corepack enable
corepack prepare pnpm@9.15.4 --activate
# Or: npm install -g pnpm
```

### Install Ollama (optional)

Ollama provides free local LLM inference. RUSVEL auto-detects it on first run.

```bash
# macOS
brew install ollama

# Linux
curl -fsSL https://ollama.com/install.sh | sh

# Start the service
ollama serve
```

If you skip Ollama, you can configure Claude API or OpenAI keys instead.

## Clone and Build

```bash
git clone https://github.com/your-org/all-in-one-rusvel.git
cd all-in-one-rusvel
cargo build
```

The workspace has 20+ crates. First build takes a few minutes; subsequent builds are incremental.

## Build the Frontend

```bash
cd frontend
pnpm install
pnpm build
```

The built frontend is served by the Rust binary at runtime. During development you can run `pnpm dev` for hot-reload on port 5173.

## Verify the Installation

Run the full test suite to confirm everything works:

```bash
cargo test
```

You should see all 197 tests passing across multiple crates:

```
test result: ok. 197 passed; 0 failed
```

### Test individual crates

```bash
cargo test -p rusvel-core       # Core domain types and port traits
cargo test -p rusvel-db         # SQLite stores (41 tests)
cargo test -p forge-engine      # Agent orchestration (15 tests)
cargo test -p content-engine    # Content creation (7 tests)
cargo test -p harvest-engine    # Opportunity discovery (12 tests)
```

## Next Steps

Once installed, proceed to [First Run](/getting-started/first-run/) to launch RUSVEL for the first time.
