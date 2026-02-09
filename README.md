# context-core

[![Crates.io](https://img.shields.io/crates/v/context-core.svg)](https://crates.io/crates/context-core)
[![Docs.rs](https://docs.rs/context-core/badge.svg)](https://docs.rs/context-core)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

Deterministic core library for context caching and selection.

`context-core` provides the foundational types and algorithms for the Context platform: document ingestion, cache building, content-hash versioning, term-frequency scoring, and token-budgeted selection. All operations are deterministic — identical inputs always produce identical outputs.

## Crate overview

| Module | Purpose |
|--------|---------|
| `document` | Document model with content-hash versioning (`DocumentId`, `Document`, `Metadata`) |
| `cache` | Cache build pipeline (`CacheBuilder`, `CacheManifest`, `ContextCache`) |
| `selection` | Deterministic context selection with scoring and token budgeting (`ContextSelector`) |
| `types` | Shared types (`Query`, `ScoreDetails`, `ContextBundle`) |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
context-core = { path = "../context-core" }
```

### Build a cache

```rust
use context_core::cache::{CacheBuildConfig, CacheBuilder};
use context_core::document::{Document, DocumentId, Metadata};

let root = std::path::Path::new("/docs");
let id = DocumentId::from_path(root, &root.join("guide.md")).unwrap();
let doc = Document::ingest(id, "guide.md".into(), b"Hello world".to_vec(), Metadata::default()).unwrap();

let builder = CacheBuilder::new(CacheBuildConfig::v0());
let cache = builder.build(vec![doc], std::path::Path::new("/tmp/my-cache")).unwrap();
```

### Select context

```rust
use context_core::selection::ContextSelector;
use context_core::types::Query;

let selector = ContextSelector::default();
let query = Query::new("deployment");
let result = selector.select(&cache, query, 4000).unwrap();
```

## Build

```bash
make build     # debug build
make test      # run all tests
make check     # cargo check + clippy
make release   # optimized build
make clean     # remove artifacts
```

## Spec references

See `spec_refs.md` for links to the governing specifications.

---

"Context Engine" is a trademark of Context Engine Contributors. The software is open source under the [Apache License 2.0](LICENSE). The trademark is not licensed for use by third parties to market competing products or services without prior written permission.
