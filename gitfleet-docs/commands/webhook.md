# webhook

## Purpose

`webhook` manages webhooks and deliveries.

## Why This Exists

Webhooks connect repositories to external automation and need consistent
inventory, creation, deletion, and test operations.

## When To Use It

Use `webhook` when auditing integrations, adding an endpoint, removing an
endpoint, or testing delivery.

## Before You Run

Confirm that you control the destination URL and understand which events will be
sent. Webhook secrets should be treated as sensitive values. Testing a webhook
may send a real delivery to the configured endpoint.

## Common Commands

- `gitfleet webhook list --repo owner/repository`
- `gitfleet webhook create --repo owner/repository --url https://hooks.example.com/gitfleet`
- `gitfleet webhook test <hook-id> --repo owner/repository`
- `gitfleet webhook delete <hook-id> --repo owner/repository --yes`

## Provider Support

GitHub and GitLab both expose webhook support.

## Safety Notes

Creating, testing, and deleting webhooks affect external integrations. Delete
requires confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for integration inventory.

## Related Commands

See [pipeline](./pipeline.md), [deploy](./deploy.md), and [api](./api.md).
