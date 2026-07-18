# repo

## Purpose

`repo` manages repository lifecycle, forks, stars, cloning, and metadata.

## Why This Exists

Repository lifecycle is the base workflow for every provider. Gitfleet keeps
creation, discovery, local cloning, and retirement under one family.

## When To Use It

Use `repo` when creating a repository, listing repositories, inspecting details,
cloning one repository, cloning an owner set, editing metadata, starring,
forking, archiving, or deleting.

## Before You Run

Decide whether the repository belongs to a user or organization with
`--owner-type` when creating or listing repositories. Visibility is selected
with flags such as `--public`, `--private`, and `--internal`; only one
visibility flag can be used.

For single-clone commands, pass `owner/repository`. For bulk clone, use
`repo clone --all` with exactly one of `--org` or `--user`. Bulk clone defaults
to active non-fork repositories, skips local directories that already exist, and
continues after individual clone failures so the final report can show the full
owner set.

## Common Commands

- `gitfleet repo create service-api --private --owner platform --owner-type org --initialize`
- `gitfleet repo list --owner platform`
- `gitfleet repo view platform/service-api`
- `gitfleet repo clone platform/service-api`
- `gitfleet repo clone platform/service-api --ssh --directory service-api`
- `gitfleet repo clone --all --org platform --directory repos --dry-run`
- `gitfleet repo clone --all --org platform --directory repos --concurrency 4`
- `gitfleet repo clone --all --user alice --include-forks --include-archived --ssh`
- `gitfleet repo edit platform/service-api --description "Service API"`
- `gitfleet repo archive platform/service-api --yes`
- `gitfleet repo unarchive platform/service-api`
- `gitfleet repo rename platform/service-api platform/service-core`
- `gitfleet repo star owner/repository`
- `gitfleet repo unstar owner/repository`
- `gitfleet repo fork list owner/repository`
- `gitfleet repo fork create owner/repository`
- `gitfleet repo delete platform/service-core --yes`

## Provider Support

GitHub and GitLab both support repository operations. Fork destination and owner
semantics can differ by provider.

## Safety Notes

Delete, archive, rename, and metadata edits mutate repository state. Delete and
archive require confirmation or `--yes` where applicable.

Bulk clone writes local directories. It is not destructive to provider state,
but it can create many local folders and consume bandwidth. Use `--dry-run`
before cloning a large owner. Existing local directories are skipped rather than
overwritten or pulled.

## JSON/Automation Notes

Use `--json` for repository inventory, migration scripts, and bulk clone
reports. Bulk clone JSON includes the owner scope, destination root, protocol,
summary counts, and per-repository statuses.

## Related Commands

See [workspace](./workspace.md), [browse](./browse.md), [code](./code.md), and
[license](./license.md).
