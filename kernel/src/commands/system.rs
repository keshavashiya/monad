use crate::os::session::Session;
use crate::os::process_table::ProcessTable;
use crate::os::dmesg::Dmesg;
use crate::terminal::ansi::Style;
use crate::terminal::table::{Table, Column};

pub fn uname(args: &str, _session: &Session) -> String {
    match args {
        "-a" | "--all" => {
            format!(
                "MONAD {} {} {} vault:{} {}",
                crate::VERSION,
                "compiled identity kernel",
                "wasm32-unknown-unknown",
                short_hash(),
                "MONAD"
            )
        }
        "-r" | "--kernel-release" => crate::VERSION.to_string(),
        "-m" | "--machine" => "wasm32-unknown-unknown".to_string(),
        "-n" | "--nodename" => "monad".to_string(),
        _ => format!(
            "MONAD {} {}",
            crate::VERSION,
            Style::dim("(use -a for full details)")
        ),
    }
}

pub fn uptime(session: &Session) -> String {
    format!(" {} {}", session.format_uptime(), Style::dim("(since session start)"))
}

pub fn date() -> String {
    // In WASM we can't access the real system clock easily; return a simulated UTC time.
    // The console adapter may override this with real JS Date.
    "Session time: UTC (use `date` from your host for real time)".to_string()
}

pub fn neofetch(session: &Session) -> String {
    format!(
        r#"{}  {}
    {}  OS: MONAD {} wasm32
    {}  Host: compiled identity kernel
    {}  Kernel: Rust (stable toolchain)
    {}  Shell: one core, many adapters
    {}  Resolution: 80x24 (terminal)
    {}  CPU: wasm32 — single-threaded, no preemption
    {}  Queries: {} commands this session
    {}  Uptime: {}
"#,
        Style::amber("    λ"),   Style::bold(&Style::amber("MONAD")),
        Style::amber("    λ"),   crate::VERSION,
        Style::amber("    λ"),
        Style::amber("    λ"),
        Style::amber("    λ"),
        Style::amber("    λ"),
        Style::amber("    λ"),
        Style::amber("    λ"),   session.command_count,
        Style::amber("    λ"),   session.format_uptime(),
    )
}

pub fn ps(session: &Session) -> String {
    let processes = ProcessTable::snapshot(&session.vault.processes);
    let headers = &[
        Column { header: "USER", width: 8 },
        Column { header: "PID", width: 5 },
        Column { header: "PPID", width: 5 },
        Column { header: "%CPU", width: 6 },
        Column { header: "COMMAND", width: 20 },
    ];
    let rows: Vec<Vec<String>> = processes.iter().map(|p| {
        vec![
            p.user.clone().unwrap_or_else(|| "?".to_string()),
            p.pid.to_string(),
            p.ppid.to_string(),
            format!("{:.1}", p.cpu.unwrap_or(0.0)),
            p.command.clone(),
        ]
    }).collect();
    Table::render(headers, &rows, None)
}

pub fn top(session: &Session) -> String {
    let uptime = session.format_uptime();
    let processes = ProcessTable::snapshot(&session.vault.processes);
    let header = format!(
        "{} {}  {}  {}",
        Style::bold("MONAD"),
        Style::dim("top -"),
        uptime,
        Style::dim(&format!("{} queries", session.command_count)),
    );
    let table_headers = &[
        Column { header: "PID", width: 5 },
        Column { header: "USER", width: 8 },
        Column { header: "STATE", width: 10 },
        Column { header: "%CPU", width: 6 },
        Column { header: "COMMAND", width: 20 },
    ];
    let rows: Vec<Vec<String>> = processes.iter().map(|p| {
        vec![
            p.pid.to_string(),
            p.user.clone().unwrap_or_else(|| "?".to_string()),
            p.state.clone(),
            format!("{:.1}", p.cpu.unwrap_or(0.0)),
            p.command.clone(),
        ]
    }).collect();
    let table = Table::render(table_headers, &rows, None);
    format!("{}\r\n{}", header, table)
}

pub fn dmesg(session: &Session) -> String {
    let entries = Dmesg::read(session);
    entries.iter().map(|e| {
        format!("{} {}", Style::dim(&e.timestamp), e.message)
    }).collect::<Vec<_>>().join("\r\n")
}

pub fn clear() -> String {
    "\x1b[2J\x1b[H".to_string() // ANSI clear screen + home cursor
}

pub fn panic() -> String {
    format!(
        r#"
{}
{}
{}
{}
"#,
        Style::bold(&Style::red("KERNEL PANIC!")),
        Style::dim("A fatal exception has occurred."),
        Style::dim("MONAD has encountered an unrecoverable error and must halt."),
        Style::cyan("System halted. Restart the terminal to continue."),
    )
}

/// First 12 hex chars of the embedded vault hash — enough to eyeball.
fn short_hash() -> &'static str {
    &crate::BUILD_HASH[..12.min(crate::BUILD_HASH.len())]
}

/// `verify` — prove the running identity matches the committed source.
pub fn verify(_session: &Session) -> String {
    format!(
        r#"{}
{}

  kernel       : MONAD {}
  target       : wasm32 / native / wasi (one core, many adapters)
  vault sha256 : {}

{}
  {}
  {}

{}"#,
        Style::bold(&Style::amber("MONAD — verifiable identity")),
        Style::dim("This build embeds a reproducible hash of its identity data."),
        crate::VERSION,
        Style::bold(&Style::amber(crate::BUILD_HASH)),
        Style::dim("Reproduce it yourself from the source vault:"),
        Style::cyan("shasum -a 256 vault/profile.json    # macOS"),
        Style::cyan("sha256sum    vault/profile.json     # Linux"),
        Style::dim("If the hex matches, the identity you are talking to is exactly the committed source."),
    )
}
