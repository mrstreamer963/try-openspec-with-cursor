#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRATE="$ROOT/packages/game-core"
OUT="$ROOT/packages/client/src/wasm"
export CARGO_TARGET_DIR="$ROOT/target"
PROFILE="release"

if [[ "${1:-}" == "--dev" ]]; then
  PROFILE="dev"
fi

mkdir -p "$OUT"

if [[ "$PROFILE" == "dev" ]]; then
  cargo build --manifest-path "$CRATE/Cargo.toml" --target wasm32-unknown-unknown
  WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/debug/game_core.wasm"
else
  cargo build --manifest-path "$CRATE/Cargo.toml" --target wasm32-unknown-unknown --release
  WASM="$CARGO_TARGET_DIR/wasm32-unknown-unknown/release/game_core.wasm"

  if command -v wasm-opt >/dev/null 2>&1; then
    wasm-opt -Oz "$WASM" -o "$WASM"
  fi
fi

wasm-bindgen "$WASM" \
  --out-dir "$OUT" \
  --target web \
  --no-typescript
