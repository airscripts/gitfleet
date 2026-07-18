# environment

## Purpose

`environment` manages deployment environments.

## Why This Exists

Deployment environments provide named targets for releases, deployments,
approval policy, and automation.

## When To Use It

Use `environment` to list, create, or delete repository deployment
environments.

## Before You Run

Choose the exact environment name used by your deployment workflows, such as
`staging` or `production`. Removing an environment can affect deployment
history, approvals, secrets, variables, or provider UI state depending on the
provider.

## Common Commands

- `gitfleet environment list --repo owner/repository`
- `gitfleet environment create production --repo owner/repository`
- `gitfleet environment delete production --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both expose environment support.

## Safety Notes

Deleting an environment is destructive and requires confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for deployment inventory and environment audits.

## Related Commands

See [deploy](./deploy.md), [pipeline](./pipeline.md), [secret](./secret.md), and
[variable](./variable.md).
