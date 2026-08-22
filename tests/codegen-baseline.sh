#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
COMPILER=${GAUT_OS_IMAGE:-"$ROOT/dist/gaut-os.img"}

if ! command -v qemu-system-aarch64 >/dev/null 2>&1; then
    echo "qemu-system-aarch64 is required for code-generation baseline verification." >&2
    exit 2
fi

TEST_TEMP=$(/usr/bin/mktemp -d "${TMPDIR:-/tmp}/gaut-codegen-baseline.XXXXXX")
cleanup() {
    /bin/rm -rf "$TEST_TEMP"
}
trap cleanup EXIT INT TERM

EXPECTED="$ROOT/tests/fixtures/codegen-baseline/EXPECTED.tsv"
ACTUAL="$TEST_TEMP/actual.tsv"

/usr/bin/printf 'program\timage_bytes\tused_bytes\tinstructions\tevaluation_pushes\tevaluation_pops\n' > "$ACTUAL"

for PROGRAM in constant binary call memory; do
    IMAGE="$TEST_TEMP/$PROGRAM.elf"
    "$ROOT/os/build.sh" \
        "$COMPILER" \
        linux \
        "$ROOT/tests/fixtures/codegen-baseline/$PROGRAM.gaut" \
        "$IMAGE" >/dev/null

    IMAGE_BYTES=$(/usr/bin/wc -c < "$IMAGE" | /usr/bin/tr -d ' ')
    USED_BYTES=$(
        /usr/bin/od -An -tu1 -v "$IMAGE" \
            | /usr/bin/awk '{ for (field = 1; field <= NF; field++) { offset++; if ($field != 0) last = offset } } END { print last + 0 }'
    )
    INSTRUCTIONS=$(((USED_BYTES - 120) / 4))
    EVALUATION_PUSHES=$(
        /usr/bin/od -An -tu4 -v "$IMAGE" \
            | /usr/bin/awk '{ for (field = 1; field <= NF; field++) if ($field == 2432705172) count++ } END { print count + 0 }'
    )
    EVALUATION_POPS=$(
        /usr/bin/od -An -tu4 -v "$IMAGE" \
            | /usr/bin/awk '{ for (field = 1; field <= NF; field++) if ($field == 3506446996) count++ } END { print count + 0 }'
    )

    /usr/bin/printf '%s\t%s\t%s\t%s\t%s\t%s\n' \
        "$PROGRAM" "$IMAGE_BYTES" "$USED_BYTES" "$INSTRUCTIONS" \
        "$EVALUATION_PUSHES" "$EVALUATION_POPS" >> "$ACTUAL"
done

if ! /usr/bin/cmp -s "$EXPECTED" "$ACTUAL"; then
    echo "Gaut code-generation baseline changed." >&2
    /usr/bin/diff -u "$EXPECTED" "$ACTUAL" >&2 || true
    exit 1
fi

echo "Gaut code-generation size and evaluation-stack baseline passed."
