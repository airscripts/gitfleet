# Contributing

When contributing to this repository, please first discuss the change you wish
to make via issue, email, or any other method with the owners of this
repository, ensuring you follow the
[Code of Conduct](https://github.com/airscripts/gitfleet/blob/main/CODE_OF_CONDUCT.md).

## Development Setup

Gitfleet is a Rust workspace. Build and test with Cargo:

```bash
CARGO_BUILD_JOBS=4 cargo build --workspace
cargo fmt --check
CARGO_BUILD_JOBS=4 cargo clippy -- -D warnings
CARGO_BUILD_JOBS=4 cargo check --workspace
CARGO_BUILD_JOBS=4 cargo test --workspace
CARGO_BUILD_JOBS=4 cargo llvm-cov --fail-under-lines 80 --workspace
CARGO_BUILD_JOBS=4 cargo build --release
```

Git hooks are managed by [Lefthook](https://github.com/evilmartians/lefthook).
Install hooks once after cloning:

```bash
lefthook install
```

The configured pre-commit hook runs formatting, clippy, a workspace build, and
the workspace coverage gate. There is no configured pre-push hook.

## Architecture

Read [AGENTS.md](./AGENTS.md) for the full architecture contract, crate
boundaries, code style, testing strategy, and release rules. Key points:

- `gitfleet-core` holds provider-neutral types, traits, and infrastructure.
- `gitfleet-providers` is the only crate that calls `reqwest`.
- `gitfleet-cli` is a thin surface over shared operations.
- Provider wire types must be normalized before crossing the crate boundary.
- Never add raw `println!` or `eprintln!` outside the output layer.
- Use `GitfleetError` enum variants for expected failures.

## Testing

Unit tests live inside source files in `#[cfg(test)] mod tests {}` blocks.
Integration tests live in each crate's `tests/` directory. Provider tests use
`wiremock` for HTTP mocking and `insta` for snapshot normalization. Automated
tests must never make real HTTP requests.

Live playbooks under `gitfleet-playbooks/` validate command families against
the real GitHub and GitLab APIs. They require an explicit token and test
repository for the active provider, and clean up mutations during teardown.
Run them only against a dedicated test repository.

## Commit Convention

All commit messages must use a lowercase prefix followed by a colon and space:

- `feat:` — new user-visible behavior
- `fix:` — bug fix
- `refactor:` — code restructure without behavior change
- `chore:` — build, release, dependency, or metadata changes
- `tests:` — test additions or modifications
- `ci:` — CI/CD workflow changes
- `documentation:` — documentation-only changes
- `repo:` — project scaffolding

Subject line: imperative mood, no period, under 50 characters. No scopes. No
body.

## Pull Requests

- Use the pull request template provided in the repository.
- Ensure all gates pass before submitting.
- Rebase your branch on `main` before opening a PR.
- One logical change per PR.
