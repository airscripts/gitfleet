# access

## Purpose

`access` manages organizations, groups, teams, and membership.

## Why This Exists

Access management is provider-specific at the API layer, but teams need one
workflow for checking membership, inviting users, removing users, and inspecting
teams.

## When To Use It

Use `access` when onboarding or offboarding people, checking organization
membership, creating teams, or listing team members.

## Before You Run

Use a profile whose token can read organization or group membership. Invitation,
team creation, and removal commands need elevated organization or group
permissions from the provider. Decide whether you are managing organization
membership directly with `access org` or team-level visibility with
`access team`; those operations affect different provider resources.

## Common Commands

- `gitfleet access org list-members example`
- `gitfleet access org invite example alice --role member`
- `gitfleet access org remove example alice --yes`
- `gitfleet access team list example`
- `gitfleet access team create example platform`
- `gitfleet access team list-members example platform`

## Provider Support

GitHub and GitLab both expose access capabilities. Exact permission names and
organization or group semantics remain provider-specific.

## Safety Notes

Removing members is destructive and requires confirmation in human mode or
`--yes` in automation.

## JSON/Automation Notes

Use `--json` for inventory and audit scripts.

## Related Commands

See [auth](./auth.md), [identity](./identity.md), [govern](./govern.md), and
[policy](./policy.md).
