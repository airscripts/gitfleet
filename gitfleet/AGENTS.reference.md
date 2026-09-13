# AGENTS.reference.md

## Provenance And Decisions

- Agentskill Version: `2.1.0`.
- Evidence Schema Version: `4`.
- Repository Revision: `7f407aead045dabd74f15c664b42fac5113cfffd`.
- Configuration: inherited from the repository root; default signature enabled.
- Maintainer-Confirmed Decisions: adopt the gitfleet crate as a nested scope; keep commands thin over `src/service/`; keep `gitfleet` and `gf` equivalent; do not restore legacy aliases.
- Unresolved Uncertainty: none specific to this crate beyond the root merge-strategy gap.

## Boundaries And Ownership

This crate is the nearest owner of Clap command definitions, alias expansion, and service orchestration. It depends on gitfleet-core and gitfleet-providers. `src/commands/site.rs` is the CLI `gitfleet site` family, not the Astro homepage.

## Testing Topology

CLI tests use `assert_cmd`, `predicates`, `tempfile`, and insta. Wiremock appears in this crate's dev-dependencies for tests that stub provider HTTP through the service layer. Automated tests must not make live requests.

## Evidence Summary

`Cargo.toml` defines binary entries for `gitfleet` and `gf`. `src/main.rs` uses Clap `Parser`/`Subcommand` with global `--json`, `--debug`, `--theme`, `--yes`, and `--dry-run`. Command modules live under `src/commands/`; services live under `src/service/`.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
