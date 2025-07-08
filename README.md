# 🗓️ dailylog

A minimal journaling tool

## ✨ Features

- Opens your `$EDITOR` (defaults to `vim`) to write an entry
- **Git commit style parsing**: First line becomes title, body after blank line
- Saves entries with timestamps and markdown formatting to `YYYY-MM-DD.txt`
- View previous day's log entry with `dailylog previous`
- **Git sync support**: Sync logs across devices with automatic push/pull
- Configurable log directory via a simple TOML config
- Clean, dependency-light, and terminal-native

## 📦 Installation

Requires Rust toolchain. [Install Rust](https://www.rust-lang.org/tools/install)

### Build from source

```bash
git clone https://github.com/cetanu/dailylog.git
cd dailylog
cargo build --release
cp target/release/dailylog ~/.local/bin/
```

### Install via Cargo

```bash
cargo install dailylog
```


## ⚙️ Configuration

Create a config file at `~/.dailylog.toml`:

```toml
# Directory where your daily logs are stored
log_dir = "/path/to/your/log/folder"

# Optional: Git repository URL for syncing logs across devices
git_repo = "https://github.com/yourusername/dailylogs.git"

# Optional: Enable automatic git sync after each log entry (default: false)
git_auto_sync = true
```

Make sure the log directory exists, or `dailylog` will try to create it on first run.

## Usage

### Create a new log entry

```bash
dailylog
```

This will:
1. Open your editor (via `$EDITOR` or default to `vim`)
2. Parse your input using git commit style (title on first line, body after blank line)
3. Save formatted entry with timestamp to a file like `2025-05-31.txt`
4. Auto-sync with git if enabled

**Entry format example:**
```
Fixed authentication bug

Updated the login system to handle edge cases.
This resolves the timeout issues users reported.

- Improved error messages
- Added validation
```

**Becomes:**
```
## 14:30 - Fixed authentication bug

Updated the login system to handle edge cases.
This resolves the timeout issues users reported.

- Improved error messages
- Added validation
```

### View previous day's log entry

```bash
dailylog previous
```

### Add to previous day's log

```bash
dailylog yesterday
```

### Git sync commands

```bash
# Sync logs (pull then push)
dailylog sync

# Pull latest logs from repository
dailylog pull

# Push local logs to repository
dailylog push
```

**Setting up git sync:**
1. Create a git repository (GitHub, GitLab, etc.)
2. Add `git_repo = "your-repo-url"` to `~/.dailylog.toml`
3. Run `dailylog sync` to initialize
4. Optionally enable `git_auto_sync = true` for automatic syncing

Note: on shells like `fish`, sometimes `$EDITOR` is not set to propagate to child processes. You can fix this with `set -Ux EDITOR myEditor`
