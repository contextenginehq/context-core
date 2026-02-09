//! Deterministic context selection engine for AI agents and LLMs.
//!
//! `context-core` provides document ingestion, cache building, content-hash
//! versioning, term-frequency scoring, and token-budgeted selection. All
//! operations are deterministic — identical inputs always produce identical
//! outputs, byte-for-byte.
//!
//! See <https://github.com/contextenginehq/context-engine> for the full platform.

pub mod cache;
pub mod compression;
pub mod document;
pub mod selection;
pub mod types;
