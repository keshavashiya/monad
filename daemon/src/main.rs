//! MONAD daemon — an optional, self-hosted HTTP adapter over `monad-kernel`.
//!
//! The same kernel that powers the browser console and the CLI, exposed over a
//! tiny HTTP/1.1 server. No async runtime, no web framework — just `std::net`
//! and the kernel itself (the daemon links `monad-kernel` directly; there is no
//! wasmtime and no `monad.wasm` to load).
//!
//!   GET  /          → health + usage (JSON)
//!   POST /execute   → run one command; body {"input":"whoami"} → {"output":"…"}
//!
//! Each request runs against a fresh, stateless kernel — the same guarantee as
//! the rest of MONAD: nothing about a caller is ever stored.
//!
//! Usage:
//!   monad-daemon                 # bind 127.0.0.1:7373 (localhost only)
//!   monad-daemon 0.0.0.0:7373    # bind all interfaces (self-hosted, public)

use monad_kernel::{Host, Kernel};
use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

/// Native host: wall clock from the system clock.
struct NativeHost;
impl Host for NativeHost {
    fn now_secs(&self) -> f64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
    }
}

/// Cap the request body — commands are tiny; anything larger is rejected.
const MAX_BODY: usize = 64 * 1024;

fn main() {
    let addr = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:7373".to_string());

    let listener = match TcpListener::bind(&addr) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("monad-daemon: cannot bind {addr}: {e}");
            std::process::exit(1);
        }
    };

    let version = Kernel::new(Box::new(NativeHost)).version().to_string();
    eprintln!("MONAD daemon v{version} — HTTP adapter over the kernel");
    eprintln!("    listening on http://{addr}");
    eprintln!("    try:  curl -s http://{addr}/execute -d '{{\"input\":\"whoami\"}}'");
    if addr.starts_with("127.") || addr.starts_with("localhost") {
        eprintln!("    (localhost only — pass an address like 0.0.0.0:7373 to expose it)");
    }

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(move || handle(stream));
            }
            Err(e) => eprintln!("monad-daemon: connection error: {e}"),
        }
    }
}

/// Handle a single connection: read one request, route it, write one response.
fn handle(stream: TcpStream) {
    let request = match read_request(&stream) {
        Ok(Some(req)) => req,
        Ok(None) => return, // empty/closed connection
        Err(_) => {
            let _ = write_response(&stream, 400, "Bad Request", &error_json("malformed request"));
            return;
        }
    };

    let (status, reason, body) = route(&request);
    let _ = write_response(&stream, status, reason, &body);
}

struct Request {
    method: String,
    path: String,
    body: String,
}

/// Read the request line, headers, and (length-delimited) body.
fn read_request(stream: &TcpStream) -> std::io::Result<Option<Request>> {
    let mut reader = BufReader::new(stream);

    let mut request_line = String::new();
    if reader.read_line(&mut request_line)? == 0 {
        return Ok(None);
    }
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();

    // Headers, until a blank line. We only care about Content-Length.
    let mut content_length = 0usize;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':') {
            if name.trim().eq_ignore_ascii_case("content-length") {
                content_length = value.trim().parse().unwrap_or(0);
            }
        }
    }

    let to_read = content_length.min(MAX_BODY);
    let mut buf = vec![0u8; to_read];
    if to_read > 0 {
        reader.read_exact(&mut buf)?;
    }
    let body = String::from_utf8_lossy(&buf).into_owned();

    Ok(Some(Request { method, path, body }))
}

/// Map a request to (status, reason, JSON body).
fn route(req: &Request) -> (u16, &'static str, String) {
    // Strip any query string for matching.
    let path = req.path.split('?').next().unwrap_or(&req.path);

    match (req.method.as_str(), path) {
        // CORS preflight.
        ("OPTIONS", _) => (204, "No Content", String::new()),

        ("GET", "/") => (200, "OK", health()),

        ("POST", "/execute") => match serde_json::from_str::<Value>(&req.body) {
            Ok(v) => {
                let input = v.get("input").and_then(Value::as_str).unwrap_or("");
                let mut kernel = Kernel::new(Box::new(NativeHost));
                let output = kernel.execute(input);
                (200, "OK", json!({ "input": input, "output": output }).to_string())
            }
            Err(_) => (
                400,
                "Bad Request",
                error_json("body must be JSON: {\"input\":\"<command>\"}"),
            ),
        },

        ("GET", "/execute") => (
            405,
            "Method Not Allowed",
            error_json("use POST with body {\"input\":\"<command>\"}"),
        ),

        _ => (404, "Not Found", error_json("not found")),
    }
}

fn health() -> String {
    let kernel = Kernel::new(Box::new(NativeHost));
    json!({
        "name": "monad-daemon",
        "kernel": format!("MONAD v{}", kernel.version()),
        "endpoints": {
            "GET /": "this message",
            "POST /execute": "{\"input\":\"<command>\"} → {\"output\":\"<ansi>\"}"
        },
        "hint": "every command runs against a fresh, stateless kernel"
    })
    .to_string()
}

fn error_json(message: &str) -> String {
    json!({ "error": message }).to_string()
}

/// Write an HTTP/1.1 response with permissive CORS and `Connection: close`.
fn write_response(mut stream: &TcpStream, status: u16, reason: &str, body: &str) -> std::io::Result<()> {
    let head = format!(
        "HTTP/1.1 {status} {reason}\r\n\
         Content-Type: application/json; charset=utf-8\r\n\
         Content-Length: {len}\r\n\
         Access-Control-Allow-Origin: *\r\n\
         Access-Control-Allow-Methods: GET, POST, OPTIONS\r\n\
         Access-Control-Allow-Headers: Content-Type\r\n\
         Connection: close\r\n\
         \r\n",
        len = body.len(),
    );
    stream.write_all(head.as_bytes())?;
    stream.write_all(body.as_bytes())?;
    stream.flush()
}
