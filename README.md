# cfm — Contextual File Manager

A directory listing with instant context.

## What It Does

When you run `f`, you see:
- File listing (like `ls`/`exa`/`lsd`)
- Git status
- Build status
- TODO count
- Project type
- Ports in use
- Docker status

**All instantly, no extra commands needed.**

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
```

## Flags

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

## Environment Variables

| Variable | Description |
|----------|-------------|
| `CFM_NO_TODOS` | Set to `1` to disable TODO scanning |
| `CFM_NO_PORTS` | Set to `1` to disable port detection |
| `CFM_NO_DOCKER` | Set to `1` to disable Docker detection |
| `CFM_NO_METRICS` | Set to `1` to disable code metrics |
| `NO_COLOR` | Disable colors (per spec) |

## Testing

```bash
cargo test    # 77 tests pass
cargo clippy  # 0 warnings
```

## License

MIT
