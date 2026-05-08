# who — Architecture

## Overview

`who` is a semantic code intelligence runtime for humans and AI agents. It parses source code via tree-sitter, builds a symbol/call index in SQLite, and answers semantic questions like "who calls this function?" and "who implements this trait?"

```
Source Files
     |
  tree-sitter AST Parse
     |
  Symbol + Import + Call Extraction
     |
  SQLite Index  (.who-ast/index.sqlite)
     |
  Target Resolution + Query
     |
  Output (human / JSON / NDJSON / quickfix)
```

---

## Core Binaries

```
who-call     Find callers, definitions, references, impact — and build the index
who-impl     Find implementations of traits/interfaces
```

### Usage

```bash
who-call index .                          # build the index
who-call render_text                      # who calls render_text?
who-call src/ui/button.rs:42              # who calls the function at this line?
who-call src/ui/button.rs:42 --json       # structured output for AI agents
who-call def src/main.rs:10               # resolve definition
who-call refs src/text/render.rs:3        # find references
who-call impact src/text/render.rs:3      # transitive caller chain
who-impl Renderable                       # who implements Renderable?
who-impl index .                          # build the index (also available here)
```

---

## Workspace Layout

```
who/
├── Cargo.toml                          # workspace root
├── crates/
│   ├── who-core/                       # data model, index, resolution engine
│   │   └── src/
│   │       ├── target.rs               # CLI target parser (file:line, file#symbol, etc.)
│   │       ├── symbol.rs               # Symbol, Import, FileEntry, SourceRange
│   │       ├── refs.rs                 # Reference, RefKind
│   │       ├── calls.rs                # CallEdge, Resolution
│   │       ├── confidence.rs           # scoring builder + labels
│   │       ├── resolve.rs              # target → symbol resolution, caller lookup
│   │       ├── index.rs                # SQLite schema, reads, writes
│   │       ├── lang.rs                 # LanguageParser trait, detect_language()
│   │       └── error.rs               # WhoError, ExitCode
│   │
│   ├── who-cli/                        # binary crate (who-call, who-impl)
│   │   └── src/
│   │       ├── bin_whocall.rs          # `who-call` — callers, def, refs, impact, index
│   │       ├── bin_whoimpl.rs          # `who-impl` — impl queries, index
│   │       ├── cmd_index.rs            # `who-call index .` / `who-impl index .`
│   │       ├── cmd_callers.rs          # `who-call <target>`
│   │       ├── cmd_def.rs              # `who-call def <target>`
│   │       ├── cmd_refs.rs             # `who-call refs <target>`
│   │       ├── cmd_impl.rs             # `who-impl <target>`
│   │       ├── cmd_impact.rs           # `who-call impact <target>`
│   │       └── output.rs              # human, JSON, quickfix formatters
│   │
│   └── who-lang-rust/                  # Rust language support
│       └── src/
│           ├── lib.rs
│           └── parser.rs              # tree-sitter Rust extraction
│
├── samples/
│   └── rust-project/                   # sample codebase for demos
│
└── .github/workflows/
    ├── ci.yml                          # build, test, clippy, fmt
    ├── showcase.yml                    # index + query the sample project
    └── release.yml                     # build binaries + update Homebrew tap
```

### Crate Dependency Graph

```
who-cli
├── who-core
│   ├── rusqlite          (SQLite storage)
│   ├── serde / serde_json (serialization)
│   ├── ignore            (gitignore-aware file walking)
│   ├── thiserror         (error types)
│   └── anyhow            (error propagation)
├── who-lang-rust
│   ├── who-core
│   ├── tree-sitter       (AST parsing framework)
│   └── tree-sitter-rust  (Rust grammar)
└── clap                  (CLI argument parsing)
```

---

## Core Data Model

Five entities stored in SQLite:

```
┌──────────┐     ┌──────────┐     ┌──────────┐
│  files   │◄────│ symbols  │◄────│  imports │
│          │     │          │     │          │
│ path     │     │ name     │     │ local_   │
│ language │     │ qual_name│     │   name   │
│ hash     │     │ kind     │     │ qual_    │
│ mtime    │     │ range    │     │  target  │
└──────────┘     │ signature│     │ alias    │
                 │ visibility     └──────────┘
                 └─────┬────┘
                       │
              ┌────────┴────────┐
              ▼                 ▼
        ┌──────────┐     ┌──────────┐
        │   refs   │◄────│  calls   │
        │          │     │          │
        │ target_  │     │ caller_  │
        │  sym_id  │     │  sym_id  │
        │ source_  │     │ callee_  │
        │  file_id │     │  sym_id  │
        │ kind     │     │ ref_id   │
        │ text     │     │ confid.  │
        │ confid.  │     │ resolut. │
        └──────────┘     └──────────┘
```

---

## Call Resolution

After indexing, `resolve_all_calls()` runs a second pass over every file's unresolved call refs. Three strategies are tried in order:

```
 Strategy              Confidence   When
 ──────────────────────────────────────────────────────────
 1. Import match       0.75         callee name matches an import's local_name,
                                    and the import's qualified_target resolves to
                                    exactly one symbol
 2. Same-file match    0.60         callee name matches a symbol defined in the
                                    same file
 3. Global unique      0.45         callee name matches exactly one symbol across
                                    the entire index
```

If global lookup finds multiple candidates, an `Ambiguous` call edge is stored with all candidate IDs and confidence 0.25.

---

## Release & Installation

### Release Pipeline (`.github/workflows/release.yml`)

Triggered on GitHub release publish:

```
 release published
     │
     ├─ Build 4 targets in parallel:
     │   ├─ aarch64-apple-darwin   (macOS ARM, native)
     │   ├─ x86_64-apple-darwin    (macOS Intel, native)
     │   ├─ x86_64-unknown-linux-gnu (Linux x86_64, native)
     │   └─ aarch64-unknown-linux-gnu (Linux ARM, cross)
     │
     ├─ Package who-call + who-impl as who-<target>.tar.gz
     │
     ├─ Upload to GitHub release assets
     │
     └─ Homebrew job: generate who.rb formula → push to meloalright/homebrew-tap
```

### Installation

```sh
# Homebrew (macOS / Linux)
brew tap meloalright/tap
brew install who

# From source
cargo install --path crates/who-cli

# From GitHub release
gh release download --repo meloalright/who-ast --pattern 'who-*.tar.gz'
```

---

## Design Principles

1. **Semantic-first** — resolve to meaning, not syntax shape
2. **Incremental by default** — hash-based re-indexing avoids reparsing unchanged files
3. **AI-agent native** — structured output with confidence scoring
4. **Multi-language architecture** — pluggable `LanguageParser` trait per language
5. **Unix-like UX** — minimal, composable, scriptable CLI commands
