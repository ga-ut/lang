#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for Gaut comparison verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-fixed-point-comparison.XXXXXX")
cleanup() {
    /usr/bin/find "$TEST_TEMP" -depth -delete
}
trap cleanup EXIT INT TERM

make_request() {
    FIXTURE=$1
    DESTINATION=$2
    SOURCE_ROOT="$TEST_TEMP/$FIXTURE"
    /bin/mkdir "$SOURCE_ROOT"
    /bin/cp "$ROOT/tests/fixtures/fixed-point/$FIXTURE.gaut" "$SOURCE_ROOT/app.gaut"
    /bin/cp "$ROOT/gaut/adapters/common/verification.gaut" "$SOURCE_ROOT/verification.gaut"
    "$ROOT/gaut/request.sh" test "$SOURCE_ROOT" "$DESTINATION"
}

EQUAL="$TEST_TEMP/equal.request"
BYTE_MISMATCH="$TEST_TEMP/byte-mismatch.request"
SIZE_MISMATCH="$TEST_TEMP/size-mismatch.request"
STREAM="$TEST_TEMP/serial.stream"

make_request equal "$EQUAL"
make_request byte-mismatch "$BYTE_MISMATCH"
make_request size-mismatch "$SIZE_MISMATCH"
/bin/cat "$EQUAL" "$BYTE_MISMATCH" "$SIZE_MISMATCH" > "$STREAM"
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
gaut-os: received 1
gaut-os: test passed
gaut-os: ready
gaut-os: received 2
gaut-os: test failed
gaut-os: ready
gaut-os: received 3
gaut-os: test failed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Gaut image comparison verdicts differed." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Gaut image comparison pass, byte-mismatch, and size-mismatch verdicts passed."
