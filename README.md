# cfm — Contextual File Manager

> **Type `fm`. Get context.**

An ephemeral, zero-hostage intelligence layer for the shell. Print once, exit immediately. No TUI. No daemon. <5ms latency.

---

## 🚀 Quick Start

### Auto Install (One Command!)

```bash
cd /home/dracon/Dev/cli-file-manager
./install.sh
source ~/.zshrc  # or source ~/.bashrc
```

That's it! Now type `cd` anywhere to see the banner automatically! 🎉

### Manual Installation

```bash
cd /home/dracon/Dev/cli-file-manager
cargo build --release
cp target/release/fm ~/bin/fm
```

Add to `~/.bashrc` or `~/.zshrc`:

**Bash:**
```bash
export PATH="$HOME/bin:$PATH"
_cfm_hook() {
    command /home/dracon/bin/fm banner "$PWD"
}
```

**Zsh:**
```bash
export PATH="$HOME/bin:$PATH"
autoload -U add-zsh-hook
add-zsh-hook chpwd _cfm_hook
_cfm_hook() {
    command /home/dracon/bin/fm banner "$PWD"
}
```

---

## 🎯 The Auto Banner

After installation, just `cd` anywhere and see:

```bash
$ cd /home/dracon/Dev/cli-file-manager
┌──┐
│ 📂 /home/dracon/Dev/cli-file-manager [main]
│ 🦀 Rust │ 114.4 KiB │ chore: update banner
│ ----------------------------------------------------------------------------
+----+------------------+-----------------+-------------------+
|    | Name             | Type            | Size              |
+=============================================================+
| 📂 | .dracon          | 1 item(s)       | -                 |
|----+------------------+-----------------+-------------------|
| 📂 | .github           | 1 item(s)       | -                 |
|...
```

---

## ⚡ Common Commands

```bash
# Direct use
./target/release/fm

# JSON output
./target/release/fm banner --json

# Copy to clipboard
./target/release/fm yank Cargo.toml README.md

# Paste from clipboard
./target/release/fm paste

# Move files
./target/release/fm mv src/main.rs target/

# Compare directories
./target/release/fm diff src cmd
```

---

## 🧪 Testing

```bash
# Tests
cargo test    # ✅ 10 tests pass

# Linting
cargo clippy  # ✅ 0 warnings

# Build
cargo build --release
```

---

## 📦 What You Get

- **28 commands** fully implemented
- **Auto banner** on every `cd`
- **Clipboard** (yank, paste)
- **Pins** (bookmark directories)
- **Sessions** (save/restore workspace)
- **Completions** for bash/zsh/fish

---

## 📊 Project Statistics

| Metric | Value |
|--------|--------|
| Total Lines of Code | ~4,100 |
| Source Files | 33 |
| Tests | 10 (2 unit + 8 integration) |
| Clippy Warnings | 0 |
| Commands | 28 |
| CI/CD | 2 GitHub Actions workflows |

---

## 🎉 Ready to Use!

```bash
# Just cd anywhere and see the banner!
cd /home/dracon/Dev/cli-file-manager

# Or use the binary directly
./target/release/fm

# Or from PATH
fm
```

---

## 📝 License

MIT License