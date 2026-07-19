#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Deploy List"
expect_exit_0 "deploy list succeeds" gitfleet deploy list --repo "$GITFLEET_PLAYBOOK_REPO" --limit 5 --page 1
