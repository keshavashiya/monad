mod system;
mod profile;
mod filesystem;
mod contact;
mod meta;

use crate::os::session::Session;

/// Dispatch a command string to the appropriate handler.
/// Returns Ok(ANSI-formatted output) or Err(error message).
pub fn dispatch(input: &str, session: &mut Session) -> Result<String, String> {
    let input = input.trim();

    // Handle empty input
    if input.is_empty() {
        return Ok(String::new());
    }

    // Split into command and args
    let parts: Vec<&str> = input.splitn(2, ' ').collect();
    let cmd = parts[0];
    let args = parts.get(1).copied().unwrap_or("");

    match cmd {
        // System commands
        "uname"    => Ok(system::uname(args, session)),
        "uptime"   => Ok(system::uptime(session)),
        "date"     => Ok(system::date()),
        "neofetch" => Ok(system::neofetch(session)),
        "ps"       => Ok(system::ps(session)),
        "top"      => Ok(system::top(session)),
        "dmesg"    => Ok(system::dmesg(session)),
        "verify"   => Ok(system::verify(session)),
        "clear"    => Ok(system::clear()),

        // Profile commands
        "whoami"   => Ok(profile::whoami(session)),
        "id"       => Ok(profile::id(session)),
        "roles"    => Ok(profile::roles(session)),
        "stacks"   => Ok(profile::stacks(session)),
        "systems"  => Ok(profile::systems(session)),
        "projects" => Ok(profile::projects(session)),

        // Filesystem commands
        "ls"       => filesystem::ls(args, session),
        "cat"      => filesystem::cat(args, session),
        "tree"     => filesystem::tree(args, session),
        "pwd"      => Ok(filesystem::pwd(session)),
        "cd"       => filesystem::cd(args, session),
        "find"     => filesystem::find(args, session),

        // Contact commands
        "links"    => Ok(contact::links(session)),
        "pgp"      => Ok(contact::pgp(session)),
        "resume"   => Ok(contact::resume(session)),
        "meeting"  => Ok(contact::meeting(session)),

        // Meta commands
        "help" | "--help" | "-h" => Ok(meta::help()),
        "version" | "--version" | "-V" => Ok(meta::version()),
        "hint"     => Ok(meta::hint()),
        "fortune"  => Ok(meta::fortune(session)),
        "history"  => Ok(meta::history(session)),
        "exit"     => Ok(meta::exit()),
        "logout"   => Ok(meta::exit()),

        // Easter eggs
        "sudo"     => Err("permission denied: you are already root. there is no higher privilege.".to_string()),
        "su"       => Err("this system has no superuser switch. you are already at the top.".to_string()),
        "shutdown" => Err("cannot shutdown: this system runs until you close the terminal.".to_string()),
        "reboot"   => Err("cannot reboot: this is not a real kernel. close and reopen.".to_string()),
        "panic"    => Ok(system::panic()),

        _ => Err(format!("monad: command not found: {}", cmd)),
    }
}

/// Return tab completion candidates for a given prefix.
pub fn completions(prefix: &str) -> Vec<String> {
    let all_commands = vec![
        "whoami", "id", "uname", "uptime", "date", "neofetch",
        "ps", "top", "dmesg", "verify", "clear",
        "roles", "stacks", "systems", "projects",
        "ls", "cat", "tree", "pwd", "cd", "find",
        "links", "pgp", "resume", "meeting",
        "help", "version", "hint", "fortune", "history", "exit", "logout",
        "sudo", "su", "shutdown", "reboot", "panic",
    ];
    all_commands.into_iter()
        .filter(|c| c.starts_with(prefix))
        .map(|c| c.to_string())
        .collect()
}
