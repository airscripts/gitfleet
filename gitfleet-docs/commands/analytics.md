# analytics

## Purpose

`analytics` inspects repository traffic.

## Why This Exists

Repository traffic and clone data help maintainers understand usage and impact
without leaving the Gitfleet command surface.

## When To Use It

Use `analytics` for maintainer reporting, adoption checks, or release impact
reviews.

## Before You Run

Use a token with permission to read repository traffic data. Providers may limit
traffic windows, retention, and visibility to repository administrators or
maintainers. Treat the output as a recent activity signal, not a complete audit
log.

## Common Commands

- `gitfleet analytics views --repo owner/repository`
- `gitfleet analytics clones --repo owner/repository`

## Provider Support

GitHub supports analytics operations. GitLab currently reports this capability
as unsupported.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for scheduled reporting.

## Related Commands

See [repo](./repo.md), [pipeline](./pipeline.md), and [release](./release.md).
