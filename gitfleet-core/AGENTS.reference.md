# AGENTS.reference.md

## Provenance And Decisions

- Agentskill Version: `2.1.0`.
- Evidence Schema Version: `4`.
- Repository Revision: `7f407aead045dabd74f15c664b42fac5113cfffd`.
- Configuration: inherited from the repository root; default signature enabled.
- Maintainer-Confirmed Decisions: adopt gitfleet-core as a nested scope; keep provider-neutral types and contracts here; do not flatten parent guidance into this document.
- Unresolved Uncertainty: none specific to this crate beyond the root merge-strategy gap.

## Boundaries And Ownership

This crate is the nearest owner of provider-neutral DTOs, capability traits, `GitfleetError`, `UnsupportedCapabilityError`, `output::Renderer`, prompts, configuration, and `src/operations.rs`. Provider HTTP and wire payloads belong in gitfleet-providers. CLI parsing belongs in the gitfleet crate.

## Testing Topology

Unit tests live in source modules. Integration tests live in `tests/`. Snapshot helpers use insta where present. Automated tests must not make live requests.

## Evidence Summary

`Cargo.toml` describes domain types, the provider trait, operations, and infrastructure. `src/lib.rs` exports one-concept modules. Workspace tests run with `cargo test --workspace` and `CARGO_BUILD_JOBS=4`.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
