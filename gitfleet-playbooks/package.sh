#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Package List"
if [ "$GITFLEET_PLAYBOOK_PROVIDER" = "gitlab" ]; then
  PACKAGE_OWNER="$GITFLEET_PLAYBOOK_REPO"
else
  PACKAGE_OWNER="$GITFLEET_PLAYBOOK_OWNER"
fi

expect_exit_0 "package list succeeds" gitfleet registry list --owner "$PACKAGE_OWNER" --limit 5 --page 1
