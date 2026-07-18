# Documentation Guide

This guide defines the writing standard for Gitfleet user docs. Use it when
adding a command, changing command behavior, or expanding an existing page.

## User-first Rule

Start from the user task, not the implementation. Explain what the user is
trying to accomplish, why Gitfleet has a command for it, and what the command
changes or reads.

Prefer product terms:

- Use `change` for pull requests and merge requests.
- Use `pipeline` for provider CI/CD runs and definitions.
- Use `planning` for milestones and projects.
- Use `registry` for package and container registry operations.
- Use `review` for comments and reactions.

Only mention Rust modules, provider client internals, or crate boundaries in
maintainer-facing docs.

## Command Page Checklist

Every command-family page should answer these questions:

- What does this command family do?
- Why does this command exist in Gitfleet instead of only in provider CLIs?
- When should a user reach for it?
- What are the common read-only commands?
- What are the common mutating commands?
- Which commands need `--repo`, owner, ID, branch, ref, or file input?
- Which provider capabilities are required?
- Does GitHub behave differently from GitLab?
- Which operations are destructive or state-changing?
- When should the user use `--dry-run`?
- When is `--yes` required?
- What should scripts parse with `--json`?
- Which related command family should the user read next?

Do not leave a page with only command syntax. Syntax tells users how to type the
command; the docs should also explain whether the command is the right tool.

## Recommended Page Shape

Use this structure for command-family pages unless a page has a good reason to
deviate:

```markdown
# command-name

## Purpose

One or two direct paragraphs about what the family manages.

## Why This Exists

Explain the provider-neutral value and the product vocabulary.

## When To Use It

Describe concrete user situations.

## Before You Run

List prerequisites: profile, repo target, permissions, token scopes, branch/ref,
file input, or provider feature requirements.

## Common Commands

Show realistic examples. Include read commands first, then mutating commands.

## Provider Support

Call out GitHub/GitLab support and unsupported capabilities.

## Safety Notes

Document confirmation, `--yes`, `--dry-run`, and sensitive output.

## JSON/Automation Notes

Describe what automation should parse and any non-interactive requirements.

## Related Commands

Link to nearby command families and workflows.
```

The current first-pass command pages may omit `Before You Run` when the
prerequisites are obvious. Add it when expanding a page or when the command has
important inputs, provider permissions, or safety concerns.

## Example Quality

Examples should be runnable in shape, even when placeholders are used:

- Use `owner/repository`, `group/project`, `<run-id>`, `<workflow-id>`,
  `<ruleset-id>`, and similar placeholders consistently.
- Use real Gitfleet option names from `gitfleet help`.
- Include `--repo` when a command normally needs repository context.
- Include `--yes` on destructive examples.
- Include `--dry-run` before broad fleet mutations when supported.
- Avoid provider-specific endpoint paths unless documenting `api`.

Prefer a small set of high-value examples over an exhaustive option dump. The
installed CLI help remains the source of truth for every flag.

## Provider Notes

Document provider support from code, not assumptions. Check:

- `gitfleet-core/src/operations.rs` for public operation families.
- `gitfleet/src/main.rs` for public Clap command families.
- `gitfleet-providers/src/github/provider.rs` for GitHub capabilities.
- `gitfleet-providers/src/gitlab/provider.rs` for GitLab capabilities.

If a provider does not declare a capability, say it is unsupported. Do not
suggest unofficial workarounds as Gitfleet behavior.

## Safety Language

Use direct safety wording:

- "Read-only" for commands that only inspect state.
- "Mutates provider state" for create, update, set, trigger, test, mark-read,
  archive, unarchive, rename, merge, and similar actions.
- "Destructive" for delete, remove, logout, archive, cancellation, and policy
  removal.
- "Requires confirmation or `--yes`" when the command prompts in human mode or
  must run non-interactively.

Mention sensitive data explicitly for `secret`, `variable`, `security`, `auth
token`, and raw `api` commands.

## Workflow Guides

Workflow guides should combine command families into a realistic job. They
should tell the user:

- What outcome the workflow produces.
- Which profile or repository context is expected.
- Which commands are safe to run first.
- Where state changes happen.
- What to use in CI or scripts.
- Which reference pages give the detailed command behavior.

## README Role

Keep the root `README.md` concise. It should help a new reader understand what
Gitfleet is, install it, run the first commands, and find the full docs. Move
long command tables, provider details, configuration details, and workflows into
`gitfleet-docs/`.
