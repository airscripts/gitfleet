#!/usr/bin/env bash
set -euo pipefail
source "$(dirname "$0")/env.sh"

setup() { :; }
teardown() { print_summary; }
trap teardown EXIT
setup

step "License List"
expect_exit_0 "license list succeeds" gitfleet license list

step "License View"
expect_exit_0 "license view succeeds" gitfleet license view mit