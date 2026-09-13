# AGENTS.reference.md

## Provenance And Decisions

- Agentskill Version: `2.1.0`.
- Evidence Schema Version: `4`.
- Repository Revision: `7f407aead045dabd74f15c664b42fac5113cfffd`.
- Configuration: inherited from the repository root; default signature enabled.
- Maintainer-Confirmed Decisions: adopt gitfleet-providers as a nested scope; keep `reqwest` and wire types here; normalize before crossing crate boundaries.
- Unresolved Uncertainty: none specific to this crate beyond the root merge-strategy gap.

## Boundaries And Ownership

This crate is the nearest owner of GitHub and GitLab HTTP clients, wire payloads, endpoint modules, retry behavior, and capability implementation details. gitfleet-core remains the owner of DTOs and traits. Live API checks belong in gitfleet-playbooks, not in this crate's automated tests.

## Testing Topology

Provider tests use wiremock and insta JSON snapshots. `Cargo.toml` lists `wiremock`, `insta`, `http`, and `serial_test` as dev-dependencies. Automated tests must not make live requests.

## Evidence Summary

`src/lib.rs` exports `GitHubProvider`, `GitLabProvider`, and `ProviderRegistry`. Provider trees are nested under `src/github/` and `src/gitlab/`. `reqwest` is a crate dependency with the `json` feature.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
