#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)

if [ ! -x "$ROOT/runtime/arm64-lab" ]; then
    echo "Run $ROOT/setup.sh first." >&2
    exit 1
fi

exec "$ROOT/runtime/arm64-lab"
