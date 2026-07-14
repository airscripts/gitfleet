#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

LABEL_NAME="gitfleet-test-api-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
LABEL_CREATED=false

if provider_is github; then
  LABEL_BODY="{\"name\":\"$LABEL_NAME\",\"color\":\"f29513\"}"
else
  LABEL_BODY="{\"name\":\"$LABEL_NAME\",\"color\":\"#f29513\"}"
fi

ENCODED_LABEL=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$LABEL_NAME")
LABEL_ENDPOINT="$GITFLEET_PLAYBOOK_API_LABELS_ENDPOINT/$ENCODED_LABEL"
setup() { :; }
teardown() {
  if [ "$LABEL_CREATED" = true ]; then
    gitfleet api delete --endpoint "$LABEL_ENDPOINT" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "API GET"
expect_exit_0 "api get succeeds" gitfleet api get --endpoint "$GITFLEET_PLAYBOOK_API_REPO_ENDPOINT"

step "API POST"
if gitfleet api post --endpoint "$GITFLEET_PLAYBOOK_API_LABELS_ENDPOINT" --body "$LABEL_BODY" --yes >/dev/null 2>&1; then
  pass "api post succeeds"
  LABEL_CREATED=true
else
  fail "api post failed"
fi

step "API UPDATE"
if provider_is github; then
  expect_exit_0 "api patch succeeds" gitfleet api patch --endpoint "$LABEL_ENDPOINT" --body '{"description":"Updated by the Gitfleet API playbook."}' --yes
else
  expect_exit_0 "api put succeeds" gitfleet api put --endpoint "$LABEL_ENDPOINT" --body '{"description":"Updated by the Gitfleet API playbook."}' --yes
fi

step "API DELETE"
expect_exit_0 "api delete succeeds" gitfleet api delete --endpoint "$LABEL_ENDPOINT" --yes
LABEL_CREATED=false

step "API GET with --json"
expect_exit_0 "api get --json succeeds" gitfleet --json api get --endpoint "$GITFLEET_PLAYBOOK_API_REPO_ENDPOINT"

step "API POST invalid JSON rejected"
expect_exit_non0 "api post with invalid json fails" gitfleet api post --endpoint "$GITFLEET_PLAYBOOK_API_REPO_ENDPOINT" --body 'not-json'
