#!/usr/bin/env bash
set -euo pipefail

# One-command setup for MONAD. Installs every prerequisite the build and dev
# loops need. Safe to re-run — each step is skipped if already satisfied.

echo "==> SETUP  MONAD toolchain"

# 1. Rust + required compile targets
if ! command -v rustc >/dev/null 2>&1; then
  echo "    [error] Rust not found. Install it from https://rustup.rs and re-run."
  exit 1
fi
echo "    rustc      $(rustc --version | awk '{print $2}')"

echo "    targets    wasm32-unknown-unknown, wasm32-wasip1"
rustup target add wasm32-unknown-unknown wasm32-wasip1 >/dev/null 2>&1 || true

# 2. wasm-pack (firmware → browser wasm)
if command -v wasm-pack >/dev/null 2>&1; then
  echo "    wasm-pack  present"
else
  echo "    wasm-pack  installing..."
  cargo install wasm-pack
fi

# 3. cargo-watch (used by `make dev`)
if command -v cargo-watch >/dev/null 2>&1; then
  echo "    cargo-watch present"
else
  echo "    cargo-watch installing..."
  cargo install cargo-watch
fi

# 4. Node toolchain check + console deps (includes the ajv validator)
if ! command -v node >/dev/null 2>&1; then
  echo "    [error] Node.js not found. Install Node 20.19+ (or 22.12+) and re-run."
  exit 1
fi
echo "    node       $(node --version)  (Vite 8 needs ^20.19 or >=22.12)"
echo "    console    npm install..."
( cd console && npm install --silent )

echo ""
echo "    ✓ setup complete."
echo "      next:  make dev     (live console at http://localhost:5173)"
echo "         or:  make         (build the static site into dist/)"
