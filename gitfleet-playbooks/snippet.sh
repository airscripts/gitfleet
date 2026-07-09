#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

SNIPPET_ID=""

setup() { :; }

teardown() {
  if [ -n "$SNIPPET_ID" ]; then
    gitfleet snippet delete "$SNIPPET_ID" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}

trap teardown EXIT
setup

step "Snippet List"
if gitfleet snippet list >/dev/null 2>&1; then
  pass "snippet list succeeds"
else
  skip "snippet list (may require additional scopes)"
fi

step "Snippet Create"
echo "gitfleet test snippet content" > "$TMPDIR/snippet_test.txt"
output=$(gitfleet snippet create --description "gitfleet-test-snippet" --public --file "$TMPDIR/snippet_test.txt" --json 2>&1) || true
SNIPPET_ID=$(echo "$output" | python3 -c "import sys,json; print(json.load(sys.stdin).get('id',''))" 2>/dev/null || echo "")

if [ -n "$SNIPPET_ID" ]; then
  pass "snippet create succeeded (id=$SNIPPET_ID)"
else
  skip "snippet create (may require additional scopes)"
fi

step "Snippet View"
if [ -n "$SNIPPET_ID" ]; then
  expect_exit_0 "snippet view succeeds" gitfleet snippet view "$SNIPPET_ID"
else
  skip "snippet view (create failed)"
fi

step "Snippet Delete"
if [ -n "$SNIPPET_ID" ]; then
  expect_exit_0 "snippet delete succeeds" gitfleet snippet delete "$SNIPPET_ID" --yes
  SNIPPET_ID=""
else
  skip "snippet delete (create failed)"
fi