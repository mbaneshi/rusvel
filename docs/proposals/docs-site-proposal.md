# Documentation Site Proposal for RUSVEL

> Evaluated: 2026-03-24
> Context: RUSVEL is a Rust + SvelteKit monorepo, 34 crates, 12 departments, targeting solo devs/founders.

---

## Our Requirements

Based on RUSVEL's vision, mission, and repo:

1. **Developer-focused** — code examples, API references, architecture diagrams
2. **Rust ecosystem aligned** — familiar to Rust developers
3. **Single binary philosophy** — docs should be simple, not another heavy stack
4. **Beautiful by default** — dark mode, search, responsive, professional
5. **GitHub Pages deployable** — free hosting, automated CI
6. **Low maintenance** — content is Markdown, minimal framework churn
7. **Fast builds** — we have 20+ pages, growing to 50+
8. **Monorepo friendly** — lives in `docs-site/` alongside Rust crates and SvelteKit frontend

---

## Options Evaluated

### 1. mdBook (Rust-native)
- **What:** Rust's official doc tool. Used by The Rust Book, Tokio, Bevy.
- **Pros:**
  - Written in Rust — `cargo install mdbook` — aligns with our stack
  - Zero JavaScript, zero Node.js — no version conflicts
  - Builds in milliseconds (10K pages in <2 seconds)
  - Built-in search, syntax highlighting, dark mode
  - Dead simple: `SUMMARY.md` defines structure
  - Used by every major Rust project — familiar to our audience
- **Cons:**
  - Simpler design (no hero sections, no fancy components)
  - Less customizable than JS-based frameworks
  - No component-based pages (pure Markdown)
- **Build:** `mdbook build` → static HTML
- **Stars:** 19K+ GitHub stars

### 2. VitePress (Vue-powered)
- **What:** Vue-powered SSG by the Vue.js team. Used by Vite, Vitest, Pinia.
- **Pros:**
  - Beautiful default theme
  - Dark mode, search, i18n out of the box
  - Blazing fast (Vite-powered)
  - Vue components in Markdown
  - Active community, stable API
- **Cons:**
  - Vue.js — we're a SvelteKit project (philosophical mismatch)
  - Adds Node.js + Vue to the dependency tree
  - Yet another JS framework in the repo
- **Stars:** 15K+ GitHub stars

### 3. Astro Starlight (Astro-powered)
- **What:** Astro's official docs framework. Used by Astro, Clerk, Biome.
- **Pros:**
  - Most feature-rich (search, i18n, versioning, components)
  - Tailwind integration (matches our frontend stack)
  - Can use Svelte components (our existing stack!)
  - Beautiful design out of the box
- **Cons:**
  - **Currently broken in our repo** — Astro 5.x + Starlight version conflicts with zod
  - Heavy dependency tree (Astro + Starlight + zod + shiki + etc.)
  - Frequent breaking changes between versions
  - Adds significant complexity vs simpler alternatives
- **Stars:** 5.5K+ GitHub stars (Starlight), 50K+ (Astro)

### 4. Docusaurus (React-powered)
- **What:** Facebook's doc framework. Used by React, Babel, Jest.
- **Pros:**
  - Most mature (since 2017), battle-tested
  - Versioned docs, blog, plugin system
  - MDX support (React components in Markdown)
- **Cons:**
  - React-based — completely foreign to our Rust + SvelteKit stack
  - Heaviest of all options (React + webpack/turbopack)
  - Overkill for our current scale
- **Stars:** 58K+ GitHub stars

### 5. Zola (Rust-native)
- **What:** Fast static site generator in Rust. Single binary.
- **Pros:**
  - Written in Rust — single binary, no dependencies
  - Fast builds, Sass support, shortcodes, search
  - Template-based (Tera) — more flexible than mdBook
  - Good for docs + blog + landing page
- **Cons:**
  - Smaller community than mdBook
  - Less Rust-ecosystem recognition
  - Requires learning Tera template syntax
- **Stars:** 14K+ GitHub stars

### 6. Custom SvelteKit Docs
- **What:** Build docs with our existing SvelteKit frontend.
- **Pros:**
  - Same stack — reuse components, design system, Tailwind tokens
  - Full control, zero new dependencies
  - Could serve from the same binary (rust-embed)
- **Cons:**
  - Have to build everything ourselves (search, navigation, ToC)
  - Significant effort for features other frameworks give for free
  - Reinventing the wheel

---

## Comparison Matrix

