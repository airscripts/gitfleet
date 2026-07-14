#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Analytics Views"
expect_capability_or_unsupported "analytics views" "analytics" gitfleet analytics views --repo "$REPO"

step "Analytics Clones"
expect_capability_or_unsupported "analytics clones" "analytics" gitfleet analytics clones --repo "$REPO"
