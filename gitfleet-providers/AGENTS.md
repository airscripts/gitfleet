# AGENTS.md

## Scope

- Path: gitfleet-providers
- Parent: .
- Inheritance: additive

## Mission And Repository Map

This crate owns GitHub and GitLab clients, wire payloads, endpoint wrappers, normalization, and capability implementations.

- `src/github/` contains the GitHub client, provider, and API modules.
- `src/gitlab/` contains the GitLab client, provider, and API modules.
- `src/registry.rs` constructs the provider registry.
- `src/lib.rs` is the last crate surface that may reshape provider payloads.

## Non-Negotiables

- Keep all `reqwest` calls and provider wire types in this crate.
- Finish mapping wire payloads onto core DTOs before values leave this crate.
- Implement capabilities from gitfleet-core traits; return `UnsupportedCapabilityError` when a provider cannot perform an operation.
- Keep GitHub and GitLab code in nested provider folders.

## Don’ts

- Do not expose raw provider JSON as core DTOs.
- Do not invent GitHub or GitLab behavior that the provider does not support.
- Do not call live GitHub or GitLab APIs from automated tests.
- Do not add provider-neutral domain types that belong in gitfleet-core.

## Quick Start

```bash
CARGO_BUILD_JOBS=4 cargo test -p gitfleet-providers --locked
```

## Change Routing

Provider endpoint, payload, retry, and capability-support changes land here. Update gitfleet-core first when a new canonical DTO or capability is required, then both provider implementations where supported, then provider notes in gitfleet-docs.

## Architecture Rules

GitHub and GitLab modules wrap HTTP endpoints and map wire types onto core DTOs. Capability support is declared on each provider implementation, not inferred by the CLI.

## Implementation Conventions

Follow workspace Rust style. Nest API modules per provider. Keep retry and shared HTTP helpers crate-local.

## Testing And Validation

Use wiremock for HTTP and insta for normalization snapshots. Unit tests stay beside source; crate integration tests live in `tests/`. Automated tests must not make live requests.

## Common Change Playbooks

For a new provider operation, add the capability in core if needed, implement or reject it in both providers, add wiremock coverage, snapshot normalized payloads, and update provider documentation.

## Free Region

No additional maintainer policy for this scope.

## Further Context

See [AGENTS.reference.md](AGENTS.reference.md) for provenance and crate-local evidence. Parent guidance lives in the repository-root AGENTS.md.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
