# RUSVEL — Task Runner

# List all recipes
default:
    @just --list

# Build all workspace crates
build:
    cargo build --workspace

# Run all tests
test:
    cargo test --workspace

# Run tests for a single crate
test-crate crate:
    cargo test -p {{crate}}

# Check everything (Rust + frontend)
check: check-rust check-frontend

# Check Rust workspace
check-rust:
    cargo check --workspace

# Check frontend types
check-frontend:
    cd frontend && pnpm check

# Format Rust code
fmt:
    cargo fmt --all

# Format and check
fmt-check:
    cargo fmt --all -- --check

# Run clippy
lint:
    cargo clippy --workspace -- -D warnings

# Full CI check (format + lint + test)
ci: fmt-check lint test

# Start API server on :3000
serve:
    cargo run

# Start dev frontend on :5173
dev-frontend:
    cd frontend && pnpm dev

# Build frontend for embedding
build-frontend:
    cd frontend && pnpm install --frozen-lockfile && pnpm build

# Run coverage report
coverage:
    ./scripts/coverage.sh

# Count lines per crate (flag any over 2000)
crate-lines:
    @for dir in crates/*/; do \
        name=$(basename "$dir"); \
        lines=$(find "$dir" -name "*.rs" -exec cat {} + 2>/dev/null | wc -l | tr -d ' '); \
        if [ "$lines" -gt 2000 ]; then \
            echo "⚠ $name: $lines lines (over 2000)"; \
        else \
            echo "  $name: $lines lines"; \
        fi; \
    done

# Check engine crates for adapter imports (architecture violation)
check-boundaries:
    @echo "Checking engine crates for adapter imports..."
    @for engine in crates/*-engine/; do \
        name=$(basename "$engine"); \
        if grep -q 'rusvel-db\|rusvel-llm\|rusvel-agent\|rusvel-channel' "$engine/Cargo.toml" 2>/dev/null; then \
            echo "⚠ $name: imports adapter crate!"; \
        else \
            echo "  $name: OK"; \
        fi; \
    done

# Start MCP server
mcp:
    cargo run -- --mcp

# Start TUI dashboard
tui:
    cargo run -- --tui

# Start REPL shell
shell:
    cargo run -- shell

# Workspace stats
stats:
    @echo "Rust crates: $(ls -d crates/*/ | wc -l | tr -d ' ')"
    @echo "Rust files: $(find crates -name '*.rs' | wc -l | tr -d ' ')"
    @echo "Rust lines: $(find crates -name '*.rs' -exec cat {} + | wc -l | tr -d ' ')"
    @echo "Frontend files: $(find frontend/src -type f 2>/dev/null | wc -l | tr -d ' ')"
    @echo "Test count: $(cargo test --workspace -- --list 2>/dev/null | grep -c ': test$' || echo '?')"

# Tag and push a release (usage: just release 0.2.0)
release version:
    @echo "Releasing v{{version}}..."
    sed -i '' 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    cargo check --workspace
    git add -A
    git commit -m "chore: release v{{version}}"
    git tag "v{{version}}"
    @echo "Run 'git push origin main --tags' to trigger release workflow"

# Build Docker image
docker-build:
    docker build -t rusvel:latest .

# Run with docker-compose
docker-up:
    docker compose up -d

# Stop docker-compose
docker-down:
    docker compose down

# Build mdBook docs
docs-build:
    mdbook build docs-site

# Serve mdBook docs locally
docs-serve:
    mdbook serve docs-site
