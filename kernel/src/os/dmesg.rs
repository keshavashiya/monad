use crate::vault::LogEntry;
use crate::os::session::Session;

/// The dmesg ring buffer — kernel boot messages.
/// Derived from vault data, with one dynamic entry appended for the current session.
pub struct Dmesg;

impl Dmesg {
    /// Returns all boot log entries from the vault.
    pub fn read(session: &Session) -> Vec<LogEntry> {
        let mut entries = session.vault.bootlog.clone();
        // Append a dynamic entry showing command count (simulates /dev/kmsg)
        entries.push(LogEntry {
            timestamp: format!("[{:>10.6}]", session.uptime_seconds()),
            message: format!(
                "monad: {} commands executed this session",
                session.command_count
            ),
        });
        entries
    }
}
