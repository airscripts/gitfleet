#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Pipeline List-Def"
if provider_is github; then
  expect_exit_0 "pipeline list-def succeeds" gitfleet pipeline list-def --repo "$GITFLEET_PLAYBOOK_REPO"
else
  expect_exit_non0 "pipeline list-def is explicitly unsupported" gitfleet pipeline list-def --repo "$GITFLEET_PLAYBOOK_REPO"
fi

step "Pipeline List-Runs"
expect_exit_0 "pipeline list-runs succeeds" gitfleet pipeline list-runs --repo "$GITFLEET_PLAYBOOK_REPO" --limit 5 --page 1
