#!/usr/bin/env bash
set -euo pipefail

# Build the entire MONAD system image.
# Usage: ./build.sh [dev]

MODE="${1:-release}"

echo "==> MONAD Build"
echo "    mode: $MODE"
echo ""

# Step 1: Validate vault
./scripts/validate.sh

# Step 2: Build firmware (Rust → WASM)
echo "==> COMPILE   firmware (wasm32-unknown-unknown)"
pushd firmware >/dev/null
if [ "$MODE" = "dev" ]; then
  wasm-pack build --target web --out-name monad --dev --no-opt
else
  wasm-pack build --target web --out-name monad --release --no-opt
fi
popd >/dev/null
echo "    firmware/pkg/monad.wasm ready"

# Step 3: Build console adapter
echo "==> ASSEMBLE  console adapter"
pushd console >/dev/null
npm ci --silent
npm run build
popd >/dev/null
echo "    console/dist/ ready"

# Step 4: Assemble system image
echo "==> ASSEMBLE  system image"
mkdir -p dist
cp -r console/dist/* dist/
cp firmware/pkg/monad_bg.wasm dist/monad.wasm
echo "    dist/ assembled"

echo ""
echo "==> Build complete. System image in dist/"
echo "    Deploy dist/ to any static host."
