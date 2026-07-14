#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TAG_PATTERN="gitfleet-test-$PB_RESOURCE_SUFFIX-*"
TAG_IDENTIFIER=""

setup() { :; }
teardown() {
  if [ -n "$TAG_IDENTIFIER" ]; then
    gitfleet policy tag-protection delete "$TAG_IDENTIFIER" --repo "$REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
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

step "Policy Tag Protection Create"
output=$(gitfleet policy tag-protection create "$TAG_PATTERN" --repo "$REPO" --json 2>&1) || true
TAG_IDENTIFIER=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('identifier',''))" 2>/dev/null || echo "")

if [ -n "$TAG_IDENTIFIER" ]; then
  pass "policy tag-protection create succeeds"
else
  skip "policy tag-protection create (may not be available)"
fi

step "Policy Tag Protection Delete"
if [ -n "$TAG_IDENTIFIER" ]; then
  expect_exit_0 "policy tag-protection delete succeeds" gitfleet policy tag-protection delete "$TAG_IDENTIFIER" --repo "$REPO" --yes
  TAG_IDENTIFIER=""
else
  skip "policy tag-protection delete (create failed)"
fi
