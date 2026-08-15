#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE="$ROOT/gaut/compiler"

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for self-host verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-self-host.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

STAGE1="$TEST_TEMP/stage1.img"
STAGE2="$TEST_TEMP/stage2.img"
STAGE3="$TEST_TEMP/stage3.img"

"$ROOT/os/build.sh" "$ROOT/dist/gaut-os.img" gaut-os "$SOURCE" "$STAGE1"
"$ROOT/os/build.sh" "$STAGE1" gaut-os "$SOURCE" "$STAGE2"
"$ROOT/os/build.sh" "$STAGE2" gaut-os "$SOURCE" "$STAGE3"

if ! /usr/bin/cmp -s "$STAGE1" "$STAGE2" || ! /usr/bin/cmp -s "$STAGE2" "$STAGE3"; then
    echo "Gaut compiler did not reach a byte-identical fixed point." >&2
    exit 1
fi

HASH=$(/usr/bin/shasum -a 256 "$STAGE3" | /usr/bin/awk '{print $1}')
echo "QEMU self-host fixed point passed: $HASH"
