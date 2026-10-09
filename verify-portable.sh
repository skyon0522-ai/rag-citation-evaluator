#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
work=$(mktemp -d "${TMPDIR:-/tmp}/rag-citation-verify.XXXXXX")
trap 'rm -rf "$work"' EXIT

: "${CARGO_HOME:=$work/cargo-home}"
: "${CARGO_TARGET_DIR:=$work/target}"
export CARGO_HOME CARGO_TARGET_DIR
mkdir -p "$CARGO_HOME" "$CARGO_TARGET_DIR"

if [ "$#" -gt 1 ]; then
    echo "usage: $0 [fixture-output-directory]" >&2
    exit 2
fi
if [ "$#" -eq 1 ]; then
    results=$1
else
    results=$work/fixture-results
fi

cargo build --locked --manifest-path "$root/cli/Cargo.toml"
python3 -B "$root/cli/tests/check_fixtures.py" \
    "$CARGO_TARGET_DIR/debug/rag-reference-contract" "$results"
