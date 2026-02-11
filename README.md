# context-core

[![Crates.io](https://img.shields.io/crates/v/context-core.svg)](https://crates.io/crates/context-core)
[![Docs.rs](https://docs.rs/context-core/badge.svg)](https://docs.rs/context-core)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

> `context-core` is the deterministic engine powering the Context platform. It provides content-addressed document modeling, immutable cache management, and auditably-reproducible context selection for LLM agents.

It replaces non-deterministic retrieval pipelines with a reproducible, inspectable alternative suitable for regulated and production environments.

## Core principles

`context-core` is designed for environments where reproducibility and security are non-negotiable:

- **Deterministic Selection**: For a given cache and query, the selection of context is identical across every platform and version.
- **Content-Addressed**: Every document is versioned by its content hash (SHA-256), ensuring integrity and preventing "hidden state" bugs.
- **Token Budget First**: Precision context retrieval designed specifically for LLM window constraints.
- **Deterministic Token Accounting**: Token counts are derived from a stable byte-level algorithm defined in the specification. They do not depend on model-specific tokenizers.
- **Zero Network**: No external services, no vector database sidecars, no runtime network dependencies.

## When to use context-core

Use this library when you need:

- Reproducible context selection across environments
- Local or air-gapped deployments
- Inspectable and versioned knowledge caches
- Deterministic inputs for LLM systems

## Crate structure

The library is organized into specialized modules that form the platform's foundation:

| Module | Responsibility |
|--------|----------------|
| `document` | Content-hash versioned modeling (`DocumentId`, `Document`, `Metadata`) |
| `cache` | Immutable cache build-and-load pipeline (`CacheBuilder`, `ContextCache`) |
| `selection` | The core selection logic with scoring and token budgeting (`ContextSelector`) |
| `types` | Shared contracts (`Query`, `ScoreDetails`, `ContextBundle`) |

## Usage

### Integration

Add to your `Cargo.toml`:

```toml
[dependencies]
context-core = "0.1"
```

*Note: During local development of the platform, internal crates use path dependencies to ensure they track local changes.*

### Context Selection

```rust
use context_core::cache::ContextCache;
use context_core::selection::ContextSelector;
use context_core::types::Query;

// Load an immutable cache
let cache = ContextCache::load("./path/to/my-cache").expect("Valid cache");

// Execute deterministic selection
let selector = ContextSelector::default();
let query = Query::new("deployment architecture");
let budget = 4000; // tokens

let result = selector.select(&cache, query, budget).expect("Deterministic result");
```

## Determinism & Reproducibility

Determinism is the primary "invariant" of this library. The engine guarantees stable result ordering and byte-identical output across:

1. Different hardware architectures (x86_64, aarch64)
2. Different compiler versions (within the supported rust-version window)
3. Different operating systems

Determinism applies to:
- document ordering
- scores and floating-point representations
- serialized output structure
- cache loading and inspection results

This is verified by a "golden snapshot" test harness in the `tests/` directory which prevents selection logic regressions.

## Platform Architecture Role

`context-core` is the engine that drives both the `context-cli` for build-time operations and the `mcp-context-server` for runtime agent interaction. Most users interact with this engine indirectly via the context CLI or the MCP server. Direct library integration is intended for systems-level embedding.

## Build

```bash
make build     # debug build
make test      # run all tests (including selection invariants)
make check     # cargo check + clippy
make release   # optimized build
make clean     # remove artifacts
```

## Spec references

See `spec_refs.md` for links to the governing specifications.

---

"Context Engine" is a trademark of Context Engine Contributors. The software is open source under the [Apache License 2.0](LICENSE). The trademark is not licensed for use by third parties to market competing products or services without prior written permission.
