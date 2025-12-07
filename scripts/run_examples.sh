#!/usr/bin/env bash
set -euo pipefail

ROOT=$(cd -- "$(dirname "$0")/.." && pwd)

examples=(hello calc record)

for ex in "${examples[@]}"; do
  echo "==> interp $ex"
  cargo test -p interp -- tests::$ex -- --nocapture >/dev/null || true
done

echo "==> cgen sample build (calc)"
calc_c="$ROOT/target/calc.c"
cargo test -p cgen -- tests::simple_program -- --nocapture >/dev/null
cargo test >/dev/null

echo "Done"
