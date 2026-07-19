# pipeline

## Purpose

`pipeline` manages pipeline definitions, runs, triggers, cancellations, and
reruns.

## Why This Exists

CI/CD terminology differs across providers. Gitfleet provides one command
family for workflow definitions and execution history.

## When To Use It

Use `pipeline` to inspect workflows, check run status, trigger a run, cancel a
run, or rerun a failed pipeline.

## Before You Run

Use a token with CI/CD read permission, and write permission for trigger,
cancel, or rerun operations. GitHub requires a pipeline definition ID when
triggering; GitLab may use different provider concepts underneath the normalized
command.

## Common Commands

- `gitfleet pipeline list-def --repo owner/repository`
- `gitfleet pipeline view-def <workflow-id> --repo owner/repository`
- `gitfleet pipeline list-runs --repo owner/repository`
- `gitfleet pipeline list-runs --repo owner/repository --limit 25 --page 2`
- `gitfleet pipeline view-run <run-id> --repo owner/repository`
- `gitfleet pipeline trigger <workflow-id> --repo owner/repository --ref main`
- `gitfleet pipeline cancel <run-id> --repo owner/repository --yes`
- `gitfleet pipeline rerun <run-id> --repo owner/repository`

## Provider Support

GitHub and GitLab both support pipeline operations.

## Safety Notes

Triggering, canceling, and rerunning pipelines mutate CI/CD state. Canceling
requires confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for status checks, release gates, and deployment automation.
For list definitions and list runs, `--limit` is the page size and `--page`
selects a 1-based provider page.

## Related Commands

See [release](./release.md), [deploy](./deploy.md), [environment](./environment.md),
[runner](./runner.md), and [registry](./registry.md).
