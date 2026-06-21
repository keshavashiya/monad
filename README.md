# MONAD — A Compiled Identity Kernel

> A monad is an indivisible unit of being. This one is mine.

MONAD is a compiled identity kernel: a single Rust core that **AI agents can interview**, that **runs anywhere** from one binary, and that **remembers its visitors on their own machine — never on a server**. It is compiled, not served. There is no database, no required backend — only a kernel and its embedded vault.

The browser page is not "a website with a terminal theme." It is one **adapter** over the kernel — the same core also runs on the CLI (`npx keshavashiya`), in `wasmtime`, and as an MCP server an AI agent can connect to.

> One core, many protocols — the same local-first, memory-first philosophy behind [`brain`](https://github.com/keshavashiya/brain), [`docify`](https://github.com/keshavashiya/docify), and [`codesage`](https://github.com/keshavashiya/codesage). Free and open source; runs on any static host.

## Quick Start

```bash
make setup     # install all prerequisites (rust targets, wasm-pack,
               # cargo-watch, console deps + vault validator) — idempotent
make dev       # firmware watcher + vite dev server → http://localhost:5173

# or, for a one-off static build:
make build     # assemble the site into dist/
```

Requires [Rust](https://rustup.rs) and [Node 20.19+](https://nodejs.org) on your
`PATH`; `make setup` installs everything else. Run `make doctor` any time to
check what's missing, or `make` (no target) to list every command.

## Project Structure

```
monad/
├── kernel/       # Pure Rust core — all logic, host-agnostic
├── firmware/     # Thin wasm-bindgen shim (browser adapter)
├── console/      # Web serial adapter (Vite + TypeScript)
├── cli/          # Native + WASI binary; MCP server; npx launcher
├── vault/        # Compiled-in identity data (JSON)
├── daemon/       # Optional HTTP server over the kernel (self-hosted)
├── lib/          # Shared cross-crate data structures
├── scripts/      # Build and dev tooling
└── docs/         # Forkable vault-schema protocol
```

## Architecture

```
vault/profile.json ──include_str!──▶ kernel/ (pure Rust, host-agnostic)
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    │                     │                     │
              firmware/ (wasm)      cli/ (native+WASI)     daemon/ (opt)
                    │                     │                     │
              console/ (TS)         npx keshavashiya       HTTP /execute
                    │                     │
              TTY renderer          stdout / MCP
```

**Key constraint**: adapters never read vault data directly. All logic lives in the `kernel` crate; each adapter only injects a `Host` (clock/rand) and renders what `execute()` returns. The terminal is a display driver.

## The Vault

The `vault/profile.json` file is the **root filesystem** of MONAD. It defines everything the kernel knows: identity, roles, stacks, systems, projects, the virtual filesystem, processes, boot log, and fortunes.

The schema is versioned at `vault/schema.json` and documented as a **forkable
protocol** in [docs/vault-schema.md](docs/vault-schema.md). Anyone can fork
MONAD, write their own `profile.json`, and build their own identity kernel —
website, CLI, WASI binary, and MCP server included.

## Verifiable identity

Each build embeds a **reproducible SHA-256 of the vault**, computed at build time
([`kernel/build.rs`](kernel/build.rs)). Recompute it with standard tools and
compare against what the running kernel reports — if they match, the identity you
are talking to is exactly the committed source:

```bash
monad verify                        # or the `verify` command in the console
shasum -a 256 vault/profile.json    # macOS  (sha256sum on Linux)
```

## Run it without a browser

The kernel is the artifact; the website is just one adapter. The same core runs
over npm, in any WASM runtime, and natively — no Rust toolchain required for the
first two:

```bash
npx keshavashiya              # boots the kernel via Node's built-in WASI
npx keshavashiya whoami       # one-shot command
wasmtime monad.wasm whoami    # the exact same binary, standalone

cargo run -p monad-cli -- repl   # native interactive session
```

`npx keshavashiya` runs the bundled `monad.wasm` through Node's WASI runtime —
no native binary, no `wasmtime`, no install. (v4 of this package was a static
business card; v5 boots the real kernel.)

## Let an AI agent interview me (MCP)

MONAD ships as an [MCP](https://modelcontextprotocol.io) server, so the tools
that increasingly screen candidates can query the source of truth directly. It
runs over stdio on the *agent's* machine — no server to host.

```bash
claude mcp add keshav -- npx -y keshavashiya mcp
```

Exposed tools: `get_experience`, `query_systems`, `list_projects`, `ask`.

## Self-host it as an HTTP service (optional)

The `daemon` is the same kernel behind a tiny HTTP adapter — `std::net` only, no
async runtime, no `monad.wasm` to load. Optional and self-hosted; the core
experience never depends on a server.

```bash
make daemon                 # build ./target/release/monad-daemon
./target/release/monad-daemon            # bind 127.0.0.1:7373 (localhost)
./target/release/monad-daemon 0.0.0.0:7373   # expose it

curl -s localhost:7373/execute -d '{"input":"whoami"}'
# → {"input":"whoami","output":"keshavashiya"}
```

`GET /` returns health + usage; `POST /execute` runs one command against a fresh,
stateless kernel. Responses carry permissive CORS, so a browser can call it too.

## Commands

### System
`whoami`, `id`, `uname`, `uptime`, `date`, `neofetch`, `ps`, `top`, `dmesg`, `clear`

### Profile
`roles`, `stacks`, `systems`, `projects`

### Filesystem
`ls`, `cat`, `tree`, `pwd`, `cd`, `find`

### Contact
`links`, `pgp`, `resume`, `meeting`

### Meta
`help`, `hint`, `fortune`, `history`, `exit`, `logout`

### Client memory (browser only — *the kernel forgets; your machine remembers*)
`memory` — show what *your* browser remembers about your visits · `forget` — erase it.
The kernel is stateless; this state lives only in your browser's IndexedDB (with a
localStorage fallback) and is **never sent to a server**. Returning visitors get a
"welcome back" and skip the host-key prompt.

## Build

```bash
make setup     # install prerequisites (run once)
make doctor    # check prerequisites without installing
make           # list all targets
make build     # validate + firmware + console → dist/ (the website)
make wasi      # build monad.wasm → npm/ (for npx keshavashiya / wasmtime)
make cli       # build the native `monad` binary
make daemon    # build the self-hosted HTTP server (POST /execute)
make dev       # concurrent watchers + vite dev server
make clean     # remove build artifacts
```

Output of `make` is a fully static `dist/` folder deployable to GitHub Pages,
Cloudflare Pages, Netlify, or any HTTP server.

To publish the npm launcher (after `make wasi` produces `npm/monad.wasm`):

```bash
npm publish ./npm   # publishes the `keshavashiya` package
```

## Deployment

```bash
make build
npx gh-pages -d dist
```

## CI/CD

GitHub Actions builds everything automatically:

| Workflow | Trigger | What it does |
|---|---|---|
| `.github/workflows/ci.yml` | every push / PR | tests the kernel, validates the vault, builds **`dist/`** and the **npm package** (incl. a WASI smoke test), and uploads both as artifacts |
| `.github/workflows/deploy.yml` | push to `main` | builds `dist/` and deploys to GitHub Pages |
| `.github/workflows/publish-npm.yml` | version tag `v*` / release | builds `monad.wasm` and publishes the `keshavashiya` package to npm, attaching the binary to the release |

`publish-npm.yml` needs an `NPM_TOKEN` repository secret (Settings → Secrets →
Actions). Tag a release to ship: `git tag v5.0.0 && git push --tags`.

## License

MIT — see [LICENSE](LICENSE).

---

*"A monad is an indivisible unit of being. This one is mine."*
