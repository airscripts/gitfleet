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
  completion
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
  security
  attestation
  code
  template
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

record_playbook_result() {
  local name="$1"
  local output_file="$2"
  local exit_code="$3"
  local output
  output=$(<"$output_file")

  echo "$output"

  local p=0 f=0 s=0 summary
  summary=$(echo "$output" | grep '^  Passed:' | tail -1 || true)

  if [[ "$summary" =~ Passed:\ ([0-9]+).*Failed:\ ([0-9]+).*Skipped:\ ([0-9]+) ]]; then
    p="${BASH_REMATCH[1]}"
    f="${BASH_REMATCH[2]}"
    s="${BASH_REMATCH[3]}"
  else
    f=$(echo "$output" | grep -c '^\[ERROR\]' || true)
  fi

  TOTAL_PASS=$((TOTAL_PASS + p))
  TOTAL_FAIL=$((TOTAL_FAIL + f))
  TOTAL_SKIP=$((TOTAL_SKIP + s))

  if [ "$exit_code" -eq 0 ] && [ "$f" -eq 0 ]; then
    RESULTS+=("$name: PASSED (pass:$p fail:$f skip:$s)")
  elif [ "$exit_code" -ne 0 ]; then
    RESULTS+=("$name: ERRORED (exit $exit_code)")
    if [ "$f" -eq 0 ]; then
      TOTAL_FAIL=$((TOTAL_FAIL + 1))
    fi
  else
    RESULTS+=("$name: FAILED (pass:$p fail:$f skip:$s)")
  fi
}

run_playbook() {
  local name="$1"
  local playbook="$SCRIPT_DIR/${name}.sh"

  if [ ! -f "$playbook" ]; then
    echo "[ERROR] Playbook not found: $playbook"
    RESULTS+=("$name: MISSING")
    TOTAL_FAIL=$((TOTAL_FAIL + 1))
    return
  fi

  echo ""
  echo "[INFO] Running playbook: $name"

  local output_file
  output_file=$(mktemp "$TMPDIR/gitfleet-playbook-${name}.XXXXXX")
  local exit_code=0
  bash "$playbook" >"$output_file" 2>&1 || exit_code=$?

  record_playbook_result "$name" "$output_file" "$exit_code"
  rm -f "$output_file"
}

echo "[INFO] gitfleet playbook pipeline"
echo "[INFO] REPO=$REPO  ORG=$ORG  TMPDIR=$TMPDIR"
echo ""

if [ "$PARALLEL" -eq 1 ]; then
  echo "[WARN] Parallel mode: running playbooks concurrently."
  echo "[WARN] Teardown order is not guaranteed in parallel mode."
  echo ""

  PARALLEL_DIR=$(mktemp -d "$TMPDIR/gitfleet-all.XXXXXX")
  PARALLEL_NAMES=()
  PARALLEL_PIDS=()
  PARALLEL_OUTPUTS=()

  for playbook in "${PLAYBOOKS[@]}"; do
    if should_skip "$playbook"; then
      RESULTS+=("$playbook: SKIPPED (in SKIP list)")
      TOTAL_SKIP=$((TOTAL_SKIP + 1))
      continue
    fi
    if [ ! -f "$SCRIPT_DIR/$playbook.sh" ]; then
      echo "[ERROR] Playbook not found: $SCRIPT_DIR/$playbook.sh"
      RESULTS+=("$playbook: MISSING")
      TOTAL_FAIL=$((TOTAL_FAIL + 1))
      continue
    fi
    output_file="$PARALLEL_DIR/$playbook.log"
    bash "$SCRIPT_DIR/$playbook.sh" >"$output_file" 2>&1 &
    PARALLEL_NAMES+=("$playbook")
    PARALLEL_PIDS+=("$!")
    PARALLEL_OUTPUTS+=("$output_file")
  done

  for index in "${!PARALLEL_PIDS[@]}"; do
    exit_code=0
    wait "${PARALLEL_PIDS[$index]}" || exit_code=$?
    record_playbook_result \
      "${PARALLEL_NAMES[$index]}" \
      "${PARALLEL_OUTPUTS[$index]}" \
      "$exit_code"
  done

  rm -rf "$PARALLEL_DIR"
else
  for playbook in "${PLAYBOOKS[@]}"; do
    if should_skip "$playbook"; then
      RESULTS+=("$playbook: SKIPPED (in SKIP list)")
      TOTAL_SKIP=$((TOTAL_SKIP + 1))
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
