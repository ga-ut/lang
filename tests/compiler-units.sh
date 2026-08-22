#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
SOURCE_ROOT="$ROOT/gaut/compiler"

if [ ! -d "$SOURCE_ROOT" ]; then
    echo "Gaut compiler source is not split into source units." >&2
    exit 1
fi

for UNIT in compiler emitter lexer parser request storage symbols; do
    if [ ! -f "$SOURCE_ROOT/$UNIT.gaut" ]; then
        echo "Missing compiler source unit: $UNIT.gaut" >&2
        exit 1
    fi
done

for ADAPTER in linux gaut-os; do
    if [ ! -f "$ROOT/gaut/adapters/$ADAPTER/platform.gaut" ]; then
        echo "Missing platform adapter: $ADAPTER" >&2
        exit 1
    fi
done
if [ ! -f "$ROOT/gaut/adapters/common/verification.gaut" ]; then
    echo "Missing common verification module." >&2
    exit 1
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-compiler-units.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

REQUEST="$TEST_TEMP/compiler.request"
"$ROOT/gaut/request.sh" gaut-os "$SOURCE_ROOT" "$REQUEST"

UNIT_COUNT=$(/usr/bin/od -An -tu8 -j24 -N8 "$REQUEST" | /usr/bin/tr -d ' ')
if [ "$UNIT_COUNT" -ne 9 ]; then
    echo "Expected 7 compiler units, 1 platform adapter, and 1 verification module, got $UNIT_COUNT." >&2
    exit 1
fi

for MODE in workspace-test workspace-fixed-point; do
    WORKSPACE_REQUEST="$TEST_TEMP/$MODE.request"
    "$ROOT/gaut/request.sh" "$MODE" - "$WORKSPACE_REQUEST"
    REQUEST_SIZE=$(/usr/bin/wc -c < "$WORKSPACE_REQUEST" | /usr/bin/tr -d ' ')
    COMMAND=$(/usr/bin/od -An -tu8 -j16 -N8 "$WORKSPACE_REQUEST" | /usr/bin/tr -d ' ')
    if [ "$REQUEST_SIZE" -ne 24 ] || [ "$COMMAND" -ne 6 ]; then
        echo "$MODE must be one 24-byte command 6 request without source bytes." >&2
        exit 1
    fi
done

echo "Compiler source-unit layout passed."
