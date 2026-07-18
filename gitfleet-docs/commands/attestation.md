# attestation

## Purpose

`attestation` inspects artifact provenance.

## Why This Exists

Build provenance is part of supply-chain security and release confidence.

## When To Use It

Use `attestation` when validating release artifacts, deployment inputs, or
repository compliance.

## Before You Run

Use a profile that can read the repository and its artifact provenance metadata.
Attestation availability depends on the provider and on whether the repository
publishes provenance for the artifacts you care about.

## Common Commands

- `gitfleet attestation list --repo owner/repository`

## Provider Support

GitHub supports attestation operations. GitLab currently reports this capability
as unsupported.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for release gates and compliance checks.

## Related Commands

See [release](./release.md), [security](./security.md), and [deps](./deps.md).
