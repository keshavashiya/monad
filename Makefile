.PHONY: all help setup doctor build validate firmware console wasi cli daemon verify clean dev deploy

# Default target: print the available commands so `make` alone is informative.
help:
	@echo "MONAD — make targets"
	@echo ""
	@echo "  make setup     install toolchain prerequisites (run this first)"
	@echo "  make doctor    check that all prerequisites are present"
	@echo "  make dev       firmware watcher + vite dev server (http://localhost:5173)"
	@echo "  make build     assemble the static site into dist/"
	@echo "  make cli       build the native ./target/release/monad binary"
	@echo "  make daemon    build the self-hosted HTTP server (POST /execute)"
	@echo "  make wasi      build npm/monad.wasm (for npx keshavashiya / wasmtime)"
	@echo "  make verify    print and independently check the vault hash"
	@echo "  make clean     remove build artifacts"
	@echo "  make deploy    build, then print Pages deploy hint"

all: build

# One command to install everything a contributor needs. Idempotent.
setup:
	@./scripts/setup.sh

# Verify the toolchain without installing anything.
doctor:
	@./scripts/doctor.sh

build: validate firmware console
	@mkdir -p dist
	@cp -r console/dist/* dist/
	@cp firmware/pkg/monad_bg.wasm dist/monad.wasm
	@echo "MONAD system image assembled in dist/"

validate:
	@./scripts/validate.sh

firmware:
	@echo "==> COMPILE   firmware (wasm32-unknown-unknown, release)"
	@wasm-pack build firmware/ --target web --release --out-name monad --no-opt

console:
	@echo "==> ASSEMBLE  console adapter"
	@cd console && npm ci --silent && npm run build

wasi:
	@echo "==> COMPILE   wasi binary (wasm32-wasip1, release)"
	@rustup target add wasm32-wasip1 2>/dev/null || true
	@cargo build -p monad-cli --target wasm32-wasip1 --release
	@cp target/wasm32-wasip1/release/monad.wasm npm/monad.wasm
	@echo "    npm/monad.wasm ready  (npx keshavashiya)"

cli:
	@echo "==> COMPILE   native cli (monad)"
	@cargo build -p monad-cli --release
	@echo "    target/release/monad ready"

daemon:
	@echo "==> COMPILE   daemon (http adapter)"
	@cargo build -p monad-daemon --release
	@echo "    target/release/monad-daemon ready  (run: ./target/release/monad-daemon)"

verify: cli
	@./target/release/monad verify
	@echo ""
	@echo "==> Independent check:"
	@printf "    shasum: " && shasum -a 256 vault/profile.json | cut -d' ' -f1

dev:
	@./scripts/dev.sh

clean:
	@rm -rf firmware/pkg console/dist dist target npm/monad.wasm
	@echo "==> Clean."

deploy: build
	@echo "==> Deploy ready. Push dist/ to your Pages provider."
	@echo "    GitHub: npx gh-pages -d dist"
