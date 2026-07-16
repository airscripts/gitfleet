#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 2 ]; then
    echo "usage: $0 <tag> <output-file>" >&2
    exit 2
fi

tag="$1"
output_file="$2"

case "$tag" in
    v[0-9]*.[0-9]*.[0-9]*)
        version="${tag#v}"
        base_version="${version%%-*}"
        ;;
    *)
        echo "release tag must use vX.Y.Z or vX.Y.Z-prerelease format: $tag" >&2
        exit 1
        ;;
esac

declared_version="$(tr -d '[:space:]' < VERSION)"

if [ "$base_version" != "$declared_version" ]; then
    echo "release tag $tag does not match VERSION $declared_version" >&2
    exit 1
fi

if [ "$version" != "$base_version" ]; then
    cat > "$output_file" <<EOF
Release candidate for \`$base_version\`.

This prerelease is intended to validate release packaging and platform artifacts.
See the final \`v$base_version\` release for changelog notes.
EOF
    exit 0
fi

awk -v version="$base_version" '
    $0 ~ "^## \\[" version "\\]" {
        found = 1
        next
    }

    found && /^## \[/ {
        exit
    }

    found {
        print
    }

    END {
        if (!found) {
            exit 1
        }
    }
' CHANGELOG.md > "$output_file"

if [ ! -s "$output_file" ]; then
    echo "CHANGELOG.md section for $version is empty or missing" >&2
    exit 1
fi
