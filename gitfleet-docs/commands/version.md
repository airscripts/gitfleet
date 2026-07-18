# version

## Purpose

`version` prints Gitfleet version information.

## Why This Exists

Version output is required for bug reports, support requests, and automation
that validates installed tooling.

## When To Use It

Use `version` after installation, in CI images, or when reporting issues.

## Before You Run

No profile or provider token is required. `version` is safe to run even when
credentials are missing or malformed.

## Common Commands

- `gitfleet version`
- `gf version`
- `gitfleet --version`

## Provider Support

Version output is local and does not require a provider context.

## Safety Notes

This command is read-only.

## JSON/Automation Notes

Use plain version output for environment checks.

## Related Commands

See [completion](./completion.md), [config](./config.md), and [auth](./auth.md).
