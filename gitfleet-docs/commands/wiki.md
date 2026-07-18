# wiki

## Purpose

`wiki` manages repository wiki pages.

## Why This Exists

Some providers expose repository wiki content as an API-backed documentation
surface.

## When To Use It

Use `wiki` to list, view, create, edit, or delete wiki pages.

## Before You Run

Use a GitLab profile and repository path. Choose stable page slugs and keep
local source-of-truth expectations clear; wiki edits change provider-hosted
documentation directly.

## Common Commands

- `gitfleet wiki list --repo group/project`
- `gitfleet wiki view home --repo group/project`
- `gitfleet wiki create usage --repo group/project --content "Usage notes"`
- `gitfleet wiki edit usage --repo group/project --content "Updated usage notes"`
- `gitfleet wiki delete usage --repo group/project --yes`

## Provider Support

GitLab supports wiki operations. GitHub reports wiki operations as unsupported
instead of emulating them through Git.

## Safety Notes

Creating, editing, and deleting pages mutates documentation state. Delete
requires confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for wiki inventory and migration.

## Related Commands

See [site](./site.md), [repo](./repo.md), and [code](./code.md).
