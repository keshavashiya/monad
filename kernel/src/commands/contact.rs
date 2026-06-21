use crate::os::session::Session;
use crate::terminal::ansi::Style;

pub fn links(session: &Session) -> String {
    let l = &session.vault.links;
    let mut output = String::new();
    output.push_str(&Style::bold(&Style::amber("Contact & Links")));
    output.push_str("\r\n");
    output.push_str(&Style::rule(session.cols().unwrap_or(54)));
    output.push_str("\r\n");

    let items: Vec<(&str, &str, bool)> = vec![
        ("email", l.email.as_str(), false),
        ("github", l.github.as_deref().unwrap_or(""), true),
        ("linkedin", l.linkedin.as_deref().unwrap_or(""), true),
        ("twitter", l.twitter.as_deref().unwrap_or(""), true),
        ("website", l.website.as_deref().unwrap_or(""), true),
        ("resume", l.resume.as_deref().unwrap_or(""), true),
        ("meeting", l.meeting.as_deref().unwrap_or(""), true),
    ];

    for (label, value, is_hyperlink) in items {
        if value.is_empty() { continue; }
        let display = if is_hyperlink && value.starts_with("http") {
            Style::hyperlink(value, value)
        } else {
            value.to_string()
        };
        output.push_str(&format!("  {}: {}\r\n", Style::dim(&format!("{:>9}", label)), display));
    }

    output.push_str(&Style::dim("  ─────────────────────────────────────────────────"));
    output.push_str(&format!("\r\n  {}  {}\r\n", Style::dim("pgp fingerprint:"), Style::cyan("15A2 E8F4 7B91 C3D0 5F91  6F3E A1B2 C3D4 E5F6 7890")));
    output
}

/// Render a single labelled link (or a friendly note if the vault omits it).
/// Shared by the `resume` and `meeting` commands. The URL is printed as a
/// terminal hyperlink; adapters that can (the web console) also act on it.
fn link_action(url: Option<&str>, label: &str, missing: &str) -> String {
    match url {
        Some(u) if !u.is_empty() => {
            format!("  {}: {}\r\n", Style::dim(label), Style::hyperlink(u, u))
        }
        _ => format!("{}\r\n", Style::dim(missing)),
    }
}

pub fn resume(session: &Session) -> String {
    link_action(
        session.vault.links.resume.as_deref(),
        "resume",
        "No résumé link on file.",
    )
}

pub fn meeting(session: &Session) -> String {
    link_action(
        session.vault.links.meeting.as_deref(),
        "meeting",
        "No meeting link on file.",
    )
}

pub fn pgp(session: &Session) -> String {
    let id = &session.vault.identity;
    format!(
        "{}  {}\r\n{}",
        Style::dim("pub   ed25519 2026-01-01"),
        Style::cyan("15A2 E8F4 7B91 C3D0 5F91  6F3E A1B2 C3D4 E5F6 7890"),
        Style::dim(&format!("uid                 {} <{}@monad>", id.name, id.handle))
    )
}
