# secret

## Purpose

`secret` manages repository secrets.

## Why This Exists

Automation secrets configure CI/CD and deployment workflows and require careful
handling.

## When To Use It

Use `secret` to list, set, delete, or fetch the public key used for encrypted
secret submission.

## Before You Run

Use a profile with permission to manage repository automation secrets. Secret
values are passed as command arguments in the current CLI, so avoid doing that
on shared machines or shells that persist history. Prefer short-lived tokens
and rotate values after testing.

## Common Commands

- `gitfleet secret list --repo owner/repository`
- `gitfleet secret public-key --repo owner/repository`
- `gitfleet secret set API_TOKEN value --repo owner/repository`
- `gitfleet secret delete API_TOKEN --repo owner/repository --yes`

## Provider Support

GitHub supports repository secrets. GitLab currently reports this capability as
unsupported; use [variable](./variable.md) for GitLab CI/CD variables.

## Safety Notes

Setting and deleting secrets mutates sensitive automation configuration. Delete
requires confirmation or `--yes`.

## JSON/Automation Notes

Avoid printing secret values in logs. Use `--json` for metadata, not secret
disclosure.

## Related Commands

See [variable](./variable.md), [pipeline](./pipeline.md), and
[environment](./environment.md).
