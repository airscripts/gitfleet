# AGENTS.md

## Overview

Gitfleet is a Rust CLI for provider-neutral repository
management. GitHub and GitLab are built-in providers. The dependency direction
is:

```text
gitfleet-cli ->	gitfleet-core
             -> gitfleet-providers -> gitfleet-core
```

`PLAN.md` is the implementation contract and `ROADMAP.md` contains deferred
milestones.

## Crates

- `gitfleet-core` contains provider-neutral entities, identifiers, capabilities,
  errors, operations, and all infrastructure (config, git, output, prompts,
  secrets, workspace).
- `gitfleet-providers` owns provider clients, wire types, endpoint wrappers,
  normalization, and capability implementations for GitHub and GitLab. This is
  the only crate that calls reqwest.
- `gitfleet-cli` is a thin surface over shared operations.
- `gitfleet-playbooks` contains live API test scripts (bash, not Rust).

## Boundaries

- `gitfleet-core/provider.rs` contains the GitProvider trait and capability
  subtraits. This is the contract that providers implement.
- `gitfleet-providers` is the only crate that calls reqwest. Normalize provider
  payloads before they leave the crate. Unsupported behavior must use
  `UnsupportedCapabilityError` instead of emulating another provider.
- `gitfleet-cli/commands/` is a thin surface
  over shared operations. No business logic in command handlers.
- `gitfleet-core/types.rs` are the canonical DTOs. Provider wire types live
  inside `gitfleet-providers` and must be normalized to domain types before
  crossing the boundary.

## Product Conventions

- Human-readable output is the default; `--json` is explicit.
- Use `output::Renderer` for rendering and `tracing` for status/logging.
- `tracing` writes to stderr. Structured output writes to stdout.
- Use `GitfleetError` enum variants for expected failures.
- Destructive human-mode operations require confirmation via `inquire`.
- Destructive JSON or non-interactive operations require `--yes`.
- Bulk mutations provide `--dry-run` behavior when a preview is meaningful.
- CLI command labels use provider-neutral terminology from the
  operation registry.
- No legacy aliases. Canonical names only.
- Configuration lives under `~/.config/gitfleet/`; environment variables use
  the `GITFLEET_` prefix. Config format is TOML.
- Exit codes: 0 for success, 1 for general error, 2 for usage errors.

## Code Style

- rustfmt defaults: 4-space indentation, 100-column width.
- snake_case for functions, variables, and modules.
- PascalCase for types, structs, enums, and traits.
- SCREAMING_SNAKE_CASE for constants.
- Group imports as std, external crates, then crate-internal with blank lines.
- Format Rust with breathing room between logical code blocks. A block of code
  that is preceded and followed by code should normally have a blank line on
  both sides.
- Separate enum variants, match arms, trait methods, impl methods, and test
  cases with blank lines.
- Separate setup, validation, execution, rendering, and return phases inside
  functions. For example, keep provider lookup, capability lookup, request
  execution, response shaping, renderer calls, and `Ok(())` visually distinct.
- Separate completed multiline expressions from the next operation. This
  includes chains ending in `.await?`, `.collect()`, `.map_err(...)`, request
  calls, renderer calls, and destructive confirmation calls.
- In provider API code, keep endpoint construction, request body construction,
  request execution, response parsing, and return values as separate visual
  blocks.
- In command handlers, keep command parsing/resolution, dry-run handling,
  confirmation, provider operation calls, rendering, and final return separated.
- In tests, separate arrange, act, assert, and cleanup phases. Keep related
  assertions together only when they describe the same object or behavior.
- Do not insert blank lines inside tight syntax-bound constructs where they
  reduce clarity, such as simple struct literals, short argument lists, chained
  method calls, or adjacent assertions that intentionally read as one group.
- Prefer readable grouping over sorting by line length. Import ordering is by
  group, not by visual shape.
- Keep command registration thin and business behavior typed.
- Never add raw `println!` or `eprintln!` outside established output
  boundaries.
- Never call `reqwest` outside a provider client module.
- Cargo.toml crate names use kebab-case. Module imports use snake_case.
- Flat module layout in `gitfleet-core`: one file per concept at `src/` root.
- Nested subsystem layout in `gitfleet-providers`: each provider is a folder.
- Technical group layout in `gitfleet-cli`: `commands/` then topic files.

## Testing

Unit tests live inside source files in `#[cfg(test)] mod tests {}` blocks.
Integration tests live in each crate's `tests/` directory. Coverage must remain
at or above 80 percent.

- `gitfleet-core/tests/` — integration tests + fixtures
- `gitfleet-providers/tests/` — provider integration tests + fixtures
- `gitfleet-cli/tests/` — CLI integration tests (assert_cmd)
- `gitfleet-playbooks/` — live API playbooks (bash, not cargo)

Do not make real HTTP requests in automated tests. Mock provider clients with
wiremock and use insta for normalization snapshots. Every retained command
family must have unit, integration, and reversible live-playbook coverage.

Required gates:

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo check
cargo test --workspace
cargo llvm-cov --fail-under-lines 80
cargo build --release
```

## Playbooks

Playbooks live under `gitfleet-playbooks/`, source `gitfleet-playbooks/env.sh`,
test positive and negative cases, and always clean up mutations with
`trap teardown EXIT`. Resources use a `gitfleet-test-` or `gitfleet_` prefix.
Output uses `[INFO]`, `[OK]`, `[ERROR]`, `[WARN]`, and `[DEBUG]` without
decorative lines.

## Release and Git Rules

- Keep implementation changes unstaged unless the owner explicitly requests
  staging or publication.
- Do not commit, tag, push, publish, alter remotes, rename the remote
  repository, or delete releases.
- Release metadata must agree across `VERSION`, all `Cargo.toml` files,
  `CITATION.cff`, `CHANGELOG.md`, and documentation.
- Conventional commits use a lowercase prefix, colon, space, and a short
  imperative subject.

## Red Lines

- Never call `reqwest` outside a provider client module.
- Never add provider wire types to gitfleet-core or gitfleet-cli.
- Never bypass the shared output layer for structured rendering.
- Never use bare `Error` or `anyhow` for expected failures.
- Never add a public command family outside the operation registry.
- Never add a command without tests and a corresponding playbook.
- Never restore legacy `ghg`, automatic `gh` proxying, or parity-only
  behavior.
- Never assume unsupported provider capabilities.
- Never stage or publish as part of the Gitfleet 0.1.0 implementation handoff.
