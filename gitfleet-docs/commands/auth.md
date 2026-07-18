# auth

## Purpose

`auth` manages provider accounts, profiles, hosts, and tokens.

## Why This Exists

Gitfleet works across providers and hosts. Profiles keep those identities
explicit and switchable.

## When To Use It

Use `auth` before provider-backed commands, when changing accounts, when setting
up CI, or when letting Gitfleet detect a profile from the current Git remote.

## Before You Run

Decide the provider, host, and profile name you want to create. For enterprise
or self-managed hosts, pass `--host` during login so later repository commands
resolve against the correct provider API. For CI, prefer environment tokens and
`GITFLEET_PROFILE` instead of interactive login.

## Common Commands

- `gitfleet auth login`
- `gitfleet auth login --provider gitlab --host git.example.com --profile work-gitlab`
- `gitfleet auth status`
- `gitfleet auth token`
- `gitfleet auth list`
- `gitfleet auth switch work`
- `gitfleet auth detect`
- `gitfleet auth setup-git`
- `gitfleet auth logout --profile old --yes`

## Provider Support

Authentication is Gitfleet infrastructure and supports GitHub and GitLab
profiles.

## Safety Notes

`auth logout` removes stored credentials and prompts unless `--yes` is supplied
for automation.

## JSON/Automation Notes

Use `GITFLEET_GITHUB_TOKEN`, `GITFLEET_GITLAB_TOKEN`, and `GITFLEET_PROFILE` in
CI. See [../authentication.md](../authentication.md).

## Related Commands

See [config](./config.md), [identity](./identity.md), and
[../configuration.md](../configuration.md).
