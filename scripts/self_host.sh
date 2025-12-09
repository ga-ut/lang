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

echo "C output and binaries in $OUT"
