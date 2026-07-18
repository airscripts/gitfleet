# repo

## Purpose

`repo` manages repository lifecycle, forks, stars, cloning, and metadata.

## Why This Exists

Repository lifecycle is the base workflow for every provider. Gitfleet keeps
creation, discovery, local cloning, and retirement under one family.

## When To Use It

Use `repo` when creating a repository, listing repositories, inspecting details,
cloning, editing metadata, starring, forking, archiving, or deleting.

## Before You Run

Decide whether the repository belongs to a user or organization with
`--owner-type`. Visibility is selected with flags such as `--public`,
`--private`, and `--internal`; only one visibility flag can be used. For
commands that accept a repository argument, use `owner/repository` or run from a
clone whose remote Gitfleet can parse.

## Common Commands

- `gitfleet repo create service-api --private --owner platform --owner-type org --initialize`
- `gitfleet repo list --owner platform`
- `gitfleet repo view platform/service-api`
- `gitfleet repo clone platform/service-api`
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

## JSON/Automation Notes

Use `--json` for repository inventory and migration scripts.

## Related Commands

See [workspace](./workspace.md), [browse](./browse.md), [code](./code.md), and
[license](./license.md).
