#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
[[ $(uname -s) == Linux ]] || { echo 'Run this preview inside Linux.' >&2; exit 1; }
cargo build --release --locked
export QUIET_MD_HOME="${QUIET_MD_HOME:-$(mktemp -d "${TMPDIR:-/tmp}/quiet-md-linux-preview.XXXXXX")}"
printf 'Preview library: %s\n' "$QUIET_MD_HOME"
exec "${CARGO_TARGET_DIR:-target}/release/quiet_md"
