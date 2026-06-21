#!/usr/bin/env bash
set -euo pipefail

# Development environment — concurrent watchers for firmware and console.
# Run `make setup` first to install prerequisites (wasm-pack, cargo-watch).

echo "==> MONAD Development Environment"
echo ""

# Fail fast with an actionable message if the watcher tool is missing, rather
# than letting cargo print a cryptic "no such command: watch".
if ! command -v cargo-watch >/dev/null 2>&1; then
  echo "    [error] cargo-watch is not installed (needed for the firmware watcher)."
  echo "            run:  make setup     (or: cargo install cargo-watch)"
  exit 1
fi
if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "    [error] wasm-pack is not installed."
  echo "            run:  make setup     (or: cargo install wasm-pack)"
  exit 1
fi

# Validate vault first.
./scripts/validate.sh

# Initial firmware build so the console has something to load immediately.
echo "==> COMPILE   firmware (dev)"
( cd firmware && wasm-pack build --target web --out-name monad --dev --no-opt )

# Ensure console deps are present.
( cd console && npm install --silent )

echo ""
echo "==> Starting watchers (Ctrl+C stops everything)..."
echo "    firmware: cargo-watch + wasm-pack"
echo "    console:  vite dev server on http://localhost:5173"
echo ""

# Tear down every child process on exit/interrupt so nothing is left running.
cleanup() {
  trap - INT TERM EXIT
  echo ""
  echo "==> Stopping development environment..."
  kill 0 2>/dev/null || true
}
trap cleanup INT TERM EXIT

# Firmware watcher (background).
( cd firmware && cargo watch -w src/ -w ../vault/ -w ../kernel/src/ \
    -s "wasm-pack build --target web --out-name monad --dev --no-opt" ) &

# Vite dev server (foreground). When it exits (or Ctrl+C), the trap fires and
# kills the watcher too.
( cd console && npx vite dev --host )
