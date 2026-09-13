# AGENTS.md

## Scope

- Path: gitfleet
- Parent: .
- Inheritance: additive

## Mission And Repository Map

This crate owns the product CLI: Clap parsing, command handlers, service orchestration, and the `gitfleet` and `gf` binaries.

- `src/main.rs` defines the public command surface.
- `src/commands/` parses arguments and stays thin.
- `src/service/` executes typed workflows against provider contracts.
- `src/bin/gf.rs` exposes the same command surface as `gitfleet`.

## Non-Negotiables

- Keep command handlers thin and delegate to services.
- Render through `gitfleet_core::output::Renderer` and send tracing to stderr.
- Register public commands in the operation registry and Clap surface together.
- Keep the `gitfleet` and `gf` binaries equivalent.

## Don’ts

- Do not add public commands outside the operation registry.
- Do not restore legacy command aliases.
- Do not put `reqwest` calls or provider wire types in this crate.

## Quick Start

```bash
CARGO_BUILD_JOBS=4 cargo test -p gitfleet --locked
```

Install from this crate with `cargo install --path gitfleet`.

## Change Routing

CLI flags, help text, and orchestration land here. Shared DTOs and capabilities go to gitfleet-core first. Provider HTTP stays in gitfleet-providers. User-facing command pages live in gitfleet-docs.

## Architecture Rules

`src/commands/` maps Clap input onto services. Global flags include `--json`, `--debug`, `--theme`, `--yes`, and `--dry-run`. Destructive human operations require confirmation; JSON and non-interactive destructive operations require `--yes`.

## Implementation Conventions

Follow workspace Rust style. Keep command files focused on parsing and delegation. Use `assert_cmd` for CLI integration tests.

## Testing And Validation

Unit tests stay beside source. CLI integration tests live in `tests/` and use `assert_cmd`. Automated tests must not make live HTTP requests.

## Common Change Playbooks

For a command family, update the operation registry, service, command module, command tests, gitfleet-docs, and a reversible playbook. Implement confirmation, `--yes`, and `--dry-run` together for destructive bulk behavior.

## Free Region

No additional maintainer policy for this scope.

## Further Context

See [AGENTS.reference.md](AGENTS.reference.md) for provenance and crate-local evidence. Parent guidance lives in the repository-root AGENTS.md.

---

> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).
> Do not touch this file. It is automatically managed by Agentskill.
