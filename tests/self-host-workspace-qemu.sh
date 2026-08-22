#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for workspace self-host verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-workspace-self-host.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

STREAM="$TEST_TEMP/serial.stream"
: > "$STREAM"

INDEX=1
for UNIT in compiler emitter lexer parser request storage symbols; do
    REQUEST="$TEST_TEMP/store.$INDEX"
    "$ROOT/gaut/request.sh" store "$ROOT/gaut/compiler/$UNIT.gaut" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STREAM"
    INDEX=$((INDEX + 1))
done

for UNIT in \
    "$ROOT/gaut/adapters/gaut-os/platform.gaut" \
    "$ROOT/gaut/adapters/common/verification.gaut"; do
    REQUEST="$TEST_TEMP/store.$INDEX"
    "$ROOT/gaut/request.sh" store "$UNIT" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STREAM"
    INDEX=$((INDEX + 1))
done

REQUEST_ONE="$TEST_TEMP/workspace-fixed-point-one.request"
REQUEST_TWO="$TEST_TEMP/workspace-fixed-point-two.request"
"$ROOT/gaut/request.sh" workspace-fixed-point - "$REQUEST_ONE"
"$ROOT/gaut/request.sh" workspace-fixed-point - "$REQUEST_TWO"
/bin/cat "$REQUEST_ONE" "$REQUEST_TWO" >> "$STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null

ACTUAL=$(qemu-system-aarch64 \
    -machine virt,virtualization=off \
    -cpu cortex-a53 \
    -m 128M \
    -display none \
    -monitor none \
    -serial stdio \
    -nic none \
    -no-reboot \
    -kernel "$COMPILER" \
    < "$STREAM")

EXPECTED='gaut-os: ready
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
gaut-os: ready
gaut-os: ready
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Stored Gaut compiler sources did not reach a native fixed point." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Stored Gaut compiler workspace fixed point passed."
