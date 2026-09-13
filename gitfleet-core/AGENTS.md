# AGENTS.md

## Scope

- Path: gitfleet-core
- Parent: .
- Inheritance: additive

## Mission And Repository Map

This crate owns provider-neutral domain types, capability contracts, the operation registry, configuration, output, prompts, and shared infrastructure.

- `src/provider.rs` defines provider identity, context, and capability traits.
- `src/types.rs` holds canonical DTOs that cross crate boundaries.
- `src/operations.rs` is the public operation family registry.
- `src/errors.rs` defines `GitfleetError` and `UnsupportedCapabilityError`.
- `src/output.rs` is the `Renderer` used by command handlers.

## Non-Negotiables

- Keep DTOs and capability contracts provider-neutral.
- Define expected-failure types in `src/errors.rs` so callers can route through `GitfleetError`.
- Render user-facing CLI output through `output::Renderer`.
- Keep one concept per source file.

## Don’ts

- Do not add provider wire types or HTTP clients.
- Do not call `reqwest`.
- Do not add provider-specific GitHub or GitLab behavior that belongs in gitfleet-providers.

## Quick Start

```bash
CARGO_BUILD_JOBS=4 cargo test -p gitfleet-core --locked
```

## Change Routing

Add shared types, traits, errors, output, prompts, and operation families here before CLI or provider code consumes them.

## Architecture Rules

Capability traits in `src/provider.rs` are the contract providers implement. Configuration is TOML under the user configuration directory, with `GITFLEET_` environment variables. Human output is the default; JSON is explicit. Confirmation, `--yes`, and `--dry-run` helpers belong in this crate.

## Implementation Conventions

Four-space Rust, 100-column limit, grouped imports, snake_case items, PascalCase types, SCREAMING_SNAKE_CASE constants. Keep setup, validation, execution, rendering, and return phases visually separated.

## Testing And Validation

Keep unit tests beside source. Crate integration tests live in `tests/`. Automated tests must not make live HTTP requests.

## Common Change Playbooks

For a new DTO or capability, add the type or trait here, then update both providers and the CLI only after the contract compiles. For output or confirmation changes, update `Renderer` and prompt helpers together with command tests.

## Free Region

No additional maintainer policy for this scope.

## Further Context

See [AGENTS.reference.md](AGENTS.reference.md) for provenance and crate-local evidence. Parent guidance lives in the repository-root AGENTS.md.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
