use colored::*;
use std::fs;
use std::path::PathBuf;

/// Get the path to the config file
pub fn get_config_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("struct").join("ignores.txt")
}

/// Load patterns from config file
pub fn load_config_patterns() -> Vec<String> {
    let config_path = get_config_path();
    if let Ok(content) = fs::read_to_string(&config_path) {
        content.lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && !s.starts_with('#'))
            .collect()
    } else {
        Vec::new()
    }
}

/// Save patterns to config file
fn save_config_patterns(patterns: &[String]) -> std::io::Result<()> {
    let config_path = get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&config_path, patterns.join("\n"))
}

/// Add a pattern to the config file
pub fn add_config_pattern(pattern: String) {
    let mut patterns = load_config_patterns();
    if patterns.contains(&pattern) {
        println!("{} already in config", pattern.yellow());
        return;
    }
    patterns.push(pattern.clone());
    if let Err(e) = save_config_patterns(&patterns) {
        eprintln!("failed to save config: {}", e);
        return;
    }
    println!("{} added to config", pattern.green());
    println!("config file: {}", get_config_path().display().to_string().bright_black());
}

/// Remove a pattern from the config file
pub fn remove_config_pattern(pattern: String) {
    let mut patterns = load_config_patterns();
    let before_len = patterns.len();
    patterns.retain(|p| p != &pattern);
    
    if patterns.len() == before_len {
        println!("{} not found in config", pattern.yellow());
        return;
    }
    
    if let Err(e) = save_config_patterns(&patterns) {
        eprintln!("failed to save config: {}", e);
        return;
    }
    println!("{} removed from config", pattern.red());
}

/// List all patterns in the config file
pub fn list_config_patterns() {
    let patterns = load_config_patterns();
    if patterns.is_empty() {
        println!("no custom patterns configured");
        println!("add some with: struct add \"pattern\"");
        return;
    }
    
    println!("{}", "custom ignore patterns:".bright_black());
    for pattern in patterns {
        println!("  {}", pattern.cyan());
    }
    println!("\nconfig file: {}", get_config_path().display().to_string().bright_black());
}

/// Clear all patterns from the config file
pub fn clear_config_patterns() {
    let config_path = get_config_path();
    if config_path.exists() {
        if let Err(e) = fs::remove_file(&config_path) {
            eprintln!("failed to clear config: {}", e);
            return;
        }
        println!("{}", "cleared all custom patterns".green());
    } else {
        println!("no config file to clear");
    }
}