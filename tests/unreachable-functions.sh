#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for unreachable-function verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-unreachable-functions.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

FORWARD_CLEAN="$TEST_TEMP/forward-clean"
FORWARD_DEAD="$TEST_TEMP/forward-dead"
BACKWARD_CLEAN="$TEST_TEMP/backward-clean"
BACKWARD_DEAD="$TEST_TEMP/backward-dead"
/bin/mkdir "$FORWARD_CLEAN" "$FORWARD_DEAD" "$BACKWARD_CLEAN" "$BACKWARD_DEAD"

for DIRECTORY in "$FORWARD_CLEAN" "$FORWARD_DEAD"; do
    /usr/bin/printf '%s\n' \
        'fn main() { return sub(c_live.value(), 7); }' > "$DIRECTORY/a_app.gaut"
    /usr/bin/printf '%s\n' \
        'fn value() { return 7; }' > "$DIRECTORY/c_live.gaut"
done
/usr/bin/printf '%s\n' \
    'fn unused() {' \
    '  let value = add(40, 2);' \
    '  return value;' \
    '}' > "$FORWARD_DEAD/b_dead.gaut"

for DIRECTORY in "$BACKWARD_CLEAN" "$BACKWARD_DEAD"; do
    /usr/bin/printf '%s\n' \
        'fn value() {' \
        '  let current = 0;' \
        '  while lt(current, 7) {' \
        '    current = add(current, 1);' \
        '  }' \
        '  if eq(current, 7) {' \
        '  } else {' \
        '    current = 0;' \
        '  }' \
        '  return current;' \
        '}' > "$DIRECTORY/b_live.gaut"
    /usr/bin/printf '%s\n' \
        'fn main() { return sub(b_live.value(), 7); }' > "$DIRECTORY/c_app.gaut"
done
/usr/bin/printf '%s\n' \
    'fn unused() { return 99; }' > "$BACKWARD_DEAD/a_dead.gaut"

for DIRECTION in forward backward; do
    "$ROOT/os/build.sh" "$COMPILER" linux \
        "$TEST_TEMP/$DIRECTION-clean" "$TEST_TEMP/$DIRECTION-clean.elf" >/dev/null
    "$ROOT/os/build.sh" "$COMPILER" linux \
        "$TEST_TEMP/$DIRECTION-dead" "$TEST_TEMP/$DIRECTION-dead.elf" >/dev/null

    for IMAGE in "$TEST_TEMP/$DIRECTION-clean.elf" "$TEST_TEMP/$DIRECTION-dead.elf"; do
        MAGIC=$(/usr/bin/od -An -tx1 -N4 "$IMAGE" | /usr/bin/tr -d ' \n')
        if [ "$MAGIC" != "7f454c46" ]; then
            echo "The $DIRECTION-call fixture did not produce a Linux ELF." >&2
            exit 1
        fi
    done

    if ! /usr/bin/cmp -s \
        "$TEST_TEMP/$DIRECTION-clean.elf" "$TEST_TEMP/$DIRECTION-dead.elf"; then
        echo "An unreachable $DIRECTION-call function still changed the generated image." >&2
        exit 1
    fi
done

UNRESOLVED="$TEST_TEMP/unresolved.gaut"
CYCLE="$TEST_TEMP/cycle.gaut"
SUCCESS="$TEST_TEMP/success.gaut"
CHAIN="$TEST_TEMP/chain.gaut"
/usr/bin/printf '%s\n' \
    'fn unused() { return missing(); }' \
    'fn main() { return 0; }' > "$UNRESOLVED"
/usr/bin/printf '%s\n' \
    'fn first() { return second(); }' \
    'fn second() { return first(); }' \
    'fn main() { return 0; }' > "$CYCLE"
/usr/bin/printf 'fn main() { return 0; }\n' > "$SUCCESS"
/usr/bin/printf 'fn f254() { return 0; }\n' > "$CHAIN"
INDEX=253
while [ "$INDEX" -ge 0 ]; do
    /usr/bin/printf 'fn f%s() { return f%s(); }\n' "$INDEX" $((INDEX + 1)) >> "$CHAIN"
    INDEX=$((INDEX - 1))
done
/usr/bin/printf 'fn main() { return f0(); }\n' >> "$CHAIN"

STREAM="$TEST_TEMP/serial.stream"
: > "$STREAM"
for SOURCE in "$FORWARD_DEAD" "$BACKWARD_DEAD" "$CHAIN" "$UNRESOLVED" "$CYCLE" "$SUCCESS"; do
    REQUEST="$TEST_TEMP/$(basename "$SOURCE").request"
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
gaut-os: test passed
gaut-os: ready
gaut-os: received 4
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Unreachable-function validation or execution transcript differed." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Unreachable functions were removed after full validation and calls still resolved."
