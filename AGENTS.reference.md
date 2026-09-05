# AGENTS.reference.md

## Provenance And Decisions

- Agentskill Version: `2.1.0`.
- Evidence Schema Version: `4`.
- Repository Revision: `411b5812305ff61eb40fcfcb91a00ea82d4b1cfb`.
- Configuration: default signature enabled.
- Maintainer-Confirmed Decisions: replace the existing root guidance with an Agentskill-managed operational document; preserve maintainer policy in its `## Free Region`; keep nested scope candidates advisory until each repository-relative scope is explicitly selected.
- Unresolved Uncertainty: Git history does not establish a preferred merge strategy; `gitfleet-site` is detected as a separate package candidate.

## Boundaries And Ownership

`gitfleet-core/src/provider.rs` defines provider and capability contracts. `gitfleet-core/src/types.rs` contains canonical DTOs. `gitfleet-providers` contains GitHub and GitLab clients, endpoint wrappers, wire payloads, and normalization. It is the only crate that may call `reqwest`. `gitfleet/src/commands/` parses CLI input and delegates to services under `gitfleet/src/service/`. `gitfleet-playbooks/` exercises live APIs with Bash and cleanup traps. Documentation is maintained under `gitfleet-docs/` and linked from `README.md`.

## Development Workflow

The root `Makefile` exposes install, formatting, Clippy, workspace check, tests, coverage, debug and release builds, metrics, and the aggregate `verify` target. CI separately checks Rust formatting, Clippy, workspace compilation, MSRV compilation, Bash syntax, and the `gitfleet-site` package. Use `CARGO_BUILD_JOBS=4` for the workspace Rust gates.

## Testing Topology

Rust unit tests are colocated in source modules. Crate integration tests live in `gitfleet-core/tests/`, `gitfleet-providers/tests/`, and `gitfleet/tests/`; CLI tests use `assert_cmd`. Provider tests use wiremock and insta snapshots. Playbooks under `gitfleet-playbooks/` are live API checks and must test positive and negative cases while cleaning created resources with `trap teardown EXIT`.

## Product Safety

Human-readable output is the default and structured output is opt-in with `--json`. Destructive human operations require `inquire` confirmation. JSON and non-interactive destructive operations require `--yes`. Bulk mutations should expose `--dry-run`. Provider-neutral terminology belongs in the operation registry, and unsupported capabilities must fail explicitly rather than emulate another provider.

## Scope Candidates

Agentskill detected `gitfleet`, `gitfleet-core`, `gitfleet-providers`, and `gitfleet-site` as nested candidates. They inherit the root document until explicitly adopted. No scoped documents are created by this migration.

## Evidence Summary

The repository is a Rust workspace with a Makefile, TOML configuration and templates, YAML CI, Bash playbooks, and an Astro/TypeScript site. Git history contains 85 commits with `feat`, `fix`, `chore`, `ci`, `docs`, `perf`, `refactor`, `repo`, and `test` prefixes. The analyzer reports Rust tests under `cargo test --workspace`; site tests use Jest conventions.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
