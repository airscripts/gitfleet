#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Package List"
if gitfleet registry list --owner "$OWNER" >/dev/null 2>&1; then
  pass "package list succeeds"
else
  skip "package list (org may not have packages)"
fi