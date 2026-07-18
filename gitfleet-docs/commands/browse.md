# browse

## Purpose

`browse` opens provider resources in a browser.

## Why This Exists

Some workflows end in a provider UI. `browse` keeps resource resolution inside
Gitfleet while handing visual inspection to the browser.

## When To Use It

Use `browse` when you want to open a repository from the active provider and
profile.

## Before You Run

Make sure the current shell has browser-opening support. If you are inside SSH,
CI, or a minimal container, prefer copying the provider URL from `repo view` or
using `--json` with a modeled read command.

## Common Commands

- `gitfleet browse open --repo owner/repository`

## Provider Support

GitHub and GitLab both expose browsing support.

## Safety Notes

This command is read-only, but it launches a browser or opens a URL in the local
environment.

## JSON/Automation Notes

Use provider URLs directly in automation; `browse` is primarily interactive.

## Related Commands

See [repo](./repo.md), [code](./code.md), and [search](./search.md).
