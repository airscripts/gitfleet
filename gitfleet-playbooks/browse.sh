#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Browse Repo"
expect_exit_0 "browse repo succeeds" gitfleet browse open --repo "$GITFLEET_PLAYBOOK_REPO"