#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Pipeline List-Def"
expect_exit_0 "pipeline list-def succeeds" gitfleet pipeline list-def --repo "$REPO"

step "Pipeline List-Runs"
expect_exit_0 "pipeline list-runs succeeds" gitfleet pipeline list-runs --repo "$REPO" --limit 5