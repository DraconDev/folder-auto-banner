# cfm tasks

## Header redesign

- [x] Replace `12M` with `💾 12MB` (add icon + full unit for clarity)
- [x] Replace `15 files` with `📄 15 files` (correct file icon)
- [x] Replace `5 dirs` with `📂 5 dirs` (correct folder icon)
- [x] Change `16k LOC` to `16k lines` (clearer than jargon)
- [x] Drop LOC breakdown `(md: 10k, rs: 5.9k, no-ext: 149)` from header
- [x] Drop commit hash from header
- [x] Drop `FILES:1 DIRS:src` from header (unclear meaning)
- [x] Drop `DELTA:` label from git delta, keep just `+13 -6`
- [x] Show `*` suffix on branch name when repo is dirty
- [x] Add `✓ clean` when repo is clean
- [x] Fix symlinks: show target's contents (line count for files, item count for dirs)
- [x] Build, install, verify output

## Daemon (cfmd)

- [x] Design daemon architecture (IPC, lifecycle, data structures)
- [x] Create daemon binary (cfmd) with Unix socket server
- [x] Implement inotify-based directory watching (recompute on change)
- [x] Implement pre-computation: directory sizes, git status, TODO/LOC
- [x] Implement IPC protocol (request/response for cached data)
- [x] Modify banner to read from daemon cache (fallback to direct scan)
- [x] Add daemon management commands (start, stop, status, restart, clear-cache)
- [x] Add systemd/user service file for auto-start
- [x] Add resource limits (nice=10, ionice=idle)
- [x] Test: daemon startup, cache population, banner speed

## UX improvements

- [x] Alternating row tints (subtle gray on odd/even rows)
- [x] Color scheme: dirs=blue, scripts=red, size/contents=orange
- [x] Directory size: show `-` instead of misleading inode size
- [x] Symlinks: follow target for contents and metadata
