#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
ACTUAL=$("$ROOT/os/run.sh" "$ROOT/tests/fixtures/platform-module")

EXPECTED='Gaut OS is compiling and running 1 child request(s).
gaut-os: ready
gaut-os: received 1
platform module ok
gaut-os: ready
Gaut OS shut down cleanly.'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "platform.run did not resolve as an ordinary Gaut module function." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Platform module resolution passed."
