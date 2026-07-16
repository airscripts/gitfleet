#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

TEST_SUFFIX="$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO_NAME="gitfleet-test-change-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"
TEST_REPO="$GITFLEET_PLAYBOOK_TEST_REPO_OWNER/$TEST_REPO_NAME"
REPO_CREATED=false
BASE_BRANCH="gitfleet-test-pr-base-$TEST_SUFFIX"
HEAD_BRANCH="gitfleet-test-pr-head-$TEST_SUFFIX"
TEST_PR_NUMBER=""

setup() {
  local default_branch base_sha content
  if ! gitfleet repo create "$TEST_REPO_NAME" --owner "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER" --owner-type "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" --private --initialize --yes >/dev/null 2>&1; then
    fail "change test repository creation failed"
    return
  fi
  REPO_CREATED=true

  if provider_is github; then
    default_branch=$(gitfleet api get --endpoint "/repos/$TEST_REPO" --json 2>&1 | python3 -c "import sys,json; print(json.load(sys.stdin).get('default_branch','main'))" 2>/dev/null || echo "main")
    base_sha=$(gitfleet api get --endpoint "/repos/$TEST_REPO/git/ref/heads/$default_branch" --json 2>&1 | python3 -c "import sys,json; print(json.load(sys.stdin).get('object',{}).get('sha',''))" 2>/dev/null || echo "")

    if [ -z "$base_sha" ]; then
      fail "could not resolve the GitHub default branch SHA"
      return
    fi

    if ! gitfleet api post --endpoint "/repos/$TEST_REPO/git/refs" --body "{\"ref\":\"refs/heads/$BASE_BRANCH\",\"sha\":\"$base_sha\"}" --json --yes >/dev/null 2>&1; then
      fail "GitHub base branch creation failed"
      return
    fi
    if ! gitfleet api post --endpoint "/repos/$TEST_REPO/git/refs" --body "{\"ref\":\"refs/heads/$HEAD_BRANCH\",\"sha\":\"$base_sha\"}" --json --yes >/dev/null 2>&1; then
      fail "GitHub head branch creation failed"
      return
    fi

    content=$(printf 'PR playbook %s\n' "$TEST_SUFFIX" | base64 | tr -d '\n')
    if ! gitfleet api put --endpoint "/repos/$TEST_REPO/contents/gitfleet-test-pr-$TEST_SUFFIX.txt" --body "{\"message\":\"test: add PR playbook fixture\",\"content\":\"$content\",\"branch\":\"$HEAD_BRANCH\"}" --json --yes >/dev/null 2>&1; then
      fail "GitHub change commit creation failed"
      return
    fi
  else
    local encoded_test_repo
    encoded_test_repo=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$TEST_REPO")
    default_branch="main"
    if ! gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/branches" --body "{\"branch\":\"$BASE_BRANCH\",\"ref\":\"$default_branch\"}" --json --yes >/dev/null 2>&1; then
      fail "GitLab base branch creation failed"
      return
    fi
    if ! gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/branches" --body "{\"branch\":\"$HEAD_BRANCH\",\"ref\":\"$default_branch\"}" --json --yes >/dev/null 2>&1; then
      fail "GitLab head branch creation failed"
      return
    fi

    content=$(printf 'PR playbook %s\n' "$TEST_SUFFIX" | base64 | tr -d '\n')
    if ! gitfleet api post --endpoint "/projects/$encoded_test_repo/repository/files/gitfleet-test-pr-$TEST_SUFFIX.txt" --body "{\"branch\":\"$HEAD_BRANCH\",\"content\":\"$content\",\"encoding\":\"base64\",\"commit_message\":\"test: add change playbook fixture\"}" --json --yes >/dev/null 2>&1; then
      fail "GitLab change commit creation failed"
      return
    fi
  fi

  local result
  result=$(gitfleet change create --repo "$TEST_REPO" "[noop] gitfleet PR lifecycle test" --body "Created by the PR playbook." --base "$BASE_BRANCH" --head "$HEAD_BRANCH" --draft --json 2>&1) || true
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

  step "PR Merge"
  expect_exit_0 "pr merge succeeds" gitfleet change merge "$TEST_PR_NUMBER" --repo "$TEST_REPO" --method merge --yes
else
  skip "pr view (no test PR)"
  skip "pr merge (no test PR)"
fi
