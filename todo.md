# cfm TODO

## ✅ Done

### Table / Display
- [x] Replace manual `println!` box-drawing with `comfy-table` v6
- [x] Custom preset matching original `│`, `─`, `├`, `┼`, `┤` style
- [x] Responsive column widths (caps at 100, min 60, name column gets remainder)
- [x] Merge icon into name column (`📁 .dracon` instead of wasted icon column)
- [x] `max_height(1)` prevents row wrapping; `truncate_ansi()` handles overflow with `…`

### File-Type Icons (lifted from lsd)
- [x] 3-tier icon lookup: exact filename → extension → type fallback
- [x] 100+ mappings: `Cargo.toml` → 🦀, `install.sh` → 🐚, `README.md` → 📖, `.lock` → 🔒, etc.
- [x] Separate `src/icon.rs` module with unit tests

### Permissions & Symlinks
- [x] `DirEntry` fields: `is_exec`, `perms`, `symlink_target`
- [x] Unix permissions via `std::os::unix::fs::PermissionsExt::mode()` → `rwxrwxrwx`
- [x] Symlink target resolution via `std::fs::read_link`
- [x] `truncate_ansi()` handles ANSI escape sequences without breaking width calculation

### Install / Shell
- [x] Consolidated 4 redundant install scripts into 1
- [x] `install.sh` tears down old binary before copying new one
- [x] Cleanup handles all hook-name variants: `_cfm_hook`, `_cfm_on_directory_change`, `_cfm_on_startup`
- [x] Cleanup removes old `~/bin` PATH, orphaned function fragments, fused braces
- [x] Hook fires on new tabs/shell startup (not just `cd`)
- [x] Bash `PROMPT_COMMAND` support
- [x] Typo `add-zash-hook` → `add-zsh-hook` fixed in `install_hook.rs`
- [x] Hook names unified to `_cfm_hook` everywhere

### Docs / CI
- [x] `INSTALL.md` — removed hardcoded `/home/dracon` paths, added bash section
- [x] `README.md` — mentions both zsh and bash
- [x] `.ralph/cfm-build.md` — fixed claims (no builds, no completions installed)
- [x] `release.yml` — fixed step ordering, switched to `softprops/action-gh-release@v2`

### Code Cleanup
- [x] Removed orphaned `banner_new.rs`
- [x] Removed empty unused `src/shell/`, `src/banner/` dirs
- [x] Added `unicode-width` for proper CJK/emoji truncation
- [x] All 13 tests pass (5 unit + 8 integration)

---

## 🔜 Next Up

### 1. Color in Table (requires comfy-table Cell styling)
- [ ] Directories → blue, executables → green, symlinks → magenta, hidden → dim
- [ ] Use `Cell::set_style()` instead of raw ANSI escape codes

### 2. Permission Column
- [ ] Add optional `PERM` column to table (like `lsd -l`)
- [ ] Configurable via `--compact` or config flag
- [ ] Show owner/group optionally

### 3. Header Improvements
- [ ] Last modified date of directory itself
- [ ] Free/used filesystem space
- [ ] Parent directory size

### 4. Sorting & Filtering
- [ ] `--sort name|size|modified` flag
- [ ] `--filter` flag (type, size range, name pattern)
- [ ] `--max N` flag (limit items shown)
- [ ] `--hidden` flag to always show dotfiles

### 5. Table Views
- [ ] `--tree` recursive view (like `lsd --tree`)
- [ ] `--grid` compact multi-column layout (like `ls` default)

### 6. Git Per-File Status
- [ ] Show `M` / `?` / `+` per-file git status icons
- [ ] Color modified/staged/untracked items differently
- [ ] Show .gitignore'd files dimmed

### 7. Config System
- [ ] `fm config` TUI or file-based preferences
- [ ] Remember sort order, icon theme, column layout
- [ ] Configurable visible columns

### 8. Size Visualization
- [ ] Size bar next to file size (like `du` or `dust`)
- [ ] Highlight largest files

### 9. Performance
- [ ] Cache directory scan results (TTL-based)
- [ ] `--depth N` for shallow recursive view
- [ ] Bench: <50ms for 10k files

### 10. Safety / Polish
- [ ] Deduplicate `autoload -U add-zsh-hook` in install.sh
- [ ] Test install.sh idempotency (run twice = no duplicates)
- [ ] `cargo clippy` pass
- [ ] `cargo fmt` pass

---

## Testing Backlog
- [ ] Test with 1000+ items in directory
- [ ] Test with very long filenames
- [ ] Test with CJK/emoji filenames
- [ ] Test with symlinks + broken symlinks
- [ ] Performance benchmark
