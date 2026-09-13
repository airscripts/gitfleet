# AGENTS.md

## Mission And Repository Map

Gitfleet is a provider-neutral Rust CLI for managing GitHub and GitLab repositories.

- `gitfleet-core/` owns domain types, provider contracts, infrastructure, output, and prompts.
- `gitfleet-providers/` owns provider clients, wire payloads, normalization, and capabilities.
- `gitfleet/` owns the thin CLI surface and service orchestration.
- `gitfleet-playbooks/` contains Bash live API checks; it is not a Cargo crate.
- `gitfleet-docs/` contains user-facing command, workflow, and provider documentation.
- `gitfleet-site/` owns the static Astro homepage.
- `gitfleet-assets/` contains canonical cover, logo, and favicon graphics.

Scoped `AGENTS.md` files exist in `gitfleet-core/`, `gitfleet-providers/`, `gitfleet/`, and `gitfleet-site/`. They inherit this document additively.

## Non-Negotiables

- Keep provider-neutral DTOs and contracts in `gitfleet-core`; keep provider wire types and all `reqwest` calls in `gitfleet-providers`.
- Normalize provider responses before they cross into `gitfleet-core` or `gitfleet`.
- Route expected failures through `GitfleetError` and unsupported capabilities through `UnsupportedCapabilityError`.
- Keep command handlers thin, use shared services, render through `output::Renderer`, and send tracing to stderr.
- Update the relevant documentation whenever public commands, flags, output, provider capability support, or homepage behavior changes.

## Don’ts

- Do not add raw `println!` or `eprintln!` outside established output boundaries.
- Do not call `reqwest` outside provider client modules.
- Do not add provider wire types to `gitfleet-core` or `gitfleet`.
- Do not add public commands outside the operation registry or restore legacy aliases.
- Do not emulate unsupported provider behavior.

## Quick Start

```bash
make install
make verify
```

The required CLI gates are formatting, warnings-as-errors Clippy, workspace check and tests, 80% line coverage, release build, and repository metrics. The homepage gate is `pnpm verify` inside `gitfleet-site/`.

## Change Routing

Put shared behavior and canonical DTOs in `gitfleet-core`, provider behavior in `gitfleet-providers`, CLI parsing and orchestration in `gitfleet`, live API coverage in `gitfleet-playbooks`, user behavior changes in `gitfleet-docs`, homepage and visual changes in `gitfleet-site`, and brand graphics in `gitfleet-assets`.

## Architecture Rules

Use provider capability traits from `gitfleet-core/src/provider.rs`. Keep `gitfleet/src/commands/` as a thin surface over typed services. Configuration is TOML under the user configuration directory, with `GITFLEET_` environment variables. Human output is the default; JSON is explicit. Destructive operations require confirmation, or `--yes` in JSON and non-interactive modes. Bulk mutations should provide meaningful `--dry-run` previews. The Astro homepage is not the CLI `gitfleet site` command family.

## Implementation Conventions

Use four-space Rust formatting with a 100-column limit, grouped imports, snake_case functions and variables, PascalCase types, and SCREAMING_SNAKE_CASE constants. Keep setup, validation, execution, rendering, and return phases visually separated. Use one concept per file in `gitfleet-core` and nested provider folders in `gitfleet-providers`. Format `gitfleet-site` with Prettier at print width 100.

## Testing And Validation

Keep unit tests beside source, integration tests in `gitfleet-core/tests/`, `gitfleet-providers/tests/`, and `gitfleet/tests/`, and Bash playbooks under `gitfleet-playbooks/`. Mock HTTP with wiremock and use insta for normalization snapshots. Site tests use Vitest beside `gitfleet-site/src/`, build-output checks in `gitfleet-site/tests/`, and Playwright against the Astro production preview. Automated tests must not make live requests. Refresh LOC, test-count, and coverage shields after CLI implementation or test changes.

## Common Change Playbooks

For provider changes, update the trait, both implementations where supported, normalization tests, and provider notes. For a command family, update the operation registry, service, command tests, documentation, and reversible playbook. For destructive bulk behavior, implement confirmation, `--yes`, and a dry-run path together. For homepage changes, keep the header, ASCII hero, terminal, and footer skeleton, lock sonar cyan tokens, mock the GitHub stars API, and update `gitfleet-site/README.md`.

## Free Region

Maintainer policy: keep implementation changes unstaged unless explicitly requested. Do not commit, tag, push, publish, alter remotes, rename repositories, or delete releases. Keep `PLAN.md` and `ROADMAP.md` for planning only. Use conventional commits with a lowercase prefix and optional submodule scope. Keep release metadata synchronized across `VERSION`, Cargo manifests, `CITATION.cff`, `CHANGELOG.md`, and documentation.

For every implementation handoff, include ready-to-run conventional commit commands grouped by affected submodule. Use only submodule scopes without the `gitfleet-` prefix; use unscoped commits for root or CI changes. Do not run the commands unless explicitly requested.

## Further Context

See [AGENTS.reference.md](AGENTS.reference.md) for provenance, detailed boundaries, evidence, workflows, and unresolved decisions.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
