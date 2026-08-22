#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for capacity baseline verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-capacity-baseline.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

write_source_size() {
    DESTINATION=$1
    TARGET_SIZE=$2
    /usr/bin/printf 'fn main() { return 0; }\n' > "$DESTINATION"
    CURRENT_SIZE=$(/usr/bin/wc -c < "$DESTINATION" | /usr/bin/tr -d ' ')
    REMAINING=$((TARGET_SIZE - CURRENT_SIZE))
    /bin/dd if=/dev/zero bs=1 count="$REMAINING" 2>/dev/null \
        | /usr/bin/tr '\000' ' ' >> "$DESTINATION"
}

AT_LIMIT_DIR="$TEST_TEMP/request-at-limit"
OVER_LIMIT_DIR="$TEST_TEMP/request-over-limit"
/bin/mkdir "$AT_LIMIT_DIR" "$OVER_LIMIT_DIR"
write_source_size "$AT_LIMIT_DIR/app.gaut" 131021
write_source_size "$OVER_LIMIT_DIR/app.gaut" 131022

AT_LIMIT_REQUEST="$TEST_TEMP/request-at-limit.bin"
"$ROOT/gaut/request.sh" gaut-child "$AT_LIMIT_DIR/app.gaut" "$AT_LIMIT_REQUEST"
AT_LIMIT_SIZE=$(/usr/bin/wc -c < "$AT_LIMIT_REQUEST" | /usr/bin/tr -d ' ')
if [ "$AT_LIMIT_SIZE" -ne 131072 ]; then
    echo "Expected the largest request to occupy 131072 bytes, got $AT_LIMIT_SIZE." >&2
    exit 1
fi

if "$ROOT/gaut/request.sh" gaut-child "$OVER_LIMIT_DIR/app.gaut" "$TEST_TEMP/request-over-limit.bin" 2>"$TEST_TEMP/request-over-limit.err"; then
    echo "A request larger than 131072 bytes was accepted." >&2
    exit 1
fi
if ! /usr/bin/grep -q 'exceeds the 131072-byte limit' "$TEST_TEMP/request-over-limit.err"; then
    echo "The oversized request did not report its capacity boundary." >&2
    exit 1
fi

UNITS_AT_LIMIT="$TEST_TEMP/units-at-limit"
UNITS_OVER_LIMIT="$TEST_TEMP/units-over-limit"
/bin/mkdir "$UNITS_AT_LIMIT" "$UNITS_OVER_LIMIT"
UNIT=0
while [ "$UNIT" -lt 65 ]; do
    /usr/bin/printf 'fn f%s() { return %s; }\n' "$UNIT" "$UNIT" > "$UNITS_OVER_LIMIT/unit$UNIT.gaut"
    if [ "$UNIT" -lt 64 ]; then
        /bin/cp "$UNITS_OVER_LIMIT/unit$UNIT.gaut" "$UNITS_AT_LIMIT/unit$UNIT.gaut"
    fi
    UNIT=$((UNIT + 1))
done

"$ROOT/gaut/request.sh" gaut-child "$UNITS_AT_LIMIT" "$TEST_TEMP/units-at-limit.bin"
UNIT_COUNT=$(/usr/bin/od -An -tu8 -j24 -N8 "$TEST_TEMP/units-at-limit.bin" | /usr/bin/tr -d ' ')
if [ "$UNIT_COUNT" -ne 64 ]; then
    echo "Expected a 64-unit request, got $UNIT_COUNT units." >&2
    exit 1
fi
if "$ROOT/gaut/request.sh" gaut-child "$UNITS_OVER_LIMIT" "$TEST_TEMP/units-over-limit.bin" 2>"$TEST_TEMP/units-over-limit.err"; then
    echo "A request containing 65 source units was accepted." >&2
    exit 1
fi
if ! /usr/bin/grep -q 'between 1 and 64 source units' "$TEST_TEMP/units-over-limit.err"; then
    echo "The 65-unit request did not report its capacity boundary." >&2
    exit 1
fi

MEMORY_AT_LIMIT="$TEST_TEMP/memory_at_limit.gaut"
MEMORY_OVER_LIMIT="$TEST_TEMP/memory_over_limit.gaut"
FUNCTIONS_AT_LIMIT="$TEST_TEMP/functions_at_limit.gaut"
FUNCTIONS_OVER_LIMIT="$TEST_TEMP/functions_over_limit.gaut"
LOCALS_AT_LIMIT="$TEST_TEMP/locals_at_limit.gaut"
LOCALS_OVER_LIMIT="$TEST_TEMP/locals_over_limit.gaut"
OUTPUT_NEAR_LIMIT="$TEST_TEMP/output_near_limit.gaut"
OUTPUT_OVER_LIMIT="$TEST_TEMP/output_over_limit.gaut"
SUCCESS="$TEST_TEMP/success.gaut"

