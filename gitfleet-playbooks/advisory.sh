#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_REPO_NAME="gitfleet-test-advisory-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false

setup() {
  if has_capability "advisories" && gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --public --initialize --yes >/dev/null 2>&1; then
    REPO_CREATED=true
  fi
}
teardown() {
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "Advisory List"
if ! has_capability "advisories"; then
  expect_exit_non0 "advisories are explicitly unsupported" gitfleet advisory list
  exit 0
fi

expect_exit_0 "advisory list succeeds" gitfleet advisory list --repo "$TEST_REPO"
