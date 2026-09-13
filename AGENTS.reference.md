# AGENTS.reference.md

## Provenance And Decisions

- Agentskill Version: `2.1.0`.
- Evidence Schema Version: `4`.
- Repository Revision: `7f407aead045dabd74f15c664b42fac5113cfffd`.
- Configuration: default signature enabled.
- Maintainer-Confirmed Decisions: replace the existing root guidance with an Agentskill-managed operational document; preserve maintainer policy in its `## Free Region`; adopt nested scopes at `gitfleet-core/`, `gitfleet-providers/`, `gitfleet/`, and `gitfleet-site/` after explicit selection; keep homepage star-count fetches in the browser and mock GitHub in automated site tests.
- Unresolved Uncertainty: Git history does not establish a preferred merge strategy.

## Boundaries And Ownership

`gitfleet-core/src/provider.rs` defines provider and capability contracts. `gitfleet-core/src/types.rs` contains canonical DTOs. `gitfleet-core/src/operations.rs` is the public operation family registry. `gitfleet-providers` contains GitHub and GitLab clients, endpoint wrappers, wire payloads, and normalization. It is the only crate that may call `reqwest`. `gitfleet/src/commands/` parses CLI input and delegates to services under `gitfleet/src/service/`. `gitfleet-playbooks/` exercises live APIs with Bash and cleanup traps. Documentation is maintained under `gitfleet-docs/` and linked from `README.md`. `gitfleet-site/` is the Astro marketing homepage; `gitfleet-docs/commands/site.md` documents the CLI `gitfleet site` family. Canonical brand files live in `gitfleet-assets/`.

## Development Workflow

The root `Makefile` exposes install, formatting, Clippy, workspace check, tests, coverage, debug and release builds, metrics, and the aggregate `verify` target. CI separately checks Rust formatting, Clippy, workspace compilation, MSRV compilation, Bash syntax, Agentskill document validation, and the `gitfleet-site` package. Use `CARGO_BUILD_JOBS=4` for the workspace Rust gates. The site gate is `pnpm verify` in `gitfleet-site/`.

## Testing Topology

Rust unit tests are colocated in source modules. Crate integration tests live in `gitfleet-core/tests/`, `gitfleet-providers/tests/`, and `gitfleet/tests/`; CLI tests use `assert_cmd`. Provider tests use wiremock and insta snapshots. Playbooks under `gitfleet-playbooks/` are live API checks and must test positive and negative cases while cleaning created resources with `trap teardown EXIT`. Site unit tests live beside `gitfleet-site/src/`. Site integration tests read `gitfleet-site/dist/`. Playwright covers keyboard navigation, theme toggle, and mocked GitHub star counts. Automated tests must not make live HTTP requests.

## Product Safety

Human-readable output is the default and structured output is opt-in with `--json`. Destructive human operations require `inquire` confirmation. JSON and non-interactive destructive operations require `--yes`. Bulk mutations should expose `--dry-run`. Provider-neutral terminology belongs in the operation registry, and unsupported capabilities must fail explicitly rather than emulate another provider.

## Nested Scopes

Adopted scopes inherit this root document additively and keep independent `## Free Region` sections:

- `gitfleet-core/` — domain types, contracts, output, and prompts.
- `gitfleet-providers/` — GitHub and GitLab clients, wire types, and HTTP.
- `gitfleet/` — CLI parsing, services, and `gitfleet`/`gf` binaries.
- `gitfleet-site/` — Astro homepage, visual tokens, and Node quality gates.

`gitfleet-docs/`, `gitfleet-playbooks/`, and `gitfleet-assets/` remain root-owned until separately selected.

## Evidence Summary

The repository is a Rust workspace with a Makefile, TOML configuration and templates, YAML CI, Bash playbooks, an Astro/TypeScript site, and PNG brand assets. Git history uses conventional prefixes including `feat`, `fix`, `chore`, `ci`, `docs`, `perf`, `refactor`, `repo`, `test`, and `build`. Rust tests run under `cargo test --workspace`. Site tests use Vitest and Playwright, not live GitHub requests.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
