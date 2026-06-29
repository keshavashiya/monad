#!/usr/bin/env node
'use strict';

// MONAD — npx launcher.
//
// Boots the compiled identity kernel (a WASI build of monad-kernel) using
// Node's built-in WASI runtime. No native binary, no wasmtime, no toolchain:
// `npx keshavashiya` just runs. The exact same monad.wasm also runs under
// `wasmtime monad.wasm`.
//
//   npx keshavashiya            interactive session (or a command, e.g. whoami)
//   npx keshavashiya whoami     one-shot command
//   npx keshavashiya mcp        run as an MCP server over stdio (for AI agents)

const { readFileSync } = require('node:fs');
const { join } = require('node:path');

// Quiet Node's experimental WASI warning for a clean terminal. Re-exec once
// with --no-warnings if it is not already set.
if (!process.env.MONAD_REEXEC) {
  const { spawnSync } = require('node:child_process');
  const r = spawnSync(
    process.execPath,
    ['--no-warnings', __filename, ...process.argv.slice(2)],
    { stdio: 'inherit', env: { ...process.env, MONAD_REEXEC: '1' } }
  );
  process.exit(r.status === null ? 1 : r.status);
}

async function main() {
  let WASI;
  try {
    ({ WASI } = require('node:wasi'));
  } catch (e) {
    console.error('MONAD requires Node 20+ with WASI support. Detected: ' + process.version);
    process.exit(1);
  }

  // WASI has no TIOCGWINSZ, so the kernel can't read the terminal width itself;
  // and shells don't export $COLUMNS. Node knows the width, so pass it in as
  // COLUMNS (the kernel reads it) — without this, tables fall back to a narrow
  // fixed layout that truncates. Omitted when stdout isn't a TTY (piped).
  const env = { ...process.env };
  if (process.stdout.isTTY && process.stdout.columns) {
    env.COLUMNS = String(process.stdout.columns);
  }

  const wasi = new WASI({
    version: 'preview1',
    args: ['monad', ...process.argv.slice(2)],
    env,
    returnOnExit: true,
  });

  const bytes = readFileSync(join(__dirname, 'monad.wasm'));
  const module = await WebAssembly.compile(bytes);
  const instance = await WebAssembly.instantiate(module, wasi.getImportObject());
  const code = wasi.start(instance);
  process.exit(typeof code === 'number' ? code : 0);
}

main().catch((e) => {
  console.error(e && e.message ? e.message : e);
  process.exit(1);
});
