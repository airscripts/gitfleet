# Gitfleet

[![Main](https://github.com/airscripts/gitfleet/actions/workflows/main.yml/badge.svg)](https://github.com/airscripts/gitfleet/actions/workflows/main.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Coverage](https://img.shields.io/badge/coverage-88%25-brightgreen)](./coverage)

Command every repository as one fleet.

![Gitfleet](gitfleet-assets/gitfleet.png)

## Table of Contents

- [Overview](#overview)
- [Providers](#providers)
- [Architecture](#architecture)
- [Features](#features)
- [Installation](#installation)
- [Authentication](#authentication)
- [Configuration](#configuration)
- [Profiles](#profiles)
- [Commands](#commands)
- [Common Workflows](#common-workflows)
- [Output Formats](#output-formats)
- [Safety](#safety)
- [Playbooks](#playbooks)
- [Development](#development)
- [Repository Structure](#repository-structure)
- [Contributing](#contributing)
- [Security](#security)
- [Support](#support)
- [Sponsorship](#sponsorship)
- [License](#license)

## Overview

Gitfleet manages repositories from creation and cloning through collaboration,
delivery, security, analytics, and governance. It keeps portable concepts such
as changes, pipelines, planning, and development environments consistent while
retaining provider-specific capabilities when they exist.

It is a provider-neutral fleet-management CLI, not a replacement name for a
single provider's CLI. Where `gh` is focused on GitHub and `glab` is focused on
GitLab, Gitfleet gives teams that use both providers one vocabulary, one command
surface, and named profiles for working across their repositories. It preserves
real provider differences: a command only runs when the selected provider
supports the required capability.

The `gitfleet` and `gf` binaries are equivalent. Human-readable output is the
default, and every automation workflow can opt into structured output with
`--json`.

## Providers

Gitfleet currently includes these providers:

| Provider | Default host | Use with Gitfleet |
| -------- | ------------ | ----------------- |
| GitHub   | `github.com` | GitHub repositories and GitHub Enterprise hosts configured with `--host` |
| GitLab   | `gitlab.com` | GitLab repositories through the same provider-neutral command families |

Configure a profile per provider or account, then switch profiles or let
`gitfleet auth detect` select the profile from the current repository remote.
This makes mixed GitHub and GitLab fleets manageable without changing tools or
learning provider-specific root command families.

## Architecture

| Crate                | Responsibility                                                      |
| -------------------- | ------------------------------------------------------------------- |
| `gitfleet-core`      | Provider-neutral entities, capabilities, operations, infrastructure |
| `gitfleet-providers` | GitHub and GitLab provider clients, wire types, normalization       |
| `gitfleet-cli`       | Thin CLI surface over shared operations                             |

Only `gitfleet-providers` performs HTTP requests. Unsupported operations return
a capability error instead of pretending another provider supports them.

## Features

- Repository lifecycle, cloning, forks, synchronization, stale branches, and
  multi-repository workspaces.
- Proposed changes, stacked changes, merge queues, reviews, suggestions,
  issues, discussions, notifications, and mentions.
- Pipeline definitions, runs, logs, artifacts, caches, releases, deployments,
  environments, registries, and runners.
- Planning boards, milestones, labels, templates, wikis, static sites, and
  license discovery.
- Dependency review, advisories, attestations, secret scanning, CodeQL,
  compliance, audit logs, secrets, and variables.
- Repository governance, policies, access management, account keys, analytics,
  code navigation, snippets, browser integration, and raw provider API access.
- Named profiles, explicit JSON output, terminal themes, aliases, completion,
  reversible live playbooks.

## Installation

Gitfleet is a Rust CLI. Install the CLI package from this checkout with Cargo:

```bash
cargo install --path gitfleet-cli

gitfleet version
gf version
```

This builds optimized release binaries and installs both commands into Cargo's
binary directory.

Cargo installs binaries into `~/.cargo/bin`; make sure that directory is on
your `PATH` if the commands are not found.

After pulling new source changes, reinstall the local package to refresh the
commands:

```bash
cargo install --path gitfleet-cli --force
```

## Authentication

Authenticate with a provider token and inspect the resulting profile:

```bash
gitfleet auth login --token <token>
gitfleet auth status
gitfleet auth token
```

GitHub Enterprise is supported through an explicit host:

```bash
gitfleet auth login \
  --host github.example.com \
  --profile work \
  --token <token>
```

For GitLab, select the provider explicitly. This is required for a
self-managed GitLab host whose name does not include `gitlab`:

```bash
gitfleet auth login \
  --provider gitlab \
  --host git.example.com \
  --profile work-gitlab \
  --token <token>
```

For CI, set `GITFLEET_GITHUB_TOKEN` or `GITFLEET_GITLAB_TOKEN`. Destructive
operations in JSON or non-interactive mode require `--yes`.

### GitHub Token Scopes

Choose the narrowest scopes required by the commands you use. For a classic
personal access token, the relevant scopes are:

| Scope                                 | Required for                                           |
| ------------------------------------- | ------------------------------------------------------ |
| `repo`                                | Private repositories, changes, issues, code, and CI    |
| `public_repo`                         | Public-repository access without the full `repo` scope |
| `workflow`                            | Adding or updating workflow files                      |
| `notifications`                       | Inbox notification operations                          |
| `read:discussion`, `write:discussion` | Reading or modifying GitHub discussions                |
| `read:user`                           | Account and activity information                       |
| `read:org`                            | Organization, team, and runner reads                   |
| `admin:org`                           | Organization membership and team mutations             |
| `read:project`, `project`             | Reading or modifying GitHub planning projects          |
| `gist`                                | Hosted snippet operations                              |
| `read:packages`, `write:packages`     | Reading or publishing package registry content         |
| `delete:packages`                     | Deleting package versions                              |
| `admin:repo_hook`, `admin:org_hook`   | Repository and organization webhook management         |
| `security_events`                     | Code scanning and security event operations            |
| `read:audit_log`                      | Organization audit log access                          |
| `codespace`                           | Hosted development environment operations              |
| `admin:public_key`, `admin:gpg_key`   | SSH and GPG key management                             |
| `delete_repo`                         | Destructive repository retirement workflows            |

Fine-grained tokens use repository and organization permissions instead of
classic scopes. Grant each selected repository the permissions required by the
commands you intend to run; some user-scoped endpoints may still require a
classic token. See GitHub's official documentation for
[classic scopes](https://docs.github.com/en/apps/oauth-apps/building-oauth-apps/scopes-for-oauth-apps)
and
[fine-grained permissions](https://docs.github.com/en/rest/authentication/permissions-required-for-fine-grained-personal-access-tokens).

## Configuration

Credentials are stored at `~/.config/gitfleet/credentials.toml` with mode
`0600`. Repository-local profile selection uses `.gitfleetrc`.

Resolution follows explicit flags, environment configuration, the
repository-local profile, the active profile, and then defaults. Repository
targets come from `--repo` or the current Git remote.

Environment variables:

- `GITFLEET_GITHUB_TOKEN` supplies the GitHub token in automation.
- `GITFLEET_GITLAB_TOKEN` supplies the GitLab token in automation.
- `GITFLEET_PROFILE` selects a named profile.
- `CI=true` enables non-interactive behavior.

## Profiles

Named profiles support separate provider accounts and hosts:

```bash
gitfleet auth login --profile personal --token <token>
gitfleet auth login --profile work --token <token>
gitfleet auth list
gitfleet auth switch work
gitfleet auth detect
```

The active profile supplies the provider, host, and token for subsequent
operations. `auth detect` selects a profile from the current repository.

## Commands

Use `gitfleet help` for the complete surface or target nested help directly:

```bash
gitfleet help
gitfleet help change
gitfleet help pipeline run
```

| Family                                         | Purpose                                                    |
| ---------------------------------------------- | ---------------------------------------------------------- |
| `auth`                                         | Provider accounts and profiles                             |
| `repo`                                         | Repository lifecycle, forks, synchronization, and branches |
| `change`                                       | Proposed changes, stacks, checks, and merge automation     |
| `review`                                       | Threads, comments, suggestions, reactions, and resolution  |
| `issue`, `discussion`, `inbox`                 | Collaboration and notification workflows                   |
| `pipeline`                                     | Definitions, runs, logs, artifacts, and caches             |
| `release`, `deploy`, `environment`             | Delivery lifecycle                                         |
| `workspace`                                    | Named fleets and bounded multi-repository execution        |
| `govern`, `policy`                             | Fleet governance and repository protection                 |
| `planning`                                     | Boards, work items, milestones, and iterations             |
| `wiki`, `site`                                 | Repository documentation and publishing                    |
| `search`, `code`, `browse`, `api`              | Discovery, navigation, and provider escape hatches         |
| `label`, `template`, `license`                 | Repository metadata                                        |
| `deps`, `advisory`, `attestation`, `security`  | Dependency and security operations                         |
| `registry`, `dev`, `runner`, `webhook`         | Build and delivery infrastructure                          |
| `secret`, `variable`                           | Automation configuration                                   |
| `access`, `identity`                           | Organization access and account keys                       |
| `analytics`, `snippet`                         | Reporting and hosted snippets                              |
| `alias`, `completion`, `config`, `help`        | Gitfleet utilities                                         |
| `version`                                      | Version information                                        |

### Selected Nested Commands

- `pipeline definition` manages pipeline definitions.
- `pipeline run` inspects, watches, reruns, and debugs runs.
- `pipeline cache` manages pipeline caches.
- `security audit`, `security leaks`, `security dependabot`,
  `security compliance`, and `security codeql` expose security capabilities.
- `analytics repo` and `analytics pipeline` provide repository and pipeline
  reporting.
- `access org` and `access team` manage organizations, groups, and teams.
- `identity ssh` and `identity gpg` manage provider account keys.
- `change queue`, `planning milestone`, `policy branch`, and `repo fork`
  expose provider capabilities without adding provider-specific root names.

## Common Workflows

### Change Review

```bash
gitfleet change create --title "Add feature"
gitfleet change list --state open
gitfleet change stack create --base main
gitfleet review threads 42
gitfleet review suggest 42 --file src/index.ts --line 12 --body "Use this"
```

### Pipeline Delivery

```bash
gitfleet pipeline definition list
gitfleet pipeline run list
gitfleet pipeline run watch <run-id>
gitfleet pipeline cache inspect
gitfleet release list
gitfleet deploy list
```

### Security Governance

```bash
gitfleet security compliance check --repos owner/repository
gitfleet security leaks scan --repo owner/repository
gitfleet policy list --repo owner/repository
gitfleet govern report --org example
```

### Workspaces

```bash
gitfleet workspace define \
  --name platform \
  --repos owner/api \
  --repos owner/web
gitfleet workspace list
gitfleet workspace run --name platform --command "issue list --state open"
```

Workspace targets may be provider-qualified:

```text
github@github.com:owner/repository
github@github.example.com:platform/repository
gitlab@gitlab.com:group/project
```

## Output Formats

Human-readable output is the default:

```bash
gitfleet issue list
```

Use structured output explicitly:

```bash
gitfleet issue list --json
```

Global options include:

- `--json` for machine-readable results.
- `--debug` for a redacted diagnostic log.
- `--theme dark|light|auto` for terminal color selection.

## Safety

Destructive human-mode operations request confirmation. JSON and
non-interactive operations require `--yes`; bulk mutations provide `--dry-run`
when a useful preview is possible.

## Playbooks

Live playbooks under `gitfleet-playbooks/` validate command families against
the GitHub API. They require explicit credentials and revert mutations during
teardown.

```bash
REPO=owner/test-repository ORG=example bash gitfleet-playbooks/all.sh
SKIP="pipeline-run.sh,planning.sh" bash gitfleet-playbooks/all.sh
PARALLEL=1 bash gitfleet-playbooks/all.sh
```

Use a dedicated test repository and review each playbook before running it.
Automated tests never make real HTTP requests.

## Development

Install Lefthook once after cloning or when recreating the local Git metadata:

```bash
lefthook install
```

Make sure `lefthook` and `cargo-llvm-cov` are available on your `PATH` before
committing. Install the coverage tool with `cargo install cargo-llvm-cov` if
needed.

The pre-commit hook runs formatting, linting, a workspace build, and
`cargo llvm-cov --fail-under-lines 80 --workspace`, each capped at four Cargo
build jobs to keep local resource usage manageable. Coverage runs the test
suite, so a separate `cargo test --workspace` step is unnecessary for the
normal pre-commit workflow. To run the hook manually:

```bash
lefthook run pre-commit
```

Build release binaries separately when you want to inspect the optimized
artifacts or perform the final release check:

```bash
CARGO_BUILD_JOBS=4 cargo build --release
./target/release/gitfleet version
./target/release/gf version
```

The workspace currently lists 2,239 Rust test entries as reported by Cargo:

```bash
CARGO_BUILD_JOBS=4 cargo test --workspace -- --list | grep -c ': test$'
```

Coverage must remain at or above 80 percent. See [AGENTS.md](./AGENTS.md) for
architecture, testing, style, playbook, and release requirements.

## Repository Structure

```text
gitfleet-core/        provider-neutral entities, capabilities, errors, operations,
                      infrastructure (config, git, output, prompts, secrets, workspace)
gitfleet-providers/   GitHub and GitLab provider clients, wire types, normalization
gitfleet-cli/         thin CLI surface over shared operations
gitfleet-playbooks/   live API test scripts (bash)
```

`PLAN.md` is the Gitfleet refactor contract. `ROADMAP.md` tracks the Rust
rewrite and GitLab provider.

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) before proposing a change.

## Security

Report vulnerabilities through the private process in
[SECURITY.md](./SECURITY.md), not a public issue.

## Support

For usage help, run `gitfleet help [command...]`. For reproducible bugs, open an
issue with the Gitfleet version, operating system, command, expected behavior,
actual behavior, and redacted `--debug` output.

## Sponsorship

Gitfleet is maintained by Airscripts. If it helps your team manage repositories
more effectively, sponsor ongoing maintenance and provider coverage through
[GitHub Sponsors](https://github.com/sponsors/airscripts).

## License

MIT. See [LICENSE](./LICENSE).
