# cfm tasks

## Approved features to implement

### 1. Build status
Show if project compiles without errors
- Rust: `cargo check --message-format=short`
- Node/TS: `tsc --noEmit`
- Go: `go build ./...`
- Python: `python -m py_compile`
- Fast check only, don't run full build

### 2. Docker integration
Show running containers for this project
- Check `docker ps` for containers with matching labels/volumes
- Or check `docker-compose.yml` in project root
- Show: container count, status (running/stopped)

### 3. Port usage
Show what ports are in use by processes in this directory
- `lsof -i -P -n | grep <dir>` or `ss -tlnp`
- Show: port numbers, process names
- Useful for web dev projects

### 4. TODO/FIXME count
Scan code for TODOs and FIXMEs
- Pattern: `- [ ]`, `TODO:`, `FIXME:`, `HACK:`
- Quick scan of source files (skip node_modules, target, etc.)
- Show count in header

### 5. Code metrics
Lines of code, file count by type
- Fast: just count lines in source files
- Skip binary files, vendored deps
- Show: total LOC, file count by extension

### 6. Branch comparison
Show if deviating from main/master
- Compare HEAD to origin/main or origin/master
- Show: commits ahead/behind
- Already have ahead/behind, but make it more visible

### 7. Stale files ordering
Sort files by modification time, oldest first
- Option to sort by: name, size, date, type
- Default: directories first, then by date
- Could add `--sort` flag

## Implementation notes

- All features should be fast (<100ms)
- Default to hidden/off, enable via flags or env vars
- Use existing patterns (like git integration)
- Don't block on slow operations
- Cache results where possible

## Priority

1. Build status - most useful for dev workflow
2. TODO count - quick win, useful
3. Branch comparison - enhance existing git info
4. Code metrics - nice to have
5. Port usage - useful for web dev
6. Docker - useful for containerized projects
7. Stale files - nice to have
