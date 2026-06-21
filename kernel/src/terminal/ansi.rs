/// Minimal ANSI escape sequence builder.
/// We use a small subset of ANSI for output formatting:
///   - SGR parameters (colours, bold, dim)
///   - Carriage return + newline for line endings
///   - OSC-8 hyperlinks for clickable links
pub struct Style;

// Some palette/style helpers are part of the kernel's public formatting surface
// but not used by every build target; keep them available without warnings.
#[allow(dead_code)]
impl Style {
    // Reset
    pub fn reset() -> &'static str { "\x1b[0m" }

    // Bold
    pub fn bold(s: &str) -> String { format!("\x1b[1m{}\x1b[22m", s) }

    // Dim
    pub fn dim(s: &str) -> String { format!("\x1b[2m{}\x1b[22m", s) }

    // Underline
    pub fn underline(s: &str) -> String { format!("\x1b[4m{}\x1b[24m", s) }

    // Italic
    pub fn italic(s: &str) -> String { format!("\x1b[3m{}\x1b[23m", s) }

    // Foreground colours (8-bit)
    pub fn fg(code: u8, s: &str) -> String { format!("\x1b[38;5;{}m{}\x1b[0m", code, s) }

    // Background colours (8-bit)
    pub fn bg(code: u8, s: &str) -> String { format!("\x1b[48;5;{}m{}\x1b[0m", code, s) }

    // Named colours
    pub fn amber(s: &str) -> String   { Self::fg(214, s) }  // #e6b91e
    pub fn gold(s: &str) -> String    { Self::fg(220, s) }  // #ffd866
    pub fn cyan(s: &str) -> String    { Self::fg(81, s) }   // #78dce8
    pub fn green(s: &str) -> String   { Self::fg(113, s) }  // #87d96c
    pub fn red(s: &str) -> String     { Self::fg(131, s) }  // #d47766
    pub fn white(s: &str) -> String   { Self::fg(251, s) }  // #c6c6c6
    pub fn grey(s: &str) -> String    { Self::fg(240, s) }  // #585858

    // Header line (amber, bold)
    pub fn header(s: &str) -> String { Self::bold(&Self::amber(s)) }

    // OSC-8 hyperlink: clickable in modern terminals
    pub fn hyperlink(uri: &str, text: &str) -> String {
        format!("\x1b]8;;{}\x1b\\{}\x1b]8;;\x1b\\", uri, Self::cyan(text))
    }

    // Horizontal rule using dim dashes
    pub fn hr() -> String {
        format!("\x1b[2m{}\x1b[22m\r\n", "─".repeat(56))
    }

    /// A dim horizontal rule `width` columns wide (no trailing newline), so a
    /// heading rule can flex to the terminal width the adapter reported.
    pub fn rule(width: usize) -> String {
        Self::dim(&"─".repeat(width))
    }
}

/// Strip ANSI escape sequences (SGR and OSC-8) from a string, returning only visible text.
pub fn strip_ansi(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            match chars.peek() {
                Some('[') => {
                    chars.next(); // consume '['
                    while let Some(&n) = chars.peek() {
                        if n.is_ascii_uppercase() || n.is_ascii_lowercase() {
                            chars.next(); // consume terminator letter
                            break;
                        }
                        chars.next();
                    }
                }
                Some(']') => {
                    chars.next(); // consume ']'
                    while let Some(&n) = chars.peek() {
                        if n == '\x1b' {
                            chars.next();
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                                break;
                            }
                        } else {
                            chars.next();
                        }
                    }
                }
                _ => {}
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Count visible characters in a string (excluding ANSI escapes).
pub fn visible_width(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

/// Truncate a string so its visible width ≤ max_width, preserving ANSI codes.
/// Appends '…' if truncation occurs. If max_width < 2, always returns '…'.
pub fn truncate_visible(s: &str, max_width: usize) -> String {
    if max_width < 2 {
        return '…'.to_string();
    }
    let mut out = String::new();
    let mut vis = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            out.push(c);
            match chars.peek() {
                Some('[') => {
                    out.push('['); chars.next();
                    while let Some(&n) = chars.peek() {
                        out.push(n); chars.next();
                        if n.is_ascii_uppercase() || n.is_ascii_lowercase() { break; }
                    }
                }
                Some(']') => {
                    out.push(']'); chars.next();
                    while let Some(&n) = chars.peek() {
                        out.push(n); chars.next();
                        if n == '\x1b' {
                            if chars.peek() == Some(&'\\') { out.push('\\'); chars.next(); }
                            break;
                        }
                    }
                }
                _ => {}
            }
        } else {
            if vis >= max_width - 1 { out.push('…'); return out; }
            out.push(c);
            vis += 1;
        }
    }
    out
}
