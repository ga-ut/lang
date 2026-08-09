#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
WORKSPACE="$ROOT/runtime/workspace.img"

if [ -e "$WORKSPACE" ]; then
    echo "Workspace already exists: $WORKSPACE"
    /bin/ls -lh "$WORKSPACE"
    /usr/bin/du -h "$WORKSPACE"
    exit 0
fi

/bin/mkdir -p "$ROOT/runtime"
/usr/bin/truncate -s 268435456 "$WORKSPACE"

DEVICE=$(/usr/bin/hdiutil attach -nomount "$WORKSPACE" | /usr/bin/awk 'NR==1{print $1}')
cleanup() {
    /usr/bin/hdiutil detach "$DEVICE" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

/usr/sbin/diskutil eraseDisk FAT32 OURWORK MBRFormat "$DEVICE" >/dev/null
/usr/bin/hdiutil detach "$DEVICE" >/dev/null
trap - EXIT INT TERM

echo "Created a 256 MB logical sparse workspace. Current physical use:"
/usr/bin/du -h "$WORKSPACE"
