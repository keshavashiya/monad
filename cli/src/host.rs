use monad_kernel::Host;

/// Native host: the wall clock comes from the system clock.
pub struct NativeHost;

impl Host for NativeHost {
    fn now_secs(&self) -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }
}
