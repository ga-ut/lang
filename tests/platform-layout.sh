#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for platform-layout verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-platform-layout.XXXXXX")
cleanup() {
    /usr/bin/find "$TEST_TEMP" -depth -delete
}
trap cleanup EXIT INT TERM

IMAGE="$TEST_TEMP/platform-layout.img"
"$ROOT/os/build.sh" \
    "$COMPILER" \
    gaut-os \
    "$ROOT/tests/fixtures/platform-layout" \
    "$IMAGE" >/dev/null

ACTUAL=$(qemu-system-aarch64 \
    -machine virt,virtualization=off \
    -cpu cortex-a53 \
    -m 128M \
    -display none \
    -monitor none \
    -serial stdio \
    -nic none \
    -no-reboot \
    -kernel "$IMAGE" \
    </dev/null)

EXPECTED='platform layout ok'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Platform layout accessors returned unexpected values." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Platform layout accessors passed."
