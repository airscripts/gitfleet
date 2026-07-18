# advisory

## Purpose

`advisory` lists and views security advisories.

## Why This Exists

Security advisories are part of repository risk management and should be
inspectable without switching provider tools.

## When To Use It

Use `advisory` during vulnerability triage, release review, or dependency
response work.

## Before You Run

Select a profile with access to the repository or organization that owns the
advisory data. Advisory IDs are provider IDs, so capture them from `advisory
list`, provider security views, or prior JSON output before calling
`advisory view`.

## Common Commands

- `gitfleet advisory list --repo owner/repository`
- `gitfleet advisory view <advisory-id> --repo owner/repository`

## Provider Support

GitHub supports advisory operations. GitLab currently reports this capability as
unsupported.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for vulnerability dashboards or policy checks.

## Related Commands

See [security](./security.md), [deps](./deps.md), and
[attestation](./attestation.md).
