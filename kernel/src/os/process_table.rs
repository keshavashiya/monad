use crate::vault::Process;

/// The process table is derived from the vault but includes runtime jitter
/// to simulate a live system. CPU values drift slightly on each call.
pub struct ProcessTable;

impl ProcessTable {
    /// Returns a copy of the base process list with simulated CPU jitter.
    pub fn snapshot(base: &[Process]) -> Vec<Process> {
        let mut processes = base.to_vec();
        // Add slight jitter to CPU values to simulate a live snapshot
        for proc in &mut processes {
            if proc.command == "monad" {
                proc.cpu = Some(0.3 + fastrand::f64() * 0.4);
            } else if proc.command == "monad/console" {
                proc.cpu = Some(0.1 + fastrand::f64() * 0.2);
            } else {
                proc.cpu = Some(0.0);
            }
        }
        processes
    }
}
