# deploy

## Purpose

`deploy` manages deployment records.

## Why This Exists

Deployment state belongs with release and pipeline context, but provider APIs
model it differently.

## When To Use It

Use `deploy` to list deployments or create a deployment record for a repository.

## Before You Run

Know the target repository, ref, and environment name. Deployment creation
records state in the provider; it does not by itself build, ship, or roll back
an artifact unless the provider or surrounding automation reacts to that record.

## Common Commands

- `gitfleet deploy list --repo owner/repository`
- `gitfleet deploy create --repo owner/repository --ref main --environment production`

## Provider Support

GitHub and GitLab both expose deployment support.

## Safety Notes

Creating deployments mutates provider state. Use clear environment and ref
values.

## JSON/Automation Notes

Use `--json` in deployment systems and keep status logs on stderr.

## Related Commands

See [pipeline](./pipeline.md), [release](./release.md), and
[environment](./environment.md).
