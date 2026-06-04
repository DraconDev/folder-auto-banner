# fab tasks

---

## ✅ Done

### Core Features
- [x] Directory listing with permissions, owner, group, size, date, name
- [x] File type icons (100+ mappings)
- [x] Git status per file (modified, added, deleted, untracked)
- [x] Git header (branch, ahead/behind, stash count)
- [x] Project type detection (Rust, Node, Python, Go, etc.)
- [x] Build status detection
- [x] TODO/FIXME count
- [x] Code metrics (LOC, files by type)
- [x] Port detection
- [x] Docker container detection
- [x] TTY detection (rich/raw modes)
- [x] Alternating row tints
- [x] Color scheme (dirs=blue, scripts=red, size/contents=orange)

### Daemon
- [x] Background daemon with Unix socket IPC
- [x] Inotify-based directory watching
- [x] Pre-computation of expensive operations
- [x] TTL-based caching (5 min)
- [x] Proactive home directory scanning
- [x] Resource limits (nice=10, ionice=idle)
- [x] Systemd service for auto-start
- [x] `f daemon stop/status` commands

### CLI Flags (Actions)
- [x] `--sort name|size|date|type|git|extension|version` — sorting
- [x] `-t`, `--timesort` — sort by time
- [x] `-S`, `--sizesort` — sort by size
- [x] `-X`, `--extensionsort` — sort by extension
- [x] `-G`, `--gitsort` — sort by git status
- [x] `--versionsort` — natural sort
- [x] `--no-sort` — directory order
- [x] `--reverse` — reverse sort
- [x] `--group-dirs first|last` — group directories
- [x] `-a`, `--hidden` — show dotfiles
- [x] `--tree [depth]` — tree view
- [x] `--group` — group by type
- [x] `--filter <pattern>` — filter by name
- [x] `--max <N>` — limit items
- [x] `--compact` — less info
- [x] `--verbose` — more info
- [x] `--json` — JSON output
- [x] `--raw` — plain text output

### Shell Integration
- [x] Zsh hook (chpwd)
- [x] Bash hook (PROMPT_COMMAND)
- [x] Install script

### Safety
- [x] Symlink loop prevention
- [x] Broken symlink display (✗→ indicator)
- [x] Full resolved symlink paths
- [x] Path protection for sensitive directories

### Simplification (v0.3.0)
- [x] Removed file ops commands (cp, mv, rm, trash, open)
- [x] Removed clipboard commands (yank, paste, clipboard)
- [x] Removed navigation commands (pin, unpin, pins, jump, root)
- [x] Removed session commands (save, load, list, delete)
- [x] Removed other commands (diff, do, peek, stats, config)
- [x] Renamed binary from fm to f

### Git Enhancements
- [x] Last commit time — "2h ago", "just now"
- [x] Commits today — "5 today"
- [x] Branch count — "3 branches"

### Languages Breakdown
- [x] Top 3 languages with percentages — "Rust 90% Markdown 4% Shell 1%"

### Build Timing
- [x] Build duration — "✓ builds (12s)"

### Cached Test Results
- [x] Last test run — "✓ 42 tests (3m ago)"
- [x] Auto-expires after 1 hour

### Display Improvements
- [x] Two-row layout for better readability
- [x] Dynamic truncation for narrow terminals
- [x] Classify mode (append */=>@|)

### Configuration (v0.4.0)
- [x] `f config` command — opens config in $EDITOR
- [x] Config file at `~/.config/fab/config.toml`
- [x] Permission mode (rwx, octal, disable)
- [x] Size mode (default, short, bytes)
- [x] Date mode (date, relative)
- [x] Classify mode (append */=>@|)
- [x] Column selection (show/hide columns)
- [x] Feature toggles (git, build, todos, languages, ports, docker)

---

## 🔴 Current Issues

### Performance (Critical)
- [ ] **file_statuses bloat** — `get_git_info()` stores every untracked file in `file_statuses` HashMap. In this repo: 36,501 entries (from `target/` build dir) = 3.4MB serialized over IPC on every request. Fix: count only, don't store paths.

### Daemon
- [ ] Fix daemon log spam for non-existent directories
- [ ] Fix daemon to handle dead symlinks gracefully
- [ ] Fix daemon connection issues (sometimes "daemon not available")

### Display
- [ ] Fix permission display for octal mode calculation
- [ ] Ensure config settings are applied correctly

---

## 🟡 Future Improvements

### Display Enhancements
- [ ] Add `--hyperlink` flag to attach hyperlinks
- [ ] Add `--header` flag to show block headers
- [ ] Add `--total-size` flag to show total directory size
- [ ] Add `--truncate-owner` flag to truncate long names

### Sorting Enhancements
- [ ] Sort by git status (modified first) — needs git status per file
- [ ] Sort by file creation time

### Performance
- [ ] Optimize banner for large directories (1000+ files)
- [ ] Cache directory scan results more aggressively
- [ ] Parallel file system operations

### Context Enhancements
- [ ] Show last commit date per file
- [ ] Show file count in directory header
- [ ] Show total size in directory header
- [ ] Show git stash count
- [ ] Show git worktree state (rebase, cherry-pick)
- [ ] Show outdated dependencies count

---

## ⚪ Polish

### Documentation
- [x] Update README with vision and scope
- [x] Update CHANGELOG with v0.4.0 changes
- [x] Create VISION.md with project direction
- [x] Create AUDIT.md with comparison to lsd/exa
- [ ] Add man page
- [x] Document config file options

### CI/CD
- [ ] Add cross-compilation (macOS, Linux ARM)
- [ ] Ship both binaries in GitHub releases
- [ ] Add `--locked` flag to cargo build in CI
- [ ] Add MSRV to Cargo.toml

### Code Quality
- [ ] Remove unused code (dead_code warnings)
- [ ] Add unit tests for new features
- [ ] Add integration tests for config command

---

## 📊 Summary

**Current state:** Simple, focused tool with config-based customization
**Core feature:** Directory listing with instant context
**Commands:** `f` (banner), `f env` (shell aliases), `f config` (edit config), `f daemon` (manage daemon)
**CLI flags:** Sorting, filtering, tree view, compact/verbose, JSON/raw
**Config file:** Display preferences, column selection, feature toggles

**Design principle:**
- CLI flags = actions (change often)
- Config file = preferences (set once)

**Result:** A better `ls`/`exa`/`lsd` that shows what matters instantly, with clean config for persistent settings.
