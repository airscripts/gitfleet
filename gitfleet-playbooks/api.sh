#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

LABEL_NAME="gitfleet-test-api-$PB_RESOURCE_SUFFIX"
LABEL_CREATED=false
setup() { :; }
teardown() {
  if [ "$LABEL_CREATED" = true ]; then
    gitfleet api delete --endpoint "/repos/$REPO/labels/$LABEL_NAME" >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "API GET"
expect_exit_0 "api get succeeds" gitfleet api get --endpoint "/repos/$REPO"

step "API POST"
if gitfleet api post --endpoint "/repos/$REPO/labels" --body "{\"name\":\"$LABEL_NAME\",\"color\":\"f29513\"}" >/dev/null 2>&1; then
  pass "api post succeeds"
  LABEL_CREATED=true
else
  fail "api post failed"
fi

step "API DELETE"
expect_exit_0 "api delete succeeds" gitfleet api delete --endpoint "/repos/$REPO/labels/$LABEL_NAME"
LABEL_CREATED=false

step "API GET with --json"
expect_exit_0 "api get --json succeeds" gitfleet --json api get --endpoint "/repos/$REPO"

step "API POST invalid JSON rejected"
expect_exit_non0 "api post with invalid json fails" gitfleet api post --endpoint "/repos/$REPO" --body 'not-json'
