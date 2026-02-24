# struct

Struct: tree with a developer brain.
Stop drowning in site-packages — struct shows you the code you care about.

## The Problem

Running `tree` in a project directory gives you this:

```bash
$ tree -L 3
venv/
├── lib/
│   ├── python3.11/
│   │   ├── site-packages/
│   │   │   ├── pip/
│   │   │   │   ├── __init__.py
│   │   │   │   ├── ... (2000+ files you didn't ask for)
```

I needed something that shows project structure without drowning me in dependency folders.

## What This Does

`struct` shows your project's actual structure while automatically hiding the noise:

```bash
$ struct 3
venv/ (2741 files ignored)
src/
├── main.rs
└── lib.rs
```

The folder still appears, but you get a clean file count instead of thousands of irrelevant paths.

## Installation

### Option 1: Install from crates.io
```bash
cargo install struct-cli
```

View on [crates.io](https://crates.io/crates/struct-cli)

### Option 2: Install from source
```bash
git clone https://github.com/caffienerd/struct-cli.git
cd struct-cli
chmod +x install.sh && ./install.sh
```

## Uninstallation

```bash
git clone https://github.com/caffienerd/struct-cli.git && cd struct-cli
chmod +x uninstall.sh && ./uninstall.sh
```

---

## Quick Start

```bash
struct                          # Full tree (current dir, infinite depth)
struct 0                        # Detailed summary of current directory
struct 3                        # 3 levels deep
struct ~/dir                    # Full tree of a specific directory
struct 2 ~/dir                  # Specific directory, 2 levels deep
struct ~/dir 2                  # Same — order doesn't matter
struct 5 ~/dir -z               # 5 levels with file sizes
```

---

## Complete Usage Guide

### Syntax

```
struct [DEPTH] [PATH] [FLAGS]
struct search "PATTERN" [PATH] [DEPTH] [FLAGS]
struct 0 [PATH]                       → detailed summary view
```

Both `DEPTH` and `PATH` are optional positional arguments — no flags needed.
Order doesn't matter: `struct 2 ~/dir` and `struct ~/dir 2` both work.

---

### struct 0 — Directory Summary Mode

```bash
struct 0
struct 0 ~/projects
```

**Output:**
```
/home/user/projects/myproject (main)

src/
  /home/user/projects/myproject/src
  total:    10 dirs · 45 files · 125.3K
  visible:  8 dirs · 42 files · 120.1K
  types:    rs(30) toml(5) md(3) json(2) txt(2)
  ignored:  target(948 files)

README.md
  12.5K

── ignored (top level) ──
  .git(60 files), target(948 files) · 1008 files · 45.2M
```

---

### Git Integration

Filter output by git status. All git flags can be combined with any other flag.

When multiple git flags conflict, priority is: `--gc` > `--gs` > `--gu` > `-g` > `--gh`

#### `-g, --git` — tracked files only
```bash
struct -g
struct 2 -g ~/git-project
```

#### `--gu` — untracked files only
```bash
struct --gu
struct 2 --gu ~/git-project
```

#### `--gs` — staged files only
```bash
struct --gs
```

#### `--gc` — changed/modified files only
```bash
struct --gc
```

#### `--gh` — last commit per directory
```bash
struct --gh
```

#### Root variants — start from git root regardless of current directory
```bash
struct --gr        # tracked, from git root
struct --gur       # untracked, from git root
struct --gsr       # staged, from git root
struct --gcr       # changed, from git root
struct --ghr       # history, from git root
```

**Examples:**
```bash
struct 3 -g -z                  # Tracked files with sizes
struct --gcr ~/git-project/myapp   # Changed files from repo root
struct 2 --gur                  # Untracked files from git root, 2 levels
```

---

### Flags

#### `-z, --size` — show file sizes
```bash
struct -z
struct 3 -z ~/dir
```

**Output:**
```
main.rs (8.5K)
venv/ (156.3M, 2741 files ignored)
```

#### `-s, --skip-large SIZE` — skip large directories
```bash
struct -s 100                   # Skip dirs > 100MB
struct 3 -s 500 ~/dir
```

#### `-i, --ignore PATTERNS` — inline ignore patterns
Comma-separated, wildcards supported. Merged with config patterns.

```bash
struct -i "*.log"
struct -i "*.tmp,cache*,build"
struct 3 ~/dir -i "*.log,screenshots"
```

#### `-n, --no-ignore TARGET` — un-ignore
Show things that are normally hidden. Can be given multiple times.

| Value | Effect |
|---|---|
| `all` | Disable ALL ignores |
| `defaults` | Disable built-in defaults (venv, node_modules, etc.) |
| `config` | Disable config file patterns only |
| `PATTERN` | Un-ignore one specific name (e.g. `venv`, `__pycache__`) |

```bash
struct -n all                       # Show everything
struct -n defaults                  # Show venv, __pycache__, etc.
struct -n config                    # Show config-ignored items
struct -n venv                      # Peek inside venv only
struct -n __pycache__               # Show __pycache__ contents
struct -n defaults -n config        # Same as -n all
```

---

### Config File Management

Save ignore patterns permanently so you don't have to type `-i` every time.

**Location:** `~/.config/struct/ignores.txt`

```bash
struct add "chrome_profile"     # Add a pattern
struct add "*.log"
struct remove "*.log"           # Remove a pattern
struct list                     # Show all saved patterns
struct clear                    # Delete all saved patterns
```

**Output of `struct list`:**
```
custom ignore patterns:
  chrome_profile
  *.log

config file: /home/user/.config/struct/ignores.txt
```

---

### Search

Find files and directories by pattern. Respects the same ignore rules as the tree view.

```
struct search "PATTERN" [PATH] [DEPTH] [FLAGS]
```

**Pattern matching rules:**
- **Plain text** (no `*` or `?`) → case-insensitive **substring** match
  - `search "gui"` finds `gui.py`, `gui_utils.rs`, `penguin.txt`
  - `search "cache"` finds `__pycache__`, `.cache`, `cache.json`
- **Glob patterns** (has `*` or `?`) → exact glob match
  - `search "*.py"` finds only files ending in `.py`
  - `search "test*"` finds files starting with `test`

**Basic examples:**
```bash
struct search "*.py"                    # All Python files (current dir)
struct search "gui"                     # Anything containing "gui"
struct search "__pycache__"             # Find all __pycache__ dirs
struct search "*.env" ~/dir        # .env files in ~/dir
struct search "config*" ~/dir 2    # Files starting with "config", 2 levels deep
```

**Search flags:**

#### `[DEPTH]` — limit search depth (positional, default: infinite)
```bash
struct search "*.py" . 2                # 2 levels deep
struct search "*.toml" ~/dir 1     # Top level only
```

#### `-f, --flat` — flat list instead of tree
```bash
struct search "*.py" -f
struct search "*.env" ~/dir -f
```

**Tree output (default):**
```
found 6 item(s) matching *.py

timebomb/
└── Linux/
    └── python/
        ├── app_manager.py (11.1K)
        ├── gui.py (19.4K)
        └── timer.py (18.5K)
```

**Flat output (`-f`):**
```
found 6 item(s) matching *.py

timebomb/Linux/python/app_manager.py (11.1K)
timebomb/Linux/python/gui.py (19.4K)
timebomb/Linux/python/timer.py (18.5K)
```

#### `-i, --ignore PATTERNS` — ignore patterns during search
```bash
struct search "*.wav" . -i "windows"
struct search "*.py" ~/dir -i "venv,__pycache__"
```

---

## Auto-Ignored Directories

These are hidden by default (shown with file count instead):

**Python:** `__pycache__`, `.pytest_cache`, `.mypy_cache`, `venv`, `.venv`, `env`, `virtualenv`, `*.egg-info`, `dist`, `build`

**JavaScript:** `node_modules`, `.npm`, `.yarn`

**Version Control:** `.git`, `.svn`, `.hg`

**IDEs:** `.vscode`, `.idea`, `.obsidian`

**Build Artifacts:** `target`, `bin`, `obj`, `.next`, `.nuxt`

**Caches:** `chrome_profile`, `GPUCache`, `ShaderCache`, `Cache`, `blob_storage`

**macOS:** `.DS_Store`

Use `-n all` to show everything, or `-n PATTERN` to peek at one specific folder.

---

## Real-World Examples

```bash
# Check project structure without clutter
struct 3 ~/myproject

# Find all config files in current dir
struct search "*.env"
struct search "config" . 2

# See what's actually tracked in git
struct 2 -g

# Peek inside an ignored folder
struct -n venv
struct -n node_modules

# Find large folders
struct -z                       # Show all sizes
struct -s 100                   # Skip folders > 100MB

# Search with flat output for piping
struct search "*.py" -f | grep test

# Find __pycache__ dirs across your project
struct search "__pycache__" ~/dir -f

# Git: see what you're about to commit
struct --gsr                    # Staged files from repo root
```

---

## Features

- **Clean by default**: hides noise (venv, node_modules, .git, caches, build artifacts)
- **Smart search**: substring match for plain text, glob match for patterns with wildcards
- **Git integration**: filter to tracked / untracked / staged / changed files
- **Size awareness**: show sizes with `-z`, skip large dirs with `-s`
- **Configurable ignores**: save patterns permanently with `struct add`
- **Flexible output**: tree or flat format for search
- **Color-coded**: directories in blue, executables in green
- **Fast**: written in Rust

---

## Why Rust

Started as a learning project. Turned out to be genuinely useful, so it got polished up. The performance is a nice bonus.

## Contributing

Found a bug? Want a feature? Open an issue. PRs welcome.

Drop a star if you find it useful — it helps!

## License

MIT