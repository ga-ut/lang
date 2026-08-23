#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for local-slot reuse verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-local-slot-reuse.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

write_locals() {
    DESTINATION=$1
    PREFIX=$2
    COUNT=$3
    LAST_VALUE=$4
    INDEX=0

    while [ "$INDEX" -lt "$COUNT" ]; do
        VALUE=0
        if [ "$INDEX" -eq $((COUNT - 1)) ]; then
            VALUE=$LAST_VALUE
        fi
        /usr/bin/printf '  let %s%s = %s;\n' "$PREFIX" "$INDEX" "$VALUE" >> "$DESTINATION"
        INDEX=$((INDEX + 1))
    done
}

REUSE="$TEST_TEMP/reuse.gaut"
/usr/bin/printf '%s\n' \
    'fn main() {' \
    '  return sub(add(add(first(), second()), third()), 6);' \
    '}' > "$REUSE"
for SPECIFICATION in 'first 1' 'second 2' 'third 3'; do
    set -- $SPECIFICATION
    /usr/bin/printf 'fn %s() {\n' "$1" >> "$REUSE"
    write_locals "$REUSE" "$1" 200 "$2"
    /usr/bin/printf '  return %s199;\n}\n' "$1" >> "$REUSE"
done

PRESERVE="$TEST_TEMP/preserve.gaut"
/usr/bin/printf 'fn main() {\n' > "$PRESERVE"
write_locals "$PRESERVE" main 256 7
/usr/bin/printf '%s\n' \
    '  drop child();' \
    '  return sub(main255, 7);' \
    '}' \
    'fn child() {' >> "$PRESERVE"
write_locals "$PRESERVE" child 256 9
/usr/bin/printf '%s\n' \
    '  return child255;' \
    '}' >> "$PRESERVE"

OVER="$TEST_TEMP/over.gaut"
/usr/bin/printf 'fn main() {\n' > "$OVER"
write_locals "$OVER" main 256 0
/usr/bin/printf '%s\n' \
    '  drop child();' \
    '  return main255;' \
    '}' \
    'fn child() {' >> "$OVER"
write_locals "$OVER" child 256 0
/usr/bin/printf '%s\n' \
    '  drop grandchild();' \
    '  return child255;' \
    '}' \
    'fn grandchild() {' \
    '  let extra = 0;' \
    '  return extra;' \
    '}' >> "$OVER"

CYCLE="$TEST_TEMP/cycle.gaut"
/usr/bin/printf '%s\n' \
    'fn main() { return first(); }' \
    'fn first() { return second(); }' \
    'fn second() { return first(); }' > "$CYCLE"

SUCCESS="$TEST_TEMP/success.gaut"
/usr/bin/printf 'fn main() { return 0; }\n' > "$SUCCESS"

STREAM="$TEST_TEMP/serial.stream"
: > "$STREAM"
for SOURCE in "$REUSE" "$PRESERVE" "$OVER" "$CYCLE" "$SUCCESS"; do
    REQUEST="$SOURCE.request"
    "$ROOT/gaut/request.sh" test "$SOURCE" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STREAM"
done
/bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null

ACTUAL=$(GAUT_OS_IMAGE="$COMPILER" "$ROOT/os/boot.sh" "$STREAM")
EXPECTED='gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready
gaut-os: received 2
gaut-os: test passed
gaut-os: ready
gaut-os: received 3
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Gaut local-slot lifetime reuse transcript differed." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Gaut local-slot lifetime reuse and active-chain boundary passed."
