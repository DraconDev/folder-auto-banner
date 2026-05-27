# cfm — Installation Guide

## Quick Start

### Method 1: Auto Install Script (Recommended)

```bash
cd cfm
./install.sh
exec zsh   # or: source ~/.bashrc
```

### Method 2: Build + Install

```bash
cd cfm
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
# cfm auto-banner hook
autoload -U add-zsh-hook
add-zsh-hook chpwd _cfm_hook
_cfm_hook() {
    eval "$(command fm env "$PWD")"
    command fm banner "$PWD"
}
```

### Bash (`~/.bashrc`)
```bash
# cfm auto-banner hook
_cfm_hook() {
    eval "$(command fm env "$PWD")"
    command fm banner "$PWD"
}
PROMPT_COMMAND="_cfm_hook${PROMPT_COMMAND:+;$PROMPT_COMMAND}"
```

Then reload: `exec zsh` or `source ~/.bashrc`

---

## Usage

```bash
fm                          # Show banner for current directory
fm banner /some/path        # Show banner for specific directory
fm banner --json            # JSON output
fm stats                    # Directory statistics
fm yank file.txt            # Copy to clipboard
fm paste                    # Paste from clipboard
fm mv src/file target/      # Move file
fm diff src cmd             # Compare directories
fm pin myproject            # Pin current directory
fm jump myproject           # Jump to pinned directory
fm install-hook             # Print shell hook for manual setup
```

## Testing

```bash
cargo run                   # Test the banner
cargo test                  # Run test suite
```
