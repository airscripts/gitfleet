# code

## Purpose

`code` searches source code and views file contents.

## Why This Exists

Repository code discovery should work with one command surface across supported
providers.

## When To Use It

Use `code` to find references, inspect files, or fetch source content from a
provider repository.

## Before You Run

Use a token that can read the target repository contents. Provider search syntax
and indexing freshness can differ, so treat `code search` as provider-backed
search rather than a local grep replacement. Use `code view` when you already
know the path.

## Common Commands

- `gitfleet code search "Renderer" --repo owner/repository`
- `gitfleet code search "Renderer" --repo owner/repository --limit 25 --page 2`
- `gitfleet code view src/main.rs --repo owner/repository`

## Provider Support

GitHub and GitLab both support code operations.

## Safety Notes

These commands are read-only.

## JSON/Automation Notes

Use `--json` for indexing, audit, or migration scripts.
For code search, `--limit` is the page size and `--page` selects a 1-based
provider page.

## Related Commands

See [search](./search.md), [repo](./repo.md), and [browse](./browse.md).
