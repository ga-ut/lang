#!/bin/sh
set -eu

if [ "$#" -ne 1 ]; then
    echo "usage: $0 <serial-stream>" >&2
    exit 2
fi

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}
STREAM=$1

if [ ! -f "$COMPILER" ]; then
    echo "Freestanding Gaut compiler not found: $COMPILER" >&2
    exit 2
fi
if [ ! -f "$STREAM" ]; then
    echo "Gaut serial stream not found: $STREAM" >&2
    exit 2
fi
if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required to boot Gaut OS." >&2
    exit 2
fi

if [ -n "${GAUT_WORKSPACE_FLASH:-}" ]; then
    case "$GAUT_WORKSPACE_FLASH" in
        *,*)
            echo "The QEMU workspace flash path cannot contain a comma." >&2
            exit 2
            ;;
    esac
    if [ ! -e "$GAUT_WORKSPACE_FLASH" ]; then
        if ! command -v truncate >/dev/null 2>&1; then
            echo "truncate is required to create the QEMU flash backing file." >&2
            exit 2
        fi
        truncate -s 64m "$GAUT_WORKSPACE_FLASH"
    fi
    FLASH_SIZE=$(/usr/bin/wc -c < "$GAUT_WORKSPACE_FLASH" | /usr/bin/tr -d ' ')
    if [ "$FLASH_SIZE" -ne 67108864 ]; then
        echo "The Gaut workspace flash must occupy exactly 67108864 bytes." >&2
        exit 2
    fi

    qemu-system-aarch64 \
        -machine virt,virtualization=off,pflash1=workspace-flash \
        -blockdev driver=file,filename="$GAUT_WORKSPACE_FLASH",node-name=workspace-file \
        -blockdev driver=raw,file=workspace-file,node-name=workspace-flash \
        -cpu cortex-a53 \
        -m 128M \
        -display none \
        -monitor none \
        -serial stdio \
        -nic none \
        -no-reboot \
        -kernel "$COMPILER" \
        < "$STREAM"
else
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
        < "$STREAM"
fi
