# security

## Purpose

`security` inspects security advisories and scanning alerts.

## Why This Exists

Security alerts are operational signals that should be available from the same
repository management CLI.

## When To Use It

Use `security` during triage, compliance review, or release readiness checks.

## Before You Run

Use a token with access to security alert data. Results can include sensitive
repository, dependency, or secret-scanning metadata. Decide whether output is
safe for logs before using `--json` in CI.

## Common Commands

- `gitfleet security advisories --repo owner/repository`
- `gitfleet security secret-scans --repo owner/repository`
- `gitfleet security codeql --repo owner/repository`

## Provider Support

GitHub supports security alert operations. GitLab currently reports this
capability as unsupported.

## Safety Notes

These commands are read-only, but results can contain sensitive security
metadata.

## JSON/Automation Notes

Use `--json` for audit pipelines and vulnerability dashboards.

## Related Commands

See [advisory](./advisory.md), [deps](./deps.md), [attestation](./attestation.md),
and [policy](./policy.md).
