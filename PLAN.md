# Gitfleet Remaining Work

This document tracks the work remaining to complete the Gitfleet 0.1.0
Rust rewrite. Architecture, conventions, and boundaries are defined in
`AGENTS.md` and are not repeated here.

## Current State

All prior milestones (P1–P7) are complete. P8–P18 are also
complete. The Rust workspace passes all gates: `cargo fmt --check`,
`cargo clippy -D warnings`, `cargo check`, `cargo test --workspace`
(2,398+ test executions), `cargo llvm-cov` (87.21%), and
`cargo build --release`.

| Crate              | Line Coverage | Tests |
|---------------------|---------------|-------|
| gitfleet-core       | 93.5%         | 367   |
| gitfleet-providers  | 86.1%         | 467   |
| gitfleet-cli        | 88.4%         | 705   |
| **Total**           | **87.21%**    | **1,539** |

## Completed Work

### P8 — Legacy TypeScript Cleanup ✓

Deleted all TypeScript-era files (`src/`, `tests/`, `dist/`, `coverage/`,
`node_modules/`, `package.json`, `pnpm-lock.yaml`, `pnpm-workspace.yaml`,
`tsconfig.json`, `vite.config.ts`, `eslint.config.mjs`,
`.prettierrc.json`, `.prettierignore`, `.npmrc`, `templates/`, duplicate
root `CODEOWNERS`). Updated `.gitignore`. Renamed `scripts/` to
`gitfleet-scripts/`.

### P9 — CI Migration to Rust ✓

Rewrote all GitHub Actions workflows for the Rust workspace. Deleted
`deploy.yml`. Uses `dtolnay/rust-toolchain@stable`,
`Swatinem/rust-cache@v2`, `actions/upload-artifact@v4`.

### P10 — Pre-commit Hook Migration ✓

Deleted `.husky/`. Created `lefthook.yml` with pre-commit (fmt + clippy)
and pre-push (test) hooks.

### P11a — TUI Render Tests ✓

Added 43 inline render tests using `TestBackend` + `Terminal::draw()`.
All 6 modes tested (Dashboard, Normal, Insert, Palette, Confirm, Visual).
TUI coverage rose from 14% to 85.7%. (The TUI crate has since been removed.)

### P11b — CLI Command Handler Tests ✓

Added shared `test_helpers.rs` with `MockProvider` implementing all
capability traits. Added `ProviderRegistry::with_provider()` for
dependency injection. ~543 CLI unit tests covering every command variant,
JSON/silent/human modes, dry-run paths, no-caps error paths. CLI
coverage rose from 40.2% to 88.4%.

### P11d — Verify Gate ✓

Total workspace coverage 80.79% — above the 80% gate.

### P12 — Alias Command Implementation ✓

Added `aliases: HashMap<String, String>` to `CredentialsFile` with
`#[serde(default)]` for backward compatibility. Implemented
`set_alias`, `get_alias`, `list_aliases`, `delete_alias` in `config.rs`.
Wired `alias set/get/list/delete` CLI commands to persist to TOML.
Added `--force` for overwrite, `--yes` for delete confirmation.
12 config unit tests + 20 CLI serial tests + updated playbook.

### P11c — Provider Branch Coverage ✓

Added 72 GitHub wiremock integration tests covering all previously
untested trait methods: RepoOps (get/update/delete/fork/archive),
IssueOps (get/create/update/comment), ReviewOps (reactions CRUD),
PlanningOps (milestones + projects via GraphQL), ReleaseOps
(update/delete), PolicyOps (branch + tag protection), SiteOps (pages
CRUD), SnippetOps (gists CRUD), DevEnvOps (codespaces), RegistryOps
(packages), LicenseOps, DependencyOps, AdvisoryOps, AttestationOps,
BrowseOps, RawApiOps, TemplateOps, NotificationOps (mark_read),
AccessOps (teams/members), IdentityOps (GPG keys), DiscussionOps
(get/create). Added 11 GitLab wiremock integration tests covering
RawApiOps, BrowseOps, TemplateOps, DiscussionOps, PipelineOps.
Provider coverage rose from 67.6% to 86.1%.

## Required Gates

```bash
cargo fmt --check
cargo clippy -- -D warnings
cargo check
cargo test --workspace
cargo llvm-cov --workspace
cargo build --release
```

## Remaining Work

### P13 — Missing CLI Commands ✓

Added three CLI commands:
- **`api`** — raw provider API passthrough via `RawApiOps`:
  `gitfleet api get|post|delete --endpoint <endpoint>`. Post accepts
  `--body` as a JSON string. 13 unit tests + 5 integration tests.
- **`site`** — GitHub Pages management via `SiteOps`:
  `gitfleet site get|create|delete`. Create accepts `--source` and
  optional `--build-type`. 12 unit tests + 5 integration tests.
  Added `SiteOps` impl and `site_ops()` to `MockProvider`.
- **`tui`** — CLI launcher that spawns the `gitfleet-tui` binary as a
  subprocess. 1 unit test + 1 integration test. (The TUI crate has since
  been removed.)

