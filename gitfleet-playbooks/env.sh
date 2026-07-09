#!/usr/bin/env bash
# env.sh — Centralized configuration and helpers for gitfleet playbooks.
#
# Override defaults with environment variables:
#   REPO=owner/repo  — Repository for repo-scoped commands (default: clawdeeo/gitfleet-test)
#   ORG=orgname      — Organization for org-scoped commands (default: clawdeeo)
#   TMPDIR=/path     — Scratch directory (default: /tmp/gitfleet-playbooks)
#
# Every playbook sources this file. Change pointings here or via env vars.
set -euo pipefail

export REPO="${REPO:-clawdeeorg/gitfleet}"
export ORG="${ORG:-clawdeeorg}"
export TMPDIR="${TMPDIR:-/tmp/gitfleet-playbooks}"
mkdir -p "$TMPDIR"

export OWNER="${REPO%%/*}"
export REPO_NAME="${REPO#*/}"

if [ -z "${GITFLEET_GITHUB_TOKEN:-}" ]; then
  GITFLEET_GITHUB_TOKEN=$(gitfleet auth token --raw 2>/dev/null || true)
  if [ -n "$GITFLEET_GITHUB_TOKEN" ]; then
    export GITFLEET_GITHUB_TOKEN
  else
    echo "[ERROR] GITFLEET_GITHUB_TOKEN is not set. Export your GitHub token before running playbooks."
    echo "        export GITFLEET_GITHUB_TOKEN=ghp_..."
    exit 1
  fi
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
  fi
}

# Playbooks should define a teardown() function and register:
#   trap teardown EXIT
# This file does NOT set a trap — each playbook owns its own.