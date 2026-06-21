use crate::os::filesystem::VirtualFS;
use crate::os::session::Session;
use crate::terminal::ansi::Style;

/// Split `args` into (flags, operands), where flags are tokens starting with `-`.
/// Lets commands accept (and harmlessly ignore) the flags users reflexively type,
/// e.g. `ls -al`, instead of treating `-al` as a path.
fn parse_args(args: &str) -> (Vec<&str>, Vec<&str>) {
    let mut flags = Vec::new();
    let mut operands = Vec::new();
    for tok in args.split_whitespace() {
        if tok.starts_with('-') && tok.len() > 1 {
            flags.push(tok);
        } else {
            operands.push(tok);
        }
    }
    (flags, operands)
}

pub fn ls(args: &str, session: &Session) -> Result<String, String> {
    let (_flags, operands) = parse_args(args);
    let path = match operands.first() {
        None => session.cwd.clone(),
        Some(p) => VirtualFS::resolve(&session.cwd, p),
    };
    let entries = VirtualFS::ls(&session.vault.filesystem, &path)?;
    let mut output = String::new();
    for entry in &entries {
        let e = &entry.entry;
        let perms = e.permissions.as_deref().unwrap_or("?????????");
        let owner = e.owner.as_deref().unwrap_or("?");
        let group = e.group.as_deref().unwrap_or("?");
        // Prefer the real byte length of the file's content over any declared
        // `size`, so the vault never has to hand-maintain (and drift) sizes.
        let size = match (e.entry_type.as_str(), e.content.as_deref(), e.size) {
            ("dir", _, _) => "     -".to_string(),
            (_, Some(content), _) => format!("{:>6}", content.len()),
            (_, None, Some(s)) => format!("{:>6}", s),
            _ => "     -".to_string(),
        };
        let modified = e.modified.as_deref().unwrap_or("");
        let name = if e.entry_type == "dir" {
            Style::cyan(&format!("{}/", entry.name))
        } else {
            entry.name.clone()
        };
        output.push_str(&format!(
            "{} {:>4} {:>4} {} {} {}\r\n",
            perms, owner, group, size, modified, name
        ));
    }
    Ok(output)
}

pub fn cat(args: &str, session: &Session) -> Result<String, String> {
    let (_flags, operands) = parse_args(args);
    if operands.is_empty() {
        return Err("usage: cat <path>".to_string());
    }
    // Concatenate each operand, matching real `cat`'s multi-file behaviour.
    let mut output = String::new();
    for (i, p) in operands.iter().enumerate() {
        let path = VirtualFS::resolve(&session.cwd, p);
        let content = VirtualFS::cat(&session.vault.filesystem, &path)?;
        if i > 0 {
            output.push('\n');
        }
        output.push_str(&content);
    }
    Ok(output)
}

pub fn tree(args: &str, session: &Session) -> Result<String, String> {
    let (_flags, operands) = parse_args(args);
    let path = match operands.first() {
        None => session.cwd.clone(),
        Some(p) => VirtualFS::resolve(&session.cwd, p),
    };
    VirtualFS::tree(&session.vault.filesystem, &path)
}

pub fn pwd(session: &Session) -> String {
    session.cwd.clone()
}

pub fn cd(args: &str, session: &mut Session) -> Result<String, String> {
    let (_flags, operands) = parse_args(args);
    let arg = operands.first().copied().unwrap_or("");
    let target = if arg.is_empty() || arg == "~" {
        session.home.clone()
    } else if let Some(rest) = arg.strip_prefix("~/") {
        VirtualFS::resolve(&session.home, rest)
    } else {
        VirtualFS::resolve(&session.cwd, arg)
    };
    let target_clean = target.trim_end_matches('/');
    if target_clean.is_empty() {
        session.cwd = "/".to_string();
        return Ok(String::new());
    }
    // Verify path exists and is a directory
    match session.vault.filesystem.get(target_clean) {
        Some(entry) if entry.entry_type == "dir" => {
            session.cwd = target_clean.to_string();
            Ok(String::new())
        }
        Some(_) => Err(format!("monad: cd: {}: Not a directory", args)),
        None => Err(format!("monad: cd: {}: No such file or directory", args)),
    }
}

pub fn find(args: &str, session: &Session) -> Result<String, String> {
    let (_flags, operands) = parse_args(args);

    // `find <root> -name <pattern>` or just `find <pattern>` — we treat any
    // non-flag operand that isn't an absolute path as the search pattern, and
    // an absolute path (or the implicit cwd) as the root to search under.
    let mut root = session.cwd.clone();
    let mut pattern: Option<String> = None;
    let mut tokens = operands.iter().peekable();
    while let Some(tok) = tokens.next() {
        if *tok == "-name" || *tok == "-iname" {
            if let Some(p) = tokens.next() {
                pattern = Some(p.trim_matches('*').to_string());
            }
        } else if tok.starts_with('/') || *tok == "." || *tok == "~" {
            root = VirtualFS::resolve(&session.cwd, tok);
        } else {
            pattern = Some(tok.trim_matches('*').to_string());
        }
    }

    let root_prefix = if root == "/" { "/".to_string() } else { format!("{}/", root) };
    let mut matches: Vec<&String> = session
        .vault
        .filesystem
        .keys()
        .filter(|k| *k == &root || k.starts_with(&root_prefix))
        .filter(|k| match &pattern {
            None => true,
            Some(pat) => k.rsplit('/').next().unwrap_or(k).contains(pat.as_str()),
        })
        .collect();
    matches.sort();

    if matches.is_empty() {
        return Err(format!("find: no matches under {}", root));
    }
    Ok(matches
        .iter()
        .map(|p| {
            let is_dir = session
                .vault
                .filesystem
                .get(*p)
                .map(|e| e.entry_type == "dir")
                .unwrap_or(false);
            if is_dir { Style::cyan(p) } else { (*p).clone() }
        })
        .collect::<Vec<_>>()
        .join("\r\n"))
}
