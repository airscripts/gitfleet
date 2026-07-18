# Architecture Overview

This is a compact maintainer overview. The authoritative contributor contract
is [../AGENTS.md](../AGENTS.md).

## Crates

| Crate | Responsibility |
| ----- | -------------- |
| `gitfleet-core` | Provider-neutral entities, identifiers, capabilities, errors, operations, and shared infrastructure. |
| `gitfleet-providers` | GitHub and GitLab clients, wire types, endpoint wrappers, normalization, and capability implementations. |
| `gitfleet` | Product CLI crate and thin command surface over shared operations. |

## Boundaries

`gitfleet-core/provider.rs` defines the provider traits and capability subtraits.
Provider wire types stay inside `gitfleet-providers` and are normalized before
crossing the crate boundary.

Only `gitfleet-providers` calls `reqwest`. Unsupported behavior returns
`UnsupportedCapabilityError`.

## Supporting Directories

`gitfleet-playbooks/` is not a Rust crate. It is a live API validation directory
containing Bash playbooks for provider-backed smoke coverage. Maintainers run
those scripts against dedicated test repositories, and they stay outside Cargo's
automated test suite.

## CLI Shape

Public command families are registered through the operation vocabulary in
`gitfleet-core/src/operations.rs` and exposed by the Clap surface in
`gitfleet/src/main.rs`. Command handlers in `gitfleet/src/commands/` should stay
thin and delegate business behavior to shared operations or provider
capabilities.

## Validation

Automated tests use mocks and snapshots. Live playbooks intentionally call
provider APIs and must clean up mutations with teardown traps.
