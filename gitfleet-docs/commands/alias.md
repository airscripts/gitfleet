# alias

## Purpose

`alias` manages local Gitfleet command aliases.

## Why This Exists

Aliases shorten repeated workflows without adding legacy command names or shell
expansion behavior.

## When To Use It

Use `alias` for personal shortcuts such as common repository filters or repeated
workflow commands.

## Before You Run

Choose aliases that are clear to you later. Gitfleet blocks aliases that shadow
canonical commands, but it cannot tell whether a shortcut is too cryptic for
your team. Aliases are expanded as argument lists, not through a shell, so shell
operators, environment expansion, and pipes do not run as part of the alias.

## Common Commands

- `gitfleet alias set mine "repo list --owner alice"`
- `gitfleet alias get mine`
- `gitfleet alias list`
- `gitfleet alias delete mine`

## Provider Support

Aliases are local Gitfleet configuration and do not require a provider context.

## Safety Notes

Alias names cannot shadow canonical Gitfleet commands. Alias arguments are
forwarded without invoking a shell.

## JSON/Automation Notes

Aliases are mainly interactive conveniences. Prefer canonical commands in CI.

## Related Commands

See [config](./config.md), [completion](./completion.md), and
[version](./version.md).
