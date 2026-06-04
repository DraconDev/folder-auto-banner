# fab — Installation Guide

## Quick Start

### Method 1: Auto Install Script (Recommended)

```bash
cd fab
./install.sh
exec zsh   # or: source ~/.bashrc
```

### Method 2: Build + Install

```bash
cd fab
cargo build --release
./install.sh
exec zsh   # or: source ~/.bashrc
```

### Method 3: Cargo Install

```bash
cargo install --path .
```

Then add the hook to your shell config manually — see the section below.

---

## Shell Hook Setup

Add the appropriate section to your shell config:

### Zsh (`~/.zshrc`)
```bash
# fab auto-banner hook
autoload -U add-zsh-hook
add-zsh-hook chpwd _fab_hook
_fab_hook() {
    eval "$(command f env "$PWD")"
    command f banner "$PWD"
}
_fab_hook  # fire on new shell/tab startup
```

### Bash (`~/.bashrc`)
```bash
# fab auto-banner hook
_fab_hook() {
    eval "$(command f env "$PWD")"
    command f banner "$PWD"
}
PROMPT_COMMAND="_fab_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

Then reload: `exec zsh` or `source ~/.bashrc`

---

## Usage

```bash
f                          # Show banner for current directory
f banner /some/path        # Show banner for specific directory
f banner --json            # JSON output
f stats                    # Directory statistics
f yank file.txt            # Copy to clipboard
f paste                    # Paste from clipboard
f mv src/file target/      # Move file
f diff src cmd             # Compare directories
f pin myproject            # Pin current directory
f jump myproject           # Jump to pinned directory
f install-hook             # Print shell hook for manual setup
```

## Testing

```bash
cargo run                   # Test the banner
cargo test                  # Run test suite
```
audit_test_1780438846
