/**
 * MONAD Kernel bridge — loads monad.wasm and exposes the execute() interface.
 *
 * The console never reads vault data directly. All logic lives in the WASM binary.
 * This module is the only bridge between the TTY and the kernel.
 *
 * The wasm-pack build output lives in firmware/pkg/. The Vite config aliases
 * 'monad-firmware' to this path (see vite.config.ts).
 */

export interface MonadKernel {
  init(): void;
  execute(input: string): string;
  set_cols(cols: number): void;
  completions(prefix: string): string[];
  get_cwd(): string;
  get_home(): string;
  get_user(): string;
  get_link(name: string): string;
  build_hash(): string;
  version(): string;
}

let kernel: MonadKernel | null = null;

/**
 * Load the MONAD WASM binary and initialise the kernel.
 * Returns the kernel interface once ready.
 */
export async function loadKernel(): Promise<MonadKernel> {
  if (kernel) return kernel;

  try {
    // wasm-pack produces a JS loader at ../../firmware/pkg/monad.js
    // @ts-ignore — resolved by Vite at build time; wasm-pack output replaces stub
    const wasm = await import('../../firmware/pkg/monad.js');
    // Call the default export (__wbg_init) to load and instantiate the WASM binary
    await wasm.default();
    // Then call the named init() to initialise the kernel session
    wasm.init();
    kernel = wasm as unknown as MonadKernel;
    console.log('[monad] kernel loaded, session initialised');
    return kernel;
  } catch (err) {
    console.error('[monad] failed to load kernel:', err);
    throw err;
  }
}

/**
 * Execute a command via the kernel. Returns ANSI-formatted output string.
 */
export function execute(input: string): string {
  if (!kernel) return '';
  return kernel.execute(input);
}

/**
 * Tell the kernel how wide the terminal is (character columns) so tables fill
 * the viewport. No-op until the kernel is loaded.
 */
export function setCols(cols: number): void {
  if (!kernel) return;
  kernel.set_cols(cols);
}

/**
 * Get tab completion candidates.
 */
export function completions(prefix: string): string[] {
  if (!kernel) return [];
  return kernel.completions(prefix);
}

/**
 * Get current working directory for prompt.
 */
export function getCwd(): string {
  if (!kernel) return '/';
  return kernel.get_cwd();
}

/**
 * Canonical home directory (the `~` target), for prompt abbreviation.
 */
export function getHome(): string {
  if (!kernel) return '/home/user';
  return kernel.get_home();
}

/**
 * Login short-name (`<user>@monad`) for the shell prompt, from the vault.
 */
export function getUser(): string {
  if (!kernel) return 'user';
  return kernel.get_user();
}

/**
 * A named vault link (`resume`, `meeting`, …), or '' if undefined. Lets the
 * console act on a URL (open/download) without parsing rendered output.
 */
export function getLink(name: string): string {
  if (!kernel) return '';
  return kernel.get_link(name);
}

/**
 * Reproducible SHA-256 of the embedded vault (for the boot banner / verify).
 */
export function getBuildHash(): string {
  if (!kernel) return '';
  return kernel.build_hash();
}

/**
 * MONAD version (from the kernel crate). Empty until the kernel is loaded.
 */
export function getVersion(): string {
  if (!kernel) return '';
  return kernel.version();
}
