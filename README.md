# 🗓️ dailylog

A minimal journaling tool that launches your `$EDITOR`, captures your thoughts, and saves them into a daily file automatically named by date.

## ✨ Features

- Opens your `$EDITOR` (defaults to `vim`) to write an entry
- Saves or appends your input to `YYYY-MM-DD.txt`
- Configurable log directory via a simple TOML config
- Clean, dependency-light, and terminal-native

## 📦 Installation

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

(Requires Rust toolchain. [Install Rust](https://www.rust-lang.org/tools/install))

### Download precompiled binary

Visit the [Releases](https://github.com/cetanu/dailylog/releases) page and download the appropriate binary for your platform. Then:

```bash
chmod +x dailylog
mv dailylog ~/.local/bin/
```

Make sure `~/.local/bin` is in your `$PATH`.

## ⚙️ Configuration

Create a config file at `~/.dailylog.toml`:

```toml
log_dir = "/path/to/your/log/folder"
```

Make sure this directory exists, or `dailylog` will try to create it on first run.

## Usage

Just run:

```bash
dailylog
```

This will:
1. Open your editor (via `$EDITOR` or default to `vim`)
2. Save whatever you type into a file named like `2025-05-31.txt` inside your configured directory
3. Append if the file already exists
