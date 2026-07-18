# dev

## Purpose

`dev` manages hosted development environments.

## Why This Exists

Hosted development environments are useful for onboarding, reproduction, and
isolated work.

## When To Use It

Use `dev` to list, create, or delete hosted environments such as GitHub
Codespaces.

## Before You Run

Use a provider profile with development-environment permission. Creating an
environment may consume provider quota or billable resources. Deleting an
environment is keyed by environment ID and resolves the repository from the
current Git remote.

## Common Commands

- `gitfleet dev list --repo owner/repository`
- `gitfleet dev create --repo owner/repository --branch main`
- `gitfleet dev delete <environment-id> --yes`

## Provider Support

GitHub supports development environments. GitLab currently reports this
capability as unsupported.

## Safety Notes

Deleting a development environment is destructive and requires confirmation or
`--yes`.

## JSON/Automation Notes

Use `--json` for inventory and cleanup automation.

## Related Commands

See [repo](./repo.md), [code](./code.md), and [pipeline](./pipeline.md).
