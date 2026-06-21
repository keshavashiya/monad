use crate::os::session::Session;
use crate::terminal::ansi::Style;

const HELP_TEXT: &str = r#"Available commands
──────────────────────────────────────────────────────────────────────

System
  whoami       Print current user
  id           Print user and group identity
  uname        Print system information
  uptime       Show how long the system has been running
  date         Display current time (UTC)
  neofetch     Display system overview
  ps           List running processes
  top          Show process table
  dmesg        Print kernel ring buffer
  verify       Show the reproducible vault hash (cryptographic identity check)
  clear        Clear the terminal

Filesystem
  ls           List directory contents
  cat          Print file contents
  tree         Display directory tree
  pwd          Print working directory
  cd           Change working directory
  find         Search for files

Profile
  roles        Current roles and responsibilities
  stacks       Technology stack and proficiency
  systems      Systems designed and built
  projects     Recent and active projects

Contact
  links        Contact info and social links
  pgp          PGP public key fingerprint
  resume       Link to download my résumé (PDF)
  meeting      Link to schedule a meeting

Meta
  help         Print this help text
  hint         Suggestions for first commands
  fortune      A curated thought
  history      Command history
  exit         Close this connection"#;

const HINT_TEXT: &str = r#"New here? Try some of these:

    help          → see everything available
    whoami        → who you're talking to
    roles         → what I do
    stacks        → tools I work with
    systems       → things I've built
    projects      → what I'm working on
    links         → connect with me
    neofetch      → system overview
    fortune       → something interesting
    ls /home/     → explore the filesystem

All commands run instantly. Type help for the full list."#;

const EXIT_TEXT: &str = r#"logout

[connection closed.]"#;

pub fn help() -> String {
    let mut output = String::new();
    let title = format!("MONAD v{} — help", crate::VERSION);
    output.push_str(&Style::bold(&Style::amber(&title)));
    output.push_str("\r\n");
    output.push_str(HELP_TEXT);
    output
}

pub fn version() -> String {
    format!(
        "MONAD v{} — compiled identity kernel (one core, many adapters)",
        crate::VERSION
    )
}

pub fn hint() -> String {
    Style::dim(HINT_TEXT)
}

pub fn fortune(session: &Session) -> String {
    use fastrand;
    let fortunes = &session.vault.fortune;
    if fortunes.is_empty() {
        return String::new();
    }
    let idx = fastrand::usize(..fortunes.len());
    format!("  {}", fortunes[idx])
}

pub fn history(session: &Session) -> String {
    if session.history.is_empty() {
        return "  no history".to_string();
    }
    let mut output = String::new();
    for (i, cmd) in session.history.iter().enumerate() {
        output.push_str(&format!("  {:>4}  {}\r\n", i + 1, cmd));
    }
    output
}

pub fn exit() -> String {
    EXIT_TEXT.to_string()
}
