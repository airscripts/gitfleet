# label

## Purpose

`label` manages repository labels and synchronizes them from declarative templates.

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
- `gitfleet label rename enhancement feature --repo owner/repository --yes`
- `gitfleet label template list`
- `gitfleet label template show gitfleet`
- `gitfleet label sync --template gitfleet --repo owner/repository --yes`
- `gitfleet label sync --file labels.toml --repo owner/repository --dry-run --json`

## Provider Support

GitHub and GitLab both support label operations.

## Safety Notes

Deleting labels mutates repository metadata and requires confirmation or
`--yes`. Renaming updates the existing label object, preserving its issue and
change references. If a template declares `rename_from`, synchronization uses
that same in-place update. A rename is rejected when both the old and new names
already exist because merging assignments requires separate issue and change
operations.

`--replace` deletes labels that are not in the template. Without `--replace`,
those labels are preserved.

Custom templates use versioned TOML:

```toml
version = 1
name = "team"

[[labels]]
name = "feature"
rename_from = ["enhancement"]
color = "1d7a1d"
description = "New feature or request."
```

The built-in `github` and `gitfleet` templates are maintained as TOML assets in
`gitfleet-core/templates/` and are compiled into the CLI.

## JSON/Automation Notes

Use `--json` for synchronization and audit reports. Real synchronization always
requires confirmation; use `--dry-run` to inspect the complete plan without
mutating provider state. Reports include created, updated, renamed, deleted,
preserved, skipped, and failed counts.

## Related Commands

See [issue](./issue.md), [change](./change.md), and [template](./template.md).
