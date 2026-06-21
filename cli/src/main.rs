//! MONAD CLI — the native adapter over `monad-kernel`.
//!
//!   monad <command...>   run a single kernel command and print the result
//!   monad repl           interactive session (reads lines until EOF)
//!   monad mcp            run as an MCP server over stdio (for AI agents)
//!
//! Same kernel as the browser. The only thing that differs is the host adapter.

mod host;
mod mcp;

use host::NativeHost;
use monad_kernel::Kernel;
use std::io::{self, BufRead, Write};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("mcp") => mcp::serve(),
        Some("repl") => repl(),
        None => {
            eprintln!("monad — a compiled identity kernel");
            eprintln!();
            eprintln!("usage:");
            eprintln!("  monad <command...>   run a single command (e.g. monad whoami)");
            eprintln!("  monad repl           interactive session");
            eprintln!("  monad mcp            run as an MCP server over stdio");
            std::process::exit(2);
        }
        Some(_) => {
            // Treat all args as a single command line.
            let mut kernel = Kernel::new(Box::new(NativeHost));
            if let Some(c) = term_cols() {
                kernel.set_cols(c);
            }
            let line = args.join(" ");
            let output = kernel.execute(&line);
            // Guarantee exactly one trailing newline so the shell prompt starts
            // on a fresh line (otherwise zsh shows a stray `%`).
            print!("{}", output.trim_end_matches(['\r', '\n']));
            println!();
            io::stdout().flush().ok();
        }
    }
}

/// Terminal width in character columns. Ask the TTY directly via `TIOCGWINSZ`
/// — this works even when the shell doesn't export `$COLUMNS` (zsh and bash
/// don't, by default). Fall back to `$COLUMNS`, then to `None`, which keeps the
/// kernel's fixed table layout — safe when the real width is unknown (e.g. when
/// stdout is a pipe rather than a terminal).
fn term_cols() -> Option<usize> {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
        let fd = io::stdout().as_raw_fd();
        if unsafe { libc::ioctl(fd, libc::TIOCGWINSZ, &mut ws) } == 0 && ws.ws_col >= 24 {
            return Some(ws.ws_col as usize);
        }
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.trim().parse::<usize>().ok())
        .filter(|&n| n >= 24)
}

/// Build the shell prompt for the REPL, reflecting the kernel's current cwd
/// (so `cd notes` shows `~/notes`, matching the browser console).
fn prompt(kernel: &Kernel) -> String {
    let cwd = kernel.cwd();
    let home = kernel.home();
    let dir = if cwd == home {
        "~".to_string()
    } else if let Some(rest) = cwd.strip_prefix(&format!("{}/", home)) {
        format!("~/{}", rest)
    } else {
        cwd.to_string()
    };
    format!("[{}@monad {}]$ ", kernel.user(), dir)
}

/// Interactive read-eval-print loop. Plain line input; no raw-mode TTY.
fn repl() {
    let mut kernel = Kernel::new(Box::new(NativeHost));
    if let Some(c) = term_cols() {
        kernel.set_cols(c);
    }
    let stdin = io::stdin();
    let mut out = io::stdout();
    print!("{}", prompt(&kernel));
    out.flush().ok();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let trimmed = line.trim();
        if trimmed == "exit" || trimmed == "logout" {
            break;
        }
        let output = kernel.execute(trimmed);
        print!("{}", output);
        if !output.is_empty() && !output.ends_with('\n') {
            println!();
        }
        print!("{}", prompt(&kernel));
        out.flush().ok();
    }
}
