# license

## Purpose

`license` discovers and inspects licenses.

## Why This Exists

License data affects repository setup, compliance, and reuse decisions.

## When To Use It

Use `license` when choosing a license or inspecting provider license metadata.

## Before You Run

License commands read provider license catalogs or license details. They do not
change repository files. If you need to add a license to a repository, use the
provider UI, repository template flow, or a normal Git change.

## Common Commands

- `gitfleet license list`
- `gitfleet license view mit`

## Provider Support

GitHub and GitLab both expose license support.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for compliance and repository setup tools.

## Related Commands

See [repo](./repo.md), [deps](./deps.md), and [security](./security.md).
