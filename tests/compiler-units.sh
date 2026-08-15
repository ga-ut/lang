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

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-compiler-units.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

REQUEST="$TEST_TEMP/compiler.request"
"$ROOT/gaut/request.sh" gaut-os "$SOURCE_ROOT" "$REQUEST"

UNIT_COUNT=$(/usr/bin/od -An -tu8 -j24 -N8 "$REQUEST" | /usr/bin/tr -d ' ')
if [ "$UNIT_COUNT" -ne 7 ]; then
    echo "Expected 7 compiler source units, got $UNIT_COUNT." >&2
    exit 1
fi

echo "Compiler source-unit layout passed."
