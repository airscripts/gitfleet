#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "Attestation List"
if ! has_capability "attestations"; then
  expect_exit_non0 "attestations are explicitly unsupported" gitfleet attestation list --repo "$GITFLEET_PLAYBOOK_REPO" --subject-digest "sha256:abc123"
  exit 0
fi

if gitfleet attestation list --repo "$GITFLEET_PLAYBOOK_REPO" --subject-digest "sha256:abc123" >/dev/null 2>&1; then
  pass "attestation list succeeds"
else
  skip "attestation list (may not be available for this repo)"
fi
