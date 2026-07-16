# Gitfleet

[![Main](https://github.com/airscripts/gitfleet/actions/workflows/main.yml/badge.svg)](https://github.com/airscripts/gitfleet/actions/workflows/main.yml)
[![Coverage](https://img.shields.io/badge/coverage-91%25-brightgreen)](https://github.com/airscripts/gitfleet/actions/workflows/test.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![LOC](https://img.shields.io/badge/loc-52784-blue)](gitfleet-scripts/loc.sh)
[![Tests](https://img.shields.io/badge/tests-1778-blue)](gitfleet-scripts/tests.sh)

Command every repository as one fleet.

![Gitfleet](gitfleet-assets/gitfleet.png)

## Table of Contents

- [Overview](#overview)
- [Providers](#providers)
- [Provider Support](#provider-support)
- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Authentication](#authentication)
- [Configuration](#configuration)
- [Profiles](#profiles)
- [Commands](#commands)
- [Common Workflows](#common-workflows)
- [Output Formats](#output-formats)
- [Safety](#safety)
- [Development](#development)
- [Testing](#testing)
- [Architecture](#architecture)
- [Repository Structure](#repository-structure)
- [Security](#security)
- [Support](#support)
- [Contributing](#contributing)
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

Human-readable output is the default, and every automation workflow can opt
into structured output with `--json`.

The `gitfleet` and `gf` binaries are equivalent and expose the same command
surface.

## Providers

Gitfleet currently includes these providers:

| Provider | Default host | Use with Gitfleet |
| -------- | ------------ | ----------------- |
| GitHub   | `github.com` | GitHub repositories and GitHub Enterprise hosts configured with `--host` |
| GitLab   | `gitlab.com` | GitLab repositories through the same provider-neutral command families |

GitLab supports repository wiki operations. GitHub does not provide a supported
wiki API, so Gitfleet reports the wiki capability as unavailable instead of
emulating it through Git operations.

GitLab supports protected-tag operations through `policy tag`. GitHub reports
that capability as unavailable; use GitHub repository rulesets through
`govern ruleset` when tag policies are needed there.

Configure a profile per provider or account, then switch profiles or let
`gitfleet auth detect` select the profile from the current repository remote.
This makes mixed GitHub and GitLab fleets manageable without changing tools or
learning provider-specific root command families.

## Provider Support

Gitfleet checks capabilities at runtime and returns an explicit unsupported
capability error when the active provider cannot perform an operation.

| Command capabilities | GitHub | GitLab |
| -------------------- | ------ | ------ |
| Repositories, changes, reviews, issues, pipelines, releases | Yes | Yes |
| Milestones, webhooks, access, identity, notifications | Yes | Yes |
| Search, code, labels, templates, environments, runners | Yes | Yes |
| Variables, browse, raw API, deployments, licenses, snippets | Yes | Yes |
| Repository policies and package registry | Yes | Yes |
| Projects, Pages, discussions, security alerts, dev environments | Yes | No |
| Analytics, rulesets, merge automation, dependency and advisory APIs | Yes | No |
| Attestations and repository secrets | Yes | No |
| Wikis and protected tags | No | Yes |

Some provider APIs impose narrower constraints on individual options even when
the command family is available. Gitfleet preserves those differences instead
of emulating unsupported behavior. Use `gitfleet auth status` to inspect the
active provider and its declared capabilities.

## Features

- Repository lifecycle through `repo`: create, list, view, clone, delete,
  archive, unarchive, rename, edit, star, unstar, and fork repositories.
- Bounded multi-repository execution through `workspace`, including named
  workspace definition, listing, removal, and archive operations.
- Collaboration through `change`, `review`, `issue`, `discussion`, and `inbox`,
  with nested comment and reaction operations on changes.
- Repository metadata and documentation through `label`, `template`, and
  `wiki` commands.
- Delivery and infrastructure through `pipeline`, `release`, `deploy`,
  `environment`, `registry`, `runner`, `dev`, `site`, and `webhook` commands.
- Planning, governance, access, and automation configuration through `planning`,
  `govern`, `policy`, `access`, `identity`, `secret`, and `variable` commands.
- Discovery and security through `search`, `code`, `browse`, `license`, `deps`,
  `advisory`, `attestation`, `security`, and `analytics` commands.
- Provider escape hatches and local tooling through `api`, `snippet`, `auth`,
  `config`, `alias`, `completion`, `version`, JSON output, and terminal themes.

## Installation

Download a prebuilt archive for your operating system and architecture from
[GitHub Releases](https://github.com/airscripts/gitfleet/releases). Each
archive includes both executable names, `gitfleet` and `gf`, plus the project
license.

Use the matching archive for your platform:

| Platform | Architecture | Target |
| -------- | ------------ | ------ |
| Linux    | x64          | `x86_64-unknown-linux-gnu` |
| Linux    | ARM64        | `aarch64-unknown-linux-gnu` |
| macOS    | x64          | `x86_64-apple-darwin` |
| macOS    | ARM64        | `aarch64-apple-darwin` |
| Windows  | x64          | `x86_64-pc-windows-msvc` |
| Windows  | ARM64        | `aarch64-pc-windows-msvc` |

Verify downloads with the published `SHA256SUMS` file when installing from a
release archive.

Gitfleet is also installable from this checkout with Cargo:

```bash
cargo install --path gitfleet

gitfleet version
gf version
```

This builds optimized release binaries and installs `gitfleet` and `gf` into
Cargo's binary directory.

Cargo installs binaries into `~/.cargo/bin`; make sure that directory is on
your `PATH` if the commands are not found.

After pulling new source changes, reinstall the local package to refresh the
commands:

```bash
cargo install --path gitfleet --force
```

## Quick Start

Authenticate, inspect the active account, and run a read operation:

```bash
gitfleet auth login
gitfleet auth status
gitfleet repo list
gitfleet issue list --repo owner/repository
```

Use `gitfleet help` for the complete command surface or ask for nested help:

```bash
gitfleet help
gitfleet help change
gitfleet help pipeline list-runs
```

## Authentication

Authenticate with a provider token and inspect the resulting profile:

```bash
gitfleet auth login
gitfleet auth status
gitfleet auth token
```

GitHub Enterprise is supported through an explicit host:

```bash
gitfleet auth login \
  --host github.example.com \
  --profile work
```

For GitLab, select the provider explicitly. This is required for a
self-managed GitLab host whose name does not include `gitlab`:

```bash
gitfleet auth login \
  --provider gitlab \
  --host git.example.com \
  --profile work-gitlab
```

For headless systems and CI, set `GITFLEET_GITHUB_TOKEN` or
`GITFLEET_GITLAB_TOKEN`; this avoids requiring a desktop keyring. Destructive
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

### GitLab Token Scopes

For the complete Gitfleet command surface, use a personal access token with the
`api` scope and an account role that permits the requested operations. Narrower
tokens can be used when only a subset of commands is needed:

| Scope              | Required for                                                        |
| ------------------ | ------------------------------------------------------------------- |
| `api`              | Complete read-write Gitfleet API access, including live playbooks   |
| `read_api`         | Read-only project, group, package, registry, and metadata operations |
| `read_user`        | Authenticated user and user-directory reads only                     |
| `read_repository`  | Private repository file reads and Git-over-HTTPS pulls               |
| `write_repository` | Git-over-HTTPS pushes; this scope alone cannot authenticate API writes |
| `create_runner`    | Runner creation when using runner-registration endpoints             |
| `manage_runner`    | Runner management and removal                                        |

Do not grant `sudo` or `admin_mode` for normal Gitfleet use. Project and group
access tokens can reduce resource reach, but they cannot perform user-scoped
operations outside their project or group. See GitLab's official
[access token scope reference](https://docs.gitlab.com/security/tokens/access_token_scopes/)
and
[personal access token guidance](https://docs.gitlab.com/user/profile/personal_access_tokens/).

## Configuration

Profile metadata is stored at `~/.config/gitfleet/credentials.toml` with mode
`0600`; provider tokens are stored in the operating system credential store.
The keyring is the secure default. If no native credential store is available,
Gitfleet reports an error rather than writing a plaintext token. Users who
explicitly accept that risk can opt in to Git-style plaintext storage:

```bash
export GITFLEET_CREDENTIAL_STORE=file
gitfleet auth login
```

This stores tokens in `~/.config/gitfleet/credentials.toml`. The file is
permission-protected but is not encrypted; use this only on a trusted machine.
Repository-local profile selection is disabled by default; explicitly trust it
with `GITFLEET_TRUST_REPO_CONFIG=true`.

When set, `GITFLEET_PROFILE` selects the named profile; an unknown name is an
error. Otherwise, resolution checks a trusted repository-local profile when
`GITFLEET_TRUST_REPO_CONFIG=true`, then the active profile, then the first
configured profile in sorted order, and finally the default profile.
Within the selected profile, a stored profile token takes precedence over the
provider environment token. Environment tokens are used for the default
provider host when the profile has no stored token. Repository targets come
from `--repo` or the current Git remote.

Copy [`.env.example`](./.env.example) when preparing shell or CI variables.
Gitfleet does not automatically load dotenv files; export the values through
your shell, process manager, or CI secret store. Never commit a populated
`.env` file.

### Environment Variables

| Key                            | Example value                         | Purpose |
| ------------------------------ | ------------------------------------- | ------- |
| `GITFLEET_GITHUB_TOKEN`        | `github_pat_...`                      | Supplies a GitHub token for automation. |
| `GITFLEET_GITLAB_TOKEN`        | `glpat-...`                           | Supplies a GitLab token for automation. |
| `GITFLEET_PROFILE`             | `work`                                | Selects a named profile. |
| `GITFLEET_TRUST_REPO_CONFIG`   | `true`                                | Allows `.gitfleetrc` to select a profile. Any other value keeps repository configuration untrusted. |
| `GITFLEET_CREDENTIAL_STORE`    | `file`                                | Uses permission-protected plaintext credential storage. Unset or any other value uses the secure keyring default. |
| `GITFLEET_HOME`                | `/home/example`                        | Overrides the home directory used to locate `.config/gitfleet/`; otherwise the operating-system home is used. |
| `GITFLEET_CI`                  | `true`                                 | Enables non-interactive behavior when present. |
| `GITFLEET_LOG`                 | `gitfleet=debug`                       | Overrides the default tracing filter using `tracing-subscriber` directives. |
| `GITFLEET_NO_COLOR`            | `1`                                    | Disables colored output when present. |
| `GITFLEET_TERM`                | `xterm-256color`                       | Contributes to automatic terminal theme detection. |
| `GITFLEET_COLORTERM`           | `truecolor`                            | Enables true-color-aware automatic theme detection. |
| `GITFLEET_COLORFGBG`           | `15;0`                                 | Supplies terminal foreground/background hints for automatic theme selection. |
| `GITFLEET_PLAYBOOK_REPO`         | `owner/gitfleet-test-repository`     | Selects the disposable repository required by live playbooks. |
| `GITFLEET_PLAYBOOK_ORG`          | `owner`                              | Selects the live-playbook organization or group; defaults to the owner from `GITFLEET_PLAYBOOK_REPO`. |
| `GITFLEET_PLAYBOOK_TEST_REPO_OWNER` | `owner`                           | Selects the account that owns disposable repositories; defaults to `GITFLEET_PLAYBOOK_ORG`. |
| `GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE` | `org`                         | Sets disposable repository ownership to `org` or `user`; defaults to `org`. |
| `GITFLEET_PLAYBOOK_TMPDIR`       | `/tmp/gitfleet-playbooks`            | Selects the live-playbook scratch directory. |
| `GITFLEET_PLAYBOOK_RUN_ID`       | `local-001`                          | Overrides the unique suffix used for live-playbook resources. |
| `GITFLEET_PLAYBOOK_SKIP`         | `pipeline,milestone`                 | Comma- or space-separated playbooks omitted by `all.sh`. |
| `GITFLEET_PLAYBOOK_PARALLEL`     | `1`                                  | Runs live playbooks concurrently; `0` is the safer sequential default. |
| `GITFLEET_PLAYBOOK_WEBHOOK_URL`  | `https://hooks.example.com/gitfleet` | Enables delivery testing through an owner-controlled webhook receiver. |

`GITFLEET_TEST_CREDENTIAL_STORE` is reserved for automated tests and is not a
supported user configuration variable. Variables such as
`GITFLEET_PLAYBOOK_RESOURCE_SUFFIX`, `GITFLEET_PLAYBOOK_PROVIDER`, and
`GITFLEET_PLAYBOOK_CAPABILITIES` are derived internally and should not be set
directly.

## Profiles

Named profiles support separate provider accounts and hosts:

```bash
gitfleet auth login --profile personal
gitfleet auth login --profile work
gitfleet auth list
gitfleet auth switch work
gitfleet auth detect
gitfleet auth logout --profile personal
```

The active profile supplies the provider, host, and token for subsequent
operations. `auth detect` selects a profile from the current repository.
`auth logout --profile NAME` removes only that profile; omitting `--profile`
removes all stored credentials after confirmation.

## Commands

Use `gitfleet help` for the complete surface or target nested help directly:

```bash
gitfleet help
gitfleet help change
gitfleet help pipeline list-runs
```

| Family                                         | Purpose                                                    |
| ---------------------------------------------- | ---------------------------------------------------------- |
| `auth`                                         | Provider accounts and profiles                             |
| `repo`                                         | Repository lifecycle, forks, and metadata                  |
| `change`                                       | Change creation, listing, inspection, and merging          |
| `review`                                       | Comments and reactions on changes                           |
| `issue`, `discussion`, `inbox`                 | Collaboration and notification workflows                   |
| `pipeline`                                     | Workflow definitions, runs, triggers, cancellations, and reruns |
| `release`, `deploy`, `environment`             | Delivery lifecycle                                         |
| `workspace`                                    | Named fleets and bounded multi-repository execution        |
| `govern`, `policy`                             | Ruleset governance and repository protection               |
| `planning`                                     | Milestones and projects                                    |
| `wiki`, `site`                                 | Repository documentation and publishing                    |
| `search`, `code`, `browse`, `api`              | Discovery, navigation, and provider escape hatches         |
| `label`, `template`, `license`                 | Repository metadata                                        |
| `deps`, `advisory`, `attestation`, `security`  | Dependency, advisory, attestation, and alert operations     |
| `registry`, `dev`, `runner`, `webhook`         | Build and delivery infrastructure                          |
| `secret`, `variable`                           | Automation configuration                                   |
| `access`, `identity`                           | Organization access and account keys                       |
| `analytics`, `snippet`                         | Traffic reporting and hosted snippets                      |
| `alias`, `completion`, `config`, `help`        | Gitfleet utilities                                         |
| `version`                                      | Version information                                        |

### Selected Nested Commands

- `pipeline list-def`, `pipeline view-def`, `pipeline list-runs`, and
  `pipeline view-run` inspect workflow definitions and runs; `trigger`,
  `cancel`, and `rerun` apply run operations.
- `security advisories`, `security secret-scans`, and `security codeql` expose
  provider security alerts.
- `analytics views` and `analytics clones` provide repository traffic reports.
- `access org` and `access team` manage organization and team access.
- `identity ssh-key` and `identity gpg-key` manage provider account keys.
- `planning milestone`, `planning project`, `policy branch-protection`,
  `policy tag-protection`, and `repo fork` expose their respective operations.
- `alias set NAME "COMMAND ..."` creates an executable command alias. Alias
  arguments are forwarded without invoking a shell; canonical command names
  cannot be shadowed.

## Common Workflows

### Change Review

```bash
gitfleet change create "Add feature" --head feature --base main
gitfleet change list --state open
gitfleet change merge 42 --method squash --yes
gitfleet review comment list 42
gitfleet review comment create 42 "Please add a regression test."
gitfleet review comment create 17 "I can reproduce this." --target issue
gitfleet review reaction create 42 eyes
```

### Pipeline Delivery

```bash
gitfleet pipeline list-def
gitfleet pipeline list-runs
gitfleet pipeline view-run <run-id>
gitfleet pipeline trigger <workflow-id> --ref main
gitfleet release list
gitfleet deploy list
```

### Security Governance

```bash
gitfleet security advisories --repo owner/repository
gitfleet security secret-scans --repo owner/repository
gitfleet security codeql --repo owner/repository
gitfleet policy branch-protection get main --repo owner/repository
gitfleet govern list-rulesets --repo owner/repository
```

### Workspaces

```bash
gitfleet workspace define \
  --name platform \
  --repos owner/api \
  --repos owner/web
gitfleet workspace list
gitfleet workspace archive platform --dry-run
```

Workspace targets may be provider-qualified:

```text
github@github.com:owner/repository
github@github.example.com:platform/repository
gitlab@gitlab.com:group/project
```

Workspace mutations run with bounded concurrency and preserve workspace order
in their result report. Repositories that do not match the active provider and
host profile are reported as skipped.

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

## Development

This section is for contributors and maintainers.

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

## Testing

Run the required workspace gates from the repository root:

```bash
cargo fmt --check
CARGO_BUILD_JOBS=4 cargo clippy -- -D warnings
CARGO_BUILD_JOBS=4 cargo check --workspace
CARGO_BUILD_JOBS=4 cargo test --workspace
CARGO_BUILD_JOBS=4 cargo llvm-cov --fail-under-lines 80 --workspace
CARGO_BUILD_JOBS=4 cargo build --release
```

Coverage must remain at or above 80 percent. See [AGENTS.md](./AGENTS.md) for
architecture, testing, style, playbook, and release requirements.

### Automated Tests

Unit tests live inside source files in `#[cfg(test)] mod tests {}` blocks.
Integration tests live in each crate's `tests/` directory. Provider tests use
`wiremock` for HTTP mocking and `insta` for normalization snapshots. Automated
tests must never make real HTTP requests.

### Live API Playbooks

Playbooks under `gitfleet-playbooks/` are Bash scripts for validating command
families against a real GitHub or GitLab test account and repository. They are
developer and release-validation tools, not part of the normal end-user
workflow. They use the active Gitfleet profile, require its matching explicit
credentials, and clean up mutations during teardown.

Run them only against a dedicated test repository:

```bash
GITFLEET_PLAYBOOK_REPO=owner/test-repository GITFLEET_PLAYBOOK_ORG=example bash gitfleet-playbooks/all.sh
GITFLEET_PROFILE=gitlab-test GITFLEET_PLAYBOOK_REPO=group/test-repository bash gitfleet-playbooks/all.sh
GITFLEET_PLAYBOOK_REPO=owner/test-repository GITFLEET_PLAYBOOK_SKIP="pipeline,milestone,project" bash gitfleet-playbooks/all.sh
GITFLEET_PLAYBOOK_REPO=owner/test-repository GITFLEET_PLAYBOOK_PARALLEL=1 bash gitfleet-playbooks/all.sh
```

See `gitfleet-playbooks/env.sh` for configuration. Automated tests use mocks
and make no network requests; live playbooks intentionally exercise the
provider API.

## Architecture

| Crate                | Responsibility                                                      |
| -------------------- | ------------------------------------------------------------------- |
| `gitfleet-core`      | Provider-neutral entities, capabilities, operations, infrastructure |
| `gitfleet-providers` | GitHub and GitLab provider clients, wire types, normalization       |
| `gitfleet`           | Product CLI crate and thin surface over shared operations           |

Only `gitfleet-providers` performs HTTP requests. Provider wire types are
normalized before crossing the crate boundary, and unsupported operations
return a capability error instead of pretending another provider supports
them.

Read [AGENTS.md](./AGENTS.md) for the full architecture contract, crate
boundaries, code style, testing strategy, and release requirements.

## Repository Structure

```text
gitfleet-core/        provider-neutral entities, capabilities, errors, operations,
                      infrastructure (config, git, output, prompts, secrets, workspace)
gitfleet-providers/   GitHub and GitLab provider clients, wire types, normalization
gitfleet/             product CLI crate and thin surface over shared operations
gitfleet-playbooks/   live API test scripts (Bash)
```

`PLAN.md` and `ROADMAP.md` are reserved for implementation planning and
deferred milestones.

## Security

Report vulnerabilities using the private process in [SECURITY.md](./SECURITY.md)
and do not open a public issue. Never include tokens or other credentials in
reports; use redacted `--debug` output for ordinary bug reports.

## Support

For usage help, run `gitfleet help [command...]`. For reproducible bugs, open an
issue with the Gitfleet version, operating system, command, expected behavior,
actual behavior, and redacted `--debug` output.

## Contributing

Read [CONTRIBUTING.md](./CONTRIBUTING.md) before proposing a change. The
development, testing, architecture, and repository sections above describe the
contributor workflow.

## Sponsorship

Gitfleet is maintained by Airscripts. If it helps your team manage repositories
more effectively, sponsor ongoing maintenance and provider coverage through
[GitHub Sponsors](https://github.com/sponsors/airscripts).

## License

MIT. See [LICENSE](./LICENSE).
