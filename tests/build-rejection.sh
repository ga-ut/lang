#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for build-rejection verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-build-rejection.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

SOURCE="$TEST_TEMP/invalid.gaut"
OUTPUT="$TEST_TEMP/invalid.elf"
ERROR="$TEST_TEMP/error"
/usr/bin/printf 'fn main() { return missing(); }\n' > "$SOURCE"

if "$ROOT/os/build.sh" "$COMPILER" linux "$SOURCE" "$OUTPUT" > /dev/null 2> "$ERROR"; then
    echo "The build adapter accepted a Gaut rejection diagnostic as an artifact." >&2
    exit 1
fi
if [ -e "$OUTPUT" ]; then
    echo "The rejected build left an output artifact." >&2
    exit 1
fi
if ! /usr/bin/grep -q 'non-artifact payload' "$ERROR"; then
    echo "The build adapter did not report its framing rejection." >&2
    /bin/cat "$ERROR" >&2
    exit 1
fi

echo "The build adapter rejected a diagnostic payload without replacing output."
