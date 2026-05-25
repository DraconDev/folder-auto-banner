# cfm — Installation Guide

## Quick Start

### Method 1: Auto Install Script (Recommended)

```bash
cd /home/dracon/Dev/cli-file-manager
./install.sh
source ~/.zshrc  # or source ~/.bashrc
```

### Method 2: Manual Installation

```bash
cd /home/dracon/Dev/cli-file-manager
cargo build --release
cp target/release/fm ~/bin/fm
```

Then add to your shell config:

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

Reload your shell:
```bash
source ~/.bashrc  # or source ~/.zshrc
```

## Usage

### Direct Use
```bash
cd /home/dracon/Dev/cli-file-manager
./target/release/fm
```

### After Installation (Auto Banner)
```bash
cd /home/dracon/Dev/cli-file-manager
# Auto banner appears!
```

### Common Commands
```bash
fm banner --json     # JSON output
fm yank file.txt     # Copy to clipboard
fm paste             # Paste from clipboard
fm mv src/main.rs target/  # Move file
fm diff src cmd      # Compare directories
```

## Testing

```bash
# Test the banner
./target/release/fm

# Test auto banner
source ~/.bashrc
cd /home/dracon/Dev/cli-file-manager
```
