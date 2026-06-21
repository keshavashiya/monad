//! Build script: compute a real, reproducible SHA-256 of the embedded vault and
//! expose it to the kernel as the `MONAD_VAULT_SHA256` compile-time env var.
//!
//! Anyone can reproduce it with standard tools — no MONAD toolchain required:
//!
//!   shasum -a 256 vault/profile.json     # macOS
//!   sha256sum    vault/profile.json      # Linux
//!
//! The hex it prints must equal the hash MONAD reports via `verify`. That makes
//! the running identity cryptographically checkable against the committed source.

use sha2::{Digest, Sha256};
use std::{fs, path::Path};

fn main() {
    // Build scripts run with CWD = this package's root (kernel/).
    let vault = Path::new("../vault/profile.json");
    println!("cargo:rerun-if-changed=../vault/profile.json");

    let hash = match fs::read(vault) {
        Ok(bytes) => {
            let digest = Sha256::digest(&bytes);
            let mut hex = String::with_capacity(64);
            for b in digest {
                hex.push_str(&format!("{b:02x}"));
            }
            hex
        }
        // If the vault is unreadable, emit an all-zero hash so the failure is
        // visible in `verify` rather than breaking the build.
        Err(_) => "0".repeat(64),
    };

    println!("cargo:rustc-env=MONAD_VAULT_SHA256={hash}");
}
