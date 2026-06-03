# cfm — Contextual File Manager

A directory listing with instant context.

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

**All instantly, no extra commands needed.**

## vs lsd / eza

CFM is **not a drop-in `ls` replacement** — it's a **contextual directory dashboard**. While lsd and eza focus on making `ls` pretty, CFM adds **project context** (git, TODOs, ports, docker, build status, code metrics) and **daemon caching** for instant repeated access.

| Feature | CFM | lsd | eza |
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
| Recursive (`-R`) | ❌ | ✅ | ✅ |

See [COMPETITORS.md](COMPETITORS.md) for the full comparison.

## Quick Start

```bash
./install.sh
exec zsh   # or: source ~/.bashrc
```

## Usage

```bash
f                    # Directory listing + context
f <dir>              # Listing for specific dir
f -S                 # Sort by size
f -t                 # Sort by time
f -X                 # Sort by extension
f -G                 # Sort by git status
f --versionsort      # Natural sort (file1, file2, file10)
f -a                 # Show dotfiles
f --tree             # Tree view
f --json             # JSON output
f --filter rs        # Filter by pattern
f config             # Open config file
f daemon stop        # Stop daemon
f daemon status      # Check daemon status
```

## CLI Flags (Actions)

### Sorting
| Flag | Description |
|------|-------------|
| `--sort name\|size\|date\|type\|git\|extension\|version` | Sort by field |
| `-t`, `--timesort` | Sort by time modified |
| `-S`, `--sizesort` | Sort by size |
| `-X`, `--extensionsort` | Sort by extension |
| `-G`, `--gitsort` | Sort by git status |
| `--versionsort` | Natural sort (version numbers) |
| `--no-sort` | No sort, directory order |
| `--reverse` | Reverse sort |
| `--group-dirs first\|last` | Group directories |

### Display
| Flag | Description |
|------|-------------|
| `-a`, `--hidden` | Show dotfiles |
| `--tree [depth]` | Tree view (0 = unlimited) |
| `--group` | Group by type (dirs, files, symlinks) |
| `--filter <pattern>` | Filter by name |
| `--max <N>` | Limit items |
| `--compact` | Less info |
| `--verbose` | More info |

### Output
| Flag | Description |
|------|-------------|
| `--json` | JSON output |
| `--raw` | Plain text output |

## Config File

Location: `~/.config/cfm/config.toml`

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

### Sorting
```toml
[sort]
default = "name"
reverse = false
group_dirs = "first"
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
| `CFM_NO_TODOS` | Set to `1` to disable TODO scanning |
| `CFM_NO_PORTS` | Set to `1` to disable port detection |
| `CFM_NO_DOCKER` | Set to `1` to disable Docker detection |
| `CFM_NO_METRICS` | Set to `1` to disable code metrics |
| `NO_COLOR` | Disable colors (per spec) |
| `EDITOR` | Editor for `f config` (default: vi) |

## Testing

```bash
cargo test    # 77 tests pass
cargo clippy  # 0 warnings
```

## License

MIT
# test
# test 1780433405
# test 1780433579
