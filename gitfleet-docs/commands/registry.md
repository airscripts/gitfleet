# registry

## Purpose

`registry` inspects package registry content.

## Why This Exists

Packages and container images are part of repository delivery and need a common
inventory surface.

## When To Use It

Use `registry` to list packages or view package details for an owner.

## Before You Run

Know whether the provider expects an account owner, organization, group, or
project path. Pass `--package-type` when you need a specific registry kind such
as container packages.

## Common Commands

- `gitfleet registry list --owner owner --package-type container`
- `gitfleet registry list --owner owner --package-type container --limit 25 --page 2`
- `gitfleet registry view --owner owner --package-type container --package-name app`

## Provider Support

GitHub and GitLab both expose package registry capability.

## Safety Notes

Current documented commands are read-only.

## JSON/Automation Notes

Use `--json` for package inventories and release audits.
For package inventories, `--limit` is the page size and `--page` selects a
1-based provider page. Prefer explicit paging in automation so large registries
can be processed in deterministic chunks.

## Related Commands

See [pipeline](./pipeline.md), [release](./release.md), and [deploy](./deploy.md).
