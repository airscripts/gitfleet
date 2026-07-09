#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Analytics Views"
expect_exit_0 "analytics views succeeds" gitfleet analytics views --repo "$REPO"

step "Analytics Clones"
expect_exit_0 "analytics clones succeeds" gitfleet analytics clones --repo "$REPO"