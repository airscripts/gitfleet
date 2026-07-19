# release

## Purpose

`release` manages releases and release assets.

## Why This Exists

Releases mark shipped versions and connect source, artifacts, and deployment
records.

## When To Use It

Use `release` to list, inspect, create, or delete repository releases.

## Before You Run

Choose the release tag and confirm it matches the commit or artifact workflow
you intend to publish. Release deletion can remove provider release metadata and
assets while leaving the Git tag behavior provider-dependent.

## Common Commands

- `gitfleet release list --repo owner/repository`
- `gitfleet release list --repo owner/repository --limit 20 --page 2`
- `gitfleet release view v1.0.0 --repo owner/repository`
- `gitfleet release create --tag v1.0.0 --repo owner/repository --title "v1.0.0"`
- `gitfleet release delete v1.0.0 --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both support release operations.

## Safety Notes

Creating and deleting releases mutates provider state. Delete operations require
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for release automation and changelog tooling.
For list output, `--limit` is the page size and `--page` selects a 1-based
provider page.

## Related Commands

See [pipeline](./pipeline.md), [deploy](./deploy.md), [attestation](./attestation.md),
and [license](./license.md).
