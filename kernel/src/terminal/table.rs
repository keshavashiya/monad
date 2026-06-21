use crate::terminal::ansi::{Style, visible_width, truncate_visible};

/// Simple table renderer for structured command output.
/// Takes a header row and data rows, produces aligned ANSI-styled output.
pub struct Table;

#[derive(Debug)]
pub struct Column {
    pub header: &'static str,
    pub width: usize,
}

impl Table {
    /// Render a table with headers and rows. Columns auto-space.
    ///
    /// `flex` is the terminal width in columns when the last column should grow
    /// to fill the screen (free-form text like a DESCRIPTION); pass `None` to
    /// keep every column at its fixed declared width.
    pub fn render(headers: &[Column], rows: &[Vec<String>], flex: Option<usize>) -> String {
        if rows.is_empty() {
            return String::new();
        }

        let col_count = headers.len();
        let mut output = String::new();

        // The column where the last cell begins, so wrapped continuation lines
        // line up under it instead of falling back to column 0.
        let last = col_count - 1;

        // Per-column widths. Default to the declared widths — the fixed layout
        // used when the adapter doesn't know the terminal size (e.g. the HTTP
        // daemon), so inner columns truncate to keep the grid bounded.
        let mut widths: Vec<usize> = headers.iter().map(|c| c.width).collect();

        // When the terminal width is known, grow every inner column to fit its
        // widest cell (declared width is a floor, never a ceiling) so nothing
        // truncates, and let the final free-form column absorb the remainder.
        if let Some(cols) = flex {
            for (i, w) in widths.iter_mut().enumerate() {
                let natural = rows
                    .iter()
                    .filter_map(|r| r.get(i))
                    .map(|c| visible_width(c))
                    .chain(std::iter::once(visible_width(headers[i].header)))
                    .max()
                    .unwrap_or(*w);
                *w = (*w).max(natural);
            }
            let inner: usize = widths[..last].iter().map(|w| w + 2).sum();
            // Only expand if the content-fit inner columns still leave room for
            // the last column; otherwise fall back to the fixed declared widths
            // so the grid stays aligned on a narrow terminal.
            if inner + headers[last].width <= cols {
                widths[last] = cols.saturating_sub(inner).max(headers[last].width);
            } else {
                widths = headers.iter().map(|c| c.width).collect();
            }
        }

        let indent: usize = widths[..last].iter().map(|w| w + 2).sum();
        let last_width = widths[last];

        // Header row
        for (i, col) in headers.iter().enumerate() {
            let h = Style::bold(&Style::amber(col.header));
            output.push_str(&h);
            if i < col_count - 1 {
                let pad = widths[i].saturating_sub(visible_width(&h)) + 2;
                output.push_str(&" ".repeat(pad));
            }
        }
        output.push_str("\r\n");

        // Separator
        for (i, w) in widths.iter().enumerate() {
            let sep: String = "─".repeat(*w);
            output.push_str(&Style::grey(&sep));
            if i < col_count - 1 {
                output.push_str("  ");
            }
        }
        output.push_str("\r\n");

        // Data rows
        for row in rows {
            // Inner columns: truncate + pad to keep the grid aligned.
            for (i, &w) in widths[..last].iter().enumerate() {
                let cell = row.get(i).map(|s| s.as_str()).unwrap_or("");
                let display = if visible_width(cell) > w {
                    truncate_visible(cell, w)
                } else {
                    cell.to_string()
                };
                output.push_str(&display);
                let pad = w.saturating_sub(visible_width(&display)) + 2;
                output.push_str(&" ".repeat(pad));
            }
            // Last column: word-wrap free-form text (e.g. a DESCRIPTION/FOCUS) to
            // its effective width and indent continuation lines, so long text
            // stays inside the grid rather than bleeding to column 0.
            let cell = row.get(last).map(|s| s.as_str()).unwrap_or("");
            for (j, line) in wrap_visible(cell, last_width).iter().enumerate() {
                if j > 0 {
                    output.push_str(&" ".repeat(indent));
                }
                output.push_str(line);
                output.push_str("\r\n");
            }
        }

        output
    }
}

/// Greedy word-wrap of plain text into lines of at most `width` visible columns.
/// Used for the final, free-form table column. A single word longer than `width`
/// is left on its own line rather than hard-split. Always returns ≥ 1 line.
fn wrap_visible(text: &str, width: usize) -> Vec<String> {
    if width == 0 || text.trim().is_empty() {
        return vec![text.to_string()];
    }
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if current.is_empty() {
            current.push_str(word);
        } else if visible_width(&current) + 1 + visible_width(word) <= width {
            current.push(' ');
            current.push_str(word);
        } else {
            lines.push(std::mem::take(&mut current));
            current.push_str(word);
        }
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines
}
