# cfm tasks

---

## ✅ Done

### Core Banner
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

### CLI Flags
- [x] `--sort name|size|date|type` — sorting
- [x] `--reverse` — reverse sort
- [x] `--hidden` — show dotfiles
- [x] `--tree [depth]` — tree view
- [x] `--json` — JSON output
- [x] `--raw` — plain text output
- [x] `--filter <pattern>` — filter by name
- [x] `--max <N>` — limit items displayed
- [x] `--group` — group by type

### Shell Integration
- [x] Zsh hook (chpwd)
- [x] Bash hook (PROMPT_COMMAND)
- [x] Install script

### Safety
- [x] Symlink loop prevention
- [x] Broken symlink display (✗→ indicator)
- [x] Full resolved symlink paths
- [x] Path protection for sensitive directories

---

## 🔴 Remove (Commands That Duplicate Existing Tools)

### File Operations
- [ ] Remove `cp` command — people use their own cp
- [ ] Remove `mv` command — people use their own mv
- [ ] Remove `rm` command — people use their own rm
- [ ] Remove `trash` command — people use trash-cli
- [ ] Remove `open` command — people use xdg-open

### Clipboard
- [ ] Remove `yank` command — niche use case
- [ ] Remove `paste` command — niche use case
- [ ] Remove `clipboard` command — niche use case

### Navigation
- [ ] Remove `pin` command — redundant with frecency
- [ ] Remove `unpin` command — redundant with frecency
- [ ] Remove `pins` command — redundant with frecency
- [ ] Remove `jump` command — z/zoxide is better
- [ ] Remove `root` command — git root is trivial

### Sessions
- [ ] Remove `save-session` command — over-engineering
- [ ] Remove `load-session` command — over-engineering
- [ ] Remove `sessions` command — over-engineering
- [ ] Remove `delete-session` command — over-engineering

### Other
- [ ] Remove `diff` command — people use diff/meld
- [ ] Remove `do` command — niche
- [ ] Remove `peek` command — people use bat/cat
- [ ] Remove `stats` command — covered by banner
- [ ] Remove `config` command — use env vars instead

### Flags to Remove
- [ ] Remove `--no-build-check` flag — use env var
- [ ] Remove `--no-todos` flag — use env var
- [ ] Remove `--no-ports` flag — use env var
- [ ] Remove `--no-docker` flag — use env var
- [ ] Remove `--no-metrics` flag — use env var

---

## 🟡 Add (Useful Features)

### Output Modes
- [ ] Add `--compact` flag — less info, just essentials
- [ ] Add `--verbose` flag — more info, deep dive
- [ ] Add `--format <template>` flag — custom output format

### Context Improvements
- [ ] Show last commit date per file
- [ ] Show file count in directory header
- [ ] Show total size in directory header
- [ ] Show git stash count
- [ ] Show git worktree state (rebase, cherry-pick)

### Sorting Improvements
- [ ] Sort by git status (modified first)
- [ ] Sort by file type (dirs, then files)
- [ ] Sort by extension

---

## 🔵 Fix (Issues to Address)

### Daemon
- [ ] Fix daemon log spam for non-existent directories
- [ ] Fix daemon to handle dead symlinks gracefully
- [ ] Fix install script to properly stop daemon before reinstall

### Display
- [ ] Fix broken symlink display (show ✗→ indicator)
- [ ] Fix symlink target to show full resolved path
- [ ] Fix permissions display for symlinks

### Performance
- [ ] Optimize banner for large directories (1000+ files)
- [ ] Cache directory scan results more aggressively

---

## ⚪ Polish

### Documentation
- [ ] Update README with vision and scope
- [ ] Document all flags with examples
- [ ] Add man page

### CI/CD
- [ ] Add cross-compilation (macOS, Linux ARM)
- [ ] Ship both binaries in GitHub releases

---

## 📊 Summary

**Current state:** 30+ commands, complex
**Target state:** 1 command (`f`), simple, focused

**Keep:**
- Banner (directory listing + context)
- Shell hook (auto-banner)
- Sorting, filtering, tree view
- JSON/raw output

**Remove:**
- 15+ commands that duplicate existing tools
- 5+ flags that can be env vars

**Add:**
- Compact/verbose modes
- Better context per project type
- Custom format template

**Result:** Simple, fast, useful directory listing with instant context.
