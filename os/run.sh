#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE_ONE=${1:-"$ROOT/os/examples/child1.gaut"}
SOURCE_TWO=${2:-"$ROOT/os/examples/child2.gaut"}
COMPILER="$ROOT/dist/gaut-os.img"

for SOURCE in "$SOURCE_ONE" "$SOURCE_TWO"; do
    if [ ! -f "$SOURCE" ]; then
        echo "Gaut source not found: $SOURCE" >&2
        exit 2
    fi
done
if [ ! -f "$COMPILER" ]; then
    echo "Freestanding Gaut compiler not found: $COMPILER" >&2
    exit 2
fi
if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required to run the isolated machine." >&2
    exit 2
fi

RUN_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-run.XXXXXX")
PACKET="$RUN_TEMP/source.packet"
REQUEST_ONE="$RUN_TEMP/child1.request"
REQUEST_TWO="$RUN_TEMP/child2.request"
cleanup() {
    /bin/rm -rf "$RUN_TEMP"
}
trap cleanup EXIT INT TERM

"$ROOT/gaut/request.sh" gaut-child "$SOURCE_ONE" "$REQUEST_ONE"
"$ROOT/gaut/request.sh" gaut-child "$SOURCE_TWO" "$REQUEST_TWO"
append_request() {
    REQUEST=$1
    /bin/cat "$REQUEST" >> "$PACKET"
    SIZE=$(/usr/bin/wc -c < "$REQUEST" | /usr/bin/tr -d ' ')
    PADDING=$(((8 - (SIZE % 8)) % 8))
    if [ "$PADDING" -ne 0 ]; then
        /bin/dd if=/dev/zero bs=1 count="$PADDING" >> "$PACKET" 2>/dev/null
    fi
}

: > "$PACKET"
append_request "$REQUEST_ONE"
append_request "$REQUEST_TWO"
/bin/dd if=/dev/zero bs=16 count=1 >> "$PACKET" 2>/dev/null

echo "Gaut OS is compiling and running two children."
echo "Press Control-C after the program finishes."
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
    -device "loader,file=$PACKET,addr=0x47000000,force-raw=on"
