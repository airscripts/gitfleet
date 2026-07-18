# runner

## Purpose

`runner` manages CI/CD runners.

## Why This Exists

Runner inventory and removal are infrastructure tasks tied to pipeline health
and cost control.

## When To Use It

Use `runner` to list available runners or remove stale runners.

## Before You Run

Resolve the repository with `--repo` or the current Git remote. Removing a
runner by ID should be done only after checking that active jobs do not depend
on it.

## Common Commands

- `gitfleet runner list --repo owner/repository`
- `gitfleet runner remove <runner-id> --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both expose runner support.

## Safety Notes

Removing a runner can stop jobs from executing. It requires confirmation or
`--yes`.

## JSON/Automation Notes

Use `--json` for runner inventory and cleanup jobs.

## Related Commands

See [pipeline](./pipeline.md), [environment](./environment.md), and
[deploy](./deploy.md).
