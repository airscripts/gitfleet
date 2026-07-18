# planning

## Purpose

`planning` groups milestone and project operations.

## Why This Exists

Planning artifacts connect issues, changes, and releases. Gitfleet keeps
milestones and projects under one product family.

## When To Use It

Use `planning` to track delivery milestones or manage provider project boards.

## Before You Run

Use `planning milestone` for repository-scoped delivery targets and
`planning project` for owner-scoped project boards. Project IDs are provider
IDs, so capture them from `project list` or provider output before viewing or
deleting a project.

## Common Commands

- `gitfleet planning milestone list --repo owner/repository`
- `gitfleet planning milestone create "v1.0" --repo owner/repository`
- `gitfleet planning milestone view 1 --repo owner/repository`
- `gitfleet planning milestone update 1 --repo owner/repository --description "Updated scope"`
- `gitfleet planning milestone delete 1 --repo owner/repository --yes`
- `gitfleet planning project list --owner owner`
- `gitfleet planning project create "Roadmap" --owner owner`
- `gitfleet planning project view <project-id>`
- `gitfleet planning project delete <project-id> --yes`

## Provider Support

GitHub and GitLab both expose milestone support. Project support is available
where the active provider exposes a compatible project API.

## Safety Notes

Deleting milestones or projects is destructive and requires confirmation or
`--yes`.

## JSON/Automation Notes

Use `--json` for planning reports and release dashboards.

## Related Commands

See [issue](./issue.md), [change](./change.md), and [release](./release.md).
