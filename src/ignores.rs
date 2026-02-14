use regex::Regex;

/// Check if a directory should be ignored by default
pub fn should_ignore_dir(name: &str) -> bool {
    matches!(
        name,
        "__pycache__" | ".pytest_cache" | ".mypy_cache" | ".ruff_cache" |
        ".tox" | "dist" | "build" | ".coverage" |
        "venv" | ".venv" | "env" | ".env" | "virtualenv" |
        "node_modules" | ".npm" | ".yarn" |
        ".git" | ".svn" | ".hg" |
        ".vscode" | ".idea" | ".obsidian" |
        "target" | "bin" | "obj" | ".next" | ".nuxt" |
        ".DS_Store" |
        "chrome_profile" | "lofi_chrome_profile" |
        "GPUCache" | "ShaderCache" | "GrShaderCache" |
        "Cache" | "blob_storage"
    ) || name.ends_with(".egg-info")
}

/// Check if a file should be ignored by default
pub fn should_ignore_file(name: &str) -> bool {
    matches!(
        name.split('.').last().unwrap_or(""),
        "pyc" | "pyo" | "pyd" | "swp" | "swo"
    ) || name == "package-lock.json" || name == ".DS_Store"
}

/// Check if a name matches any of the custom patterns
pub fn matches_custom_pattern(name: &str, patterns: &[Regex]) -> bool {
    patterns.iter().any(|re| re.is_match(name))
}