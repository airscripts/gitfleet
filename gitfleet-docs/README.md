# Gitfleet Docs

This folder is the product and usage guide for Gitfleet. It documents the CLI as
users experience it: provider-neutral concepts first, provider differences where
they matter, and command families by their public names.

Read these docs when you want to understand which command to use, what provider
state it reads or changes, and how to run it safely in an interactive shell or
automation. The installed CLI help remains the exact syntax reference for the
current binary; these pages add product context, workflows, safety notes, and
provider expectations.

## Start Here

- [Concepts](./concepts.md) explains the product model and vocabulary.
- [Authentication](./authentication.md) covers profiles, tokens, and hosts.
- [Configuration](./configuration.md) lists supported configuration and
  environment variables.
- [Providers](./providers.md) describes GitHub and GitLab support.
- [Output](./output.md) explains human output, JSON output, debug logs, and
  terminal themes.
- [Safety](./safety.md) documents confirmations, `--yes`, and `--dry-run`.
- [Troubleshooting](./troubleshooting.md) gives common fixes.
- [Documentation Guide](./documentation-guide.md) defines the user-first writing
  checklist for expanding these docs.

## Command Reference

Use [commands/](./commands/README.md) for every public command family. Each page
answers what the command does, why it exists, when to use it, common examples,
provider notes, safety behavior, and related commands.

Command pages are organized by product vocabulary, not Rust module names or
provider API names. For example, use `change` for pull requests and merge
requests, `planning` for milestones and projects, and `registry` for packages
and container images.

## Workflows

Use [workflows/](./workflows/README.md) for task-oriented guides:

- [First Run](./workflows/first-run.md)
- [Multi-provider Profiles](./workflows/multi-provider-profiles.md)
- [Repository Lifecycle](./workflows/repository-lifecycle.md)
- [Change Review](./workflows/change-review.md)
- [Pipeline Delivery](./workflows/pipeline-delivery.md)
- [Security Governance](./workflows/security-governance.md)
- [Workspace Fleets](./workflows/workspace-fleets.md)
- [Automation and JSON](./workflows/automation-json.md)

Workflow guides are the best place to start when the job spans multiple command
families. They show a safe order of operations: inspect first, preview when
possible, then mutate with confirmation.

## Maintainer Overview

[Architecture Overview](./architecture-overview.md) summarizes the crate
boundaries and provider contract for readers who want implementation context.
Detailed contributor rules remain in [../AGENTS.md](../AGENTS.md).
