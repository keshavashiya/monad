use std::collections::HashMap;
use crate::vault::FsEntry;

/// Virtual filesystem navigation over the vault's filesystem tree.
/// No writes allowed. No symlinks. No hard links.
/// Just ls, cat, tree, pwd, cd.
pub struct VirtualFS;

#[derive(Debug)]
pub struct DirEntry {
    pub name: String,
    pub entry: FsEntry,
}

impl VirtualFS {
    /// Resolve an absolute or relative path to an absolute path.
    pub fn resolve(cwd: &str, path: &str) -> String {
        if path.starts_with('/') {
            clean_path(path)
        } else {
            clean_path(&format!("{}/{}", cwd, path))
        }
    }

    /// List contents of a directory.
    pub fn ls(fs: &HashMap<String, FsEntry>, path: &str) -> Result<Vec<DirEntry>, String> {
        let abs = clean_path(path);

        // Check if the path itself exists
        match fs.get(&abs) {
            Some(entry) if entry.entry_type == "file" => {
                // ls on a file just shows the file
                let name = abs.rsplit('/').next().unwrap_or(&abs);
                Ok(vec![DirEntry {
                    name: name.to_string(),
                    entry: entry.clone(),
                }])
            }
            Some(_) => {
                // ls on a directory: find all direct children
                let mut entries: Vec<DirEntry> = Vec::new();
                let prefix = if abs == "/" { abs.clone() } else { format!("{}/", abs) };
                for (k, v) in fs {
                    if k.starts_with(&prefix) && k.len() > prefix.len() {
                        let rest = &k[prefix.len()..];
                        if !rest.contains('/') {
                            entries.push(DirEntry {
                                name: rest.to_string(),
                                entry: v.clone(),
                            });
                        }
                    }
                }
                entries.sort_by(|a, b| a.name.cmp(&b.name));
                Ok(entries)
            }
            None => Err(format!("monad: {}: No such file or directory", path)),
        }
    }

    /// Read a file's content.
    pub fn cat(fs: &HashMap<String, FsEntry>, path: &str) -> Result<String, String> {
        let abs = clean_path(path);
        match fs.get(&abs) {
            Some(entry) if entry.entry_type == "file" => {
                Ok(entry.content.clone().unwrap_or_default())
            }
            Some(_) => Err(format!("monad: {}: Is a directory", path)),
            None => Err(format!("monad: {}: No such file or directory", path)),
        }
    }

    /// Build a tree representation (simplified, 2 levels deep).
    pub fn tree(fs: &HashMap<String, FsEntry>, path: &str) -> Result<String, String> {
        let abs = clean_path(path);
        let mut output = String::from(&abs);
        let prefix = if abs == "/" { abs.clone() } else { format!("{}/", abs) };
        let mut entries: Vec<(&String, &FsEntry)> = fs.iter()
            .filter(|(k, _)| k.starts_with(&prefix) && k.len() > prefix.len())
            .collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        for (k, v) in &entries {
            if k.matches('/').count() > abs.matches('/').count() + 1 {
                continue; // skip deeper nesting for simplicity
            }
            let name = k[prefix.len()..].to_string();
            let prefix_char = if v.entry_type == "dir" { "├──" } else { "└──" };
            let suffix = if v.entry_type == "dir" { "/" } else { "" };
            output.push_str(&format!("\n{} {}{}", prefix_char, name, suffix));
        }
        Ok(output)
    }
}

/// Normalize a path: remove trailing slashes, resolve .. and .
fn clean_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for part in path.split('/') {
        match part {
            "" | "." => continue,
            ".." => { parts.pop(); }
            _ => parts.push(part),
        }
    }
    if parts.is_empty() {
        return "/".to_string();
    }
    format!("/{}", parts.join("/"))
}
