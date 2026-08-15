#!/bin/sh
set -eu

if [ "$#" -ne 4 ]; then
    echo "usage: $0 <compiler.img> <linux|gaut-os> <source-root> <output>" >&2
    exit 2
fi

COMPILER=$1
PROFILE=$2
SOURCE=$3
OUTPUT=$4
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)

if [ ! -f "$COMPILER" ]; then
    echo "Gaut OS compiler not found: $COMPILER" >&2
    exit 2
fi
case "$PROFILE" in
    linux|gaut-os) ;;
    *)
        echo "Gaut OS can retain only linux or gaut-os build artifacts." >&2
        exit 2
        ;;
esac
if [ ! -f "$SOURCE" ] && [ ! -d "$SOURCE" ]; then
    echo "Gaut source not found: $SOURCE" >&2
    exit 2
fi
if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required to run the isolated machine." >&2
    exit 2
fi

BUILD_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-build.XXXXXX")
cleanup() {
    /bin/rm -rf "$BUILD_TEMP"
}
trap cleanup EXIT INT TERM

REQUEST="$BUILD_TEMP/request"
STREAM="$BUILD_TEMP/stream"
SERIAL="$BUILD_TEMP/serial"
MARKER="$BUILD_TEMP/marker"
PREFIX="$BUILD_TEMP/prefix"
SUFFIX="$BUILD_TEMP/suffix"
ARTIFACT="$BUILD_TEMP/artifact"

"$ROOT/gaut/request.sh" "$PROFILE" "$SOURCE" "$REQUEST"
/bin/cp "$REQUEST" "$STREAM"
/bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null

qemu-system-aarch64 \
    -machine virt,virtualization=off \
    -cpu cortex-a53 \
    -m 128M \
    -display none \
    -monitor none \
    -serial stdio \
    -nic none \
    -no-reboot \
    -kernel "$COMPILER" \
    < "$STREAM" > "$SERIAL"

/usr/bin/printf 'gaut-os: ready\n' > "$MARKER"
MARKER_SIZE=$(/usr/bin/wc -c < "$MARKER" | /usr/bin/tr -d ' ')
SERIAL_SIZE=$(/usr/bin/wc -c < "$SERIAL" | /usr/bin/tr -d ' ')
if [ "$SERIAL_SIZE" -le $((MARKER_SIZE * 2)) ]; then
    echo "Gaut OS did not return a build artifact." >&2
    exit 1
fi

ARTIFACT_SIZE=$((SERIAL_SIZE - MARKER_SIZE * 2))
/bin/dd if="$SERIAL" of="$PREFIX" bs=1 count="$MARKER_SIZE" 2>/dev/null
/bin/dd if="$SERIAL" of="$SUFFIX" bs=1 skip=$((MARKER_SIZE + ARTIFACT_SIZE)) count="$MARKER_SIZE" 2>/dev/null
if ! /usr/bin/cmp -s "$MARKER" "$PREFIX" || ! /usr/bin/cmp -s "$MARKER" "$SUFFIX"; then
    echo "Gaut OS build channel returned an invalid frame." >&2
    exit 1
fi

/bin/dd if="$SERIAL" of="$ARTIFACT" bs=1 skip="$MARKER_SIZE" count="$ARTIFACT_SIZE" 2>/dev/null
/bin/cp "$ARTIFACT" "$OUTPUT"

echo "Gaut OS built $ARTIFACT_SIZE bytes."
