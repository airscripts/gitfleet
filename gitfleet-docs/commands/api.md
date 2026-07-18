# api

## Purpose

`api` sends raw requests to the active provider.

## Why This Exists

Provider-neutral commands cover supported Gitfleet workflows. `api` is an
escape hatch for advanced users who need a provider endpoint that is not yet
modeled.

## When To Use It

Use `api` for diagnostics, one-off provider reads, or controlled automation
against an endpoint Gitfleet does not wrap.

## Before You Run

Know which provider profile is active and read the provider endpoint
documentation for the route you are calling. `api` does not normalize responses
into Gitfleet DTOs, so scripts should expect provider-specific JSON. For
mutating requests, prepare the body file and use `--dry-run` when checking the
request shape.

## Common Commands

- `gitfleet api get /user`
- `gitfleet api post /repos/owner/repository/dispatches --body payload.json --dry-run`
- `gitfleet api put /path --body payload.json`
- `gitfleet api patch /path --body payload.json`
- `gitfleet api delete /path --yes`

## Provider Support

GitHub and GitLab both expose raw API capability.

## Safety Notes

Mutating raw requests are powerful. Use `--dry-run` where supported and `--yes`
for destructive automation.

## JSON/Automation Notes

Prefer `--json` and explicit request bodies for scripts.

## Related Commands

Use modeled commands where available, especially [repo](./repo.md),
[change](./change.md), [issue](./issue.md), and [pipeline](./pipeline.md).
