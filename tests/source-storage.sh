#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for source-storage verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-source-storage.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

STORE="$TEST_TEMP/store.request"
RUN="$TEST_TEMP/run.request"
REPLACE="$TEST_TEMP/replace.request"
RUN_REPLACEMENT="$TEST_TEMP/run-replacement.request"
STORE_BAD="$TEST_TEMP/store-bad.request"
RUN_BAD="$TEST_TEMP/run-bad.request"
RUN_AFTER_REJECTION="$TEST_TEMP/run-after-rejection.request"
STREAM="$TEST_TEMP/serial.stream"

"$ROOT/gaut/request.sh" store "$ROOT/os/examples/work.gaut" "$STORE"
"$ROOT/gaut/request.sh" run work "$RUN"
"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/replacement/work.gaut" "$REPLACE"
"$ROOT/gaut/request.sh" run work "$RUN_REPLACEMENT"
"$ROOT/gaut/request.sh" store "$ROOT/tests/fixtures/bad.gaut" "$STORE_BAD"
"$ROOT/gaut/request.sh" run bad "$RUN_BAD"
"$ROOT/gaut/request.sh" run work "$RUN_AFTER_REJECTION"
/bin/cat \
    "$STORE" \
    "$RUN" \
    "$REPLACE" \
    "$RUN_REPLACEMENT" \
    "$STORE_BAD" \
    "$RUN_BAD" \
    "$RUN_AFTER_REJECTION" > "$STREAM"
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
gaut-os: stored work
gaut-os: ready
gaut-os: child 1
gaut-os: ready
gaut-os: stored work
gaut-os: ready
gaut-os: child 2
gaut-os: ready
gaut-os: stored bad
gaut-os: ready
gaut: invalid source
gaut-os: ready
gaut-os: child 2
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Named source-storage transcript differed." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Named source storage passed."
