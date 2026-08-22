#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for persistent workspace verification." >&2
    exit 2
fi
TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-persistent-workspace.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

FLASH="$TEST_TEMP/workspace.flash"

run_with_flash() {
    STREAM=$1
    ATTACHED_FLASH=${2:-$FLASH}
    GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$ATTACHED_FLASH" \
        "$ROOT/os/boot.sh" "$STREAM"
}

STORE_HELPER="$TEST_TEMP/store-helper.request"
STORE_APP="$TEST_TEMP/store-app.request"
STORE_STREAM="$TEST_TEMP/store.stream"
VERIFY="$TEST_TEMP/verify.request"
VERIFY_STREAM="$TEST_TEMP/verify.stream"

"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/workspace/replacement/helper.gaut" "$STORE_HELPER"
"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/workspace/app.gaut" "$STORE_APP"
/bin/cat "$STORE_HELPER" "$STORE_APP" > "$STORE_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$STORE_STREAM" 2>/dev/null

FIRST_BOOT=$(run_with_flash "$STORE_STREAM")
FIRST_EXPECTED='gaut-os: ready
gaut-os: stored helper
gaut-os: ready
gaut-os: stored app
gaut-os: ready'
if [ "$FIRST_BOOT" != "$FIRST_EXPECTED" ]; then
    echo "Gaut OS did not store the initial persistent workspace." >&2
    /usr/bin/printf '%s\n' "$FIRST_BOOT" >&2
    exit 1
fi

"$ROOT/gaut/request.sh" workspace-test - "$VERIFY"
/bin/cp "$VERIFY" "$VERIFY_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$VERIFY_STREAM" 2>/dev/null

SECOND_BOOT=$(run_with_flash "$VERIFY_STREAM")
SECOND_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready'
if [ "$SECOND_BOOT" != "$SECOND_EXPECTED" ]; then
    echo "Gaut OS did not restore and run the workspace after reboot." >&2
    /usr/bin/printf '%s\n' "$SECOND_BOOT" >&2
    exit 1
fi

REPLACE_HELPER="$TEST_TEMP/replace-helper.request"
REPLACE_STREAM="$TEST_TEMP/replace.stream"
"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/workspace/helper.gaut" "$REPLACE_HELPER"
/bin/cp "$REPLACE_HELPER" "$REPLACE_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$REPLACE_STREAM" 2>/dev/null

THIRD_BOOT=$(run_with_flash "$REPLACE_STREAM")
THIRD_EXPECTED='gaut-os: ready
gaut-os: stored helper
gaut-os: ready'
if [ "$THIRD_BOOT" != "$THIRD_EXPECTED" ]; then
    echo "Gaut OS did not replace a persistent source after reboot." >&2
    /usr/bin/printf '%s\n' "$THIRD_BOOT" >&2
    exit 1
fi

FOURTH_BOOT=$(run_with_flash "$VERIFY_STREAM")
FOURTH_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut-os: test failed
gaut-os: ready'
if [ "$FOURTH_BOOT" != "$FOURTH_EXPECTED" ]; then
    echo "Gaut OS did not retain the replacement across another reboot." >&2
    /usr/bin/printf '%s\n' "$FOURTH_BOOT" >&2
    exit 1
fi

# Change one byte in a copy of the first slot payload while retaining its
# committed header. Gaut must reject the now-mismatched payload checksum.
CHECKSUM_FLASH="$TEST_TEMP/checksum.flash"
/bin/cp "$FLASH" "$CHECKSUM_FLASH"
/usr/bin/printf '\000' \
    | /bin/dd of="$CHECKSUM_FLASH" bs=1 seek=4096 count=1 conv=notrunc 2>/dev/null

CHECKSUM_BOOT=$(run_with_flash "$VERIFY_STREAM" "$CHECKSUM_FLASH")
CHECKSUM_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut: invalid source
gaut-os: ready'
if [ "$CHECKSUM_BOOT" != "$CHECKSUM_EXPECTED" ]; then
    echo "Gaut OS accepted a persistent slot with a mismatched payload checksum." >&2
    /usr/bin/printf '%s\n' "$CHECKSUM_BOOT" >&2
    exit 1
fi

# Invalidate the first slot header. The second slot remains valid, so command 6
# reaches the ordinary compiler and rejects app's now-missing helper module.
/usr/bin/printf '\000' \
    | /bin/dd of="$FLASH" bs=1 seek=0 count=1 conv=notrunc 2>/dev/null

CORRUPT_BOOT=$(run_with_flash "$VERIFY_STREAM")
CORRUPT_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut: invalid source
gaut-os: ready'
if [ "$CORRUPT_BOOT" != "$CORRUPT_EXPECTED" ]; then
    echo "Gaut OS accepted a workspace with an invalid persistent slot header." >&2
    /usr/bin/printf '%s\n' "$CORRUPT_BOOT" >&2
    exit 1
fi

echo "Persistent Gaut source restore, replacement, and header/payload corruption rejection passed."
