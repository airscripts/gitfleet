# policy

## Purpose

`policy` manages repository protection policies.

## Why This Exists

Branch and tag protection are core repository controls, but providers expose
them differently.

## When To Use It

Use `policy` to inspect, set, or remove branch protection and tag protection.

## Before You Run

Use a profile with repository administration permission. Know the branch name or
tag pattern you are changing. Policy removal can immediately weaken protections
for merges, force pushes, releases, or tag creation.

## Common Commands

- `gitfleet policy branch-protection get main --repo owner/repository`
- `gitfleet policy branch-protection set main --repo owner/repository`
- `gitfleet policy branch-protection delete main --repo owner/repository --yes`
- `gitfleet policy tag-protection list --repo group/project`
- `gitfleet policy tag-protection create "v*" --repo group/project`
- `gitfleet policy tag-protection delete <rule-id> --repo group/project --yes`

## Provider Support

GitHub and GitLab both support repository policies. GitLab supports protected
tags through `policy tag-protection`; GitHub reports protected tags as
unsupported and should use [govern](./govern.md) rulesets where appropriate.

## Safety Notes

Removing protection can expose branches or tags. Delete operations require
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for policy inventory and drift detection.

## Related Commands

See [govern](./govern.md), [security](./security.md), and [access](./access.md).
