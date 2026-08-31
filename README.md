# f — Folder Auto Banner

<p align="center">
  <img src="https://github.com/DraconDev/folder-auto-banner-fab/raw/main/public/fab_thumb.png" alt="f — Folder Auto Banner" width="800">
</p>

A contextual directory dashboard that combines a file listing with project signals such as git status, TODOs, code metrics, build status, ports, Docker state, and cached directory sizes.

Repeated views are served from the background daemon. Normal banner scans inspect at most 500 entries, so the shell hook remains responsive even in very large directories. A truncated view is marked `500+` and may show temporary `4.0k` placeholders while directory sizes refresh in the background; subsequent warm views use populated cached sizes.

## What It Does

When you run `f`, you see:
- File listing (like `ls`/`exa`/`lsd`)
- Git status, last commit, commits today, branches
- Build status with duration
- TODO count
- Languages breakdown
- Ports in use
- Docker status
- Cached test results

**Fast context without extra commands.**

## vs lsd / eza

fab is **not a drop-in `ls` replacement** — it's a **contextual directory dashboard**. While lsd and eza focus on making `ls` pretty, f adds **project context** (git, TODOs, ports, docker, build status, code metrics) and **daemon caching** for instant repeated access.

| Feature | f | lsd | eza |
|---------|-----|-----|-----|
| Pretty listing | ✅ | ✅ | ✅ |
| Icons | ✅ | ✅ | ✅ |
| Tree view | ✅ | ✅ | ✅ |
| **Git status** | ✅ (rich) | ✅ | ✅ |
| **Context banner** | ✅ | ❌ | ❌ |
| **Daemon caching** | ✅ | ❌ | ❌ |
| **TODO count** | ✅ | ❌ | ❌ |
| **Port detection** | ✅ | ❌ | ❌ |
| **Build status** | ✅ | ❌ | ❌ |
| **Language breakdown** | ✅ | ❌ | ❌ |
| Long format (`-l`) | ✅ (default) | ✅ | ✅ |
| Recursive (`-R`) | ✅ | ✅ | ✅ |

See [COMPETITORS.md](COMPETITORS.md) for the full comparison.

## Quick Start

```bash
# Build + install everything (binary, shell function, daemon, auto-banner hook)
./install.sh
exec zsh   # or: source ~/.bashrc
```

This sets up:
- The `f` binary in `~/.local/bin/`
- The shell function (enables `f N` → `cd` navigation)
- The auto-banner hook (shows a banner on every `cd`)
- The background daemon

If you only need the shell function (e.g., after a manual build):

```bash
f install
exec zsh   # or: source ~/.bashrc
```

## Usage

```bash
f                    # Directory listing + context (cwd)
f 4                  # Navigate to item 4 (cd into it or open it)
f -b ./src           # Banner for specific path
f -b tree            # Tree banner (alias + path mode)
f tree               # Tree banner (alias, cwd)
f hidden verbose     # Multiple aliases compose
f -S                 # Sort by size
f -t                 # Sort by time
f -X                 # Sort by extension
f -G                 # Sort by git status
f banner --versionsort  # Natural sort (file1, file2, file10)
f -a                 # Show dotfiles
f --tree             # Tree view
f --json             # JSON output
f -R, --recursive    # Recurse into subdirectories
f --filter rs        # Filter by pattern
f install            # Install shell function for f N → cd
f config             # Open config file
f daemon restart     # Restart daemon
f daemon status      # Check daemon status
f daemon stop        # Stop daemon
```

### Routing rules (0.7+)

The CLI accepts exactly three kinds of input:

| Input | Examples | Behavior |
|-------|----------|----------|
| **Numbers** | `f 1`, `f 4` | Navigate to item N |
| **Aliases** | `f tree`, `f hidden verbose` | Expand and run |
| **Flags** | `f -t`, `f --json` | Clap direct |
| **`-b` switch** | `f -b ./src`, `f -b tree ./src` | Banner mode (allows paths) |
| **Subcommand** | `f banner ./src`, `f install` | Pass through to clap |

Anything else (`f t`, `f foo`, `f ./src`, `f Downloads`, `f /tmp`) exits 0 with
no output. Use `f -b <path>` for path-specific banners.

## Shell Function (`f install`)

The `f install` subcommand writes shell wrappers that enable `f N` → `cd` navigation:

