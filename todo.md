# cfm TODO

## ✅ Done (since last audit)

### Table / Display
- [x] Replace manual `println!` box-drawing with `comfy-table` v6
- [x] Custom preset matching original `│`, `─`, `├`, `┼`, `┤` style
- [x] Responsive column widths (caps at 100, min 60, name column gets remainder)
- [x] Merge icon into name column (`📂 .dracon` instead of wasted icon column)
- [x] `max_height(1)` prevents row wrapping; `truncate()` handles overflow with `…`

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
- [x] All 10 tests pass

---

## 🔜 Next Up — Lifted from lsd Analysis

`lsd` source study (`src/display.rs`, `src/icon.rs`) revealed patterns we can adopt:

### 1. File-Type Icons (replaces 📂/📄 everywhere)
Current: only `📂` (dir) and `📄` (file).
Target: 3-tier icon lookup like lsd's `Icon::get()`:

| Tier | Match | Examples |
|------|-------|----------|
| Filename | exact match | `Cargo.toml` → 🦀, `package.json` → 📦, `Makefile` → ⚙ |
| Extension | `.ext` → icon | `.rs` → 🦀, `.py` → 🐍, `.js` → , `.ts` → , `.go` → , `.md` →  |
| Fallback | type-based | dir → 📁, executable → ⚙, symlink → 🔗, regular → 📄 |

- [ ] Define `IconMap` struct with `names: HashMap` + `extensions: HashMap`
- [ ] Ship a default mapping embedded in binary (no YAML theme files yet)
- [ ] Nerd Font icons for languages (` js`, ` ts`, ` c`, ` cpp`, ` lua`, etc.)
- [ ] Unicode fallback icons when Nerd Font unavailable

### 2. Permission Column (like `lsd -l`)
- [ ] Add optional `PERM` column: `drwxr-xr-x`
- [ ] Configurable via `--blocks` or config flag
- [ ] Show owner/group optionally

### 3. Fix Unicode Width Handling
Current `truncate()` counts `char`s, not display width. CJK chars (width 2) and emoji are miscounted.
- [ ] Add `unicode-width` crate dependency
- [ ] Replace `truncate()` with display-width-aware version
- [ ] Test with: 日本語, 中文, 🦀📂 emoji

### 4. Color by File Type
- [ ] Directories → blue/cyan
- [ ] Executables → green
- [ ] Symlinks → magenta
- [ ] Hidden files → dim/gray
- [ ] Use `crossterm` (already a transitive dep via comfy-table) or `anstyle`

### 5. Symlink Target Display
- [ ] Show `→ target` after symlink name (like `lsd -l`)
- [ ] Already have `is_symlink` metadata in `DirEntry`

---

## 🗺️ Bucket List

### Header Enhancements
- [ ] Last modified date of directory itself
- [ ] Free/used filesystem space
- [ ] Parent directory size

### Column Options
- [ ] `--sort name|size|modified|type` flag
- [ ] `--filter` flag (type, size range, name pattern)
- [ ] `--max N` flag (limit items shown)
- [ ] `--hidden` flag to always show dotfiles

### Table Views
- [ ] `--tree` recursive view (like `lsd --tree`)
- [ ] `--grid` compact multi-column layout (like `ls` default)

### Git Per-File Status
- [ ] Show `M` / `?` / `+` per-file git status icons
- [ ] Color modified/staged/untracked items differently
- [ ] Show .gitignore'd files dimmed

### Config System
- [ ] `fm config` TUI or file-based preferences
- [ ] Remember sort order, icon theme, column layout
- [ ] Configurable visible columns (permissions on/off, etc.)

### Size Visualization
- [ ] Size bar next to file size (like `du` or `dust`)
- [ ] Highlight largest files

### Performance
- [ ] Cache directory scan results (TTL-based)
- [ ] `--depth N` for shallow recursive view
- [ ] Bench: <50ms for 10k files

### Safety / Polish
- [ ] Deduplicate `autoload -U add-zsh-hook` in install.sh (guard with grep)
- [ ] Test install.sh idempotency (run twice = no duplicates)
- [ ] `cargo clippy` pass
- [ ] `cargo fmt` pass

---

## Testing Backlog
- [ ] Test with 1000+ items in directory
- [ ] Test with very long filenames
- [ ] Test with CJK/emoji filenames (after unicode-width fix)
- [ ] Test with symlinks + broken symlinks
- [ ] Performance benchmark
