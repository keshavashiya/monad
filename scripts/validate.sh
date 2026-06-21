#!/usr/bin/env bash
set -euo pipefail

# Validate every vault JSON file against vault/schema.json.
# Uses the ajv library (installed locally via `make setup`); falls back to a
# plain JSON parse check if the dependency isn't installed.

SCHEMA_DIR="vault"

echo "==> VALIDATE  vault/*.json"

# Schema validation needs Node + the locally-installed ajv library.
if command -v node >/dev/null 2>&1 && [ -d "console/node_modules/ajv" ]; then
  node console/scripts/validate.mjs
else
  # Fallback: structural JSON check only. Not an error — schema validation is
  # an enhancement enabled by `make setup`.
  echo "    note: ajv not installed; JSON parse check only (run 'make setup' for schema validation)."
  for f in "$SCHEMA_DIR"/*.json; do
    [ "$f" = "$SCHEMA_DIR/schema.json" ] && continue
    python3 -m json.tool "$f" >/dev/null || {
      echo "    FAILED: $f is not valid JSON"
      exit 1
    }
  done
  echo "    all vault files are valid JSON."
fi
