#!/usr/bin/env bash
set -uo pipefail

# Report on each prerequisite without changing anything. Exits non-zero if a
# required tool is missing, so it doubles as a CI / pre-build sanity check.

echo "==> DOCTOR  MONAD prerequisites"
missing=0

check() { # name, command, hint
  local name="$1" cmd="$2" hint="$3"
  if command -v "$cmd" >/dev/null 2>&1; then
    printf "    \033[38;5;113m✓\033[0m %-12s %s\n" "$name" "$($cmd --version 2>/dev/null | head -1)"
  else
    printf "    \033[38;5;131m✗\033[0m %-12s missing — %s\n" "$name" "$hint"
    missing=1
  fi
}

check "rustc"       rustc       "install from https://rustup.rs"
check "cargo"       cargo       "comes with rustup"
check "wasm-pack"   wasm-pack   "cargo install wasm-pack  (or: make setup)"
check "cargo-watch" cargo-watch "cargo install cargo-watch (or: make setup) — needed for 'make dev'"
check "node"        node        "install Node 20.19+ or 22.12+ (Vite 8 requirement)"
check "npm"         npm         "comes with Node"

# Rust targets
echo "    rust targets:"
for t in wasm32-unknown-unknown wasm32-wasip1; do
  if rustup target list --installed 2>/dev/null | grep -qx "$t"; then
    printf "      \033[38;5;113m✓\033[0m %s\n" "$t"
  else
    printf "      \033[38;5;131m✗\033[0m %s — rustup target add %s\n" "$t" "$t"
    missing=1
  fi
done

echo ""
if [ "$missing" -eq 0 ]; then
  echo "    all prerequisites present."
else
  echo "    some prerequisites are missing — run: make setup"
  exit 1
fi
