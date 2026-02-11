# context-core — Implementation Progress

## Status: v0 core complete — test consolidation and verification function remain

The core library compiles, all 37 tests pass, and the three-phase selection pipeline (score → order → budget) is fully operational. The document model, cache builder, and selection engine match the normative specs. Dead code and duplicates have been cleaned up. All spec compliance gaps have been resolved (either by code fix or spec clarification). What remains is duplicate test consolidation, edge case coverage, and a standalone cache verification function.

---

## Completed

### Document Model (`document/`)
- [x] `Document` struct with `id`, `version`, `source`, `content`, `metadata`
- [x] Single constructor `Document::ingest()` enforcing all invariants
- [x] UTF-8 validation at ingestion (rejects invalid bytes)
- [x] Content-addressed versioning: `sha256:<hex>` from content bytes only
- [x] Metadata excluded from version computation
- [x] No newline normalization (CRLF vs LF produce different versions)
- [x] `DocumentId::from_path()` with normalization: lowercase, forward slashes, no `./` prefix
- [x] `DocumentVersion::from_content()` using SHA-256
- [x] `Metadata` backed by `BTreeMap` (sorted iteration for determinism)
- [x] `MetadataValue` supports `String` and `Number(i64)` only (flat, no nesting)
- [x] `Metadata::merge()` with caller-provided precedence

### Cache System (`cache/`)
- [x] `CacheBuilder::build()` — single-threaded, non-reentrant cache construction
- [x] Documents sorted by ID before processing (determinism)
- [x] Duplicate document ID detection after sorting (adjacent-pair check, fatal error)
- [x] Filename: first 12 chars of SHA-256 hash (without `sha256:` prefix)
- [x] Filename collision detection (fatal error)
- [x] Cache version: `sha256(config_json + sorted("doc_id:doc_version"))` — `created_at` excluded
- [x] Atomic writes: temp dir → rename (all-or-nothing)
- [x] Stale temp dir cleanup from previous crashed runs
- [x] `manifest.json` (pretty-printed, sorted documents)
- [x] `index.json` (pretty-printed, `BTreeMap` ensures sorted keys, `#[serde(transparent)]` for flat map format)
- [x] `documents/{hash}.json` per document
- [x] `ContextCache` — thin read-only runtime wrapper
- [x] `load_documents()` — loads from manifest entries, verifies ID matches, verifies version (recomputes content hash against manifest)
- [x] Rejects build if output directory already exists

### Selection Engine (`selection/`)
- [x] Three-phase pipeline: score → order → budget
- [x] `TermFrequencyScorer` — naive term frequency: `term_matches / total_words`
- [x] Query normalization: lowercase + whitespace split
- [x] Scoring is pure (no side effects, no randomness)
- [x] Sort: score descending, document ID ascending (deterministic tie-break)
- [x] `debug_assert!` verifying sorted order invariant
- [x] `ApproxTokenCounter` — `ceil(len / 4)` approximation
- [x] Greedy budget filling: documents added in order, never truncated
- [x] Zero budget → empty selection
- [x] Score 0.0 documents MAY be selected (no score-based exclusion in v0)
- [x] `Scorer` and `TokenCounter` traits for future extensibility
- [x] `SelectionResult` output with `documents` + `selection` metadata
- [x] `SelectionWhy` explainability: `query_terms`, `term_matches`, `total_words`

### Types (`types/`)
- [x] `Query` — normalized query with `raw` + `terms`
- [x] `SelectedDocument` — final output document with score, tokens, why
- [x] `SelectionMetadata` — query, budget, tokens_used, counts
- [x] `SelectionResult` — top-level result container
- [x] `ScoredDocument` — internal reference-based scored document (avoids premature cloning)
- [x] `ScoreDetails` — internal scoring components
- [x] `SelectionError` — `InvalidBudget`, `CacheError`
- [x] `DocumentId`, `DocumentVersion` — identity and versioning types

### Error Handling
- [x] `DocumentError` — `InvalidUtf8`
- [x] `DocumentIdError` — `OutsideRoot`, `InvalidUtf8`
- [x] `CacheBuildError` — `Io`, `Serialization`, `OutputExists`, `FilenameCollision`, `InvalidVersionFormat`, `DuplicateDocumentId`
- [x] `SelectionError` — `InvalidBudget`, `CacheError`

### Cleanup
- [x] Removed duplicate `selection/selector.rs` (inlined copy of `selection/mod.rs` logic)
- [x] Removed duplicate `selection/types.rs` (copy of `types/context_bundle.rs`)
- [x] Removed duplicate `document/id.rs` and `document/version.rs` (copies of `types/identifiers.rs`)
- [x] Removed 6 legacy re-export shims (`cache/builder.rs`, `cache/manifest.rs`, `cache/config.rs`, `cache/index.rs`, `selection/scorer.rs`, `selection/tokenizer.rs`)
- [x] Removed migrated `mcp/` module (MCP error types live in `mcp-context-server`)
- [x] Removed unused `errors.rs` (`CoreError` wrapper had no consumers)
- [x] Removed empty `tests/mcp_error_schema.rs`

