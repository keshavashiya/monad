/// Capabilities the kernel needs from its runtime adapter.
///
/// The kernel itself is pure and host-agnostic. Every non-deterministic
/// capability (wall clock, and later randomness) is injected by the adapter
/// that hosts the kernel — the browser shim, the CLI, the WASI binary, the MCP
/// server, or the daemon. This keeps the same core answering identically across
/// every runtime given the same host inputs.
///
/// `Send` is required so a `Box<dyn Host>` can live inside the kernel behind a
/// `static Mutex<Option<Kernel>>` in the browser shim.
pub trait Host: Send {
    /// Wall-clock time in seconds. Any consistent reference works (Unix epoch,
    /// or a monotonic origin) — the kernel only ever computes differences from
    /// the value captured at boot, for uptime accounting.
    fn now_secs(&self) -> f64;
}
