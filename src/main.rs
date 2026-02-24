use clap::Parser;
use colored::*;
use git2::Repository;
use regex::Regex;
use std::ffi::OsString;
use std::path::PathBuf;

mod config;
mod display;
mod ignores;
mod search;
mod summary;
mod utils;

use crate::config::{
    add_config_pattern, clear_config_patterns, list_config_patterns, load_config_patterns,
    remove_config_pattern,
};
use display::{
    display_tree, get_git_changed_files, get_git_staged_files, get_git_tracked_files,
    get_git_untracked_files, GitMode, StructConfig,
};
use search::search_files;
use summary::display_summary;

// ─── Help ─────────────────────────────────────────────────────────────────────

const HELP: &str = "\
A smarter tree — intelligent defaults, git awareness, fast search

USAGE:
  struct [DEPTH] [PATH] [FLAGS]
  struct search \"PATTERN\" [PATH] [DEPTH] [FLAGS]
  struct 0 [PATH]                      → detailed summary view

GIT:
  struct --gr                          tracked files from git root
  struct --gur                         untracked files from git root
  struct --gsr                         staged files from git root
  struct --gcr                         changed (unstaged) from git root
  struct --gu ~/projects               untracked (from given path)
  struct --gc ~/projects               changed (from given path)
  (when multiple git flags conflict, highest priority wins:
   changed > staged > untracked > tracked > history)

SEARCH:
  struct search \"*.py\" ~/projects      find .py files (tree view)
  struct search \"*.py\" ~/projects 3    search max 3 levels deep
  struct search \"gui\" .                anything containing 'gui'
  struct search \"gui*\" . -f            flat output (full paths)
  struct search \"*.log\" . -i \"venv\"    search, ignoring venv
  struct search \"*.wav\" . -i \"win,Linux\"

CONFIG:
  struct add \"pattern\"                 add to persistent ignores
  struct remove \"pattern\"              remove from persistent ignores
  struct list                          list config patterns
  struct clear                         clear all config patterns

FLAGS:
  -i \"p1,p2\"   ignore patterns (dirs or files, comma-separated)
  -n TARGET    un-ignore: a pattern name, 'defaults', 'config', or 'all'
               (can be specified multiple times: -n defaults -n config)
  -z           show file/dir sizes
  -s SIZE      skip dirs larger than SIZE megabytes
  -g/--git     git mode flags: --gu --gs --gc --gh  (current dir)
               root variants:  --gr --gur --gsr --gcr --ghr
  -h, --help   print this help
  -V, --version";

// ─── Clap — flags only, no positionals ───────────────────────────────────────
// Positionals (DEPTH and PATH) are extracted from argv before clap sees them,
// so clap never gets confused between a number-depth and a path.

#[derive(Parser, Debug)]
#[command(name = "struct")]
#[command(version)]
#[command(disable_help_flag = true)]
#[command(override_usage = "struct [DEPTH] [PATH] [FLAGS]\n       struct search \"PATTERN\" [PATH] [DEPTH] [FLAGS]")]
struct Flags {
    #[command(subcommand)]
    command: Option<Commands>,

    #[arg(short = 'g', long = "git", hide = true)]
    git_tracked: bool,
    #[arg(long = "gu", hide = true)]
    git_untracked: bool,
    #[arg(long = "gs", hide = true)]
    git_staged: bool,
    #[arg(long = "gc", hide = true)]
    git_changed: bool,
    #[arg(long = "gh", hide = true)]
    git_history: bool,
    #[arg(long = "gr", hide = true)]
    git_root: bool,
    #[arg(long = "gur", hide = true)]
    git_untracked_root: bool,
    #[arg(long = "gsr", hide = true)]
    git_staged_root: bool,
    #[arg(long = "gcr", hide = true)]
    git_changed_root: bool,
    #[arg(long = "ghr", hide = true)]
    git_history_root: bool,

    #[arg(short = 'i', long = "ignore", value_name = "PATTERNS", hide = true)]
    ignore_patterns: Option<String>,

    #[arg(short = 's', long = "skip-large", value_name = "SIZE", hide = true)]
    max_size_mb: Option<u64>,

    #[arg(short = 'z', long = "size", hide = true)]
    show_size: bool,

    /// Can be given multiple times: -n defaults -n config
    #[arg(short = 'n', long = "no-ignore", value_name = "TARGET", action = clap::ArgAction::Append, hide = true)]
    no_ignore: Vec<String>,

