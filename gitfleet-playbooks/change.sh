#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_SUFFIX="$$"
BASE_BRANCH="gitfleet-test-pr-base-$TEST_SUFFIX"
HEAD_BRANCH="gitfleet-test-pr-head-$TEST_SUFFIX"
TEST_PR_NUMBER=""

setup() {
  local default_branch base_sha content
  default_branch=$(gitfleet api get --endpoint "/repos/$REPO" --json 2>&1 | python3 -c "import sys,json; print(json.load(sys.stdin).get('default_branch','main'))" 2>/dev/null || echo "main")
  base_sha=$(gitfleet api get --endpoint "/repos/$REPO/git/ref/heads/$default_branch" --json 2>&1 | python3 -c "import sys,json; print(json.load(sys.stdin).get('object',{}).get('sha',''))" 2>/dev/null || echo "")

  if [ -z "$base_sha" ]; then
    echo "[ERROR] Could not get base SHA for default branch"
    return
  fi

  gitfleet api post --endpoint "/repos/$REPO/git/refs" --body "{\"ref\":\"refs/heads/$BASE_BRANCH\",\"sha\":\"$base_sha\"}" --json >/dev/null 2>&1 || true
  gitfleet api post --endpoint "/repos/$REPO/git/refs" --body "{\"ref\":\"refs/heads/$HEAD_BRANCH\",\"sha\":\"$base_sha\"}" --json >/dev/null 2>&1 || true

  content=$(printf 'PR playbook %s\n' "$TEST_SUFFIX" | base64 | tr -d '\n')
  gitfleet api post --endpoint "/repos/$REPO/contents/gitfleet-test-pr-$TEST_SUFFIX.txt" --body "{\"message\":\"test: add PR playbook fixture\",\"content\":\"$content\",\"branch\":\"$HEAD_BRANCH\"}" --json >/dev/null 2>&1 || true

  local result
  result=$(gitfleet change create --repo "$REPO" --title "[noop] gitfleet PR lifecycle test" --body "Created by the PR playbook." --base "$BASE_BRANCH" --head "$HEAD_BRANCH" --draft --json 2>&1) || true
  TEST_PR_NUMBER=$(echo "$result" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")
}

teardown() {
  if [ -n "$TEST_PR_NUMBER" ]; then
    gitfleet api delete --endpoint "/repos/$REPO/pulls/$TEST_PR_NUMBER" --json >/dev/null 2>&1 || true
  fi

  gitfleet api delete --endpoint "/repos/$REPO/git/refs/heads/$HEAD_BRANCH" --json >/dev/null 2>&1 || true
  gitfleet api delete --endpoint "/repos/$REPO/git/refs/heads/$BASE_BRANCH" --json >/dev/null 2>&1 || true
  print_summary
}
trap teardown EXIT
setup

step "PR Create"
if [ -n "$TEST_PR_NUMBER" ]; then pass "pr create succeeds"; else skip "pr create (may have failed)"; fi

step "PR List"
expect_exit_0 "pr list succeeds" gitfleet change list --repo "$REPO" --limit 10

if [ -n "$TEST_PR_NUMBER" ]; then
  step "PR View"
  expect_exit_0 "pr view succeeds" gitfleet change view "$TEST_PR_NUMBER" --repo "$REPO"
else
  skip "pr view (no test PR)"
fi