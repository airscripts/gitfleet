# Gitfleet

[![Main](https://github.com/airscripts/gitfleet/actions/workflows/main.yml/badge.svg)](https://github.com/airscripts/gitfleet/actions/workflows/main.yml)
[![Coverage](https://img.shields.io/badge/coverage-90.33%25-brightgreen)](https://github.com/airscripts/gitfleet/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![LOC](https://img.shields.io/badge/loc-54149-blue)](gitfleet-scripts/loc.sh)
[![Tests](https://img.shields.io/badge/tests-1803-blue)](gitfleet-scripts/tests.sh)

Command every repository as one fleet.

![Gitfleet](gitfleet-assets/gitfleet.png)

## Overview

Gitfleet is a provider-neutral CLI for managing repositories across GitHub and
GitLab. It keeps one product vocabulary for repository lifecycle, changes,
issues, pipelines, releases, security, access, and fleet workflows while still
respecting real provider capability differences.

Human-readable output is the default. Automation can opt into structured output
with `--json`. The `gitfleet` and `gf` binaries are equivalent.

## Providers

| Provider | Default host | Use with Gitfleet |
| -------- | ------------ | ----------------- |
| GitHub   | `github.com` | GitHub repositories and GitHub Enterprise hosts configured with `--host` |
| GitLab   | `gitlab.com` | GitLab.com and self-managed GitLab hosts configured with `--host` |

Gitfleet checks capabilities at runtime. If the active provider cannot perform
an operation, Gitfleet reports that capability as unsupported instead of
pretending every provider has the same behavior.

## Installation

Download a prebuilt archive from
[GitHub Releases](https://github.com/airscripts/gitfleet/releases). Each archive
includes both executable names, `gitfleet` and `gf`, plus the project license.

Gitfleet is also installable from this checkout with Cargo:

```bash
cargo install --path gitfleet

gitfleet version
gf version
```

After pulling new source changes, reinstall the local package:

```bash
cargo install --path gitfleet --force
```

## Quick Start

```bash
gitfleet auth login
gitfleet auth status
gitfleet repo list
gitfleet issue list --repo owner/repository
```

Use nested help for the current command surface:

```bash
gitfleet help
gitfleet help repo
gitfleet help pipeline list-runs
```

## Documentation

The full Markdown documentation lives in [gitfleet-docs/](./gitfleet-docs/).
The static homepage source lives in [gitfleet-site/](./gitfleet-site/).

- [Concepts](./gitfleet-docs/concepts.md)
- [Providers](./gitfleet-docs/providers.md)
- [Authentication](./gitfleet-docs/authentication.md)
- [Configuration](./gitfleet-docs/configuration.md)
- [Command Reference](./gitfleet-docs/commands/README.md)
- [Workflows](./gitfleet-docs/workflows/README.md)
- [Safety](./gitfleet-docs/safety.md)
- [Troubleshooting](./gitfleet-docs/troubleshooting.md)
- [Architecture Overview](./gitfleet-docs/architecture-overview.md)
- [Documentation Guide](./gitfleet-docs/documentation-guide.md)

## Development

Install Lefthook once after cloning or when recreating local Git metadata:

```bash
lefthook install
```

Required gates from the repository root:

```bash
cargo fmt --check
CARGO_BUILD_JOBS=4 cargo clippy -- -D warnings
CARGO_BUILD_JOBS=4 cargo check --workspace
CARGO_BUILD_JOBS=4 cargo test --workspace
CARGO_BUILD_JOBS=4 cargo llvm-cov --fail-under-lines 80 --workspace
CARGO_BUILD_JOBS=4 cargo build --release
```

Coverage must remain at or above 80 percent. See [AGENTS.md](./AGENTS.md) for
crate boundaries, coding style, testing rules, playbooks, and release rules.

The static homepage has its own Node-based quality gate and is intentionally
kept separate from Rust CLI LOC, test, and coverage metrics:

```bash
cd gitfleet-site
pnpm verify
```

The site gate uses Prettier for formatting, ESLint plus `astro check` for
linting/type diagnostics, Vitest for unit and build-output integration tests,
and Astro's static build.

## Security

Report vulnerabilities using the private process in [SECURITY.md](./SECURITY.md)
and do not open a public issue. Never include tokens or other credentials in
reports; use redacted `--debug` output for ordinary bug reports.

## Support

For usage help, run `gitfleet help [command...]`. For reproducible bugs, open an
issue with the Gitfleet version, operating system, command, expected behavior,
actual behavior, and redacted `--debug` output.

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) before proposing a change.

## Sponsorship

Gitfleet is maintained by Airscripts. If it helps your team manage repositories
more effectively, sponsor ongoing maintenance and provider coverage through
[GitHub Sponsors](https://github.com/sponsors/airscripts).

## License

MIT. See [LICENSE](./LICENSE).
