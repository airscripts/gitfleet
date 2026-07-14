#!/usr/bin/env bash
# env.sh — Centralized configuration and helpers for gitfleet playbooks.
#
# Override defaults with environment variables:
#   GITFLEET_PLAYBOOK_REPO=owner/repo  — Disposable repository for repo-scoped commands (required)
#   GITFLEET_PLAYBOOK_ORG=orgname      — Organization for org-scoped commands (derived from GITFLEET_PLAYBOOK_REPO)
#   GITFLEET_PLAYBOOK_TEST_REPO_OWNER=owner — Account that owns disposable test repositories
#   GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE=org — Disposable repository owner type: org or user
#   GITFLEET_PLAYBOOK_TMPDIR=/path     — Scratch directory (default: /tmp/gitfleet-playbooks)
#   GITFLEET_PLAYBOOK_WEBHOOK_URL — Owner-controlled receiver for live webhook delivery tests
#
# Every playbook sources this file. Change pointings here or via env vars.
set -euo pipefail

if [ -z "${GITFLEET_PLAYBOOK_REPO:-}" ]; then
  echo "[ERROR] GITFLEET_PLAYBOOK_REPO is not set. Use a disposable test repository."
  echo "        GITFLEET_PLAYBOOK_REPO=owner/gitfleet-test bash gitfleet-playbooks/all.sh"
  exit 1
fi

export GITFLEET_PLAYBOOK_REPO
export GITFLEET_PLAYBOOK_ORG="${GITFLEET_PLAYBOOK_ORG:-${GITFLEET_PLAYBOOK_REPO%%/*}}"
export GITFLEET_PLAYBOOK_TEST_REPO_OWNER="${GITFLEET_PLAYBOOK_TEST_REPO_OWNER:-$GITFLEET_PLAYBOOK_ORG}"
export GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE="${GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE:-org}"
export GITFLEET_PLAYBOOK_TMPDIR="${GITFLEET_PLAYBOOK_TMPDIR:-/tmp/gitfleet-playbooks}"

if [ "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" != "org" ] && [ "$GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE" != "user" ]; then
  echo "[ERROR] GITFLEET_PLAYBOOK_TEST_REPO_OWNER_TYPE must be 'org' or 'user'."
  exit 1
fi
mkdir -p "$GITFLEET_PLAYBOOK_TMPDIR"

export GITFLEET_PLAYBOOK_RUN_ID="${GITFLEET_PLAYBOOK_RUN_ID:-$(date +%s)-$$}"
export GITFLEET_PLAYBOOK_RESOURCE_SUFFIX="${GITFLEET_PLAYBOOK_RUN_ID//[^a-zA-Z0-9]/-}"

export GITFLEET_PLAYBOOK_OWNER="${GITFLEET_PLAYBOOK_REPO%%/*}"
export GITFLEET_PLAYBOOK_REPO_NAME="${GITFLEET_PLAYBOOK_REPO#*/}"

PROVIDER_STATUS=$(gitfleet auth status --capabilities --json 2>/dev/null || true)
GITFLEET_PLAYBOOK_PROVIDER=$(printf '%s' "$PROVIDER_STATUS" | python3 -c \
  'import json,sys; print(json.load(sys.stdin).get("provider", ""))' 2>/dev/null || true)
GITFLEET_PLAYBOOK_CAPABILITIES=$(printf '%s' "$PROVIDER_STATUS" | python3 -c \
  'import json,sys; print(",".join(json.load(sys.stdin).get("capabilities", [])))' 2>/dev/null || true)

case "$GITFLEET_PLAYBOOK_PROVIDER" in
  github)
    if [ -z "${GITFLEET_GITHUB_TOKEN:-}" ]; then
      GITFLEET_GITHUB_TOKEN=$(gitfleet auth token --raw 2>/dev/null || true)
      export GITFLEET_GITHUB_TOKEN
    fi
    GITFLEET_PLAYBOOK_TOKEN="${GITFLEET_GITHUB_TOKEN:-}"
    ;;
  gitlab)
    if [ -z "${GITFLEET_GITLAB_TOKEN:-}" ]; then
      GITFLEET_GITLAB_TOKEN=$(gitfleet auth token --raw 2>/dev/null || true)
      export GITFLEET_GITLAB_TOKEN
    fi
    GITFLEET_PLAYBOOK_TOKEN="${GITFLEET_GITLAB_TOKEN:-}"
    ;;
  *)
    echo "[ERROR] Could not determine the active Gitfleet provider."
    echo "        Select a GitHub or GitLab profile before running playbooks."
    exit 1
    ;;
esac

if [ -z "$GITFLEET_PLAYBOOK_TOKEN" ]; then
  echo "[ERROR] No token is available for the active $GITFLEET_PLAYBOOK_PROVIDER profile."
  echo "        Export the matching GITFLEET_GITHUB_TOKEN or GITFLEET_GITLAB_TOKEN."
  exit 1