- Writes `~/.local/bin/fab-shell.zsh` and `~/.local/bin/fab-shell.bash`
- Adds source lines to `~/.zshrc` and/or `~/.bashrc`
- Is idempotent — safe to run multiple times

This is called automatically by `./install.sh`. You can also run it standalone:

```bash
f install              # Install shell wrappers
f install --debug      # Install with debug output
```

Start a new shell, or activate the wrappers immediately:

```bash
source ~/.local/bin/fab-shell.zsh   # for zsh
source ~/.local/bin/fab-shell.bash  # for bash
```

The shell wrapper is the source of truth for `f N` navigation. It calls the installed `f` binary; `f install` writes the wrappers to `~/.local/bin/fab-shell.zsh` (zsh) and `~/.local/bin/fab-shell.bash` (bash), generated from the compiled-in `src/shell_wrapper.rs` constants.

## Numbered Navigation

When `numbered = true` in config (or enabled by default), each item gets a number:

```
[ 1] 📁 .github
[ 2] 📁 src
[ 3] 📄 README.md
```

Navigate with `f N` (a number is the **only** non-alias bare word
that produces a result):

```bash
f 2          # cd into item 2 (if directory)
f 3          # open item 3 in editor (if file)
f 3 cat      # open item 3 with `cat`
f 3 krita    # open item 3 with krita
f 3 -e       # force open in editor
f 3 -x       # force run the file directly
f -b 4       # navigate to item 4 in banner mode
```

> **Note:** `f N` requires the shell wrappers installed by `./install.sh` or `f install`. To activate them in your current terminal without restarting your shell:
>
> ```bash
> source ~/.local/bin/fab-shell.zsh   # for zsh
> source ~/.local/bin/fab-shell.bash  # for bash
> ```

## CLI Flags (Actions)

### Built-in Aliases (0.7+)

Each alias is a single word that expands to one or more clap flags.
Aliases are an alternative to typing flags explicitly — they make
common invocations shorter and more memorable.

**Multiple aliases compose** by concatenating their flag lists:

```bash
f hidden verbose      # → -a -v  (show hidden + verbose output)
f new recurse         # → -t -R  (sort by time + recurse)
f tree hidden         # → -R -D -a  (tree view + show hidden)
```

**Aliases compose with explicit flags** (the flags are added after
the alias expansion):

```bash
f tree -L 2           # → -R -D -L 2  (tree with depth limit)
```

**Aliases compose with paths in banner mode** (`-b`):

```bash
f -b top ./src        # → -S -r -m 20 for ./src  (top 20 in src)
```

| Alias | Expands to | What it does |
|-------|-----------|--------------|
| **Display** | | |
| `tree` | `-R -D` | Recursive, only dirs (like `tree` command) |
| `flat` | `-o` | One file per line |
| `compact` | `-c` | Compact output |
| `verbose` | `-v` | Verbose output |
| `hidden` | `-a` | Show hidden files (dotfiles) |
| `dirs` | `-D` | Only directories |
| **Sort** | | |
| `new` | `-t` | Sort by time, newest first |
| `old` | `-t -r` | Sort by time, oldest first |
| `big` | `-S` | Sort by size, largest first |
| `small` | `-S -r` | Sort by size, smallest first |
| `ext` | `-X` | Sort by extension |
| `git` | `-G` | Sort by git status |
| `nosort` | `-U` | No sort (directory order) |
| **Limits** | | |
| `top` | `-S -r -m 20` | Top 20 largest files |
| `newest` | `-t -r -m 20` | 20 newest files |
| **Recursion** | | |
| `recurse` | `-R` | Recurse into subdirectories |
| **Actions** | | |
| `edit` | `-e` | Force open in editor |
| `run` | `-x` | Force run file |

**Unknown bare words** (`f t`, `f foo`, `f Downloads`) exit 0 with
no output. The CLI only accepts numbers, aliases, and flags. To
see a banner for a specific path, use `f -b <path>` (banner mode)
or `f banner <path>` (subcommand). To see a banner for cwd, just
type `f`.

For your personal common combinations, add a shell alias:

```bash
# ~/.zshrc or ~/.bashrc
alias ftrc='f -t -r -c'
alias ftree='f -R -D'
```

