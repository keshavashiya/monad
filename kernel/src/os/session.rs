use crate::host::Host;
use crate::vault::Vault;

/// Choose the home directory: the single `/home/<user>` directory in the vault
/// filesystem if there's exactly one, otherwise the conventional default.
fn pick_home(vault: &Vault) -> String {
    let mut candidates: Vec<&String> = vault
        .filesystem
        .iter()
        .filter(|(k, v)| {
            v.entry_type == "dir"
                && k.starts_with("/home/")
                && k.matches('/').count() == 2
        })
        .map(|(k, _)| k)
        .collect();
    candidates.sort();
    candidates
        .first()
        .map(|s| s.to_string())
        // No `/home/<user>` dir in the vault: derive the conventional path from
        // the identity handle rather than hardcoding a name, so a fork's `~`
        // still points somewhere that reflects *its* vault.
        .unwrap_or_else(|| format!("/home/{}", vault.identity.handle))
}

/// Runtime session state — the only mutable state in the entire system.
/// Reset on every boot. Nothing about a visitor is persisted server-side.
pub struct Session {
    pub vault: Vault,
    pub cwd: String,
    /// Canonical home directory (the `~` target). Must be a real path in the
    /// vault filesystem — derived from it rather than from the handle, which
    /// may differ (handle `keshavashiya`, home `/home/keshav`).
    pub home: String,
    pub history: Vec<String>,
    pub start_time: f64,
    pub command_count: u64,
    /// Terminal width in character columns, when the adapter knows it (the
    /// browser measures its viewport; the CLI reads `$COLUMNS`). `None` means
    /// unknown, in which case tables fall back to their fixed column widths.
    cols: Option<usize>,
    /// Runtime-provided capabilities (clock). Injected by the adapter.
    host: Box<dyn Host>,
}

impl Session {
    pub fn new(vault: Vault, host: Box<dyn Host>) -> Self {
        let start = host.now_secs();
        // Prefer the home declared in the vault filesystem; fall back to the
        // conventional path. This keeps `~` pointing at a directory that exists.
        let home = pick_home(&vault);
        Self {
            cwd: home.clone(),
            home,
            vault,
            history: Vec::with_capacity(100),
            start_time: start,
            command_count: 0,
            cols: None,
            host,
        }
    }

    /// The terminal width the adapter reported, if any. Tables use it to expand
    /// their free-form column to fill the screen.
    pub fn cols(&self) -> Option<usize> {
        self.cols
    }

    /// Record the adapter's terminal width (clamped to a sane minimum so layout
    /// never collapses). Called on connect and on resize.
    pub fn set_cols(&mut self, cols: usize) {
        self.cols = Some(cols.max(24));
    }

    pub fn add_history(&mut self, cmd: String) {
        if self.history.len() >= 100 {
            self.history.remove(0);
        }
        self.history.push(cmd);
        self.command_count += 1;
    }

    /// The login short-name for shell prompts (`<user>@monad`). Derived from
    /// the home directory's basename so it tracks the vault rather than a
    /// literal baked into each adapter (handle may differ, e.g. handle
    /// `keshavashiya`, home `/home/keshav` → user `keshav`).
    pub fn user(&self) -> &str {
        self.home.rsplit('/').find(|s| !s.is_empty()).unwrap_or("user")
    }

    pub fn uptime_seconds(&self) -> f64 {
        self.host.now_secs() - self.start_time
    }

    pub fn format_uptime(&self) -> String {
        let secs = self.uptime_seconds() as u64;
        let days = secs / 86400;
        let hours = (secs % 86400) / 3600;
        let minutes = (secs % 3600) / 60;
        let seconds = secs % 60;
        if days > 0 {
            format!("up {} days, {:02}:{:02}:{:02}", days, hours, minutes, seconds)
        } else if hours > 0 {
            format!("up {:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("up {} min, {} sec", minutes, seconds)
        }
    }
}
