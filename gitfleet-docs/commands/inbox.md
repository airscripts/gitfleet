# inbox

## Purpose

`inbox` manages notifications and read state.

## Why This Exists

Notifications are central to review and issue workflows, and users need a
provider-neutral way to scan and clear them.

## When To Use It

Use `inbox` to list unread work or mark notifications as read.

## Before You Run

Use a token that can read notifications. Notification state is account-scoped,
so `mark-read` affects what the active provider account sees, not just the
current terminal session.

## Common Commands

- `gitfleet inbox list`
- `gitfleet inbox list --repo owner/repository`
- `gitfleet inbox mark-read --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both expose notification support.

## Safety Notes

Marking notifications as read mutates notification state and requires
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for notification dashboards.

## Related Commands

See [issue](./issue.md), [change](./change.md), [review](./review.md), and
[discussion](./discussion.md).
