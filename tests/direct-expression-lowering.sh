#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BOOTSTRAP=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for direct-expression verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-direct-expression.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

CANDIDATE="$TEST_TEMP/compiler.img"
BINARY="$TEST_TEMP/binary.elf"
REQUEST="$TEST_TEMP/direct-expression.request"
STREAM="$TEST_TEMP/direct-expression.stream"

"$ROOT/os/build.sh" "$BOOTSTRAP" gaut-os "$ROOT/gaut/compiler" "$CANDIDATE" >/dev/null
"$ROOT/os/build.sh" "$CANDIDATE" linux \
    "$ROOT/tests/fixtures/codegen-baseline/binary.gaut" "$BINARY" >/dev/null

PUSHES=$(
    /usr/bin/od -An -tu4 -v "$BINARY" \
        | /usr/bin/awk '{ for (field = 1; field <= NF; field++) if ($field == 2432705172) count++ } END { print count + 0 }'
)
POPS=$(
    /usr/bin/od -An -tu4 -v "$BINARY" \
        | /usr/bin/awk '{ for (field = 1; field <= NF; field++) if ($field == 3506446996) count++ } END { print count + 0 }'
)

if [ "$PUSHES" -ge 43 ] || [ "$POPS" -ge 46 ]; then
    echo "Direct expression lowering did not reduce binary evaluation-stack traffic." >&2
    echo "pushes=$PUSHES pops=$POPS; previous baseline was pushes=43 pops=46" >&2
    exit 1
fi

"$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/direct-expression" "$REQUEST"
/bin/cp "$REQUEST" "$STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null

ACTUAL=$(GAUT_OS_IMAGE="$CANDIDATE" "$ROOT/os/boot.sh" "$STREAM")
EXPECTED='gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "The directly lowered expression program changed observable results." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Direct expression lowering reduced stack traffic and preserved results."