    #[arg(short = 'h', long = "help", action = clap::ArgAction::SetTrue, hide = true)]
    help: bool,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Add a pattern to the persistent ignore config
    Add { pattern: String },
    /// Remove a pattern from the persistent ignore config
    Remove { pattern: String },
    /// List all persistent ignore patterns
    List,
    /// Clear all persistent ignore patterns
    Clear,
    /// Search for files/dirs matching a pattern
    ///
    /// Plain text = substring match. Wildcards (* ?) = glob match.
    Search {
        pattern: String,
        #[arg(default_value = ".")]
        path: PathBuf,
        #[arg(value_name = "DEPTH", default_value = "0")]
        depth: usize,
        #[arg(short = 'f', long = "flat")]
        flat: bool,
        #[arg(short = 'i', long = "ignore", value_name = "PATTERNS")]
        ignore_patterns: Option<String>,
    },
}

// ─── Pre-processing ───────────────────────────────────────────────────────────

/// Inspect the subcommands to know if argv[1] is a subcommand keyword.
fn is_subcommand(s: &str) -> bool {
    matches!(s, "search" | "add" | "remove" | "list" | "clear" | "help")
}

/// Extract DEPTH and PATH from argv before handing to clap.
/// Returns (depth, path, cleaned_argv_without_those_tokens).
///
/// Rules:
///   - Skip argv[0] (binary name) and any subcommand keyword at argv[1].
///   - A token that starts with '-' is a flag — leave it alone.
///   - A token that is a flag VALUE (follows a flag that takes a value) — skip it.
///   - First remaining bare token that parses as usize → DEPTH (removed).
///   - First remaining bare token that doesn't → PATH (removed).
///   - Any further bare tokens are silently discarded (they'd cause clap
///     "unrecognized subcommand" errors since clap has no positionals defined).
fn preprocess_argv() -> (Option<usize>, Option<PathBuf>, Vec<OsString>) {
    // Flags that consume the next token as their value — we must not mistake
    // that value for a DEPTH or PATH.
    const VALUE_FLAGS: &[&str] = &["-i", "--ignore", "-s", "--skip-large", "-n", "--no-ignore"];

    let raw: Vec<String> = std::env::args().collect();
    let mut cleaned: Vec<OsString> = Vec::new();
    let mut depth: Option<usize> = None;
    let mut path: Option<PathBuf> = None;

    // Always keep argv[0]
    if let Some(bin) = raw.get(0) {
        cleaned.push(bin.into());
    }

    // If argv[1] is a subcommand, keep it and pass everything else through unchanged
    if raw.get(1).map(|s| is_subcommand(s.as_str())).unwrap_or(false) {
        for tok in raw.iter().skip(1) {
            cleaned.push(tok.into());
        }
        return (None, None, cleaned);
    }

    let mut skip_next = false;
    for tok in raw.iter().skip(1) {
        if skip_next {
            // This token is the value of a preceding flag — keep it, don't parse
            cleaned.push(tok.into());
            skip_next = false;
            continue;
        }

        if tok.starts_with('-') {
            cleaned.push(tok.into());
            // If this flag consumes a value, mark next token to skip
            if VALUE_FLAGS.contains(&tok.as_str()) {
                skip_next = true;
            }
            // Handle --flag=value form: no skip needed since value is embedded
            continue;
        }

        // Bare token — try to claim as DEPTH or PATH
        if depth.is_none() {
            if let Ok(n) = tok.parse::<usize>() {
                depth = Some(n);
                continue; // consumed — don't push to cleaned
            }
        }
        if path.is_none() {
            path = Some(PathBuf::from(tok));
            continue; // consumed
        }

        // Extra bare token (second path, extra number, etc.) — silently discard.
        // Passing these to clap causes "unrecognized subcommand" errors since clap
        // has no positionals defined and treats bare tokens as subcommand names.
        let _ = tok; // consumed, ignored
    }

    (depth, path, cleaned)
}

// ─── Ignore flag processing ───────────────────────────────────────────────────

/// Fold multiple -n values into (skip_defaults, skip_config, skip_specific_patterns).
fn parse_no_ignore(values: &[String]) -> (bool, bool, Vec<String>) {
    let mut skip_defaults = false;
    let mut skip_config = false;
    let mut specifics: Vec<String> = Vec::new();

    for v in values {
        match v.as_str() {
            "all"      => { skip_defaults = true; skip_config = true; }
            "defaults" => { skip_defaults = true; }
            "config"   => { skip_config = true; }
            pattern    => { specifics.push(pattern.to_string()); }
        }
    }
    (skip_defaults, skip_config, specifics)
}

fn build_ignores_from_patterns(patterns: Vec<String>) -> Vec<Regex> {
    patterns
        .iter()
        .filter_map(|p| {
            let p = p.trim().replace("*", ".*");
            Regex::new(&format!("^{}$", p)).ok()
        })
        .collect()
}

// ─── Main ─────────────────────────────────────────────────────────────────────

