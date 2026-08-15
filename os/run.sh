#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER="$ROOT/dist/gaut-os.img"

if [ "$#" -eq 0 ]; then
    set -- "$ROOT/os/examples/child1.gaut" "$ROOT/os/examples/child2.gaut"
fi
if [ "$#" -gt 9 ]; then
    echo "A bounded Gaut session accepts at most 9 source inputs." >&2
    exit 2
fi

for SOURCE in "$@"; do
    if [ ! -f "$SOURCE" ] && [ ! -d "$SOURCE" ]; then
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
STREAM="$RUN_TEMP/serial.stream"
cleanup() {
    /bin/rm -rf "$RUN_TEMP"
}
trap cleanup EXIT INT TERM

: > "$STREAM"
INDEX=1
for SOURCE in "$@"; do
    REQUEST="$RUN_TEMP/request.$INDEX"
    "$ROOT/gaut/request.sh" gaut-child "$SOURCE" "$REQUEST"
    /bin/cat "$REQUEST" >> "$STREAM"
    INDEX=$((INDEX + 1))
done
/bin/dd if=/dev/zero bs=16 count=1 >> "$STREAM" 2>/dev/null

echo "Gaut OS is compiling and running $# child request(s)."
/bin/cat "$STREAM" | qemu-system-aarch64 \
    -machine virt,virtualization=off \
    -cpu cortex-a53 \
    -m 128M \
    -display none \
    -monitor none \
    -serial stdio \
    -nic none \
    -no-reboot \
    -kernel "$COMPILER"
echo "Gaut OS shut down cleanly."
