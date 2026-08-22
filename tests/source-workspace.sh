#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for source-workspace verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-source-workspace.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

STORE_HELPER="$TEST_TEMP/store-helper.request"
STORE_APP="$TEST_TEMP/store-app.request"
VERIFY_FAILURE="$TEST_TEMP/verify-failure.request"
STORE_REPLACEMENT="$TEST_TEMP/store-replacement.request"
VERIFY_SUCCESS="$TEST_TEMP/verify-success.request"
STREAM="$TEST_TEMP/serial.stream"

"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/workspace/helper.gaut" "$STORE_HELPER"
"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/workspace/app.gaut" "$STORE_APP"
"$ROOT/gaut/request.sh" workspace-test - "$VERIFY_FAILURE"
"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/workspace/replacement/helper.gaut" "$STORE_REPLACEMENT"
"$ROOT/gaut/request.sh" workspace-test - "$VERIFY_SUCCESS"
/bin/cat \
    "$STORE_HELPER" \
    "$STORE_APP" \
    "$VERIFY_FAILURE" \
    "$STORE_REPLACEMENT" \
    "$VERIFY_SUCCESS" > "$STREAM"
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
gaut-os: stored helper
gaut-os: ready
gaut-os: stored app
gaut-os: ready
gaut-os: received 1
gaut-os: test failed
gaut-os: ready
gaut-os: stored helper
gaut-os: ready
gaut-os: received 2
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Gaut OS did not build and judge its stored source workspace." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Gaut-native source workspace replacement passed."
