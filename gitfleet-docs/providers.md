# Providers

Gitfleet includes GitHub and GitLab providers.

| Provider | Default host | Typical profile |
| -------- | ------------ | --------------- |
| GitHub   | `github.com` | `gitfleet auth login --provider github` |
| GitLab   | `gitlab.com` | `gitfleet auth login --provider gitlab` |

Use `--host` for GitHub Enterprise or self-managed GitLab hosts:

```bash
gitfleet auth login --provider github --host github.example.com --profile work
gitfleet auth login --provider gitlab --host git.example.com --profile work-gitlab
```

## Capability Summary

| Command capabilities | GitHub | GitLab |
| -------------------- | ------ | ------ |
| Repositories, changes, reviews, issues, pipelines, releases | Yes | Yes |
| Milestones, webhooks, access, identity, notifications | Yes | Yes |
| Search, code, labels, templates, environments, runners | Yes | Yes |
| Variables, browse, raw API, deployments, licenses, snippets | Yes | Yes |
| Repository policies and package registry | Yes | Yes |
| Projects, Pages, discussions, security alerts, dev environments | Yes | No |
| Analytics, rulesets, merge automation, dependency and advisory APIs | Yes | No |
| Attestations and repository secrets | Yes | No |
| Wikis and protected tags | No | Yes |

Provider APIs can still differ inside a supported command family. Check
`gitfleet auth status` to inspect the active provider and declared
capabilities.

## How To Choose a Profile

Use one profile per account and host. A personal GitHub account, a work GitHub
Enterprise account, and a self-managed GitLab account should be separate
profiles. That separation keeps repository targeting predictable and avoids
accidentally sending a command to the wrong provider host.

```bash
gitfleet auth login --provider github --profile personal
gitfleet auth login --provider github --host github.example.com --profile work
gitfleet auth login --provider gitlab --host git.example.com --profile work-gitlab
```

Use `gitfleet auth detect` inside a clone when you want Gitfleet to match the
current remote to a configured profile.

## Unsupported Behavior

Unsupported commands are part of normal provider-neutral use. They usually mean
the selected provider does not expose the feature through the capability Gitfleet
requires. Switch to a profile for a provider that supports the capability, or
choose the closest supported Gitfleet command family for the actual job.
