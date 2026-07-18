# Change Review

Use `change` for proposed changes and `review` for comments and reactions.

This workflow treats a pull request and merge request as the same product
concept: a proposed change. Use `change` for the lifecycle of the proposal and
`review` for conversation around it.

```bash
gitfleet change create "Add feature" --head feature --base main
gitfleet change list --repo owner/repository --state open
gitfleet change view 42 --repo owner/repository
gitfleet review comment list 42 --repo owner/repository
gitfleet review comment create 42 "Please add a regression test." --repo owner/repository
gitfleet review reaction create 42 eyes --repo owner/repository
gitfleet change merge 42 --repo owner/repository --method squash --yes
```

Use `review comment create --target issue` when adding a comment to an issue
instead of a change request.

Merging is the point where repository state changes materially. Use `change
view`, pipeline checks, and review comments before running `change merge`.
