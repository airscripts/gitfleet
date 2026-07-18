# Command Reference

This reference follows the public Gitfleet command vocabulary. Use
`gitfleet help [command...]` for exact options in the installed binary.

Each command page is intentionally more than a syntax list. It should explain
the product reason for the command, when the command is useful, what provider
capabilities it needs, which examples are safe to run first, and which actions
change provider state. See [../documentation-guide.md](../documentation-guide.md)
for the full writing checklist.

## How To Use This Reference

Start with the command family that matches the job:

- Repository setup and retirement: `repo`, `workspace`, `label`, `template`,
  `license`.
- Collaboration: `change`, `review`, `issue`, `discussion`, `inbox`,
  `planning`.
- Delivery: `pipeline`, `release`, `deploy`, `environment`, `runner`,
  `registry`, `webhook`.
- Security and governance: `security`, `advisory`, `deps`, `attestation`,
  `policy`, `govern`, `access`, `identity`.
- Automation and local tooling: `auth`, `config`, `api`, `alias`,
  `completion`, `version`.

Read the page before running mutating commands. Use `gitfleet help
[command...]` for the exact option set in your installed version.

## Families

| Family | Purpose |
| ------ | ------- |
| [access](./access.md) | Organization, group, team, and member access. |
| [advisory](./advisory.md) | Security advisory listing and inspection. |
| [alias](./alias.md) | Local Gitfleet command aliases. |
| [analytics](./analytics.md) | Repository traffic reports. |
| [api](./api.md) | Raw provider API requests. |
| [attestation](./attestation.md) | Artifact provenance inspection. |
| [auth](./auth.md) | Provider accounts, profiles, and tokens. |
| [browse](./browse.md) | Open provider resources in a browser. |
| [change](./change.md) | Pull requests and merge requests. |
| [code](./code.md) | Code search and file viewing. |
| [completion](./completion.md) | Shell completions and man pages. |
| [config](./config.md) | Gitfleet configuration values. |
| [deps](./deps.md) | Dependency lists and dependency review. |
| [deploy](./deploy.md) | Deployment records. |
| [dev](./dev.md) | Hosted development environments. |
| [discussion](./discussion.md) | Provider discussions. |
| [environment](./environment.md) | Deployment environments. |
| [govern](./govern.md) | Repository rulesets and governance. |
| [identity](./identity.md) | SSH and GPG keys. |
| [inbox](./inbox.md) | Notifications and read state. |
| [issue](./issue.md) | Issues and work items. |
| [label](./label.md) | Repository labels. |
| [license](./license.md) | License discovery. |
| [planning](./planning.md) | Milestones and projects. |
| [pipeline](./pipeline.md) | Pipeline definitions and runs. |
| [policy](./policy.md) | Branch and tag protection. |
| [registry](./registry.md) | Packages and container images. |
| [release](./release.md) | Releases and assets. |
| [repo](./repo.md) | Repository lifecycle and forks. |
| [review](./review.md) | Comments and reactions. |
| [runner](./runner.md) | CI/CD runners. |
| [search](./search.md) | Provider resource search. |
| [secret](./secret.md) | Repository secrets. |
| [security](./security.md) | Security alerts and scanning. |
| [site](./site.md) | Repository Pages sites. |
| [snippet](./snippet.md) | Provider-hosted snippets. |
| [template](./template.md) | Repository issue templates. |
| [variable](./variable.md) | Repository variables. |
| [version](./version.md) | Version information. |
| [webhook](./webhook.md) | Webhooks and deliveries. |
| [wiki](./wiki.md) | Repository wiki pages. |
| [workspace](./workspace.md) | Named fleets and multi-repository operations. |
