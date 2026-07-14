#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_SUFFIX="$PB_RESOURCE_SUFFIX"
TEST_REPO_NAME="gitfleet-test-change-$PB_RESOURCE_SUFFIX"
TEST_REPO="$ORG/$TEST_REPO_NAME"
REPO_CREATED=false
BASE_BRANCH="gitfleet-test-pr-base-$TEST_SUFFIX"
HEAD_BRANCH="gitfleet-test-pr-head-$TEST_SUFFIX"
TEST_PR_NUMBER=""

setup() {
  local default_branch base_sha content
  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$ORG" --owner-type org --private --yes >/dev/null 2>&1; then
    fail "change test repository creation failed"
    return
  fi
  REPO_CREATED=true

  content=$(printf 'Gitfleet change playbook\n' | base64 | tr -d '\n')

  if provider_is github; then
    gitfleet api post --endpoint "/repos/$TEST_REPO/contents/README.md" --body "{\"message\":\"test: initialize repository\",\"content\":\"$content\"}" --json >/dev/null 2>&1 || true

    default_branch=$(gitfleet api get --endpoint "/repos/$TEST_REPO" --json 2>&1 | python3 -c "import sys,json; print(json.load(sys.stdin).get('default_branch','main'))" 2>/dev/null || echo "main")
    base_sha=$(gitfleet api get --endpoint "/repos/$TEST_REPO/git/ref/heads/$default_branch" --json 2>&1 | python3 -c "import sys,json; print(json.load(sys.stdin).get('object',{}).get('sha',''))" 2>/dev/null || echo "")

    if [ -z "$base_sha" ]; then
      fail "could not resolve the GitHub default branch SHA"
      return
    fi

    gitfleet api post --endpoint "/repos/$TEST_REPO/git/refs" --body "{\"ref\":\"refs/heads/$BASE_BRANCH\",\"sha\":\"$base_sha\"}" --json >/dev/null 2>&1 || true
    gitfleet api post --endpoint "/repos/$TEST_REPO/git/refs" --body "{\"ref\":\"refs/heads/$HEAD_BRANCH\",\"sha\":\"$base_sha\"}" --json >/dev/null 2>&1 || true

    content=$(printf 'PR playbook %s\n' "$TEST_SUFFIX" | base64 | tr -d '\n')
    gitfleet api post --endpoint "/repos/$TEST_REPO/contents/gitfleet-test-pr-$TEST_SUFFIX.txt" --body "{\"message\":\"test: add PR playbook fixture\",\"content\":\"$content\",\"branch\":\"$HEAD_BRANCH\"}" --json >/dev/null 2>&1 || true
  else
    local encoded_test_repo
    encoded_test_repo=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$TEST_REPO")
    default_branch="main"

    gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/files/README.md" --body "{\"branch\":\"$default_branch\",\"content\":\"$content\",\"encoding\":\"base64\",\"commit_message\":\"test: initialize repository\"}" --json >/dev/null 2>&1 || true
    gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/branches" --body "{\"branch\":\"$BASE_BRANCH\",\"ref\":\"$default_branch\"}" --json >/dev/null 2>&1 || true
    gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/branches" --body "{\"branch\":\"$HEAD_BRANCH\",\"ref\":\"$default_branch\"}" --json >/dev/null 2>&1 || true

    content=$(printf 'PR playbook %s\n' "$TEST_SUFFIX" | base64 | tr -d '\n')
    gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/files/gitfleet-test-pr-$TEST_SUFFIX.txt" --body "{\"branch\":\"$HEAD_BRANCH\",\"content\":\"$content\",\"encoding\":\"base64\",\"commit_message\":\"test: add change playbook fixture\"}" --json >/dev/null 2>&1 || true
  fi

  local result
  result=$(gitfleet change create --repo "$TEST_REPO" --title "[noop] gitfleet PR lifecycle test" --body "Created by the PR playbook." --base "$BASE_BRANCH" --head "$HEAD_BRANCH" --draft --json 2>&1) || true
  TEST_PR_NUMBER=$(echo "$result" | python3 -c "import sys,json; print(json.load(sys.stdin).get('number',''))" 2>/dev/null || echo "")
}

teardown() {
  if [ "$REPO_CREATED" = true ]; then
    gitfleet repo delete "$TEST_REPO" --yes >/dev/null 2>&1 || true
  fi
  print_summary
}
trap teardown EXIT
setup

step "PR Create"
if [ -n "$TEST_PR_NUMBER" ]; then pass "change create succeeds"; else fail "change create failed"; fi

step "PR List"
expect_exit_0 "pr list succeeds" gitfleet change list --repo "$TEST_REPO" --limit 10

if [ -n "$TEST_PR_NUMBER" ]; then
  step "PR View"
  expect_exit_0 "pr view succeeds" gitfleet change view "$TEST_PR_NUMBER" --repo "$TEST_REPO"
else
  skip "pr view (no test PR)"
fi
