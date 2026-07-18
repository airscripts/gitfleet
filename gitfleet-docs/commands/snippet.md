# snippet

## Purpose

`snippet` manages provider-hosted snippets.

## Why This Exists

Snippets are lightweight hosted code or text artifacts that are useful outside a
full repository.

## When To Use It

Use `snippet` to list, view, create, or delete snippets.

## Before You Run

Prepare the file you want to publish and decide whether the snippet should be
public. Snippet visibility and ownership are provider-specific, so check the
active profile before creating content.

## Common Commands

- `gitfleet snippet list`
- `gitfleet snippet view <snippet-id>`
- `gitfleet snippet create --description "Example" --file example.txt`
- `gitfleet snippet delete <snippet-id> --yes`

## Provider Support

GitHub and GitLab both expose snippet capability.

## Safety Notes

Creating and deleting snippets mutates provider state. Delete requires
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for snippet inventory and migration.

## Related Commands

See [code](./code.md), [repo](./repo.md), and [api](./api.md).
