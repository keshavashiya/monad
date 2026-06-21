//! MONAD firmware — the browser adapter.
//!
//! A thin `wasm-bindgen` shim over the host-agnostic `monad-kernel` crate.
//! It supplies a `WebHost` (the browser clock) and exposes the kernel to the
//! web console via `init` / `execute` / `completions` / `get_cwd`.

use wasm_bindgen::prelude::*;
use std::sync::Mutex;
use monad_kernel::{Kernel, Host};

/// Browser-provided capabilities: the wall clock comes from JS `Date.now()`.
struct WebHost;

impl Host for WebHost {
    fn now_secs(&self) -> f64 {
        // In the browser the clock comes from JS `Date.now()`. The crate also
        // needs to type-check for the host target (rust-analyzer, `cargo build
        // --workspace`), where `js-sys` isn't a dependency — fall back to the
        // system clock there. Only the wasm32 path ever actually runs.
        #[cfg(target_arch = "wasm32")]
        {
            js_sys::Date::now() / 1000.0
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_secs_f64())
                .unwrap_or(0.0)
        }
    }
}

static SESSION: Mutex<Option<Kernel>> = Mutex::new(None);

#[wasm_bindgen]
pub fn init() {
    let kernel = Kernel::new(Box::new(WebHost));
    let mut guard = SESSION.lock().unwrap();
    *guard = Some(kernel);
}

#[wasm_bindgen]
pub fn execute(input: &str) -> String {
    let mut guard = SESSION.lock().unwrap();
    let kernel = guard.as_mut().expect("MONAD not initialized. Call init() first.");
    kernel.execute(input)
}

/// Report the terminal width (character columns) so tables fill the viewport.
/// The console calls this on load and on window resize.
#[wasm_bindgen]
pub fn set_cols(cols: usize) {
    let mut guard = SESSION.lock().unwrap();
    if let Some(kernel) = guard.as_mut() {
        kernel.set_cols(cols);
    }
}

#[wasm_bindgen]
pub fn completions(prefix: &str) -> Vec<String> {
    Kernel::completions(prefix)
}

/// The reproducible SHA-256 of the embedded vault, for the boot banner.
#[wasm_bindgen]
pub fn build_hash() -> String {
    monad_kernel::BUILD_HASH.to_string()
}

/// The MONAD version (from the kernel crate), so the console never hardcodes it.
#[wasm_bindgen]
pub fn version() -> String {
    monad_kernel::VERSION.to_string()
}

#[wasm_bindgen]
pub fn get_cwd() -> String {
    let guard = SESSION.lock().unwrap();
    match guard.as_ref() {
        Some(kernel) => kernel.cwd().to_string(),
        None => "/".to_string(),
    }
}

/// The canonical home directory (the `~` target), so the console prompt can
/// abbreviate it without hardcoding a path the vault may have changed.
#[wasm_bindgen]
pub fn get_home() -> String {
    let guard = SESSION.lock().unwrap();
    match guard.as_ref() {
        Some(kernel) => kernel.home().to_string(),
        None => "/".to_string(),
    }
}

/// The login short-name (`<user>@monad`) for the shell prompt, derived from
/// the vault so the console never hardcodes it.
#[wasm_bindgen]
pub fn get_user() -> String {
    let guard = SESSION.lock().unwrap();
    match guard.as_ref() {
        Some(kernel) => kernel.user().to_string(),
        None => "user".to_string(),
    }
}

/// A named vault link (`resume`, `meeting`, …) so the console can open or
/// download it. Empty string if the vault doesn't define it.
#[wasm_bindgen]
pub fn get_link(name: &str) -> String {
    let guard = SESSION.lock().unwrap();
    match guard.as_ref() {
        Some(kernel) => kernel.link(name).to_string(),
        None => String::new(),
    }
}
