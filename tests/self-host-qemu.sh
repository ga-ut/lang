#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE="$ROOT/gaut/compiler"
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for self-host verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-self-host.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

REQUEST_ONE="$TEST_TEMP/fixed-point-one.request"
REQUEST_TWO="$TEST_TEMP/fixed-point-two.request"
STREAM="$TEST_TEMP/serial.stream"

"$ROOT/gaut/request.sh" fixed-point "$SOURCE" "$REQUEST_ONE"
"$ROOT/gaut/request.sh" fixed-point "$SOURCE" "$REQUEST_TWO"
/bin/cat "$REQUEST_ONE" "$REQUEST_TWO" > "$STREAM"
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
gaut-os: ready
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Gaut did not judge its compiler generations byte-identical." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Gaut-native QEMU self-host fixed point passed."
