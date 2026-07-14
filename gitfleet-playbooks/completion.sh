#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

MAN_DIR="$GITFLEET_PLAYBOOK_TMPDIR/gitfleet-man-$GITFLEET_PLAYBOOK_RESOURCE_SUFFIX"

setup() { :; }

teardown() {
  rm -rf "$MAN_DIR"
  print_summary
}

trap teardown EXIT
setup

step "Generate Bash Completion"
expect_exit_0 "bash completion generation succeeds" gitfleet completion generate bash

step "Generate PowerShell Completion"
expect_exit_0 "PowerShell completion generation succeeds" gitfleet completion generate powershell

step "Generate Man Page"
expect_exit_0 "man page generation succeeds" gitfleet completion mangen "$MAN_DIR"

step "Reject Unsupported Shell"
expect_exit_non0 "unsupported completion shell fails" gitfleet completion generate unsupported
