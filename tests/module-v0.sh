#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ "$(/usr/bin/uname -s)" != Linux ] || [ "$(/usr/bin/uname -m)" != aarch64 ]; then
    echo "Module v0 test must run in the isolated AArch64 Linux lab." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-module-v0.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

REQUEST="$TEST_TEMP/request.bin"
PROGRAM="$TEST_TEMP/program.elf"
OUTPUT="$TEST_TEMP/output"

"$ROOT/gaut/request.sh" linux "$ROOT/gaut/examples/modules" "$REQUEST"
"$ROOT/dist/gaut.elf" < "$REQUEST" > "$PROGRAM"
/bin/chmod +x "$PROGRAM"

set +e
"$PROGRAM" > "$OUTPUT"
PROGRAM_STATUS=$?
set -e

if [ "$PROGRAM_STATUS" -ne 42 ] || ! /usr/bin/grep -q '^module ok$' "$OUTPUT"; then
    echo "Module v0 program did not run." >&2
    /bin/cat "$OUTPUT" >&2
    exit 1
fi

echo "Module v0 cross-unit call passed."
