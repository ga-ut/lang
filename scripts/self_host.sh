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
    strict="${SELF_HOST_COMPILER_STRICT:-0}"
    runtime_dir="${GAUT_RUNTIME_C_DIR:-$ROOT/runtime/c}"

    stage0_c="$OUT/gautc1.stage0.c"
    stage1_bin="$OUT/gautc1"
    stage1_c="$OUT/gautc2.stage1.c"
    stage2_bin="$OUT/gautc2"
    stage2_c="$OUT/gautc3.stage2.c"

    echo "==> compiler stage0 -> stage1 (Rust -> C -> clang)"
    cargo run -p cli -- --emit-c "$stage0_c" --build "$stage1_bin" "$compiler_entry"

    echo "==> compiler stage1 -> stage2 (gautc1 -> C)"
    GAUT_STD_DIR="$ROOT/std" GAUT_RUNTIME_C_DIR="$runtime_dir" "$stage1_bin" --emit-c "$stage1_c" "$compiler_entry"
    if [[ ! -f "$stage1_c" ]]; then
      echo "!! compiler stage1 did not produce C output (compiler is still a stub); skipping stage2."
      exit 0
    fi

    h0=$(hash_file "$stage0_c")
    h1=$(hash_file "$stage1_c")
    echo "   stage0 C hash: $h0"
    echo "   stage1 C hash: $h1"
    if [[ "$h0" != "$h1" ]]; then
      echo "!! compiler C output differs between stage0 and stage1"
      if [[ "$strict" == "1" ]]; then
        exit 1
      fi
    fi

    echo "==> build compiler stage2 binary (clang stage1 C)"
    if ! clang -std=gnu11 -O2 -I "$runtime_dir" "$stage1_c" "$runtime_dir/runtime.c" -o "$stage2_bin"; then
      echo "!! failed to build stage2 compiler (stage1 output is not valid C yet)"
      if [[ "$strict" == "1" ]]; then
        exit 1
      fi
      exit 0
    fi

    echo "==> compiler stage2 -> stage3 (gautc2 -> C)"
    GAUT_STD_DIR="$ROOT/std" GAUT_RUNTIME_C_DIR="$runtime_dir" "$stage2_bin" --emit-c "$stage2_c" "$compiler_entry"
    if [[ ! -f "$stage2_c" ]]; then
      echo "!! compiler stage2 did not produce C output; skipping stage3 hash check."
      exit 0
    fi

    h2=$(hash_file "$stage2_c")
    echo "   stage2 C hash: $h2"
    if [[ "$h1" != "$h2" ]]; then
      echo "!! compiler C output differs between stage1 and stage2"
      if [[ "$strict" == "1" ]]; then
        exit 1
      fi
    fi
  else
    echo "compiler sources detected; set SELF_HOST_COMPILER=1 to run stage1/2 loop."
  fi
else
  echo "compiler sources not found at $compiler_entry; skipping compiler self-host loop."
fi

echo "C output and binaries in $OUT"
