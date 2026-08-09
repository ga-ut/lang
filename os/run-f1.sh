#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE=${1:-"$ROOT/os/examples/compiled.gaut"}
COMPILER="$ROOT/dist/gaut-f1.img"

if [ ! -f "$SOURCE" ]; then
    echo "Gaut source not found: $SOURCE" >&2
    exit 2
fi
if [ ! -f "$COMPILER" ]; then
    echo "Freestanding Gaut compiler not found: $COMPILER" >&2
    exit 2
fi
if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required to run the isolated machine." >&2
    exit 2
fi

SIZE=$(/usr/bin/wc -c < "$SOURCE" | /usr/bin/tr -d ' ')
if [ "$SIZE" -gt 131072 ]; then
    echo "Gaut source exceeds the 131072-byte F1 input limit." >&2
    exit 2
fi

F1_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-f1.XXXXXX")
PACKET="$F1_TEMP/source.packet"
cleanup() {
    /bin/rm -rf "$F1_TEMP"
}
trap cleanup EXIT INT TERM

O0=$(/usr/bin/printf '%03o' $((SIZE & 255)))
O1=$(/usr/bin/printf '%03o' $(((SIZE >> 8) & 255)))
O2=$(/usr/bin/printf '%03o' $(((SIZE >> 16) & 255)))
O3=$(/usr/bin/printf '%03o' $(((SIZE >> 24) & 255)))
O4=$(/usr/bin/printf '%03o' $(((SIZE >> 32) & 255)))
O5=$(/usr/bin/printf '%03o' $(((SIZE >> 40) & 255)))
O6=$(/usr/bin/printf '%03o' $(((SIZE >> 48) & 255)))
O7=$(/usr/bin/printf '%03o' $(((SIZE >> 56) & 255)))
/usr/bin/printf "\\$O0\\$O1\\$O2\\$O3\\$O4\\$O5\\$O6\\$O7" > "$PACKET"
/bin/dd if="$SOURCE" of="$PACKET" bs=1 seek=8 conv=notrunc 2>/dev/null

echo "Gaut OS is compiling and running: $SOURCE"
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
