# label

## Purpose

`label` manages repository labels.

## Why This Exists

Labels organize issues and changes. Gitfleet gives teams one way to inspect and
maintain them.

## When To Use It

Use `label` when setting up a repository, standardizing taxonomy, or cleaning up
metadata.

## Before You Run

Decide whether the label name already exists and whether the color and
description match your team's taxonomy. Label deletion can affect issue and
change filters, saved searches, automation, and dashboards.

## Common Commands

- `gitfleet label list --repo owner/repository`
- `gitfleet label create bug --repo owner/repository --color d73a4a`
- `gitfleet label delete old-label --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both support label operations.

## Safety Notes

Deleting labels mutates repository metadata and requires confirmation or
`--yes`.

## JSON/Automation Notes

Use `--json` for label synchronization and audits.

## Related Commands

See [issue](./issue.md), [change](./change.md), and [template](./template.md).
