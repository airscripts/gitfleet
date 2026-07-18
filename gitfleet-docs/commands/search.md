# search

## Purpose

`search` searches provider resources.

## Why This Exists

Search spans issues, repositories, and code. Gitfleet keeps the entry point
provider-neutral.

## When To Use It

Use `search` when finding work items, repositories, or code across provider
indexes.

## Before You Run

Search syntax is provider-backed. Qualifiers that work on one provider may not
mean the same thing on another. Start with broad read-only searches, then narrow
queries once you confirm the active provider and account.

## Common Commands

- `gitfleet search issues "is:open label:bug"`
- `gitfleet search repos "topic:rust"`
- `gitfleet search code "Renderer repo:owner/repository"`

## Provider Support

GitHub and GitLab both expose search support, with provider-specific query
syntax and indexing limits.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for discovery scripts and reports.

## Related Commands

See [code](./code.md), [issue](./issue.md), and [repo](./repo.md).
