#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for native Gaut test verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-native-test.XXXXXX")
cleanup() {
    /usr/bin/find "$TEST_TEMP" -depth -delete
}
trap cleanup EXIT INT TERM

PASS_ONE="$TEST_TEMP/pass-one.request"
FAIL="$TEST_TEMP/fail.request"
PASS_TWO="$TEST_TEMP/pass-two.request"
STREAM="$TEST_TEMP/serial.stream"

"$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/native-test/success.gaut" "$PASS_ONE"
"$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/native-test/failure.gaut" "$FAIL"
"$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/native-test/success.gaut" "$PASS_TWO"
/bin/cat "$PASS_ONE" "$FAIL" "$PASS_TWO" > "$STREAM"
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
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Gaut OS did not judge child test results." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Gaut-native test result protocol passed."
