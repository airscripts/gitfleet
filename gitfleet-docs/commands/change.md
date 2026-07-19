# change

## Purpose

`change` manages proposed changes: pull requests on GitHub and merge requests
on GitLab.

## Why This Exists

Gitfleet uses one provider-neutral name for reviewable code changes.

## When To Use It

Use `change` to create, list, inspect, and merge proposed code changes.

## Before You Run

Make sure Gitfleet can resolve the repository from `--repo` or the current Git
remote. Creating a change needs a head branch and usually a base branch; if the
base is omitted Gitfleet defaults to `main`. Merge methods are limited to
`merge`, `squash`, and `rebase`.

## Common Commands

- `gitfleet change create "Add feature" --head feature --base main --repo owner/repository`
- `gitfleet change list --repo owner/repository --state open`
- `gitfleet change list --repo owner/repository --state open --limit 25 --page 2`
- `gitfleet change view 42 --repo owner/repository`
- `gitfleet change merge 42 --repo owner/repository --method squash --yes`

## Provider Support

GitHub and GitLab both support change operations.

## Safety Notes

Merging changes is destructive to branch state and requires confirmation in
human mode or `--yes` in automation.

## JSON/Automation Notes

Use `--json` for review dashboards and release automation.
For list output, `--limit` is the page size and `--page` selects a 1-based
provider page.

## Related Commands

See [review](./review.md), [issue](./issue.md), [pipeline](./pipeline.md), and
[deps](./deps.md).