| Criteria | mdBook | VitePress | Starlight | Docusaurus | Zola | Custom SvelteKit |
|---|---|---|---|---|---|---|
| **Rust alignment** | ★★★★★ | ★★☆☆☆ | ★★★☆☆ | ★☆☆☆☆ | ★★★★★ | ★★★☆☆ |
| **Build speed** | ★★★★★ | ★★★★☆ | ★★★☆☆ | ★★☆☆☆ | ★★★★★ | ★★★★☆ |
| **Design quality** | ★★★☆☆ | ★★★★★ | ★★★★★ | ★★★★☆ | ★★★☆☆ | ★★★★★ |
| **No JS deps** | ★★★★★ | ☆☆☆☆☆ | ☆☆☆☆☆ | ☆☆☆☆☆ | ★★★★★ | ☆☆☆☆☆ |
| **Maintenance** | ★★★★★ | ★★★★☆ | ★★☆☆☆ | ★★★☆☆ | ★★★★☆ | ★★☆☆☆ |
| **Search built-in** | ★★★★☆ | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★☆ | ☆☆☆☆☆ |
| **Community/familiarity** | ★★★★★ | ★★★★☆ | ★★★☆☆ | ★★★★★ | ★★★☆☆ | ★☆☆☆☆ |
| **Version stability** | ★★★★★ | ★★★★☆ | ★★☆☆☆ | ★★★★☆ | ★★★★★ | ★★★★★ |
| **GitHub Pages** | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★★ | ★★★★★ |
| **Total** | **42** | **35** | **29** | **30** | **40** | **27** |

---

## Recommendation: mdBook

**Why mdBook wins for RUSVEL:**

1. **Written in Rust.** `cargo install mdbook` — that's it. No Node.js, no npm, no version conflicts, no zod errors. This is exactly the "single binary, zero ops" philosophy RUSVEL follows.

2. **Used by every Rust project our audience knows.** The Rust Book, Tokio docs, Bevy docs, Axum docs — all mdBook. Our target users (Rust developers, solo founders) already know how to navigate mdBook sites.

3. **Millisecond builds.** Our 20 pages build in <1 second. No waiting, no debugging npm version conflicts. The Starlight build has been broken for hours — mdBook would have been deployed 5 minutes after creating the files.

4. **Zero maintenance.** mdBook is stable. It doesn't break between versions. There's no JS framework churn. We already have enough complexity in 27 Rust crates + SvelteKit frontend — docs should be boring and reliable.

5. **The content already exists.** We have 20+ markdown files in `docs-site/src/content/docs/`. They'll work in mdBook with zero changes (just needs a `SUMMARY.md`).

6. **Plugins for extras.** mdBook has plugins for mermaid diagrams, admonitions, API references, and custom themes if we need them later.

**The tradeoff:** mdBook's default design is simpler than VitePress or Starlight. But for developer documentation, simplicity IS the feature. Clean, fast, searchable, familiar.

**Runner-up: Zola** — if we want a more custom landing page or blog alongside docs, Zola gives more template flexibility while staying in Rust. But for pure docs, mdBook is better.

---

## Implementation Plan

### Step 1: Install mdBook (1 minute)
```bash
cargo install mdbook
```

### Step 2: Create book structure (5 minutes)
```
docs-site/
├── book.toml          (config)
└── src/
    ├── SUMMARY.md     (table of contents)
    ├── index.md       (landing page)
    ├── getting-started/
    │   ├── installation.md
    │   ├── first-run.md
    │   └── first-mission.md
    ├── concepts/
    │   ├── departments.md
    │   ├── agents.md
    │   └── ...
    └── ...
```

### Step 3: Move existing content (10 minutes)
Copy markdown files from `docs-site/src/content/docs/` → `docs-site/src/`. Remove Starlight frontmatter (mdBook doesn't need it). Create `SUMMARY.md`.

### Step 4: Deploy to GitHub Pages (10 minutes)
Add GitHub Action:
```yaml
- uses: peaceiris/actions-mdbook@v1
- run: mdbook build docs-site
- uses: peaceiris/actions-gh-pages@v3
  with:
    publish_dir: ./docs-site/book
```

### Step 5: Custom theme (optional, later)
Add RUSVEL branding, dark mode default, custom CSS.

**Total: ~30 minutes from zero to deployed docs site.**

---

## Sources

- [mdBook](https://github.com/rust-lang/mdBook) — 19K+ stars
- [Zola](https://github.com/getzola/zola) — 14K+ stars
- [VitePress](https://vitepress.dev/) — 15K+ stars
- [Starlight vs Docusaurus comparison](https://blog.logrocket.com/starlight-vs-docusaurus-building-documentation/)
- [Distr's Starlight migration](https://distr.sh/blog/distr-docs/)
- [Rust SSGs comparison](https://dasroot.net/posts/2026/02/static-site-generators-rust-performance-ecosystem-use-cases/)
- [Documentation frameworks overview](https://dev.to/silviaodwyer/10-open-source-documentation-frameworks-to-check-out-331f)
