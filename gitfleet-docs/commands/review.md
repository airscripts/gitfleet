# review

## Purpose

`review` manages comments and reactions on changes and issues.

## Why This Exists

Comments and reactions are review primitives shared by collaboration workflows,
even though provider APIs name them differently.

## When To Use It

Use `review` to list, create, or delete review comments and reactions.

## Before You Run

Know whether the numeric target is a change or an issue. Comment commands
default to change targets and accept `--target issue` for issue comments.
Reaction commands operate on issue-style reaction targets in the active
provider.

## Common Commands

- `gitfleet review comment list 42 --repo owner/repository`
- `gitfleet review comment create 42 "Please add a regression test." --repo owner/repository`
- `gitfleet review comment create 17 "I can reproduce this." --repo owner/repository --target issue`
- `gitfleet review reaction list 42 --repo owner/repository`
- `gitfleet review reaction create 42 eyes --repo owner/repository`
- `gitfleet review reaction delete <reaction-id> 42 --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both support review operations.

## Safety Notes

Creating comments or reactions mutates collaboration state. Deleting reactions
requires confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for bots, review reports, and migration scripts.

## Related Commands

See [change](./change.md), [issue](./issue.md), and [inbox](./inbox.md).