fi

export GITFLEET_PLAYBOOK_PROVIDER GITFLEET_PLAYBOOK_CAPABILITIES GITFLEET_PLAYBOOK_TOKEN

GITFLEET_PLAYBOOK_ENCODED_REPO=$(python3 -c 'import sys,urllib.parse; print(urllib.parse.quote(sys.argv[1], safe=""))' "$GITFLEET_PLAYBOOK_REPO")

if [ "$GITFLEET_PLAYBOOK_PROVIDER" = "github" ]; then
  export GITFLEET_PLAYBOOK_API_REPO_ENDPOINT="/repos/$GITFLEET_PLAYBOOK_REPO"
  export GITFLEET_PLAYBOOK_API_LABELS_ENDPOINT="/repos/$GITFLEET_PLAYBOOK_REPO/labels"
else
  export GITFLEET_PLAYBOOK_API_REPO_ENDPOINT="/projects/$GITFLEET_PLAYBOOK_ENCODED_REPO"
  export GITFLEET_PLAYBOOK_API_LABELS_ENDPOINT="/projects/$GITFLEET_PLAYBOOK_ENCODED_REPO/labels"
fi

PB_PASS=0
PB_FAIL=0
PB_SKIP=0
PB_STEP=0

step() {
  PB_STEP=$((PB_STEP + 1))
  echo ""
  echo "[INFO] Step ${PB_STEP}: $1"
}

pass() {
  PB_PASS=$((PB_PASS + 1))
  echo "[OK] $1"
}

fail() {
  PB_FAIL=$((PB_FAIL + 1))
  echo "[ERROR] $1"
}

skip() {
  PB_SKIP=$((PB_SKIP + 1))
  echo "[WARN] $1 (skipped)"
}

expect_exit_0() {
  local label="$1"; shift
  if "$@" >/dev/null 2>&1; then
    pass "$label"
  else
    fail "$label (exited non-zero)"
  fi
}

expect_exit_non0() {
  local label="$1"; shift
  if "$@" >/dev/null 2>&1; then
    fail "$label (expected non-zero exit, got 0)"
  else
    pass "$label"
  fi
}

provider_is() {
  [ "$GITFLEET_PLAYBOOK_PROVIDER" = "$1" ]
}

has_capability() {
  case ",$GITFLEET_PLAYBOOK_CAPABILITIES," in
    *",$1,"*) return 0 ;;
    *) return 1 ;;
  esac
}

expect_capability_or_unsupported() {
  local label="$1"
  local capability="$2"
  shift 2

  if has_capability "$capability"; then
    expect_exit_0 "$label succeeds" "$@"
  else
    expect_exit_non0 "$label is explicitly unsupported" "$@"
  fi
}

expect_rejects_missing_arg() {
  local label="$1"; shift
  local output
  output=$("$@" 2>&1) || true

  if echo "$output" | grep -qi "cancelled\|required\|Error\|must provide\|is required"; then
    pass "$label"
  else
    fail "$label (command did not reject missing argument)"
  fi
}

expect_output() {
  local label="$1"
  local needle="$2"
  shift 2
  local haystack
  haystack=$("$@" 2>&1) || true

  if echo "$haystack" | grep -qi "$needle"; then
    pass "$label"
  else
    fail "$label (output missing '$needle')"
    echo "  actual: $(echo "$haystack" | head -3)"
  fi
}

expect_json_field() {
  local label="$1"
  local field="$2"
  local value="$3"
  shift 3
  local json
  json=$("$@" --json 2>&1) || true

  if echo "$json" | python3 -c "
import sys, json
d = json.load(sys.stdin)
v = d.get('$field')
if v is None:
  sys.exit(1)
target = json.loads('$value') if '$value' in ('true','false','null') else '$value'
sys.exit(0 if v == target else 1)
" 2>/dev/null; then
    pass "$label"
  elif echo "$json" | grep -qi "\"$field\".*$value"; then
    pass "$label"
  else
    fail "$label (json missing $field=$value)"
    echo "  actual: $(echo "$json" | head -3)"
  fi
}

print_summary() {
  echo ""
  echo "[INFO] Summary"
  printf "  Passed: %d  |  Failed: %d  |  Skipped: %d  |  Steps: %d\n" \
    "$PB_PASS" "$PB_FAIL" "$PB_SKIP" "$PB_STEP"
  echo ""

  if [ "$PB_FAIL" -eq 0 ]; then
    echo "[OK] All checks passed."
  else
    echo "[ERROR] Some checks failed."
    exit 1
  fi
}

# Playbooks should define a teardown() function and register:
#   trap teardown EXIT
# This file does NOT set a trap — each playbook owns its own.
