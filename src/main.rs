use clap::Parser;
use colored::*;
use regex::Regex;
use std::path::PathBuf;

mod config;
mod display;
mod ignores;
mod search;
mod summary;
mod utils;

use config::{add_config_pattern, clear_config_patterns, list_config_patterns, load_config_patterns, remove_config_pattern};
use display::{display_tree, get_git_tracked_files, StructConfig};
use search::search_files;
use summary::display_summary;

#[derive(Parser, Debug)]
#[command(name = "struct")]
#[command(about = "A smarter tree command with intelligent defaults", long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Maximum depth to display (0 = current dir only, default = infinite)
    #[arg(value_name = "DEPTH")]
    depth: Option<usize>,

    /// Starting directory
    #[arg(short = 'p', long = "path", default_value = ".")]
    path: PathBuf,

    /// Show git-tracked files only
    #[arg(short = 'g', long = "git")]
    git_mode: bool,

    /// Custom ignore patterns (comma-separated, e.g., "*.log,temp*")
    #[arg(short = 'i', long = "ignore")]
    ignore_patterns: Option<String>,

    /// Skip folders larger than SIZE MB
    #[arg(short = 's', long = "skip-large")]
    max_size_mb: Option<u64>,

    /// Show file sizes
    #[arg(short = 'z', long = "size")]
    show_size: bool,

    /// Disable ignores: 'all', 'defaults', 'config', or specific pattern
    #[arg(short = 'n', long = "no-ignore")]
    no_ignore: Option<String>,
}

#[derive(clap::Subcommand, Debug)]
enum Commands {
    /// Add a pattern to the config file
    Add {
        /// Pattern to add (e.g., "cache", "*.log")
        pattern: String,
    },
    /// Remove a pattern from the config file
    Remove {
        /// Pattern to remove
        pattern: String,
    },
    /// List all custom ignore patterns
    List,
    /// Clear all custom ignore patterns
    Clear,
    /// Search for files matching a pattern
    Search {
        /// Pattern to search for (e.g., "*.env", "config", "test*")
        pattern: String,
        /// Maximum depth to search (0 for infinite)
        #[arg(short = 'd', long = "depth", default_value = "0")]
        depth: usize,
        /// Flat output (show full paths instead of tree)
        #[arg(short = 'f', long = "flat")]
        flat: bool,
        /// Starting directory (defaults to current directory)
        #[arg(default_value = ".")]
        path: PathBuf,
    },
}

fn main() {
    let args = Args::parse();

    // Handle subcommands
    if let Some(command) = args.command {
        match command {
            Commands::Add { pattern } => {
                add_config_pattern(pattern);
                return;
            }
            Commands::Remove { pattern } => {
                remove_config_pattern(pattern);
                return;
            }
            Commands::List => {
                list_config_patterns();
                return;
            }
            Commands::Clear => {
                clear_config_patterns();
                return;
            }
            Commands::Search { pattern, depth, flat, path } => {
                let max_depth = if depth == 0 { usize::MAX } else { depth };
                
                // Load config patterns for search
                let config_patterns = load_config_patterns();
                let mut custom_ignores = Vec::new();
                for pattern in config_patterns {
                    let pattern = pattern.replace("*", ".*");
                    if let Ok(re) = Regex::new(&format!("^{}$", pattern)) {
                        custom_ignores.push(re);
                    }
                }
                
                search_files(&pattern, &path, max_depth, flat, &custom_ignores);
                return;
            }
        }
    }

    // Depth: None = infinite, 0 = current dir only, otherwise use provided
    let depth = match args.depth {
        None => usize::MAX,         // No depth arg = infinite
        Some(0) => 1,                // 0 = current dir only  
        Some(d) => d,                // Use provided depth
    };
    
    let max_size_bytes = args.max_size_mb.map(|mb| mb * 1024 * 1024);

    // Parse no-ignore option
    let (skip_defaults, skip_config, skip_specific) = match args.no_ignore {
        Some(ref mode) => match mode.as_str() {
            "all" => (true, true, None),
            "defaults" => (true, false, None),
            "config" => (false, true, None),
            pattern => (false, false, Some(pattern.to_string())),
        },
        None => (false, false, None),
    };

    // Load config patterns
    let config_patterns = if skip_config {
        Vec::new()
    } else {
        load_config_patterns()
    };

    // Parse custom ignore patterns (from -i flag)
    let mut custom_ignores = Vec::new();
    
    // Add config file patterns
    for pattern in config_patterns {
        let pattern = pattern.replace("*", ".*");
        if let Ok(re) = Regex::new(&format!("^{}$", pattern)) {
            custom_ignores.push(re);
        }
    }
    
    // Add command-line patterns
    if let Some(patterns) = args.ignore_patterns {
        for pattern in patterns.split(',') {
            let pattern = pattern.trim().replace("*", ".*");
            if let Ok(re) = Regex::new(&format!("^{}$", pattern)) {
                custom_ignores.push(re);
            }
        }
    }

    // Get git-tracked files if in git mode
    let git_files = if args.git_mode {
        get_git_tracked_files(&args.path)
    } else {
        None
    };

    let config = StructConfig {
        depth,
        custom_ignores,
        max_size_bytes,
        git_files,
        show_size: args.show_size,
        skip_defaults,
        skip_specific,
    };

    // Special mode: depth 1 (struct 0) shows detailed summary
    if depth == 1 {
        display_summary(&args.path);
        return;
    }

    println!("{}", args.path.display().to_string().cyan());
    display_tree(&args.path, &config, 0, "", true);
}