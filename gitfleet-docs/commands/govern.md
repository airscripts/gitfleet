# govern

## Purpose

`govern` inspects and manages repository rulesets.

## Why This Exists

Governance features control how repositories accept changes and releases.

## When To Use It

Use `govern` when auditing or changing rulesets for a repository.

## Before You Run

Use a profile with repository administration permission. Ruleset creation uses
provider governance concepts and may need follow-up tuning in the provider UI or
through raw API calls as Gitfleet's modeled ruleset input evolves.

## Common Commands

- `gitfleet govern list-rulesets --repo owner/repository`
- `gitfleet govern create-ruleset --repo owner/repository --name "main protection"`
- `gitfleet govern delete-ruleset <ruleset-id> --repo owner/repository --yes`

## Provider Support

GitHub supports ruleset governance. GitLab currently reports this capability as
unsupported.

## Safety Notes

Creating and deleting rulesets changes repository policy. Deleting requires
confirmation or `--yes`.

## JSON/Automation Notes

Use `--json` for governance reports and policy checks.

## Related Commands

See [policy](./policy.md), [security](./security.md), and [access](./access.md).
