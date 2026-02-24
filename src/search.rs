use colored::*;
use regex::Regex;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::ignores::{should_ignore_dir, matches_custom_pattern};
use crate::utils::{format_size, is_executable};

// ─── Match mode ───────────────────────────────────────────────────────────────

/// Glob pattern (has * or ?) → compiled to a full ^..$ regex.
/// Plain text (no wildcards)  → case-insensitive substring match.
enum MatchMode {
    Glob(Regex),
    Substring(String),
}

impl MatchMode {
    fn build(pattern: &str) -> Result<Self, String> {
        // Empty pattern is not useful and matches everything — reject it
        if pattern.is_empty() {
            return Err("pattern cannot be empty — use \"*\" to match everything".to_string());
        }

        let has_wildcards = pattern.contains('*') || pattern.contains('?');
        if has_wildcards {
            // Escape all regex metacharacters, then restore our glob chars
            let escaped = regex::escape(pattern);
            let regex_pat = escaped.replace(r"\*", ".*").replace(r"\?", ".");
            let re = Regex::new(&format!("(?i)^{}$", regex_pat))
                .map_err(|e| e.to_string())?;
            Ok(MatchMode::Glob(re))
        } else {
            // Plain text: case-insensitive substring
            Ok(MatchMode::Substring(pattern.to_lowercase()))
        }
    }

    fn is_match(&self, filename: &str) -> bool {
        match self {
            MatchMode::Glob(re) => re.is_match(filename),
            MatchMode::Substring(needle) => filename.to_lowercase().contains(needle.as_str()),
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

pub fn search_files(
    pattern: &str,
    start_path: &Path,
    max_depth: usize,
    flat: bool,
    custom_ignores: &[Regex],
) {
    let matcher = match MatchMode::build(pattern) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("error: {}", e);
            return;
        }
    };

    let mut found_count = 0;
    let mut matching_paths: HashSet<PathBuf> = HashSet::new();
    let mut flat_results: Vec<(PathBuf, bool, u64)> = Vec::new(); // (path, is_dir, size)

    for entry in WalkDir::new(start_path)
        .follow_links(false)
        .max_depth(max_depth)
        .into_iter()
        .filter_entry(|e| {
            // Always allow the root itself
            if e.depth() == 0 {
                return true;
            }
            let name = match e.file_name().to_str() {
                Some(n) => n,
                None => return true,
            };
            // For directories: prune ignored ones UNLESS the dir itself is a match.
            // This lets `search "__pycache__"` find those dirs even though they're
            // in the default ignore list. We won't descend inside them (filter_entry
            // prunes recursion) so we just surface them as direct hits.
            if e.file_type().is_dir() {
                let is_ignored = should_ignore_dir(name)
                    || matches_custom_pattern(name, custom_ignores);
                if is_ignored {
                    return matcher.is_match(name);
                }
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        if entry.depth() == 0 {
            continue; // skip root
        }

        let filename = match entry.file_name().to_str() {
            Some(n) => n,
            None => continue,
        };

        if matcher.is_match(filename) {
            let file_path = entry.path().to_path_buf();
            let is_dir = entry.file_type().is_dir();

            if flat {
                let size = if is_dir {
                    0
                } else {
                    entry.metadata().map(|m| m.len()).unwrap_or(0)
                };
                flat_results.push((file_path, is_dir, size));
            } else {
                matching_paths.insert(file_path.clone());
                // Record all ancestor dirs so the tree renders correctly
                let mut cur = file_path.parent();
                while let Some(parent) = cur {
                    if parent == start_path {
                        break;
                    }
                    matching_paths.insert(parent.to_path_buf());
                    cur = parent.parent();
                }
            }

            found_count += 1;
        }
    }

    if found_count == 0 {
        println!(
            "{}",
            format!("no files or directories matching '{}' found", pattern).yellow()
        );
        return;
    }

    println!(
        "{} {}",
        format!("found {} item(s) matching", found_count).green(),
        pattern.cyan()
    );
    println!();

    if flat {
        flat_results.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, is_dir, size) in flat_results {
            if is_dir {
                println!("{}", format!("{}/", path.display()).blue().bold());
            } else {
                let size_str = format!(" ({})", format_size(size)).bright_black();
                println!("{}{}", path.display().to_string().cyan(), size_str);
            }
        }
    } else {
        display_search_tree(start_path, &matching_paths, "", true);
    }
}

// ─── Tree display ─────────────────────────────────────────────────────────────

fn display_search_tree(
    path: &Path,
    matching_paths: &HashSet<PathBuf>,
    prefix: &str,
    _is_last: bool,
) {
    let mut entries: Vec<_> = match fs::read_dir(path) {
        Ok(entries) => entries
            .filter_map(|e| e.ok())
            .filter(|e| {
                let ep = e.path();
                matching_paths.contains(&ep)
                    || matching_paths.iter().any(|p| p.starts_with(&ep))
            })
            .collect(),
        Err(_) => return,
    };

    // Dirs first, then alphabetical
    entries.sort_by_key(|e| {
        let is_dir = e.path().is_dir();
        let name = e.file_name().to_string_lossy().to_lowercase();
        (!is_dir, name)
    });

    let total = entries.len();

    for (idx, entry) in entries.iter().enumerate() {
        let is_last_entry = idx == total - 1;
        let entry_path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        let is_dir = entry_path.is_dir();
        let connector = if is_last_entry { "└── " } else { "├── " };

        if is_dir {
            println!(
                "{}{}{}",
                prefix,
                connector,
                format!("{}/", name).blue().bold()
            );
            let new_prefix = if is_last_entry {
                format!("{}    ", prefix)
            } else {
                format!("{}│   ", prefix)
            };
            display_search_tree(&entry_path, matching_paths, &new_prefix, is_last_entry);
        } else {
            let file_name = if is_executable(&entry_path) {
                name.green().bold()
            } else {
                name.cyan().bold()
            };
            if let Ok(metadata) = fs::metadata(&entry_path) {
                let size_str = format!(" ({})", format_size(metadata.len())).bright_black();
                println!("{}{}{}{}", prefix, connector, file_name, size_str);
            } else {
                println!("{}{}{}", prefix, connector, file_name);
            }
        }
    }
}