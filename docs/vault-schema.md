# The MONAD Vault — a forkable identity protocol

The **vault** is the only thing that makes a MONAD *yours*. The kernel is
generic; the vault is the person. Fork the repo, replace `vault/profile.json`
with your own, rebuild, and you have your own compiled identity kernel — website,
CLI, WASI binary, and MCP server included. MIT-licensed.

This document is the contract. The machine-checkable version is
[`vault/schema.json`](../vault/schema.json) (JSON Schema draft-07); this page is
the human explanation.

## Make it your own

```bash
git clone https://github.com/keshavashiya/monad   # or your fork
$EDITOR vault/profile.json                          # write yourself in
make            # build the website  → dist/
make wasi       # build npx/wasmtime  → npm/monad.wasm
make verify     # (optional) print the reproducible vault hash
```

Nothing else needs editing. No Rust string hardcodes identity — every fact comes
from the vault.

## Required top-level keys

| Key | Type | Meaning |
|---|---|---|
| `meta` | object | Kernel name + build note (the version is not here — it comes from the kernel crate, so it can't drift) |
| `identity` | object | Name, handle, title, optional location/timezone |
| `roles` | array | Work history (title, org, tenure, focus, optional url) |
| `stacks` | array | Technologies (name, proficiency, years) |
| `systems` | array | Things you've built (name, architecture, description) |
| `projects` | array | Projects (name, status, description, optional url) |
| `links` | object | email + github required; linkedin/twitter/website optional |
| `filesystem` | object | Virtual FS: keys are absolute paths, values are file/dir entries |
| `processes` | array | Fictional process table (pid, ppid, command, state) |
| `bootlog` | array | Cinematic boot lines (timestamp `[ d.dddddd]`, message) |
| `fortune` | array | Strings shown by the `fortune` command |

### Enumerations the schema enforces

- `stacks[].proficiency`: `expert` · `advanced` · `proficient` · `familiar`
- `projects[].status`: `active` · `inert` · `frozen`
- `processes[].state`: `running` · `idle` · `sleeping` · `stopped`
- `filesystem` keys: must start with `/`; entry `type` is `file` or `dir`

See `vault/schema.json` for the full set of fields and patterns.

## Verifiability

At build time the kernel embeds a reproducible **SHA-256 of `vault/profile.json`**
(see [`kernel/build.rs`](../kernel/build.rs)). Anyone can recompute it with
standard tools and compare against what the running kernel reports:

```bash
shasum -a 256 vault/profile.json    # macOS
sha256sum    vault/profile.json     # Linux
# compare to the hash printed by:  monad verify   (or the `verify` command)
```

If the hex matches, the identity you are interacting with is exactly the
committed source — no tampering, no hidden server rewriting the answers.

## What the kernel does *not* allow

- No secrets: the vault is public identity data only. No credentials, tokens, keys.
- No writes: the virtual filesystem is read-only.
- No network in the core: all data is compiled in. Adapters add I/O, never the kernel.

## Adapters (you get all of these for free)

| Adapter | Command |
|---|---|
| Website | `make` → deploy `dist/` to any static host |
| CLI | `monad whoami`, `monad repl` |
| npx | `npx <your-handle>` (publish `npm/`) |
| WASI | `wasmtime monad.wasm whoami` |
| MCP | `claude mcp add me -- npx -y <your-handle> mcp` |

One vault. One kernel. Every interface.
