#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
BOOTSTRAP=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for hexadecimal literal verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "/tmp/gaut-hex-literals.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

CANDIDATE="$TEST_TEMP/compiler.img"
SOURCE="$TEST_TEMP/main.gaut"
DECIMAL_IMAGE="$TEST_TEMP/decimal.elf"
HEX_IMAGE="$TEST_TEMP/hex.elf"

"$ROOT/os/build.sh" "$BOOTSTRAP" gaut-os "$ROOT/gaut/compiler" "$CANDIDATE" >/dev/null

/bin/cp "$ROOT/tests/fixtures/hex-literals/equivalent-decimal.gaut" "$SOURCE"
"$ROOT/os/build.sh" "$CANDIDATE" linux "$SOURCE" "$DECIMAL_IMAGE" >/dev/null
/bin/cp "$ROOT/tests/fixtures/hex-literals/equivalent-hex.gaut" "$SOURCE"
"$ROOT/os/build.sh" "$CANDIDATE" linux "$SOURCE" "$HEX_IMAGE" >/dev/null
if ! /usr/bin/cmp -s "$DECIMAL_IMAGE" "$HEX_IMAGE"; then
    echo "Decimal and hexadecimal spellings did not emit identical images." >&2
    exit 1
fi

run_case() {
    NAME=$1
    EXPECTED_RESULT=$2
    REQUEST="$TEST_TEMP/$NAME.request"
    STREAM="$TEST_TEMP/$NAME.stream"

    "$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/hex-literals/$NAME.gaut" "$REQUEST"
    /bin/cp "$REQUEST" "$STREAM"
    /bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null

    ACTUAL=$(GAUT_OS_IMAGE="$CANDIDATE" "$ROOT/os/boot.sh" "$STREAM")
    EXPECTED="gaut-os: ready
gaut-os: received 1
$EXPECTED_RESULT
gaut-os: ready"
    if [ "$ACTUAL" != "$EXPECTED" ]; then
        echo "Unexpected hexadecimal literal result for $NAME." >&2
        /usr/bin/printf '%s\n' "$ACTUAL" >&2
        exit 1
    fi
}

run_recovery_sequence() {
    STREAM="$TEST_TEMP/recovery-sequence.stream"
    : > "$STREAM"
    EXPECTED='gaut-os: ready'
    REJECTED_RECEIVED=1

    for NAME in malformed-empty malformed-digit malformed-uppercase-prefix malformed-uppercase-digit overflow; do
        REQUEST="$TEST_TEMP/$NAME-recovery.request"
        "$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/hex-literals/$NAME.gaut" "$REQUEST"
        /bin/cat "$REQUEST" >> "$STREAM"

        REQUEST="$TEST_TEMP/$NAME-following-pass.request"
        "$ROOT/gaut/request.sh" test "$ROOT/tests/fixtures/hex-literals/recovery.gaut" "$REQUEST"
        /bin/cat "$REQUEST" >> "$STREAM"

        EXPECTED="$EXPECTED
gaut-os: received $REJECTED_RECEIVED
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready"
        REJECTED_RECEIVED=2
    done

    /bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null
    ACTUAL=$(GAUT_OS_IMAGE="$CANDIDATE" "$ROOT/os/boot.sh" "$STREAM")
    if [ "$ACTUAL" != "$EXPECTED" ]; then
        echo "Unexpected hexadecimal rejection and recovery sequence." >&2
        /usr/bin/printf '%s\n' "$ACTUAL" >&2
        exit 1
    fi
}

run_case accepted 'gaut-os: test passed'
run_recovery_sequence

echo "Lowercase hexadecimal literals, rejection, recovery, and decimal equivalence passed."
