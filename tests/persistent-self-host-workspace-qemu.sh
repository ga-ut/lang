#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for persistent workspace fixed-point verification." >&2
    exit 2
fi
TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-persistent-self-host.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

FLASH="$TEST_TEMP/workspace.flash"

run_with_flash() {
    STREAM=$1
    GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$FLASH" \
        "$ROOT/os/boot.sh" "$STREAM"
}

STORE_STREAM="$TEST_TEMP/store.stream"
: > "$STORE_STREAM"
INDEX=1
for UNIT in compiler emitter lexer parser request storage symbols; do
    REQUEST="$TEST_TEMP/store.$INDEX"
    "$ROOT/gaut/request.sh" store "$ROOT/gaut/compiler/$UNIT.gaut" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STORE_STREAM"
    INDEX=$((INDEX + 1))
done
for UNIT in \
    "$ROOT/gaut/adapters/gaut-os/platform.gaut" \
    "$ROOT/gaut/adapters/common/verification.gaut"; do
    REQUEST="$TEST_TEMP/store.$INDEX"
    "$ROOT/gaut/request.sh" store "$UNIT" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STORE_STREAM"
    INDEX=$((INDEX + 1))
done
/bin/dd if=/dev/zero bs=16 count=1 >> "$STORE_STREAM" 2>/dev/null

FIRST_BOOT=$(run_with_flash "$STORE_STREAM")
FIRST_EXPECTED='gaut-os: ready
gaut-os: stored compiler
gaut-os: ready
gaut-os: stored emitter
gaut-os: ready
gaut-os: stored lexer
gaut-os: ready
gaut-os: stored parser
gaut-os: ready
gaut-os: stored request
gaut-os: ready
gaut-os: stored storage
gaut-os: ready
gaut-os: stored symbols
gaut-os: ready
gaut-os: stored platform
gaut-os: ready
gaut-os: stored verification
gaut-os: ready'
if [ "$FIRST_BOOT" != "$FIRST_EXPECTED" ]; then
    echo "Gaut OS did not persist the compiler workspace before shutdown." >&2
    /usr/bin/printf '%s\n' "$FIRST_BOOT" >&2
    exit 1
fi

REQUEST_ONE="$TEST_TEMP/workspace-fixed-point-one.request"
REQUEST_TWO="$TEST_TEMP/workspace-fixed-point-two.request"
FIXED_POINT_STREAM="$TEST_TEMP/fixed-point.stream"
"$ROOT/gaut/request.sh" workspace-fixed-point - "$REQUEST_ONE"
"$ROOT/gaut/request.sh" workspace-fixed-point - "$REQUEST_TWO"
/bin/cat "$REQUEST_ONE" "$REQUEST_TWO" > "$FIXED_POINT_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$FIXED_POINT_STREAM" 2>/dev/null

SECOND_BOOT=$(run_with_flash "$FIXED_POINT_STREAM")
SECOND_EXPECTED='gaut-os: ready
gaut-os: ready
gaut-os: test passed
gaut-os: ready'
if [ "$SECOND_BOOT" != "$SECOND_EXPECTED" ]; then
    echo "The rebooted persistent compiler workspace did not reach a native fixed point." >&2
    /usr/bin/printf '%s\n' "$SECOND_BOOT" >&2
    exit 1
fi

echo "Persistent Gaut compiler workspace fixed point across reboot passed."