### Spec Compliance Fixes
- [x] `CacheIndex` serialization: added `#[serde(transparent)]` so `index.json` serializes as flat map (was wrapped in `{"entries": {...}}`)
- [x] Duplicate document ID detection: `CacheBuilder::build()` now rejects duplicate IDs after sorting
- [x] Document version verification on load: `load_documents()` recomputes content hash and compares against manifest entry
- [x] `document_model.md`: metadata extraction and frontmatter parsing scoped to post-v0
- [x] `context_selection.md`: output schema aligned to normative `context.resolve.md`
- [x] `context_selection.md`: removed `documents_excluded_by_score` (v0 doesn't exclude by score)
- [x] `milestone_zero.md`: output contract fixed (`metadata` → `selection`, added missing fields, referenced normative spec)
- [x] `milestone_zero.md`: changed false "provenance" claim to "version and scoring explanation"

### Test Coverage (37 tests, all passing)
- [x] `cache_invariants.rs` (2) — index key sorting, filename collision detection
- [x] `cache_lifecycle.rs` (10) — determinism, config changes, UTF-8, manifest bytes, corruption, ID normalization, metadata isolation, newline handling, metadata precedence
- [x] `cache_manifest.rs` (2) — version determinism, config change effects
- [x] `context_selection.rs` (3) — zero budget, sorting order, tie-breaking
- [x] `determinism.rs` (4) — manifest serialization, document serialization, selection output, end-to-end determinism
- [x] `document_model.rs` (6) — document invariants
- [x] `end_to_end_golden.rs` (2) — manifest byte comparison, corruption detection
- [x] `golden_selection_contract.rs` (1) — selection output structure validation
- [x] `golden_selection_logic.rs` (1) — end-to-end selection determinism
- [x] `golden_serialization.rs` (2) — manifest and document serialization snapshots
- [x] `selection_invariants.rs` (1) — token bounds, scores, content accuracy
- [x] `selection_logic.rs` (3) — budget constraints, sorting, tie-breaking

---

## Remaining Work

### P1 — Functional gaps

- [ ] **Cache verification function** — `context_cache.md` specifies a verification operation that checks:
  1. Manifest exists and is valid JSON
  2. Cache version matches recomputed hash
  3. Every document file exists
  4. Every document file hash matches its filename
  5. No orphan files in `documents/`

  No standalone `verify_cache()` function exists. Individual checks are partially covered by `load_documents()` (checks 1, 3, 4 via version verification) but there is no single function that runs all 5 checks and reports results. Needed by both the CLI `inspect --verify` and MCP `inspect_cache` tool.

### P1 — Enterprise Ingestion Foundation (see `context-specs/plans/enterprise_ingest_plan.md` Phase 0)

- [ ] **`DocumentSource` trait + `RawDocument` type** — Define connector interface in `document::source` module. All enterprise connectors implement this trait. `RawDocument` carries pre-ingestion content + metadata.
- [ ] **`ConnectorError` type** — Error variants: `AuthenticationFailed`, `FetchFailed`, `InvalidContent`, `PartialFetch`.
- [ ] **Canonicalization utilities** — `document::canonicalize` module: line ending normalization, trailing whitespace trimming, trailing empty line removal, Unicode NFC normalization. Deterministic ordering of all transforms.
- [ ] **`FilesystemSource` reference connector** — Migrate existing walkdir-based ingestion to `DocumentSource` trait. Must produce byte-identical caches to current `build` path.
- [ ] **`ingest_from_source()` pipeline** — Orchestrates: `source.fetch_documents()` → UTF-8 validation → `Document::ingest()`. Configurable error policy (skip-and-warn vs abort-all).
- [ ] **`unicode-normalization` dependency** — Add with `default-features = false` for NFC normalization.

### P2 — Test gaps

- [ ] **Duplicate test consolidation** — Several test files contain identical or near-identical tests:
  - `cache_lifecycle.rs` and `document_model.rs` share 5+ identical tests
  - `cache_manifest.rs` duplicates 2 tests from `cache_lifecycle.rs`
  - `context_selection.rs` and `selection_logic.rs` contain the same 3 tests
  - `determinism.rs` and `golden_serialization.rs` share tests
  - `end_to_end_golden.rs` duplicates tests from `cache_lifecycle.rs`

  Consider consolidating to avoid maintenance burden and test confusion.

- [ ] **Cache rebuild determinism** — No test verifies that building a cache twice from the same documents produces byte-identical `manifest.json` (the `created_at` timestamp will differ). The `cache_version` field will match, but the full file will not. This is spec-correct (`created_at` is informational) but should be explicitly tested.

- [ ] **Duplicate document ID test** — No test exercises the new `DuplicateDocumentId` error path.

- [ ] **Version verification test** — No test exercises the version mismatch detection in `load_documents()` (e.g., corrupt a document file after build, verify load fails).

- [ ] **Edge cases not covered:**
  - Empty document set (build cache with 0 documents)
  - Single document cache
  - Very large document (multi-MB content)
  - Document with empty content (`""`)
  - Query with special characters, punctuation
  - Budget of 1 (smaller than any document)
  - All documents have score 0.0

### P3 — Nice to have

- [ ] **`context inspect` support** — Expose an `inspect_cache()` function returning cache metadata (document count, total size, cache version, validity). Needed by the MCP `inspect_cache` tool and CLI.

- [ ] **Cache rebuild (force)** — `CacheBuilder` rejects existing output dirs. A `rebuild()` method or `--force` equivalent that removes and rebuilds would match the spec's rebuild command.

- [ ] **`Deserialize` for `Query`** — `Query` derives `Clone` and `Debug` but not `Deserialize`. Adding it would allow JSON deserialization of queries (useful for test fixtures).

- [ ] **Document field ordering guarantee** — Spec says documents are serialized with fixed field order (`id`, `version`, `source`, `content`, `metadata`). Serde's default struct serialization preserves declaration order, which matches the spec. But this is implicit — a `#[serde(rename_all)]` or field reorder would silently break it. Consider adding a golden test that asserts field order explicitly.

---

## Spec Issues — All Resolved

| # | Issue | Resolution |
|---|---|---|
| 1 | `documents_excluded_by_score` in selection output | Removed from `context_selection.md`; `context.resolve.md` is normative and doesn't include it |
| 2 | `metadata` vs `selection` key in output | Updated `milestone_zero.md` to use `"selection"` |
| 3 | `cache_version` in output | Updated `milestone_zero.md` to match normative spec (no `cache_version`) |
| 4 | Automatic metadata extraction scope | Deferred to post-v0 in `document_model.md` |
| 5 | MCP error types — single source of truth | Deleted from context-core; MCP types live in `mcp-context-server` |

---

## File Inventory

```
context-core/
├── Cargo.toml
├── progress.md                      ← this file
├── spec_refs.md
├── src/
│   ├── lib.rs                       module declarations
│   │
│   ├── types/
│   │   ├── mod.rs                   re-exports
│   │   ├── identifiers.rs          DocumentId, DocumentVersion
│   │   └── context_bundle.rs       Query, SelectionResult, etc.
│   │
│   ├── document/
│   │   ├── mod.rs                   re-exports
│   │   ├── document.rs             Document struct + ingest()
│   │   ├── metadata.rs             Metadata, MetadataValue
│   │   └── parser.rs               placeholder (future parsing hooks)
│   │
│   ├── cache/
│   │   ├── mod.rs                   re-exports
│   │   ├── cache.rs                ContextCache (runtime read-only wrapper)
│   │   ├── versioning.rs           CacheManifest, CacheBuildConfig, CacheIndex
│   │   └── invalidation.rs         CacheBuilder (build logic)
│   │
│   ├── selection/
│   │   ├── mod.rs                   ContextSelector + three-phase pipeline
│   │   ├── ranking.rs              Scorer, TermFrequencyScorer, TokenCounter
│   │   ├── budgeting.rs            apply_budget (greedy selection)
│   │   └── filters.rs              placeholder (future filtering)
│   │
│   └── compression/
│       ├── mod.rs                   module declaration
│       └── summarizer.rs           placeholder (future compression)
│
└── tests/
    ├── cache_invariants.rs          2 tests — index sorting, collision
    ├── cache_lifecycle.rs           10 tests — determinism, config, corruption
    ├── cache_manifest.rs            2 tests — version determinism, config changes
    ├── context_selection.rs         3 tests — budget, sorting, ties
    ├── determinism.rs               4 tests — serialization + e2e determinism
    ├── document_model.rs            6 tests — document invariants
    ├── end_to_end_golden.rs         2 tests — manifest bytes, corruption
    ├── golden_selection_contract.rs 1 test — output structure
    ├── golden_selection_logic.rs    1 test — e2e selection determinism
    ├── golden_serialization.rs      2 tests — serialization snapshots
    ├── selection_invariants.rs      1 test — bounds + explainability
    └── selection_logic.rs           3 tests — budget, sorting, ties
```

---

## Dependencies

```toml
[dependencies]
sha2 = "0.10"              # SHA-256 hashing
hex = "0.4"                # Hex encoding
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
thiserror = "1.0"          # Error derive macros
chrono = { version = "0.4", features = ["serde", "clock"], default-features = false }  # created_at timestamps

[dev-dependencies]
tempfile = "3.24.0"
```
