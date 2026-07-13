# Changelog

All notable Gitfleet changes are documented here using Keep a Changelog and
Semantic Versioning.

## [Unreleased]

- Default to operating-system credential storage for provider tokens, with an
  explicit `GITFLEET_CREDENTIAL_STORE=file` compatibility option for plaintext
  storage.
- Harden credential, configuration, provider-client, and workspace handling.

## [0.1.0] - 2026-07-01

### Added

- Domain model, capability contracts, and provider registry built around
  provider-neutral abstractions, with GitHub and GitLab as built-in
  providers.
- Shared command families that the CLI uses through a single
  operation catalog.
- Workspace execution that runs in-process with bounded concurrency and
  produces stable, per-repository results.
- The `gitfleet` and `gf` executable names, which expose the same command
  surface.
- Manage pull requests, reviews, issues, discussions, and notifications with
  the `change`, `review`, `issue`, `discussion`, and `inbox` commands.
- Set up repositories, governance, policies, project planning, wikis, sites,
  labels, templates, and licenses with the `repo`, `govern`, `policy`,
  `planning`, `wiki`, `site`, `label`, `template`, and `license` commands.
- Handle CI/CD pipelines, releases, package registries, dev environments,
  deployments, environments, runners, and webhooks with the `pipeline`,
  `release`, `registry`, `dev`, `deploy`, `environment`, `runner`, and
  `webhook` commands.
- Audit dependencies, advisories, attestations, security policies, secrets,
  and variables with the `deps`, `advisory`, `attestation`, `security`,
  `secret`, and `variable` commands.
- Search code, manage access and identity, view analytics, create snippets,
  browse resources, and call the API directly with the `search`, `code`,
  `access`, `identity`, `analytics`, `snippet`, `browse`, and `api` commands.
- Authenticate, manage workspaces, set up aliases, generate shell completions,
  configure Gitfleet, and show help or version information
  with the `auth`, `workspace`, `alias`, `completion`, `config`,
  `help`, and `version` commands.
- GitLab provider capabilities including reviews, milestones, snippets,
  protected branches and tags, Pages, and package registry operations.
- Insta snapshot tests for CLI help text and provider wire payload
  normalization.
- Lefthook pre-commit checks for formatting, clippy, workspace compilation, and
  the coverage gate.

### Changed

- Replaced the old GitHub-only identity with Gitfleet's own product name,
  configuration paths, environment variables, and release line.
- Moved all GitHub HTTP and REST integrations behind the GitHub provider so
  no provider details leak into shared code.
- Renamed provider-specific command names to portable Gitfleet terms,
  including `change` for pull requests, `pipeline` for CI, `planning` for
  projects, `site` for pages, `snippet` for gists, and `dev` for codespaces.

### Removed

- Removed Copilot, agent tasks, agent skills, prompt preview, and
  GitHub-compatible extension commands.
- Removed automatic `gh` proxying and parity-only compatibility behavior.
- Removed legacy executable aliases and configuration migration from the
  previous CLI.
- Removed the `gitfleet-tui` crate and the `gitfleet tui` command. Gitfleet
  is now a CLI-only product.