### `-b` Banner Mode (0.7.3+)

The `-b` flag switches to banner mode, where paths are allowed.
This is the way to see a banner for a specific path without
typing the subcommand name.

```bash
f -b                   # default banner for cwd (same as `f`)
f -b ./src             # banner for ./src
f -b /tmp              # banner for /tmp
f -b ~/Downloads       # banner for ~/Downloads
f -b tree              # tree banner (alias still expands)
f -b tree ./src        # tree banner for ./src
f -b -t                # banner with -t flag
f -b 5                 # navigate to item 5
```

Unknown words in banner mode are dropped (e.g. `f -b foo` runs the
default banner, ignoring `foo`).

All flags below are banner flags. They work as `f banner --flag …` (or as
`f -b …` for flags the top-level CLI also exposes). Rows marked ‡ exist only
on the banner subcommand — `f --sort name` / `f --versionsort` are clap
errors (exit 2); use `f banner --sort name` etc.

### Sorting (banner flags)
| Flag | Description |
|------|-------------|
| `--sort name\|size\|date\|type\|git\|extension\|version` ‡ | Sort by field |
| `-t`, `--timesort` | Sort by time modified |
| `-S`, `--sizesort` | Sort by size |
| `-X`, `--extensionsort` | Sort by extension |
| `-G`, `--gitsort` | Sort by git status |
| `--versionsort` ‡ | Natural sort (version numbers) |
| `--no-sort` | No sort, directory order |
| `--reverse` | Reverse sort |
| `--group-dirs first\|last` ‡ | Group directories |

### Display (banner flags)
| Flag | Description |
|------|-------------|
| `-a`, `--hidden` | Show dotfiles |
| `--tree [depth]` | Tree view (0 = unlimited) |
| `--group` ‡ | Group by type (dirs, files, symlinks) |
| `--filter <pattern>` | Filter by name |
| `--max <N>` | Limit items |
| `--compact` | Less info |
| `-R`, `--recursive` | Recurse into subdirectories |
| `--verbose` | More info |

### Output (banner flags)
| Flag | Description |
|------|-------------|
| `--json` | JSON output |
| `--raw` | Plain text output |

‡ Banner-subcommand-only: `f banner --flag …` (not top-level `f --flag …`).

## Config File

Location: `~/.config/fab/config.toml`

Open with: `f config`

### Display Settings
```toml
[display]
permission = "rwx"        # rwx, octal, disable
size = "default"          # default, short, bytes
date = "date"             # date, relative
classify = true           # append */=>@|
no_symlink = false
total_size = true
```

### Column Selection
```toml
[columns]
show = ["permission", "owner", "group", "size", "date", "name"]
hide = ["inode", "links"]
```

### Feature Toggles
```toml
[features]
git_status = true
build_status = true
todo_count = true
languages = true
ports = true
docker = true
```

### Navigation
```toml
[features]
numbered = true          # Show item numbers for f N navigation
open_command = "micro"   # Default editor for f N (overridden by $EDITOR)
```

### Sorting & Layout
```toml
[sort]
default = "name"
reverse = false
group_dirs = "last"      # "last" (default, folders at bottom near prompt), "first", or "none"
smart_truncation = true  # Fold repetitive archival/log files and cap huge folders
zebra_rows = false       # Alternating row background tint
```

### Recency Gradient
```toml
[display]
# Color rows based on recency: same hues, dimmer for older files
# Bright new, dim old — at-a-glance scan of what changed recently
color_scale = "all"          # "all", "age", "size", or "" to disable
color_scale_mode = "gradient" # "gradient" (default) or "fixed"
```

Tiers: <1h = bold, <1d = normal, <1w = faded gray, <1m = dim, >1m = very dim.
Recent files pop out; old files recede into the background.

## Environment Variables

| Variable | Description |
|----------|-------------|
| `FAB_NO_TODOS` | Set to `1` to disable TODO scanning |
| `FAB_NO_PORTS` | Set to `1` to disable port detection |
| `FAB_NO_DOCKER` | Set to `1` to disable Docker detection |
| `FAB_NO_METRICS` | Set to `1` to disable code metrics |
| `NO_COLOR` | Disable colors (per spec) |
| `EDITOR` | Editor for `f config` (default: vi) |

## Testing

```bash
cargo test    # full test suite
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT
