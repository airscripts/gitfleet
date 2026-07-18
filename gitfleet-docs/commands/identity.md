# identity

## Purpose

`identity` manages account SSH and GPG keys.

## Why This Exists

Developer account keys are provider resources that affect Git access and commit
verification.

## When To Use It

Use `identity` to list, add, or delete SSH and GPG keys for the active account.

## Before You Run

Make sure the active profile is the account whose keys you intend to change.
SSH keys affect Git authentication. GPG keys affect commit and tag verification.
Keep key material out of shell history and logs when possible.

## Common Commands

- `gitfleet identity ssh-key list`
- `gitfleet identity ssh-key add "laptop" "ssh-ed25519 ..."`
- `gitfleet identity ssh-key delete <key-id> --yes`
- `gitfleet identity gpg-key list`
- `gitfleet identity gpg-key add <armored-public-key>`
- `gitfleet identity gpg-key delete <key-id> --yes`

## Provider Support

GitHub and GitLab both expose identity key support.

## Safety Notes

Deleting keys can break Git access or commit verification. It requires
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for account key audits.

## Related Commands

See [auth](./auth.md) and [access](./access.md).
