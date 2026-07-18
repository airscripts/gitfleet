# config

## Purpose

`config` manages Gitfleet configuration values.

## Why This Exists

Configuration needs a stable interface for scripts and users instead of direct
TOML editing.

## When To Use It

Use `config` to set, read, or remove Gitfleet settings.

## Before You Run

Prefer `auth` for profile and credential workflows. Use `config` for explicit
Gitfleet settings when you know the key to manage. Environment variables can
override or influence behavior at runtime, so check both local config and the
shell environment when debugging.

## Common Commands

- `gitfleet config set key value`
- `gitfleet config get key`
- `gitfleet config unset key`

## Provider Support

Configuration is local Gitfleet infrastructure and does not require a provider
context.

## Safety Notes

Unsetting a key changes local behavior. Review profile and environment
resolution in [../configuration.md](../configuration.md).

## JSON/Automation Notes

Use `--json` when a script needs to parse configuration values.

## Related Commands

See [auth](./auth.md), [alias](./alias.md), and
[../configuration.md](../configuration.md).
