# cfm tasks

## 1. Build status

- [ ] Add `build_status` field to `DirSummary` struct
- [ ] Detect project type from existing `project_type` enum
- [ ] Implement Rust build check: `cargo check --message-format=short 2>&1`
- [ ] Implement Node/TS build check: `tsc --noEmit 2>&1 || npm run build --dry-run 2>&1`
- [ ] Implement Go build check: `go build ./... 2>&1`
- [ ] Implement Python build check: `python -m py_compile *.py 2>&1`
- [ ] Add timeout (500ms max) to prevent blocking
- [ ] Cache result for 30 seconds to avoid repeated checks
- [ ] Add `--no-build-check` flag to disable
- [ ] Show in header: `✓ builds` (green) or `✗ build errors` (red)
- [ ] Add `CFM_NO_BUILD_CHECK=1` env var to disable

## 2. Docker integration

- [ ] Add `docker_info` field to `DirSummary` struct
- [ ] Check for `docker-compose.yml` or `docker-compose.yaml` in project root
- [ ] Check for `Dockerfile` in project root
- [ ] Run `docker ps --filter "label=com.docker.compose.project=<dir>" --format "{{.Names}}:{{.Status}}"` 
- [ ] Run `docker ps --filter "volume=<dir>" --format "{{.Names}}:{{.Status}}"`
- [ ] Parse output to get container count and status
- [ ] Add timeout (500ms max)
- [ ] Cache result for 10 seconds
- [ ] Show in header: `🐳 3 containers (2 running)`
- [ ] Add `--no-docker` flag to disable
- [ ] Add `CFM_NO_DOCKER=1` env var to disable

## 3. Port usage

- [ ] Add `ports` field to `DirSummary` struct
- [ ] Run `lsof -i -P -n 2>/dev/null | grep <dir> | awk '{print $9}' | sort -u`
- [ ] Or run `ss -tlnp 2>/dev/null | grep <dir> | awk '{print $4}' | sort -u`
- [ ] Parse output to get port numbers
- [ ] Add timeout (500ms max)
- [ ] Cache result for 10 seconds
- [ ] Show in header: `🔌 :3000, :8080`
- [ ] Add `--no-ports` flag to disable
- [ ] Add `CFM_NO_PORTS=1` env var to disable

## 4. TODO/FIXME count

- [ ] Add `todo_count` field to `DirSummary` struct
- [ ] Scan source files for patterns: `- [ ]`, `TODO:`, `FIXME:`, `HACK:`, `XXX:`
- [ ] Skip directories: `node_modules`, `target`, `.git`, `dist`, `build`, `vendor`, `.next`
- [ ] Skip binary files (check extension: `.exe`, `.bin`, `.o`, `.so`, `.dll`, `.dylib`)
- [ ] Use `grep -r` or Rust `walkdir` + `regex` for fast scanning
- [ ] Limit scan to first 1000 files to prevent hanging
- [ ] Add timeout (1 second max)
- [ ] Cache result for 60 seconds
- [ ] Show in header: `📝 12 TODOs`
- [ ] Add `--no-todos` flag to disable
- [ ] Add `CFM_NO_TODOS=1` env var to disable

## 5. Code metrics

- [ ] Add `code_metrics` field to `DirSummary` struct
- [ ] Count lines of code in source files
- [ ] Count files by extension (top 5)
- [ ] Skip directories: `node_modules`, `target`, `.git`, `dist`, `build`, `vendor`, `.next`
- [ ] Skip binary files
- [ ] Use `wc -l` or Rust implementation for fast counting
- [ ] Limit scan to first 1000 files
- [ ] Add timeout (1 second max)
- [ ] Cache result for 60 seconds
- [ ] Show in header: `📊 4.2k LOC (rust: 3.1k, ts: 800)`
- [ ] Add `--no-metrics` flag to disable
- [ ] Add `CFM_NO_METRICS=1` env var to disable

## 6. Branch comparison

- [ ] Enhance existing `GitInfo` struct
- [ ] Get upstream branch: `git rev-parse --abbrev-ref --symbolic-full-name @{u}`
- [ ] Get ahead count: `git rev-list --count <upstream>..HEAD`
- [ ] Get behind count: `git rev-list --count HEAD..<upstream>`
- [ ] Get divergence: `git rev-list --left-right --count <upstream>...HEAD`
- [ ] Add timeout (500ms max)
- [ ] Cache result for 30 seconds
- [ ] Show in header: `↑3 ↓2` (ahead 3, behind 2)
- [ ] Already have ahead/behind, make more prominent
- [ ] Add color: green if ahead, red if behind, yellow if both

## 7. Stale files ordering

- [ ] Add `--sort` flag with options: `name`, `size`, `date`, `type`
- [ ] Default sort: directories first, then by date (newest first)
- [ ] Implement sort by name: alphabetical
- [ ] Implement sort by size: largest first
- [ ] Implement sort by date: newest first
- [ ] Implement sort by type: group by extension
- [ ] Add `--reverse` flag to reverse sort order
- [ ] Add `CFM_SORT=date` env var for default sort
- [ ] Update `display_items` sorting logic in `output_rich`

## Implementation notes

- All features should be fast (<100ms)
- Default to hidden/off, enable via flags or env vars
- Use existing patterns (like git integration)
- Don't block on slow operations
- Cache results where possible
- Add progress indicator for slow operations
- Gracefully handle missing tools (docker, lsof, etc.)
- Log errors to stderr, don't show in banner

## Priority

1. Build status - most useful for dev workflow
2. TODO count - quick win, useful
3. Branch comparison - enhance existing git info
4. Code metrics - nice to have
5. Port usage - useful for web dev
6. Docker - useful for containerized projects
7. Stale files - nice to have

## Testing

- [ ] Add unit tests for each new feature
- [ ] Add integration tests for flag handling
- [ ] Test with different project types (Rust, Node, Go, Python)
- [ ] Test with missing tools (docker not installed, etc.)
- [ ] Test with large directories (1000+ files)
- [ ] Test with slow operations (timeout handling)
- [ ] Test caching behavior
- [ ] Test env var overrides
