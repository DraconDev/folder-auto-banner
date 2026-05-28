# cfm TODO

## ✅ Done

### Table / Display
- [x] Replaced manual box-drawing with comfy-table v6
- [x] Custom preset matching `│`, `─`, `├`, `┼`, `┤` style
- [x] Responsive column widths (caps at 120, min 60)
- [x] Merge icon into name column
- [x] `max_height(1)` prevents row wrapping; `truncate_ansi()` handles overflow
- [x] Switched to compact lsd/exa-style layout (no borders, aligned text)
- [x] Column order: `PERM OWNER GROUP SIZE DATE NAME` (matches lsd/exa)

### File-Type Icons (lifted from lsd)
- [x] 3-tier icon lookup: exact filename → extension → type fallback
- [x] 100+ mappings: `Cargo.toml` → 🦀, `install.sh` → 🐚, `.lock` → 🔒, etc.
- [x] Separate `src/icon.rs` module with unit tests

### Permissions & Owner/Group
- [x] `DirEntry` fields: `is_exec`, `perms`, `owner`, `group`, `symlink_target`
- [x] Unix permissions via `std::os::unix::fs::PermissionsExt::mode()` → `drwxr-xr-x`
- [x] Owner/group resolved from `/etc/passwd` and `/etc/group`
- [x] Symlink target resolution via `std::fs::read_link`

### Compact Size/Date Formats
- [x] Size: exa-style `4.3k`, `1.1k`, `983` (no "B" suffix)
- [x] Date: exa-style `27 May 23:42` (day month hour:minute)

### Git Integration
- [x] Header shows branch name + dirty state (yellow when dirty, blue when clean)
- [x] Header shows `*N` modified, `+N` staged, `?N` untracked, `↑N` ahead, `↓N` behind
- [x] `FileStatus` enum with per-file icons and colors
- [x] `file_statuses: HashMap<String, FileStatus>` populated during scan
- [x] Per-file git status lookup by relative path

### TTY Detection
- [x] Colors only emit when stdout is a real terminal
- [x] Non-tty output is plain text (no broken escape codes)

### Install / Shell
- [x] Consolidated 4 redundant install scripts into 1
- [x] `install.sh` tears down old binary before copying new one
- [x] Cleanup handles all hook-name variants
- [x] Hook fires on new tabs/shell startup (not just `cd`)
- [x] Bash `PROMPT_COMMAND` support
- [x] Hook names unified to `_cfm_hook` everywhere

### Docs / CI
- [x] `INSTALL.md` — removed hardcoded paths, added bash section
- [x] `README.md` — mentions both zsh and bash
- [x] `release.yml` — fixed step ordering, switched to `softprops/action-gh-release@v2`

---

## 🔜 In Progress

### 1. More Vibrant Colors (exa/lsd style)
**Current:** Only filenames are colored (blue dirs, green exec, dim hidden).
**Target:** Color the whole row like exa/lsd:

| Element | Color Scheme |
|---------|-------------|
| Permissions `r` | Green (readable) |
| Permissions `w` | Yellow (writable) |
| Permissions `x` | Red (executable) |
| Permissions `d` | Blue (directory) |
| Permissions `l` | Magenta (symlink) |
| Permissions `-` | Dim (no permission) |
| Owner/Group | Dim (less prominent) |
| Size | Bold if >1MB, dim if small |
| Date | Bright if recent, dim if old |
| Filenames | Already colored ✓ |

**Implementation:** Add `colorize_perms()` function that wraps each permission character in ANSI codes.

---

## 📋 Next Up

### 2. More Git Info
- [ ] Show last commit date per file (like `tig` or `git log --format`)
- [ ] Show which branch a file was last changed on
- [ ] Show conflict markers during merge (`!` icon)
- [ ] Show stash count in header (`⚑2`)
- [ ] Show worktree state (rebase, cherry-pick, etc.)

### 3. File Metadata
- [ ] Show number of hard links (like `ls -l`)
- [ ] Show file size as a mini bar (like `dust` or `duf`)
- [ ] Show file age relative to git repo creation
- [ ] Show MIME type or category (Code, Config, Document, Media, etc.)

### 4. Sorting & Filtering
- [ ] `--sort name|size|modified|type` flag
- [ ] `--filter` flag (type, size range, name pattern)
- [ ] `--max N` flag (limit items shown)
- [ ] `--hidden` flag to always show dotfiles
- [ ] `--group` flag to group by type (dirs, files, symlinks)

### 5. Table Views
- [ ] `--tree` recursive view (like `lsd --tree`)
- [ ] `--grid` compact multi-column layout (like `ls` default)
- [ ] `--long` full mode (like `ls -l` with all columns)
- [ ] `--short` minimal mode (just name + icon)

### 6. Color Themes
- [ ] Support `LS_COLORS` environment variable
- [ ] Support `NO_COLOR` environment variable
- [ ] Configurable color scheme via config file
- [ ] Dark/light theme support

### 7. Performance
- [ ] Cache directory scan results (TTL-based)
- [ ] `--depth N` for shallow recursive view
- [ ] Parallel stat calls for large directories
- [ ] Bench: <50ms for 10k files

### 8. Safety / Polish
- [ ] Deduplicate `autoload -U add-zsh-hook` in install.sh
- [ ] Test install.sh idempotency (run twice = no duplicates)
- [ ] `cargo clippy` pass
- [ ] `cargo fmt` pass
- [ ] Handle broken symlinks gracefully

---

## Testing Backlog
- [ ] Test with 1000+ items in directory
- [ ] Test with very long filenames
- [ ] Test with CJK/emoji filenames
- [ ] Test with symlinks + broken symlinks
- [ ] Test git status in repos with many changes
- [ ] Test in non-tty contexts (piped output)
- [ ] Performance benchmark
