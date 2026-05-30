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
- [x] `--compact` — less info
- [x] `--verbose` — more info

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
- [x] Removed flags that can be env vars
- [x] Renamed binary from fm to f

---

## 🔴 Current Issues

### Daemon
- [ ] Fix daemon log spam for non-existent directories (partially fixed)
- [ ] Fix daemon to handle dead symlinks gracefully (partially fixed)

### Display
- [ ] Fix broken symlink display (show ✗→ indicator) — DONE
- [ ] Fix symlink target to show full resolved path — DONE

---

## 🟡 Future Improvements

### Banner Enhancements
- [ ] Show last commit date per file
- [ ] Show file count in directory header
- [ ] Show total size in directory header
- [ ] Show git stash count
- [ ] Show git worktree state (rebase, cherry-pick)

### Sorting Improvements
- [ ] Sort by git status (modified first)
- [ ] Sort by file type (dirs, then files)
- [ ] Sort by extension

### Performance
- [ ] Optimize banner for large directories (1000+ files)
- [ ] Cache directory scan results more aggressively

---

## ⚪ Polish

### Documentation
- [x] Update README with vision and scope
- [x] Update CHANGELOG with v0.3.0 changes
- [x] Create VISION.md with project direction
- [ ] Add man page

### CI/CD
- [ ] Add cross-compilation (macOS, Linux ARM)
- [ ] Ship both binaries in GitHub releases

---

## 📊 Summary

**Current state:** Simple, focused tool
**Core feature:** Directory listing with instant context
**Commands:** Just `f` (banner) and `f env` (shell aliases)
**Flags:** Sorting, filtering, tree view, compact/verbose modes

**Result:** A better `ls`/`exa`/`lsd` that shows what matters instantly.
