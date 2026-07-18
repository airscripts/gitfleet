# Security Governance

Use security, policy, and governance commands together when auditing or
hardening repositories.

Start with read-only security and dependency commands. Then inspect existing
branch, tag, or ruleset policy before making changes. This order keeps security
work explainable in reviews and incident notes.

```bash
gitfleet security advisories --repo owner/repository
gitfleet security secret-scans --repo owner/repository
gitfleet security codeql --repo owner/repository
gitfleet deps list --repo owner/repository
gitfleet deps review --repo owner/repository --base main --head feature
gitfleet policy branch-protection get main --repo owner/repository
gitfleet govern list-rulesets --repo owner/repository
```

Provider support differs. GitHub supports rulesets, security alerts,
attestations, and repository secrets. GitLab supports protected tags and wiki
operations.

Security output can contain sensitive metadata. Prefer `--json` for controlled
audit pipelines and avoid pasting raw alert details into public issues.
