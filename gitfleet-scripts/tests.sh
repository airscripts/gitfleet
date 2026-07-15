#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
CRATES=(gitfleet-core gitfleet-cli gitfleet-providers)
SELECTED_CRATE=""
TEST_KIND="all"

usage() {
    printf 'Usage: %s [-a|--all] [-c|--crate CRATE]\n' "$(basename "$0")"
    printf '\nCount unique Rust unit and integration test functions in the workspace.\n'
    printf '  -a, --all          count workspace tests (default)\n'
    printf '  -c, --crate        count one crate: core, cli, or providers\n'
    printf '      --unit         count unit tests only\n'
    printf '      --integration  count integration tests only\n'
    printf '  -h, --help   show this help\n'
}

while (($# > 0)); do
    case "$1" in
        -a|--all)
            if [[ -n "$SELECTED_CRATE" ]]; then
                printf 'Error: --all and --crate cannot be combined.\n' >&2
                exit 2
            fi
            ;;
        -c|--crate)
            if (($# < 2)); then
                printf 'Error: --crate requires core, cli, or providers.\n' >&2
                exit 2
            fi
            case "$2" in
                core|gitfleet-core) SELECTED_CRATE=gitfleet-core ;;
                cli|gitfleet-cli) SELECTED_CRATE=gitfleet-cli ;;
                provider|providers|gitfleet-providers) SELECTED_CRATE=gitfleet-providers ;;
                *)
                    printf 'Error: unknown crate: %s\n' "$2" >&2
                    exit 2
                    ;;
            esac
            shift
            ;;
        --unit)
            if [[ "$TEST_KIND" != "all" ]]; then
                printf 'Error: --unit and --integration cannot be combined.\n' >&2
                exit 2
            fi
            TEST_KIND=unit
            ;;
        --integration)
            if [[ "$TEST_KIND" != "all" ]]; then
                printf 'Error: --unit and --integration cannot be combined.\n' >&2
                exit 2
            fi
            TEST_KIND=integration
            ;;
        --crate=*)
            crate_arg="${1#*=}"
            case "$crate_arg" in
                core|gitfleet-core) SELECTED_CRATE=gitfleet-core ;;
                cli|gitfleet-cli) SELECTED_CRATE=gitfleet-cli ;;
                provider|providers|gitfleet-providers) SELECTED_CRATE=gitfleet-providers ;;
                *)
                    printf 'Error: unknown crate: %s\n' "$crate_arg" >&2
                    exit 2
                    ;;
            esac
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            printf 'Error: unknown option: %s\n' "$1" >&2
            usage >&2
            exit 2
            ;;
    esac
    shift
done

count_tests() {
    local path="$1"

    if [[ ! -d "$path" ]]; then
        printf '0\n'
        return
    fi

    find "$path" -type f -name '*.rs' -print0 |
        xargs -0 awk '
            /^[[:space:]]*#\[test\][[:space:]]*$/ { count++ }
            /^[[:space:]]*#\[[[:alnum:]_]+::test\][[:space:]]*$/ { count++ }
            END { print count + 0 }
        '
}

unit_total=0
integration_total=0

if [[ -n "$SELECTED_CRATE" ]]; then
    CRATES=("$SELECTED_CRATE")
fi

for crate in "${CRATES[@]}"; do
    unit=$(count_tests "$ROOT_DIR/$crate/src")
    integration=$(count_tests "$ROOT_DIR/$crate/tests")
    total=$((unit + integration))
    unit_total=$((unit_total + unit))
    integration_total=$((integration_total + integration))

done

case "$TEST_KIND" in
    unit)
        printf '%d\n' "$unit_total"
        ;;
    integration)
        printf '%d\n' "$integration_total"
        ;;
    all)
        printf '%d\n' "$((unit_total + integration_total))"
        ;;
esac
