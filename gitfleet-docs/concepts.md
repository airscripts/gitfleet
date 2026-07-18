# Concepts

Gitfleet treats repository work as a fleet-management problem. A team may have
repositories on GitHub, GitLab, hosted enterprise instances, or multiple
accounts. Gitfleet provides one command vocabulary for common repository
operations while preserving provider-specific limits.

## Provider-neutral Families

Command labels describe the work, not the provider API. A GitHub pull request
and a GitLab merge request are both handled through `change`. GitHub Actions
and GitLab pipelines are both handled through `pipeline`. Milestones and
projects are grouped under `planning`.

This vocabulary is the main product contract. Users should not need to remember
which provider calls something a pull request, merge request, workflow, job,
project, group, organization, Pages site, or wiki page before they can start.
Gitfleet exposes the common job first and lets provider support decide whether
the selected account can run it.

## Capabilities

Each provider declares the capabilities it supports. Gitfleet checks those
capabilities before running provider-backed operations. Unsupported behavior
returns an explicit capability error instead of being emulated through another
feature.

Examples:

- GitLab supports `wiki`; GitHub reports wiki operations as unsupported.
- GitHub supports `dev` for Codespaces; GitLab reports that capability as
  unsupported.
- GitLab supports protected tags through `policy tag-protection`; GitHub users
  should use `govern` rulesets where appropriate.

This matters for automation. A script can select a profile, run a command, and
receive a clear unsupported-capability failure instead of getting a hidden
fallback with provider-specific side effects.

## Profiles

A profile stores the selected provider, host, and credential reference for an
account. Profiles let you switch between personal, work, enterprise, GitHub,
and GitLab contexts without changing command names.

Profiles are also how Gitfleet avoids guessing. If a repository remote points to
an enterprise host, the matching profile should point at that same host. If a
script needs a specific account, set `GITFLEET_PROFILE` instead of relying on
the active local profile.

## Repository Targets

Most repository commands accept `--repo owner/repository`. When omitted, Gitfleet
can infer the repository from the current Git remote for commands that require a
single repository target.

Prefer passing `--repo` in scripts. Remote inference is convenient for
interactive use, but explicit repository targets make automation easier to read
and safer to review.

Workspace targets may be provider-qualified:

```text
github@github.com:owner/repository
github@github.example.com:platform/repository
gitlab@gitlab.com:group/project
```

## Output Model

Human-readable output is the default. Use `--json` when another program needs
to consume results. Status and debug logs use stderr; structured command output
uses stdout.

Treat human output as a display format and JSON as the automation interface.
When a command mutates state, combine `--json` with `--yes` in non-interactive
contexts only after reviewing the command's safety notes.
