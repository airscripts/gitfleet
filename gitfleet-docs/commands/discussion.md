# discussion

## Purpose

`discussion` manages provider discussions.

## Why This Exists

Discussions are collaboration artifacts distinct from issues and changes.

## When To Use It

Use `discussion` for long-form repository or community conversations.

## Before You Run

Confirm that discussions are enabled for the repository and that the active
provider supports them. Creating a discussion can require a provider category
ID, which should come from provider configuration or existing discussion data.

## Common Commands

- `gitfleet discussion list --repo owner/repository`
- `gitfleet discussion view <discussion-id> --repo owner/repository`
- `gitfleet discussion create "Topic" --repo owner/repository --body "Details"`

## Provider Support

GitHub supports discussions. GitLab currently reports this capability as
unsupported.

## Safety Notes

Creating discussions mutates provider state.

## JSON/Automation Notes

Use `--json` for reporting or moderation tooling.

## Related Commands

See [issue](./issue.md), [review](./review.md), and [inbox](./inbox.md).
