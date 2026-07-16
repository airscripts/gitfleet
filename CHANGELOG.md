# Changelog

All notable Gitfleet changes are documented here using Keep a Changelog and
Semantic Versioning.

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
  protected branches and tags, environments, and package registry operations.
- Insta snapshot tests for CLI help text and provider wire payload
  normalization.
- Lefthook pre-commit checks for formatting, clippy, workspace compilation, and
  the coverage gate.
- Reversible GitHub and GitLab live API playbooks covering every retained
  command family, including positive, negative, and cleanup paths.
- Raw API support for GET, POST, PUT, PATCH, and DELETE requests.
- A documented `.env.example` covering supported Gitfleet and live-playbook
  environment variables.
- Transparent provider pagination, retry handling, and rate-limit reporting
  for read operations.
- Provider capability introspection and contract validation for reliable
  cross-provider command discovery.
- Workspace archive operations and idempotent repository state changes.
- Executable command aliases with argument forwarding, quoting, cycle
  detection, and canonical-command protection.
- Change-request merging with merge, squash, and rebase methods.

### Changed

- Replaced the old GitHub-only identity with Gitfleet's own product name,
  configuration paths, environment variables, and release line.
- Moved all provider HTTP and REST integrations behind the provider clients so
  no provider details leak into shared code.
- Renamed provider-specific command names to portable Gitfleet terms,
  including `change` for pull requests, `pipeline` for CI, `planning` for
  projects, `site` for pages, `snippet` for gists, and `dev` for codespaces.
- Defaulted provider tokens to operating-system credential storage, with an
  explicit `GITFLEET_CREDENTIAL_STORE=file` compatibility option for
  permission-protected plaintext storage.
- Standardized every Gitfleet environment variable on the `GITFLEET_` prefix.
- Routed provider clients through the resolved profile host and credentials,
  including GitHub public API routing and GitLab filter handling.
- Made unsupported provider behavior explicit through capability errors,
  including GitHub wikis and protected tags; protected-tag operations remain
  available on GitLab.
- Hardened credential, configuration, provider-client, repository, prompt,
  output, and workspace behavior for interactive and automated use.
- Added MSRV, macOS, Windows, dependency-policy, and vulnerability validation
  to continuous integration.

### Fixed

- Corrected GitHub and GitLab repository creation, initialization, forking,
  editing, archival, and deletion behavior.
- Corrected issue and change comment routing, provider project identifiers,
  raw API mutation methods, and structured mutation output.
- Corrected GitHub package enumeration, project operations, repository
  rulesets, Pages lifecycle handling, and capability reporting.
- Corrected GitLab code browsing and search, including defaulting file reads to
  `HEAD`, plus label, variable, pipeline, release, environment, package, and
  repository policy operations.
- Corrected confirmation and non-interactive safeguards for destructive
  commands, including JSON and dry-run workflows.
- Corrected profile resolution, repository detection, credential handling,
  workspace partial-failure reporting, and provider-specific remote parsing.
- Corrected provider URL and path encoding, enterprise wiki safeguards,
  response cleanup, credential persistence, workspace routing, and provider
  contract handling.
- Expanded provider integration coverage and normalization checks for both
  providers; the workspace now exceeds the required 80 percent line-coverage
  gate.

### Removed

- Removed Copilot, agent tasks, agent skills, prompt preview, and
  GitHub-compatible extension commands.
- Removed automatic `gh` proxying and parity-only compatibility behavior.
- Removed legacy executable aliases and configuration migration from the
  previous CLI.
- Removed the `gitfleet-tui` crate and the `gitfleet tui` command. Gitfleet
  is now a CLI-only product.
- Removed unreachable GitHub wiki endpoint code because GitHub does not expose
  a supported wiki API.
