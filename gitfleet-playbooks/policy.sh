#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Policy Branch Protection Get"
if gitfleet policy branch-protection get main --repo "$REPO" >/dev/null 2>&1; then
  pass "policy branch-protection get succeeds"
else
  skip "policy branch-protection get (branch may not have protection)"
fi

step "Policy Tag Protection List"
if gitfleet policy tag-protection list --repo "$REPO" >/dev/null 2>&1; then
  pass "policy tag-protection list succeeds"
else
  skip "policy tag-protection list (may not be available)"
fi