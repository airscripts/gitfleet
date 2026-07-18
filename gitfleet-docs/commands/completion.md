# completion

## Purpose

`completion` generates shell completions and man pages.

## Why This Exists

Shell integration makes the CLI easier to discover without adding runtime
provider behavior.

## When To Use It

Use `completion` after installing Gitfleet or when packaging it for a shell
environment.

## Before You Run

Pick the shell or generated artifact you need. `completion generate` writes the
completion script to stdout, so installation usually happens through shell
redirection or a package-manager recipe outside Gitfleet.

## Common Commands

- `gitfleet completion generate bash`
- `gitfleet completion generate zsh`
- `gitfleet completion generate fish`
- `gitfleet completion mangen`

## Provider Support

Completion generation is local and does not require a provider context.

## Safety Notes

These commands are read-only unless the shell redirects output to a file.

## JSON/Automation Notes

Completions are text output, not JSON API output.

## Related Commands

See [alias](./alias.md), [config](./config.md), and [version](./version.md).
