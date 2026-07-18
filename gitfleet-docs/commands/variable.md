# variable

## Purpose

`variable` manages repository variables.

## Why This Exists

Variables configure CI/CD and deployment workflows without being secret values.

## When To Use It

Use `variable` to list, set, or delete repository automation variables.

## Before You Run

Use variables for non-secret configuration. Values can still affect builds and
deployments, so review dependent pipelines before deleting or changing them. Use
`secret` for sensitive values when the provider supports repository secrets.

## Common Commands

- `gitfleet variable list --repo owner/repository`
- `gitfleet variable set ENVIRONMENT production --repo owner/repository`
- `gitfleet variable delete ENVIRONMENT --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both expose variable support.

## Safety Notes

Setting and deleting variables mutates automation configuration. Delete requires
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for configuration audits.

## Related Commands

See [secret](./secret.md), [pipeline](./pipeline.md), and
[environment](./environment.md).
