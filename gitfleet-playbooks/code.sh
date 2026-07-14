#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Code Search"
expect_exit_0 "code search succeeds" gitfleet code search "README" --repo "$GITFLEET_PLAYBOOK_REPO"

step "Code View"
expect_output "code view returns decoded content" "#" gitfleet code view README.md --repo "$GITFLEET_PLAYBOOK_REPO"
