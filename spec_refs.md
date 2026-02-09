# Specification References

This crate implements the following specs from the canonical `context-specs` repository:

- [core/document_model.md](../context-specs/core/document_model.md) (v0) - ✅ Strictly Compliant
- [core/context_cache.md](../context-specs/core/context_cache.md) (v0) - 🚧 Planned

## Compliance Mechanism

Compliance is enforced via integration tests in `tests/document_model.rs` that explicitly test spec invariants.

| Invariant | Test Function |
|-----------|---------------|
| UTF-8 Validation | `invariant_utf8_rejection` |
| Content Identicality | `invariant_same_content_same_version` |
| ID Normalization | `invariant_same_path_same_id` |
| Version Purity | `invariant_metadata_does_not_affect_version` |
| Metadata Precedence | `invariant_metadata_precedence` |
| No Normalization | `invariant_no_newline_normalization` |
