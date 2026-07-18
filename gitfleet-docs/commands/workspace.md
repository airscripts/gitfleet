# workspace

## Purpose

`workspace` manages named fleets and bounded multi-repository operations.

## Why This Exists

Fleet work needs an explicit repository set, predictable ordering, and safe
preview behavior.

## When To Use It

Use `workspace` when repeatedly operating on a group of repositories or when
archiving multiple compatible repositories.

## Before You Run

Define a small workspace first and inspect it with `workspace list`. Use
provider-qualified targets when a fleet spans hosts or providers. Workspace
mutations run only against targets that match the active provider and host.

## Common Commands

- `gitfleet workspace define --name platform --repos owner/api --repos owner/web`
- `gitfleet workspace list`
- `gitfleet workspace remove platform`
- `gitfleet workspace archive platform --dry-run`
- `gitfleet workspace archive platform --yes`

## Provider Support

Workspace definitions are local. Provider-backed workspace mutations run only
against repositories matching the active provider and host; other targets are
reported as skipped.

## Safety Notes

Use `--dry-run` before fleet mutations. `workspace archive` mutates repository
state and requires confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for fleet reports and automation results.

## Related Commands

See [repo](./repo.md), [auth](./auth.md), and
[../workflows/workspace-fleets.md](../workflows/workspace-fleets.md).
