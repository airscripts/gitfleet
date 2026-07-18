# site

## Purpose

`site` manages repository Pages sites.

## Why This Exists

Static site hosting is part of repository delivery for docs and project pages.

## When To Use It

Use `site` to inspect, enable, or disable Pages for a repository.

## Before You Run

Know the source branch or source path for the site. Pages configuration affects
published documentation or project sites, so check the repository's existing
publication settings before deleting or recreating it.

## Common Commands

- `gitfleet site get --repo owner/repository`
- `gitfleet site create --repo owner/repository --source main/docs`
- `gitfleet site delete --repo owner/repository --yes`

## Provider Support

GitHub supports Pages site operations. GitLab currently reports this capability
as unsupported.

## Safety Notes

Creating or deleting a site changes publication state. Deleting requires
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for publication audits.

## Related Commands

See [repo](./repo.md), [deploy](./deploy.md), and [wiki](./wiki.md).
