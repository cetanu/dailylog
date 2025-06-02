# 🗓️ dailylog

A minimal journaling tool

## ✨ Features

- Opens your `$EDITOR` (defaults to `vim`) to write an entry
- Saves or appends your input to `YYYY-MM-DD.txt`
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
log_dir = "/path/to/your/log/folder"
```

Make sure this directory exists, or `dailylog` will try to create it on first run.

## Usage

```bash
dailylog
```

This will:
1. Open your editor (via `$EDITOR` or default to `vim`)
2. Save whatever you type into a file named like `2025-05-31.txt` inside your configured directory
3. Append if the file already exists

Note: on shells like `fish`, sometimes `$EDITOR` is not set to propagate to child processes. You can fix this with `set -Ux EDITOR myEditor`