All three registered in `main.rs` `Commands` enum and match block.
Playbooks: `api.sh`, `site.sh`, `tui.sh` added and included in
`all.sh`. All gates pass: 2,251 test executions, 86.98% line coverage.

### P14 — Config Arbitrary Keys ✓

Added `extra: HashMap<String, String>` field to `Profile` struct with
`#[serde(default, skip_serializing_if = "...is_empty")]` for backward
compatibility. `Profile` now derives `Default`. Updated `read`,
`write`, and `unset` in `config.rs` to store/retrieve arbitrary keys
from `profile.extra`. Typed accessors for `token`/`host` remain as
convenience wrappers. `unset` now errors when the profile doesn't
exist or the key is not found in `extra`. Added 5 config unit tests
(arbitrary key write/read/unset, nonexistent key error, TOML
round-trip with extra, deserialize without extra). Updated CLI config
tests: replaced "unsupported key" tests with arbitrary key tests (4
new serial tests). Updated `config.sh` playbook to test arbitrary keys.

### P15 — TUI Stub Documentation ✓

Added documentation comments to six TUI operation stubs (`alias`,
`api`, `config`, `deps`, `site`, `workspace`) marking them as
intentional stubs for 0.1.0 with rationale for deferral. Fixed
outdated CLI command references in `api.rs` ("raw-api" → "api") and
`site.rs` ("config get site" → "site get|create|delete"). (The TUI
crate has since been removed; these stubs were removed with it.)

### P16 — Provider Parity Audit ✓

Compared every GitHub API method against its GitLab counterpart.
Classified gaps as: (a) legitimate `UnsupportedCapabilityError`,
(b) missing implementation, or (c) GitHub-only.

Implemented 6 missing GitLab trait modules with 25 methods:
- **ReviewOps** (3 methods) — GitLab Award Emojis API
- **PlanningOps** (5 milestone methods) — GitLab Milestones API;
  4 project-board methods return `GitfleetError` (GitHub-only)
- **SiteOps** (3 methods) — GitLab Pages API
- **SnippetOps** (4 methods) — GitLab Snippets API
- **PolicyOps** (6 methods) — GitLab Protected Branches + Tags API
- **RegistryOps** (2 methods) — GitLab Package Registry API

Added 6 API modules (`review.rs`, `milestones.rs`, `site.rs`,
`snippets.rs`, `policy.rs`, `registry.rs`) with normalization and
unit tests. Updated `provider.rs` with 7 new capability variants
and getter methods. Added 25 wiremock integration tests in
`gitlab_extended.rs`. Remaining GitHub-only: DevEnvOps (Codespaces),
Projects (project boards). Documented parity matrix in
`PROVIDERS.md`.

### P17 — Insta Snapshot Adoption ✓

Adopted insta snapshots across the workspace. Added insta as a
dev-dependency to `gitfleet-cli` (already present in `gitfleet-core`
and `gitfleet-providers`).

- 6 `assert_snapshot!` tests for CLI `--help` text (main, repo, api,
  site, config) in `gitfleet-cli/tests/cli.rs`. (The `tui` snapshot
  was removed with the TUI crate.)
- 3 `assert_json_snapshot!` tests for GitHub provider wire payload
  normalization (repo, issue, milestone) in
  `gitfleet-providers/tests/github_extended.rs`.
- 3 `assert_json_snapshot!` tests for GitLab provider wire payload
  normalization (milestone, snippet, tag protection) in
  `gitfleet-providers/tests/gitlab_extended.rs`.
- 12 snapshot files accepted via `cargo insta accept`.
- Fixed 48 pre-existing clippy warnings across all crates
  (assertions_on_constants, redundant_locals, needless_borrows,
  useless_vec, items_after_test_module, unnecessary_unwrap).

### P18 — Documentation Finalization ✓

- Updated README.md to reflect GitLab as a built-in provider.
- Rewrote CONTRIBUTING.md for the Rust development workflow (Cargo
  commands, Lefthook hooks, architecture pointers, testing strategy).
- Updated CHANGELOG.md 0.1.0 entry with GitLab provider, insta
  snapshots, and Lefthook hooks.
- Verified version metadata agrees across `VERSION` (0.1.0), all
  `Cargo.toml` files, `CITATION.cff`, and `CHANGELOG.md`.
- Verified `.github/CODEOWNERS` is accurate.

## Execution Order

```
P8  ✓  P9  ✓  P10 ✓  P11a ✓  P11b ✓  P11c ✓  P11d ✓
P12 ✓  P13 ✓  P14 ✓  P15 ✓  P16 ✓  P17 ✓  P18 ✓
```

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
- Never stage or publish as part of the Gitfleet 0.1.0 implementation
  handoff.

## Clean Break

When 0.1.0 is ready:

1. Delete all 24 GitHub releases.
2. Delete all 24 remote tags.
3. Close all open PRs.
4. Create an orphan `main` branch with the Rust codebase.
5. Force push to GitHub.
6. Delete stale remote branches.
7. Rename the GitHub repository from `ghitgud` to `gitfleet`.
8. Update the local remote URL.
