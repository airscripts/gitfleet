# deps

## Purpose

`deps` inspects dependencies and dependency changes.

## Why This Exists

Dependency inventory and review are part of repository health, security, and
release confidence.

## When To Use It

Use `deps` when auditing an SBOM, checking dependency drift, or reviewing a
change before merge.

## Before You Run

Use a profile with access to dependency metadata for the repository. Dependency
review compares two refs, so make sure `--base` and `--head` identify branches,
commits, or refs that the provider can compare.

## Common Commands

- `gitfleet deps list --repo owner/repository`
- `gitfleet deps review --repo owner/repository --base main --head feature`

## Provider Support

GitHub supports dependency APIs. GitLab currently reports this capability as
unsupported.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for policy checks and release gates.

## Related Commands

See [advisory](./advisory.md), [security](./security.md), and
[change](./change.md).