/usr/bin/printf '%s\n' \
    'fn main() {' \
    '  memory bytes 917504;' \
    '  store8(7, add(bytes, 917503));' \
    '  return sub(load8(add(bytes, 917503)), 7);' \
    '}' > "$MEMORY_AT_LIMIT"
/usr/bin/printf '%s\n' \
    'fn main() {' \
    '  memory bytes 917505;' \
    '  return 0;' \
    '}' > "$MEMORY_OVER_LIMIT"

: > "$FUNCTIONS_AT_LIMIT"
: > "$FUNCTIONS_OVER_LIMIT"
FUNCTION=0
while [ "$FUNCTION" -lt 256 ]; do
    if [ "$FUNCTION" -lt 255 ]; then
        /usr/bin/printf 'fn f%s() { return 0; }\n' "$FUNCTION" >> "$FUNCTIONS_AT_LIMIT"
    fi
    /usr/bin/printf 'fn f%s() { return 0; }\n' "$FUNCTION" >> "$FUNCTIONS_OVER_LIMIT"
    FUNCTION=$((FUNCTION + 1))
done
/usr/bin/printf 'fn main() { return 0; }\n' >> "$FUNCTIONS_AT_LIMIT"
/usr/bin/printf 'fn main() { return 0; }\n' >> "$FUNCTIONS_OVER_LIMIT"

: > "$LOCALS_AT_LIMIT"
for FUNCTION_NAME in first second; do
    /usr/bin/printf 'fn %s() {\n' "$FUNCTION_NAME" >> "$LOCALS_AT_LIMIT"
    LOCAL=0
    while [ "$LOCAL" -lt 256 ]; do
        /usr/bin/printf '  let %s%s = 0;\n' "$FUNCTION_NAME" "$LOCAL" >> "$LOCALS_AT_LIMIT"
        LOCAL=$((LOCAL + 1))
    done
    /usr/bin/printf '  return %s255;\n}\n' "$FUNCTION_NAME" >> "$LOCALS_AT_LIMIT"
done
/usr/bin/printf 'fn main() { return 0; }\n' >> "$LOCALS_AT_LIMIT"
/bin/cp "$LOCALS_AT_LIMIT" "$LOCALS_OVER_LIMIT"
/usr/bin/printf '%s\n' \
    'fn over() {' \
    '  let extra = 0;' \
    '  return extra;' \
    '}' >> "$LOCALS_OVER_LIMIT"

: > "$OUTPUT_NEAR_LIMIT"
/usr/bin/printf 'fn main() {\n' >> "$OUTPUT_NEAR_LIMIT"
STATEMENT=0
while [ "$STATEMENT" -lt 5039 ]; do
    /usr/bin/printf 'drop add(add(1,1),1);' >> "$OUTPUT_NEAR_LIMIT"
    STATEMENT=$((STATEMENT + 1))
done
/usr/bin/printf '  return 0;\n}\n' >> "$OUTPUT_NEAR_LIMIT"

: > "$OUTPUT_OVER_LIMIT"
/usr/bin/printf 'fn main() {\n' >> "$OUTPUT_OVER_LIMIT"
STATEMENT=0
while [ "$STATEMENT" -lt 5040 ]; do
    /usr/bin/printf 'drop add(add(1,1),1);' >> "$OUTPUT_OVER_LIMIT"
    STATEMENT=$((STATEMENT + 1))
done
/usr/bin/printf '  return 0;\n}\n' >> "$OUTPUT_OVER_LIMIT"

/usr/bin/printf 'fn main() { return 0; }\n' > "$SUCCESS"

STREAM="$TEST_TEMP/serial.stream"
: > "$STREAM"
for SOURCE in \
    "$MEMORY_AT_LIMIT" \
    "$MEMORY_OVER_LIMIT" \
    "$FUNCTIONS_AT_LIMIT" \
    "$FUNCTIONS_OVER_LIMIT" \
    "$LOCALS_AT_LIMIT" \
    "$LOCALS_OVER_LIMIT" \
    "$OUTPUT_NEAR_LIMIT" \
    "$OUTPUT_OVER_LIMIT" \
    "$SUCCESS"; do
    REQUEST="$SOURCE.request"
    "$ROOT/gaut/request.sh" test "$SOURCE" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STREAM"
done
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
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready
gaut-os: received 2
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready
gaut-os: received 2
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready
gaut-os: received 2
gaut: invalid source
gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready'

if [ "$ACTUAL" != "$EXPECTED" ]; then
    echo "Gaut compiler capacity baseline changed." >&2
    /usr/bin/printf '%s\n' "$ACTUAL" >&2
    exit 1
fi

echo "Gaut request, module, function, local, fixed-memory, and output-image capacity baseline passed."
