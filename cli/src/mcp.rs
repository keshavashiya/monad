//! Minimal MCP (Model Context Protocol) server over stdio.
//!
//! Hand-rolled, dependency-light: newline-delimited JSON-RPC 2.0 on stdin/stdout
//! (the MCP stdio transport). No SDK, no network — it runs on the *agent's*
//! machine via `npx keshavashiya mcp` or `monad mcp`, so it costs nothing to host.
//!
//! Wire it into an agent with, e.g.:
//!   claude mcp add keshav -- npx -y keshavashiya mcp
//!
//! Implements just enough of the spec to be useful: `initialize`,
//! `tools/list`, `tools/call`, and `ping`. The tools expose the kernel's
//! structured query plane so an AI can interview the author.

use monad_kernel::Kernel;
use serde_json::{json, Value};
use std::io::{self, BufRead, Write};

use crate::host::NativeHost;

const DEFAULT_PROTOCOL: &str = "2024-11-05";

/// Run the stdio MCP loop until EOF.
pub fn serve() {
    let kernel = Kernel::new(Box::new(NativeHost));
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut out = stdout.lock();

    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let Ok(msg) = serde_json::from_str::<Value>(&line) else {
            // Malformed JSON: per JSON-RPC, respond with a parse error (null id).
            send(&mut out, &error_response(Value::Null, -32700, "parse error"));
            continue;
        };

        // Notifications have no `id` and never get a response.
        let id = msg.get("id").cloned();
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");

        match (method, id) {
            ("initialize", Some(id)) => send(&mut out, &initialize(id, &msg, kernel.name())),
            ("tools/list", Some(id)) => send(&mut out, &tools_list(id, kernel.name())),
            ("tools/call", Some(id)) => send(&mut out, &tools_call(id, &msg, &kernel)),
            ("ping", Some(id)) => send(&mut out, &result_response(id, json!({}))),
            // Notifications (e.g. notifications/initialized): acknowledge silently.
            (_, None) => {}
            // Unknown method with an id: report method-not-found.
            (_, Some(id)) => {
                send(&mut out, &error_response(id, -32601, "method not found"))
            }
        }
    }
}

fn initialize(id: Value, msg: &Value, name: &str) -> Value {
    // Echo the client's requested protocol version when present.
    let protocol = msg
        .get("params")
        .and_then(|p| p.get("protocolVersion"))
        .and_then(Value::as_str)
        .unwrap_or(DEFAULT_PROTOCOL);

    result_response(
        id,
        json!({
            "protocolVersion": protocol,
            "capabilities": { "tools": {} },
            "serverInfo": {
                "name": "monad",
                "version": env!("CARGO_PKG_VERSION"),
            },
            "instructions": format!(
                "MONAD is the compiled identity of {name}. Use the tools to \
retrieve verified facts about their experience, systems, and projects, or \
`ask` a free-form question."
            )
        }),
    )
}

fn tools_list(id: Value, name: &str) -> Value {
    let no_args = json!({ "type": "object", "properties": {} });
    result_response(
        id,
        json!({
            "tools": [
                {
                    "name": "get_experience",
                    "description": format!("Return {name}'s professional roles and tenure as structured JSON."),
                    "inputSchema": no_args,
                },
                {
                    "name": "query_systems",
                    "description": format!("Return the notable systems {name} has architected, with stack and scale."),
                    "inputSchema": no_args,
                },
                {
                    "name": "list_projects",
                    "description": format!("Return {name}'s open-source projects as structured JSON."),
                    "inputSchema": no_args,
                },
                {
                    "name": "ask",
                    "description": format!("Ask a free-form question about {name}; returns relevant facts from the vault."),
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "question": { "type": "string", "description": "The question to ask." }
                        },
                        "required": ["question"]
                    },
                },
            ]
        }),
    )
}

fn tools_call(id: Value, msg: &Value, kernel: &Kernel) -> Value {
    let params = msg.get("params");
    let name = params
        .and_then(|p| p.get("name"))
        .and_then(Value::as_str)
        .unwrap_or("");
    let args = params.and_then(|p| p.get("arguments"));

    let text = match name {
        "get_experience" => pretty(kernel.query("roles")),
        "query_systems" => pretty(kernel.query("systems")),
        "list_projects" => pretty(kernel.query("projects")),
        "ask" => {
            let question = args
                .and_then(|a| a.get("question"))
                .and_then(Value::as_str)
                .unwrap_or("");
            if question.is_empty() {
                return tool_error(id, "missing required argument: question");
            }
            kernel.ask(question)
        }
        other => return tool_error(id, &format!("unknown tool: {other}")),
    };

    result_response(
        id,
        json!({
            "content": [ { "type": "text", "text": text } ],
            "isError": false,
        }),
    )
}

// --- JSON-RPC helpers ------------------------------------------------------

fn result_response(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn error_response(id: Value, code: i32, message: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
}

/// A tool-level failure is reported as a successful JSON-RPC result whose
/// content is flagged `isError` — per the MCP spec, so the model can recover.
fn tool_error(id: Value, message: &str) -> Value {
    result_response(
        id,
        json!({
            "content": [ { "type": "text", "text": message } ],
            "isError": true,
        }),
    )
}

fn pretty(v: Value) -> String {
    serde_json::to_string_pretty(&v).unwrap_or_else(|_| v.to_string())
}

fn send(out: &mut impl Write, msg: &Value) {
    let _ = writeln!(out, "{msg}");
    let _ = out.flush();
}
