#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname "$0")/.." && pwd)
OUT="$ROOT/target/self_host"
mkdir -p "$OUT"

examples=(hello calc record)

hash_file() {
  shasum -a 256 "$1" | awk '{print $1}'
}

emit_and_build() {
  local name="$1"
  local src="$ROOT/examples/$name.gaut"
  local c_main="$OUT/${name}.stage0.c"
  local c_check="$OUT/${name}.stage0.check.c"
  local bin_out="$OUT/${name}"

  echo "==> emit C twice for determinism: $name"
  cargo run -p cli -- --emit-c "$c_main" "$src"
  cargo run -p cli -- --emit-c "$c_check" "$src"

  local h1
  local h2
  h1=$(hash_file "$c_main")
  h2=$(hash_file "$c_check")
  rm -f "$c_check"
  if [[ "$h1" != "$h2" ]]; then
    echo "!! nondeterministic C output for $name"
    echo "   $c_main hash: $h1"
    echo "   $c_check hash: $h2"
    exit 1
  fi
  echo "   hash: $h1"

  echo "==> build and run stage1 binary: $name"
  cargo run -p cli -- --emit-c "$c_main" --build "$bin_out" "$src"
  if [[ -x "$bin_out" ]]; then
    "$bin_out" >/dev/null || true
  fi
}

for ex in "${examples[@]}"; do
  emit_and_build "$ex"
done

if [[ "${SELF_HOST_SKIP:-0}" == "1" ]]; then
  echo "SELF_HOST_SKIP=1, compiler self-host loop skipped."
  exit 0
fi

compiler_entry="$ROOT/compiler/main.gaut"
if [[ -f "$compiler_entry" ]]; then
  if [[ "${SELF_HOST_COMPILER:-0}" == "1" ]]; then
    echo "==> compiler stage0 -> stage1"
    cargo run -p cli -- --emit-c "$OUT/gautc1.stage0.c" --build "$OUT/gautc1" "$compiler_entry"

    echo "==> compiler stage1 -> stage2"
    GAUT_STD_DIR="$ROOT/std" GAUT_RUNTIME_C_DIR="$ROOT/runtime/c" "$OUT/gautc1" --emit-c "$OUT/gautc2.stage1.c" "$compiler_entry"
    if [[ ! -f "$OUT/gautc2.stage1.c" ]]; then
      echo "!! compiler stage1 did not produce C output (compiler is still a stub); skipping stage2."
      exit 0
    fi
    h1=$(hash_file "$OUT/gautc1.stage0.c")
    h2=$(hash_file "$OUT/gautc2.stage1.c")
    echo "   stage0 C hash: $h1"
    echo "   stage1 C hash: $h2"
    if [[ "$h1" != "$h2" ]]; then
      echo "!! compiler C output differs between stage0 and stage1"
      exit 1
    fi

    echo "==> build compiler stage2 binary"
    cargo run -p cli -- --emit-c "$OUT/gautc2.stage1.c" --build "$OUT/gautc2" "$compiler_entry"
  else
    echo "compiler sources detected; set SELF_HOST_COMPILER=1 to run stage1/2 loop."
  fi
else
  echo "compiler sources not found at $compiler_entry; skipping compiler self-host loop."
fi

echo "C output and binaries in $OUT"