fn main() {
    let raw_strs: Vec<String> = std::env::args().collect();

    // Intercept -h / --help for top-level (not subcommands)
    let top_level = !raw_strs.get(1).map(|s| is_subcommand(s)).unwrap_or(false);
    if top_level && (raw_strs.contains(&"-h".to_string()) || raw_strs.contains(&"--help".to_string())) {
        println!("{}", HELP);
        return;
    }

    // Pre-process: pull out DEPTH and PATH before clap sees argv
    let (raw_depth, raw_path, cleaned_argv) = preprocess_argv();

    // Parse only flags
    let flags = Flags::parse_from(cleaned_argv);

    // ── Subcommands ───────────────────────────────────────────────────────────
    if let Some(command) = flags.command {
        match command {
            Commands::Add { pattern } => { add_config_pattern(pattern); return; }
            Commands::Remove { pattern } => { remove_config_pattern(pattern); return; }
            Commands::List => { list_config_patterns(); return; }
            Commands::Clear => { clear_config_patterns(); return; }

            Commands::Search { pattern, path, depth, flat, ignore_patterns } => {
                let max_depth = if depth == 0 { usize::MAX } else { depth };
                let mut all_patterns = load_config_patterns();
                if let Some(inline) = ignore_patterns {
                    for p in inline.split(',') {
                        let p = p.trim().to_string();
                        if !p.is_empty() { all_patterns.push(p); }
                    }
                }
                let custom_ignores = build_ignores_from_patterns(all_patterns);
                search_files(&pattern, &path, max_depth, flat, &custom_ignores);
                return;
            }
        }
    }

    // ── Resolve path and depth ────────────────────────────────────────────────
    let path = raw_path.unwrap_or_else(|| PathBuf::from("."));

    let depth_for_tree = match raw_depth {
        None    => usize::MAX,
        Some(0) => 1,   // 0 means summary; display_tree still needs 1 internally
        Some(d) => d,
    };

    let max_size_bytes = flags.max_size_mb.map(|mb| mb * 1024 * 1024);

    // ── Git mode (conflicting flags: highest priority wins) ───────────────────
    let git_mode = if flags.git_changed || flags.git_changed_root {
        Some(GitMode::Changed)
    } else if flags.git_staged || flags.git_staged_root {
        Some(GitMode::Staged)
    } else if flags.git_untracked || flags.git_untracked_root {
        Some(GitMode::Untracked)
    } else if flags.git_tracked || flags.git_root {
        Some(GitMode::Tracked)
    } else if flags.git_history || flags.git_history_root {
        Some(GitMode::History)
    } else {
        None
    };

    let use_git_root = flags.git_root
        || flags.git_untracked_root
        || flags.git_staged_root
        || flags.git_changed_root
        || flags.git_history_root;

    if git_mode.is_some() && Repository::discover(&path).is_err() {
        eprintln!("error: not in a git repository");
        return;
    }

    let start_path = if use_git_root {
        match Repository::discover(&path) {
            Ok(repo) => repo.workdir().map(|w| w.to_path_buf()).unwrap_or_else(|| path.clone()),
            Err(_) => { eprintln!("error: not in a git repository"); return; }
        }
    } else {
        path.clone()
    };

    // ── Ignore config ─────────────────────────────────────────────────────────
    let (skip_defaults, skip_config, skip_specifics) = parse_no_ignore(&flags.no_ignore);

    // depth 0 + git flags: git filtering is ignored for summary (summary shows dir stats, not file lists)
    if raw_depth == Some(0) {
        display_summary(&start_path);
        return;
    }

    let config_patterns = if skip_config { Vec::new() } else { load_config_patterns() };
    let mut all_patterns = config_patterns;

    // Add skip_specifics as additional ignore patterns (un-ignore means remove from
    // default list, handled in display.rs via skip_specific — we pass the first one
    // for backward compat; multiple specifics: each gets its own skip_specific pass)
    if let Some(inline) = flags.ignore_patterns {
        for p in inline.split(',') {
            let p = p.trim().to_string();
            if !p.is_empty() { all_patterns.push(p); }
        }
    }
    let custom_ignores = build_ignores_from_patterns(all_patterns);

    // ── Git file sets ─────────────────────────────────────────────────────────
    let git_files = if let Some(ref mode) = git_mode {
        match mode {
            GitMode::Tracked   => get_git_tracked_files(&start_path),
            GitMode::Untracked => get_git_untracked_files(&start_path),
            GitMode::Staged    => get_git_staged_files(&start_path),
            GitMode::Changed   => get_git_changed_files(&start_path),
            GitMode::History   => None,
        }
    } else {
        None
    };

    // For multiple -n specifics, use the first one (StructConfig takes one skip_specific).
    // display.rs would need updating to support a Vec — for now first wins.
    let skip_specific = skip_specifics.into_iter().next();

    let config = StructConfig {
        depth: depth_for_tree,
        custom_ignores,
        max_size_bytes,
        git_files,
        git_mode,
        show_size: flags.show_size,
        skip_defaults,
        skip_specific,
    };

    println!("{}", start_path.display().to_string().cyan());
    display_tree(&start_path, &config, 0, "", true);
}