//! Structured query API — the data plane of the kernel.
//!
//! The terminal commands return ANSI strings for humans. This module returns
//! plain data (`serde_json::Value`) and plain text for *programs* — chiefly the
//! MCP server, which exposes these to AI agents so they can interview the
//! author. It is deterministic and dependency-light: no LLM, no network.

use serde_json::{json, Value};

use crate::os::session::Session;

/// The topics a caller may request structured data for.
pub const TOPICS: &[&str] = &[
    "identity", "roles", "stacks", "systems", "projects", "links", "all",
];

/// Return structured JSON for a topic. Unknown topics return an error object
/// listing the valid topics rather than failing — friendlier for agents.
pub fn query(session: &Session, topic: &str) -> Value {
    let v = &session.vault;
    match topic.trim() {
        "identity" => to_value(&v.identity),
        "roles" => to_value(&v.roles),
        "stacks" => to_value(&v.stacks),
        "systems" => to_value(&v.systems),
        "projects" => to_value(&v.projects),
        "links" => to_value(&v.links),
        "all" | "" => json!({
            "identity": to_value(&v.identity),
            "roles": to_value(&v.roles),
            "stacks": to_value(&v.stacks),
            "systems": to_value(&v.systems),
            "projects": to_value(&v.projects),
            "links": to_value(&v.links),
        }),
        other => json!({
            "error": format!("unknown topic: {other}"),
            "topics": TOPICS,
        }),
    }
}

/// Deterministic keyword router over the vault — answers a natural-language-ish
/// question with relevant facts as plain text. No LLM: the agent on the other
/// end supplies the language model; MONAD supplies the ground truth.
pub fn ask(session: &Session, question: &str) -> String {
    let v = &session.vault;
    let q = question.to_lowercase();

    let hit = |keys: &[&str]| keys.iter().any(|k| q.contains(k));

    if hit(&["experience", "work", "job", "role", "career", "employ"]) {
        let mut out = String::from("Roles:\n");
        for r in &v.roles {
            out.push_str(&format!("- {} @ {} ({}) — {}\n", r.title, r.org, r.tenure, r.focus));
        }
        out
    } else if hit(&["stack", "tech", "language", "skill", "framework", "tool"]) {
        let mut out = String::from("Stacks:\n");
        for s in &v.stacks {
            out.push_str(&format!("- {} ({}, {}y)\n", s.name, s.proficiency, s.years));
        }
        out
    } else if hit(&["system", "architect", "built", "scale", "platform", "design"]) {
        let mut out = String::from("Systems:\n");
        for s in &v.systems {
            out.push_str(&format!("- {} [{}] — {}\n", s.name, s.architecture, s.description));
        }
        out
    } else if hit(&["project", "open source", "oss", "github", "repo"]) {
        let mut out = String::from("Projects:\n");
        for p in &v.projects {
            out.push_str(&format!("- {} ({}) — {}\n", p.name, p.status, p.description));
        }
        out
    } else if hit(&["contact", "email", "reach", "hire", "linkedin", "connect"]) {
        format!(
            "Contact:\n- email: {}\n- github: {}\n- linkedin: {}",
            v.links.email,
            v.links.github.as_deref().unwrap_or("-"),
            v.links.linkedin.as_deref().unwrap_or("-"),
        )
    } else {
        format!(
            "{} — {}{}. Ask about experience, stacks, systems, projects, or contact.",
            v.identity.name,
            v.identity.title,
            v.identity
                .location
                .as_deref()
                .map(|l| format!(" ({l})"))
                .unwrap_or_default(),
        )
    }
}

fn to_value<T: serde::Serialize>(v: &T) -> Value {
    serde_json::to_value(v).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::host::Host;
    use crate::vault::Vault;

    struct TestHost;
    impl Host for TestHost {
        fn now_secs(&self) -> f64 {
            0.0
        }
    }

    fn session() -> Session {
        Session::new(Vault::load(), Box::new(TestHost))
    }

    #[test]
    fn query_known_topics_return_data() {
        let s = session();
        for t in TOPICS {
            let v = query(&s, t);
            assert!(v.get("error").is_none(), "topic {t} returned an error");
        }
    }

    #[test]
    fn query_unknown_topic_lists_topics() {
        let s = session();
        let v = query(&s, "nonsense");
        assert!(v.get("error").is_some());
        assert!(v.get("topics").is_some());
    }

    #[test]
    fn ask_routes_keywords() {
        let s = session();
        assert!(ask(&s, "what is your work experience?").starts_with("Roles:"));
        assert!(ask(&s, "which tech stack do you use?").starts_with("Stacks:"));
        assert!(ask(&s, "tell me about systems you built").starts_with("Systems:"));
        assert!(ask(&s, "your open source projects").starts_with("Projects:"));
        assert!(ask(&s, "how can I reach you?").starts_with("Contact:"));
    }
}
