#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE=${1:-"$ROOT/os/examples/compiled.gaut"}
COMPILER="$ROOT/dist/gaut-os.img"

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

RUN_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-run.XXXXXX")
PACKET="$RUN_TEMP/source.packet"
cleanup() {
    /bin/rm -rf "$RUN_TEMP"
}
trap cleanup EXIT INT TERM

"$ROOT/gaut/request.sh" gaut-os "$SOURCE" "$PACKET"

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
