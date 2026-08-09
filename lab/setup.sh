#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
RUNTIME="$ROOT/runtime"
ALPINE_VERSION="3.24.1"
RELEASE_URL="https://dl-cdn.alpinelinux.org/alpine/v3.24/releases/aarch64"
ISO_NAME="alpine-virt-$ALPINE_VERSION-aarch64.iso"
KERNEL_GZIP_OFFSET="51832"

mkdir -p "$RUNTIME"

if [ ! -f "$RUNTIME/Image" ] || [ ! -f "$RUNTIME/initramfs-virt" ]; then
    TEMP_DIR=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/arm64-lab.XXXXXX")
    ISO_PATH="$TEMP_DIR/$ISO_NAME"
    CHECKSUM_PATH="$TEMP_DIR/$ISO_NAME.sha256"

    cleanup() {
        /bin/rm -rf "$TEMP_DIR"
        /bin/rm -f "$RUNTIME/vmlinuz-virt.part"
    }
    trap cleanup EXIT INT TERM

    /bin/rm -f "$RUNTIME/vmlinuz-virt.part"
    /usr/bin/curl -fL --retry 3 -o "$ISO_PATH" "$RELEASE_URL/$ISO_NAME"
    /usr/bin/curl -fsSL -o "$CHECKSUM_PATH" "$RELEASE_URL/$ISO_NAME.sha256"
    expected=$(/usr/bin/awk '{print $1}' "$CHECKSUM_PATH")
    actual=$(/usr/bin/shasum -a 256 "$ISO_PATH" | /usr/bin/awk '{print $1}')
    if [ "$expected" != "$actual" ]; then
        echo "Checksum mismatch for $ISO_NAME" >&2
        exit 1
    fi

    /bin/mkdir -p "$TEMP_DIR/extracted"
    /usr/bin/tar -xf "$ISO_PATH" -C "$TEMP_DIR/extracted" \
        boot/vmlinuz-virt \
        boot/initramfs-virt
    /bin/cp "$TEMP_DIR/extracted/boot/vmlinuz-virt" "$RUNTIME/vmlinuz-virt"
    /bin/cp "$TEMP_DIR/extracted/boot/initramfs-virt" "$RUNTIME/initramfs-virt"

    /bin/dd if="$RUNTIME/vmlinuz-virt" bs=1 skip="$KERNEL_GZIP_OFFSET" 2>/dev/null \
        | /usr/bin/gzip -dc > "$RUNTIME/Image.part" 2>/dev/null || true
    if ! /usr/bin/file "$RUNTIME/Image.part" | /usr/bin/grep -q "Linux kernel ARM64 boot executable Image"; then
        echo "Unable to extract the ARM64 kernel image" >&2
        exit 1
    fi
    /bin/mv "$RUNTIME/Image.part" "$RUNTIME/Image"
    /bin/rm -f "$RUNTIME/vmlinuz-virt"
    trap - EXIT INT TERM
    /bin/rm -rf "$TEMP_DIR"
fi

/usr/bin/xcrun swiftc \
    -O \
    -framework Virtualization \
    "$ROOT/VirtualMachine.swift" \
    -o "$RUNTIME/arm64-lab"

/usr/bin/codesign \
    --force \
    --sign - \
    --entitlements "$ROOT/virtualization.entitlements" \
    "$RUNTIME/arm64-lab"

echo "ARM64 lab is ready. Run: $ROOT/run.sh"
