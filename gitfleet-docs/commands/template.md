# template

## Purpose

`template` discovers repository templates.

## Why This Exists

Templates standardize new work and issue intake.

## When To Use It

Use `template` when inspecting available issue templates for a repository.

## Before You Run

Resolve the repository with `--repo` or the current Git remote. Templates are
read from provider-supported repository metadata; missing templates usually mean
the repository has not configured them or the provider does not expose them in
the same way.

## Common Commands

- `gitfleet template list --repo owner/repository`

## Provider Support

GitHub and GitLab both expose template support.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for repository setup checks.

## Related Commands

See [issue](./issue.md), [label](./label.md), and [repo](./repo.md).
