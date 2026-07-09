#!/usr/bin/env bash
# all.sh — Orchestrator that runs every gitfleet playbook in sequence.
#
# Usage:
#   bash gitfleet-playbooks/all.sh                              # run all playbooks sequentially
#   PARALLEL=1 bash gitfleet-playbooks/all.sh                   # run playbooks concurrently
#   SKIP="pipeline.sh,dependency.sh" bash gitfleet-playbooks/all.sh # skip playbooks
#   REPO=owner/repo ORG=orgname bash gitfleet-playbooks/all.sh  # override pointings
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/env.sh"

SKIP="${SKIP:-}"
PARALLEL="${PARALLEL:-0}"

PLAYBOOKS=(
  config
  auth
  alias
  api
  version
  search
  inbox
  policy
  label
  license
  wiki
  webhook
  environment
  variable
  secret
  discussion
  deploy
  issue
  govern
  repo
  dependency
  advisory
  attestation
  code
  analytics
  access
  identity
  browse
  runner
  release
  change
  pipeline
  workspace
  milestone
  project
  reaction
  comment
  snippet
  package
  dev
  site
)

TOTAL_PASS=0
TOTAL_FAIL=0
TOTAL_SKIP=0
RESULTS=()

should_skip() {
  local name="$1"
  local item

  for item in ${SKIP//,/ }; do
    item="${item%.sh}"
    if [ "$item" = "$name" ]; then
      return 0
    fi
  done

  return 1
}

run_playbook() {
  local name="$1"
  local playbook="$SCRIPT_DIR/${name}.sh"

  if [ ! -f "$playbook" ]; then
    echo "[ERROR] Playbook not found: $playbook"
    RESULTS+=("$name: MISSING")
    TOTAL_SKIP=$((TOTAL_SKIP + 1))
    return
  fi

  echo ""
  echo "[INFO] Running playbook: $name"

  local output
  local exit_code=0
  output=$(bash "$playbook" 2>&1) || exit_code=$?

  echo "$output"

  local p f s
  p=$(echo "$output" | grep -c '^\[OK\]' || true)
  f=$(echo "$output" | grep -c '^\[ERROR\]' || true)
  s=$(echo "$output" | grep -c '^\[WARN\].*(skipped)' || true)

  TOTAL_PASS=$((TOTAL_PASS + p))
  TOTAL_FAIL=$((TOTAL_FAIL + f))
  TOTAL_SKIP=$((TOTAL_SKIP + s))

  if [ "$exit_code" -eq 0 ] && [ "$f" -eq 0 ]; then
    RESULTS+=("$name: PASSED (pass:$p fail:$f skip:$s)")
  elif [ "$exit_code" -ne 0 ]; then
    RESULTS+=("$name: ERRORED (exit $exit_code)")
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
  else
    RESULTS+=("$name: FAILED (pass:$p fail:$f skip:$s)")
  fi
}

echo "[INFO] gitfleet playbook pipeline"
echo "[INFO] REPO=$REPO  ORG=$ORG  TMPDIR=$TMPDIR"
echo ""

if [ "$PARALLEL" -eq 1 ]; then
  echo "[WARN] Parallel mode: running playbooks concurrently."
  echo "[WARN] Teardown order is not guaranteed in parallel mode."
  echo ""

  for playbook in "${PLAYBOOKS[@]}"; do
    if should_skip "$playbook"; then
      RESULTS+=("$playbook: SKIPPED (in SKIP list)")
      continue
    fi
    run_playbook "$playbook" &
  done
  wait
else
  for playbook in "${PLAYBOOKS[@]}"; do
    if should_skip "$playbook"; then
      RESULTS+=("$playbook: SKIPPED (in SKIP list)")
      continue
    fi
    run_playbook "$playbook"
  done
fi

echo ""
echo "[INFO] Final Summary"
printf "  Passed: %d  |  Failed: %d  |  Skipped: %d\n" \
  "$TOTAL_PASS" "$TOTAL_FAIL" "$TOTAL_SKIP"
echo ""
for result in "${RESULTS[@]}"; do
  echo "  $result"
done
echo ""

if [ "$TOTAL_FAIL" -eq 0 ]; then
  echo "[OK] All playbooks passed."
  exit 0
else
  echo "[ERROR] Some playbooks failed."
  exit 1
fi