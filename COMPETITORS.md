# Competitive Analysis: CFM vs lsd vs eza

**Date:** 2026-06-02  
**Scope:** Feature comparison of modern `ls` alternatives  
**Competitors:** [lsd](https://github.com/lsd-rs/lsd) v1.2.0, [eza](https://github.com/eza-community/eza) (active fork of exa)

---

## Quick Summary

| | CFM | lsd | eza |
|---|-----|-----|-----|
| **Purpose** | Contextual directory dashboard | Pretty ls replacement | Modern ls replacement |
| **Language** | Rust | Rust | Rust |
| **Status** | Active | Active | Active |
| **Config** | TOML | YAML (3 files) | YAML |
| **Daemon** | ✅ Background caching | ❌ | ❌ |
| **Icons** | ✅ 100+ mappings | ✅ Nerd Font | ✅ Nerd Font |
| **Git** | ✅ Per-file + repo stats | ✅ Per-file | ✅ Per-file + repos |
| **Tree** | ✅ `--tree` | ✅ `--tree` | ✅ `-T` |
| **Context** | ✅ TODOs, ports, docker, build | ❌ | ❌ |
| **Project detection** | ✅ Auto-detect type | ❌ | ❌ |

---

## Feature Matrix

### Display Options

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| Grid view (default) | ✅ | ✅ | ✅ |
| Long format (`-l`) | ❌ | ✅ | ✅ |
| One per line (`-1`) | ✅ | ✅ | ✅ |
| Tree view (`--tree`) | ✅ | ✅ | ✅ |
| Recursive (`-R`) | ❌ | ✅ | ✅ |
| Classification (`-F`) | ✅ `--classify` | ✅ `-F` | ✅ `-F` |
| Hyperlinks | ✅ `--hyperlink` | ✅ | ✅ |
| Color scale | ❌ | ❌ | ✅ `--color-scale` |
| Width control (`-w`) | ❌ | ❌ | ✅ |
| Icons | ✅ 100+ | ✅ | ✅ |
| Custom icon themes | ❌ | ✅ `icons.yaml` | ✅ `theme.yml` |
| Custom color themes | ❌ | ✅ `colors.yaml` | ✅ `theme.yml` |

### Sorting

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| Sort by name | ✅ | ✅ | ✅ |
| Sort by time (`-t`) | ✅ | ✅ | ✅ |
| Sort by size (`-S`) | ✅ | ✅ | ✅ |
| Sort by extension (`-X`) | ✅ | ✅ | ✅ |
| Sort by version (`-v`) | ✅ `--versionsort` | ✅ `-v` | ✅ |
| Sort by git status (`-G`) | ✅ | ❌ | ❌ |
| Sort by inode | ❌ | ❌ | ✅ |
| Reverse (`-r`) | ✅ `-R` | ✅ `-r` | ✅ `-r` |
| No sort (`-U`) | ✅ | ✅ `-U` | ✅ `-s none` |
| Group dirs first/last | ✅ `--group-dirs` | ✅ `--group-dirs` | ✅ `--group-directories-first` |

### Filtering

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| Show hidden (`-a`) | ✅ `-a` | ✅ `-a` | ✅ `-a` |
| Filter pattern (`-f`) | ✅ `--filter` | ❌ | ❌ |
| Ignore glob | ✅ `--ignore-glob` | ✅ `-I` | ✅ `-I` |
| Max items (`-m`) | ✅ `--max` | ❌ | ❌ |
| Only dirs (`-D`) | ❌ | ❌ | ✅ |
| Only files (`-f`) | ❌ | ❌ | ✅ |
| Git ignore | ❌ | ❌ | ✅ `--git-ignore` |
| Dereference symlinks (`-L`) | ❌ | ✅ `-L` | ✅ `-X` |
| No symlinks | ✅ `--no-symlink` | ❌ | ✅ `--no-symlinks` |
| Depth limit (`-L`) | ❌ | ✅ `--depth` | ✅ `-L` |

### Metadata

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| Permissions | ✅ | ✅ | ✅ |
| Owner/group | ✅ | ✅ | ✅ |
| Size | ✅ | ✅ | ✅ |
| Date | ✅ | ✅ | ✅ |
| Inode (`-i`) | ❌ | ✅ | ✅ |
| Links (`-H`) | ❌ | ❌ | ✅ |
| Extended attrs (`-@`) | ❌ | ❌ | ✅ |
| Security context (`-Z`) | ❌ | ✅ | ✅ |
| Mount details (`-M`) | ❌ | ❌ | ✅ |
| Octal permissions | ❌ | ✅ | ✅ |
| Binary sizes (`-b`) | ❌ | ❌ | ✅ |
| Total size | ✅ `--total-size` | ✅ | ✅ |
| Smart group display | ❌ | ❌ | ✅ |

### Git Integration

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| Per-file git status | ✅ | ✅ | ✅ |
| Branch/ahead/behind | ✅ | ✅ | ✅ |
| Stash count | ✅ | ❌ | ❌ |
| Merge state | ✅ | ❌ | ❌ |
| Commits today | ✅ | ❌ | ❌ |
| Branch count | ✅ | ❌ | ❌ |
| Last commit time | ✅ | ❌ | ❌ |
| Per-directory git status | ✅ (via daemon cache) | ❌ | ✅ `--git-repos` |
| Git theme customization | ❌ | ✅ | ❌ |

### Output Formats

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| JSON output | ✅ `--json` | ❌ | ❌ |
| Raw paths | ✅ `--raw` | ❌ | ❌ |
| Stdin support | ❌ | ❌ | ✅ `--stdin` |

### Configuration

| Feature | CFM | lsd | eza |
|---------|-----|-----|-----|
| Config file | ✅ TOML | ✅ YAML | ✅ YAML |
| Editable via command | ✅ `f config` | ❌ | ❌ |
| Config + CLI override | ✅ | ✅ | ✅ |
| Color config | ❌ | ✅ `colors.yaml` | ✅ `theme.yml` |
| Icon config | ❌ | ✅ `icons.yaml` | ✅ `theme.yml` |
| Generate default config | ❌ | ✅ `--generate-config` | ❌ |

### Unique to CFM (not in lsd/eza)

| Feature | Description |
|---------|-------------|
| **Contextual banner** | Git summary, TODO count, ports, docker, build status, code metrics |
| **Daemon caching** | Background process with Unix socket IPC, instant warm reads |
| **Proactive scanning** | Pre-computes banners for home directory subdirs |
| **Project type detection** | Auto-detects Rust, Node, Python, Go, etc. |
| **Build status** | Shows if project builds successfully |
| **Port detection** | Shows active listening ports |
| **Docker detection** | Shows running containers |
| **Language breakdown** | Shows top languages by LOC percentage |
| **Code metrics** | Total LOC, file counts by type |
| **TODO/FIXME count** | Scans source for task markers |
| **Config command** | `f config` opens config in editor |
| **Shell hooks** | Auto-banner on `cd` (zsh/bash) |
| **Long format by default** | No `-l` needed — detailed view is the default |
| **Rich git integration** | Branch, ahead/behind, stash, merge state, commits today, branch count, last commit time |
| **Per-file git status** | Modified, added, deleted, untracked indicators per file |

### Unique to lsd (not in CFM/eza)

| Feature | Description |
|---------|-------------|
| **Classic mode** | `--classic` for ls-compatible output |
| **3-file config** | Separate config, colors, icons YAML files |
| **Icon themes** | Full icon customization via YAML |
| **Color themes** | Full color customization via YAML |
| **Symlink arrow** | Customizable symlink target arrow |
| **Truncate owner** | `--truncate-owner` for long owner names |

### Unique to eza (not in CFM/lsd)

| Feature | Description |
|---------|-------------|
| **Color scale** | `--color-scale` gradient by age/size |
| **Git repos** | `--git-repos` per-directory git status |
| **Mount details** | `-M` show mount point info |
| **SELinux context** | `-Z` security context |
| **Extended attrs** | `-@` show xattrs |
| **Stdin support** | `--stdin` read filenames from pipe |
| **Smart group** | Only show group if different from owner |
| **Git ignore** | `--git-ignore` respect .gitignore |
| **Width control** | `-w` set terminal width |
| **Time styles** | Multiple timestamp formats |
| **Only dirs/files** | `-D` / `-f` filters |
| **Binary sizes** | `-b` binary prefix sizes |

---

## Priority Feature Gaps for CFM

### High Priority (would match competitors)

1. **`-R` / `--recursive`** — Flat recursive listing (vs tree's hierarchical view)
2. **`-D` / `--only-dirs`** — List only directories
3. **`-f` / `--only-files`** — List only files
4. **`--git-ignore`** — Respect .gitignore in listings
5. **`-L` / `--level`** — Limit tree recursion depth

Note: `-l` / `--long` is NOT needed — CFM's default output already shows detailed metadata (permissions, owner, group, size, date, name). lsd/eza need `-l` because their default is a grid view.

### Medium Priority (nice to have)

6. **`-i` / `--inode`** — Show inode numbers
7. **`-L` / `--level`** — Limit tree recursion depth
8. **`-w` / `--width`** — Control terminal width
9. **`--color-scale`** — Gradient colors by age/size
10. **`--git-repos`** — Per-directory git status in tree

### Low Priority (edge cases)

11. **`-@` / `--extended`** — Show extended attributes
12. **`-Z` / `--context`** — SELinux security context
13. **`-M` / `--mounts`** — Mount point details
14. **`--stdin`** — Read filenames from stdin
15. **`--generate-config`** — Generate default config file

---

## CFM's Competitive Advantage

CFM is **not a drop-in `ls` replacement** — it's a **contextual directory dashboard**. The key differentiators:

1. **Instant context**: See git status, TODOs, ports, docker, build status, and code metrics at a glance
2. **Daemon caching**: First access computes, subsequent accesses are instant (~7ms)
3. **Project awareness**: Auto-detects project type and shows relevant info
4. **Smart defaults**: Config file for persistent preferences, CLI flags for ad-hoc overrides

**Best for**: Developers who want to understand a directory's context without running multiple commands.

**Not ideal for**: Scripts or automation (use `ls` or `find` for that).

---

## Installation Comparison

| | CFM | lsd | eza |
|---|-----|-----|-----|
| Package managers | Build from source | ✅ Most distros | ✅ Most distros |
| Cargo | `cargo install cfm` | `cargo install lsd` | `cargo install eza` |
| Binary releases | ✅ GitHub | ✅ GitHub | ✅ GitHub |
| NixOS | Manual | ✅ `nixpkgs.lsd` | ✅ `nixpkgs.eza` |

---

## Conclusion

| Use Case | Recommendation |
|----------|----------------|
| **Daily directory navigation with context** | CFM |
| **Drop-in `ls` replacement** | lsd or eza |
| **Scripting/automation** | Standard `ls` or `find` |
| **Maximum customization** | lsd (3-file config) |
| **Best git integration** | eza (`--git-repos`) |
