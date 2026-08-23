#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for flash source-catalog verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-flash-source-catalog.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

FLASH="$TEST_TEMP/workspace.flash"
STORE_STREAM="$TEST_TEMP/store.stream"
SOURCE_DIR="$TEST_TEMP/sources"
/bin/mkdir "$SOURCE_DIR"
: > "$STORE_STREAM"

INDEX=0
while [ "$INDEX" -lt 10 ]; do
    SOURCE="$SOURCE_DIR/unit$INDEX.gaut"
    REQUEST="$TEST_TEMP/store-unit$INDEX.request"
    /usr/bin/printf 'fn value() { return %s; }\n' "$INDEX" > "$SOURCE"
    "$ROOT/gaut/request.sh" store "$SOURCE" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STORE_STREAM"
    INDEX=$((INDEX + 1))
done

APP="$SOURCE_DIR/app.gaut"
/usr/bin/printf '%s\n' 'fn main() {' '  let total = 0;' > "$APP"
INDEX=0
while [ "$INDEX" -lt 10 ]; do
    /usr/bin/printf '  total = add(total, unit%s.value());\n' "$INDEX" >> "$APP"
    INDEX=$((INDEX + 1))
done
/usr/bin/printf '%s\n' '  return sub(total, 45);' '}' >> "$APP"
"$ROOT/gaut/request.sh" store "$APP" "$TEST_TEMP/store-app.request"
/bin/cat "$TEST_TEMP/store-app.request" >> "$STORE_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$STORE_STREAM" 2>/dev/null

FIRST_BOOT=$(GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$FLASH" \
    "$ROOT/os/boot.sh" "$STORE_STREAM")
FIRST_EXPECTED='gaut-os: ready
gaut-os: stored unit0
gaut-os: ready
gaut-os: stored unit1
gaut-os: ready
gaut-os: stored unit2
gaut-os: ready
gaut-os: stored unit3
gaut-os: ready
gaut-os: stored unit4
gaut-os: ready
gaut-os: stored unit5
gaut-os: ready
gaut-os: stored unit6
gaut-os: ready
gaut-os: stored unit7
gaut-os: ready
gaut-os: stored unit8
gaut-os: ready
gaut-os: stored unit9
gaut-os: ready
gaut-os: stored app
gaut-os: ready'
if [ "$FIRST_BOOT" != "$FIRST_EXPECTED" ]; then
    echo "Gaut OS did not store eleven flash-catalog sources." >&2
    /usr/bin/printf '%s\n' "$FIRST_BOOT" >&2
    exit 1
fi

VERIFY="$TEST_TEMP/verify.request"
VERIFY_STREAM="$TEST_TEMP/verify.stream"
"$ROOT/gaut/request.sh" workspace-test - "$VERIFY"
/bin/cp "$VERIFY" "$VERIFY_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$VERIFY_STREAM" 2>/dev/null

SECOND_BOOT=$(GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$FLASH" \
    "$ROOT/os/boot.sh" "$VERIFY_STREAM")
SECOND_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut-os: test passed
gaut-os: ready'
if [ "$SECOND_BOOT" != "$SECOND_EXPECTED" ]; then
    echo "Gaut OS did not rebuild eleven catalog sources after reboot." >&2
    /usr/bin/printf '%s\n' "$SECOND_BOOT" >&2
    exit 1
fi

REPLACEMENT_DIR="$TEST_TEMP/replacement"
/bin/mkdir "$REPLACEMENT_DIR"
/usr/bin/printf 'fn main() { return 1; }\n' > "$REPLACEMENT_DIR/app.gaut"
"$ROOT/gaut/request.sh" store "$REPLACEMENT_DIR/app.gaut" "$TEST_TEMP/store-app-replacement.request"
REPLACE_STREAM="$TEST_TEMP/replace.stream"
/bin/cat \
    "$TEST_TEMP/store-app-replacement.request" \
    "$VERIFY" \
    "$TEST_TEMP/store-app.request" \
    "$VERIFY" > "$REPLACE_STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$REPLACE_STREAM" 2>/dev/null

REPLACE_BOOT=$(GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$FLASH" \
    "$ROOT/os/boot.sh" "$REPLACE_STREAM")
REPLACE_EXPECTED='gaut-os: ready
gaut-os: stored app
gaut-os: ready
gaut-os: received 1
gaut-os: test failed
gaut-os: ready
gaut-os: stored app
gaut-os: ready
gaut-os: received 2
gaut-os: test passed
gaut-os: ready'
if [ "$REPLACE_BOOT" != "$REPLACE_EXPECTED" ]; then
    echo "Gaut OS did not deterministically replace the eleventh catalog source." >&2
    /usr/bin/printf '%s\n' "$REPLACE_BOOT" >&2
    exit 1
fi

SLOT_TEN_OFFSET=$((10 * 262144))
WRONG_INDEX_FLASH="$TEST_TEMP/wrong-index.flash"
/bin/cp "$FLASH" "$WRONG_INDEX_FLASH"
/usr/bin/printf '\000' \
    | /bin/dd of="$WRONG_INDEX_FLASH" bs=1 seek=$((SLOT_TEN_OFFSET + 16)) count=1 conv=notrunc 2>/dev/null

WRONG_INDEX_BOOT=$(GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$WRONG_INDEX_FLASH" \
    "$ROOT/os/boot.sh" "$VERIFY_STREAM")
WRONG_INDEX_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut: invalid source
gaut-os: ready'
if [ "$WRONG_INDEX_BOOT" != "$WRONG_INDEX_EXPECTED" ]; then
    echo "Gaut OS accepted an eleventh catalog source with the wrong physical index." >&2
    /usr/bin/printf '%s\n' "$WRONG_INDEX_BOOT" >&2
    exit 1
fi

CHECKSUM_FLASH="$TEST_TEMP/checksum.flash"
/bin/cp "$FLASH" "$CHECKSUM_FLASH"
/usr/bin/printf '\000' \
    | /bin/dd of="$CHECKSUM_FLASH" bs=1 seek=$((SLOT_TEN_OFFSET + 4096)) count=1 conv=notrunc 2>/dev/null

CHECKSUM_BOOT=$(GAUT_OS_IMAGE="$COMPILER" GAUT_WORKSPACE_FLASH="$CHECKSUM_FLASH" \
    "$ROOT/os/boot.sh" "$VERIFY_STREAM")
CHECKSUM_EXPECTED='gaut-os: ready
gaut-os: received 1
gaut: invalid source
gaut-os: ready'
if [ "$CHECKSUM_BOOT" != "$CHECKSUM_EXPECTED" ]; then
    echo "Gaut OS accepted an eleventh catalog source with a mismatched checksum." >&2
    /usr/bin/printf '%s\n' "$CHECKSUM_BOOT" >&2
    exit 1
fi

echo "Gaut flash source catalog crossed ten slots, replaced slot eleven, and rejected corrupt records."
