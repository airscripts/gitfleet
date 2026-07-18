# issue

## Purpose

`issue` manages issues and work items.

## Why This Exists

Issues are common across providers, but field names and APIs differ.

## When To Use It

Use `issue` to create, list, or view tracked work.

## Before You Run

Resolve the repository with `--repo` or run from a cloned repository with a
provider remote. Listing defaults to open issues and a limited result set; pass
state and limit options when building triage views.

## Common Commands

- `gitfleet issue create "Bug title" --repo owner/repository --body "Details"`
- `gitfleet issue list --repo owner/repository --state open`
- `gitfleet issue view 17 --repo owner/repository`

## Provider Support

GitHub and GitLab both support issue operations.

## Safety Notes

Creating issues mutates provider state. Listing and viewing are read-only.

## JSON/Automation Notes

Use `--json` for triage dashboards and import/export workflows.

## Related Commands

See [review](./review.md), [label](./label.md), [planning](./planning.md), and
[inbox](./inbox.md).
