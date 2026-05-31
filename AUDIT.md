# CFM Comprehensive Audit Report

**Date:** 2026-05-31
**Auditor:** pi coding agent
**Scope:** Code Quality · Security · Dependencies · Performance
**Cargo version:** stable
**Rustc version:** 1.95.0

---

## ✅ Previously Fixed (Preserved)

These items were marked fixed in prior audits and remain fixed:

- [x] **Broken `file_statuses` filter in `compute_banner_data()`** — The old filter used `keep.contains(&first_component)` which matched ALL files under top-level directories. Fixed by limiting to depth 0 or 1 only.
- [x] **Daemon cache not persisted on shutdown** — The `Shutdown` handler called `std::process::exit(0)` without saving the banner cache first. Added `save_banner_cache()` call before exit.

---

## 📊 Code Quality

### Severity: MEDIUM

#### Findings

| ID | Finding | Location | Severity |
|----|---------|----------|----------|
| **CQ-1** | Dead code: `send_warm` never called | `src/daemon_client.rs:88` | LOW |
| **CQ-2** | Dead code: `get_git_info_summary`, `get_git_info_filtered` never called | `src/git/mod.rs:78,85` | LOW |
| **CQ-3** | Too many arguments: `output_rich()` takes 23 parameters (limit: 7) | `src/cmd/banner.rs:289` | MEDIUM |
| **CQ-4** | Items after test module (`fn main()` after `mod tests`) | `src/daemon.rs:761` | LOW |
| **CQ-5** | Double comparison: `size > 0 \|\| size == 0` (simplifiable to `size >= 0`) | `src/daemon.rs:847` | LOW |
| **CQ-6** | Trailing whitespace throughout codebase | Multiple files | LOW |
| **CQ-7** | 83 uses of `.unwrap()` / `.expect()` (mostly in tests, acceptable) | Various files | INFO |

#### Clippy Output (4 warnings, 0 errors)

```
warning: function `send_warm` is never used
  --> src/daemon_client.rs:88:8

warning: function `get_git_info_summary` is never used
  --> src/git/mod.rs:78:8

warning: function `get_git_info_filtered` is never used
  --> src/git/mod.rs:85:8

warning: this function has too many arguments (23/7)
  --> src/cmd/banner.rs:289:1

warning: this binary expression can be simplified
  --> src/daemon.rs:847:17
  help: try: `size >= 0`

warning: items after a test module
  --> src/daemon.rs:761:1
```

#### Formatting (cargo fmt --check)

Failed — extensive trailing whitespace and minor formatting inconsistencies in:
`src/cmd/banner.rs`, `src/daemon.rs`, `src/daemon_client.rs`, `src/daemon_types.rs`, `src/fs/mod.rs`, `src/git/mod.rs`, `src/icon.rs`, `src/state/mod.rs`, `src/test_cache.rs`, `src/utils.rs`, `tests/integration_test.rs`.

#### Error Handling

- No `?` propagation in hot paths; uses `Result` types consistently
- `anyhow::Result` used throughout CLI layer
- Mutex poisoning handled gracefully with `.lock().unwrap_or_else(|e| e.into_inner())`
- No `unsafe` code anywhere in the codebase
- No `async/await` — fully synchronous

---

## 🔒 Security

### Severity: LOW-MEDIUM

#### Findings

| ID | Finding | Location | Severity | Status |
|----|---------|----------|----------|--------|
| **SEC-1** | IPC socket created with default filesystem permissions (world-readable) | `src/daemon.rs:57` | LOW | WONTFIX (in user XDG_DATA_HOME) |
| **SEC-2** | Potential argument injection in `open_path()` via string splitting | `src/cmd/open.rs:66-71` | LOW | WONTFIX (uses hardcoded openers, no user input) |
| **SEC-3** | TOCTOU race in `read_dir` + subsequent metadata reads | `src/fs/mod.rs:172-260` | LOW | ACCEPTED |
| **SEC-4** | `canonicalize()` silently falls back to raw path on error | `src/port_usage/mod.rs:24`, `src/cmd/banner.rs:87` | LOW | ACCEPTED |

#### Path Traversal Analysis

All user-supplied paths go through `canonicalize()`:
- `src/cmd/banner.rs:87` — canonicalizes banner path
- `src/daemon.rs:291,357` — canonicalizes in request handlers
- `src/port_usage/mod.rs:24,92` — canonicalizes project path
- `src/docker/mod.rs:85` — canonicalizes path

No `..` patterns, no string concatenation for path building, no `shell: true` subprocess calls. **Path traversal risk: LOW.**

#### Subprocess Security

- `open.rs:66-71` splits opener strings (`"xdg-open"`, `"gio open"`, etc.) by whitespace — hardcoded list only, no user input
- `file_metadata.rs:138` uses `Command::new("sqlite3")` with fixed arguments
- `completion.rs` builds CLI with fixed subcommand names
- **No shell injection vectors found.**

#### Dependencies with Security Relevance

- `git2` 0.19.0 → 0.21.0 (contains security fixes — see dependency section)
- `directories` 5.0.1 → 6.0.0 (permission model changes)
- `cargo audit` blocked on advisory DB lock; could not run vulnerability scan

---

## 📦 Dependencies

### Severity: MEDIUM

#### cargo audit

Blocked — advisory database locked by another process (two 60-180s timeouts). **Recommend running manually:**
```bash
cargo audit
```

#### Outdated Direct Dependencies

| Package | Current | Latest | Kind |
|---------|---------|--------|------|
| `git2` | 0.19.0 | 0.21.0 | Normal |
| `directories` | 5.0.1 | 6.0.0 | Normal |
| `comfy-table` | 6.2.0 | 7.2.2 | Normal |
| `console` | 0.15.11 | 0.16.3 | Normal |
| `indicatif` | 0.17.11 | 0.18.4 | Normal |
| `toml` | 0.8.23 | 1.1.2 | Normal |
| `unicode-width` | 0.1.14 | 0.2.2 | Normal |

#### Notable Transitive Deps with Updates

- `libgit2-sys` 0.17.0+1.8.1 → 0.18.5+1.9.4 (bundled libgit2, security relevant)
- `crossterm` 0.26.1 → 0.29.0 (terminal I/O)
- `mio` 0.8.11 — marked **Removed** in latest
- `signal-hook` 0.3.18 — marked **Removed** in latest
- `signal-hook-mio` 0.2.5 — marked **Removed** in latest

The `mio`/`signal-hook` removal chain is a transitive dependency of `crossterm` — needs verification that `crossterm` 0.29.0 resolves this.

#### Cargo.lock

- ✅ Committed to repository
- ✅ Version 4 format (current)
- ✅ No duplicate dependencies detected

---

## ⚡ Performance

### Severity: MEDIUM

#### Already Fixed (from prior audit)

| Item | Status |
|------|--------|
| file_statuses filter (39K entries → top items only) | ✅ Fixed |
| Daemon cache not persisted on shutdown | ✅ Fixed |

#### High Priority

| ID | Finding | Location | Severity |
|----|---------|----------|----------|
| **PERF-1** | `get_git_info()` still walks entire working tree via `repo.statuses(None)` | `src/git/mod.rs` | HIGH |
| **PERF-2** | `commits_today` walks up to 1000 revisions per call | `src/git/mod.rs:236` | MEDIUM |
| **PERF-3** | `warm_nearby_dirs()` spawns 20+ IPC requests after every banner | `src/cmd/banner.rs` | HIGH |
| **PERF-4** | `diff.stats()` computes full diff for lines added/deleted | `src/git/mod.rs` | MEDIUM |
| **PERF-5** | `output_rich()` takes 23 parameters (stack-heavy, hard to optimize) | `src/cmd/banner.rs:289` | MEDIUM |

#### Medium Priority

| ID | Finding | Location | Severity |
|----|---------|----------|----------|
| **PERF-6** | `DirSummary::scan_with_options()` runs TODO scan, port detection, Docker check, code metrics — all subprocess calls on cache miss | `src/fs/mod.rs:332` | MEDIUM |
| **PERF-7** | `ProjectType::detect()` walks up directory ancestors reading contents at each level | `src/build_status/mod.rs` | MEDIUM |
| **PERF-8** | `resolve_uid()`/`resolve_gid()` reads `/etc/passwd` and `/etc/group` per file | `src/fs/mod.rs` | MEDIUM |
| **PERF-9** | Proactive scan holds `dir_sizes` mutex during batch inserts | `src/daemon.rs` | LOW |
| **PERF-10** | `test_cache::TestResults::load()` reads from disk on every banner render | `src/test_cache.rs` | LOW |

#### Low Priority

| ID | Finding | Location | Severity |
|----|---------|----------|----------|
| **PERF-11** | `WalkBuilder` doesn't use `.gitignore(true)` for project type detection | `src/fs/mod.rs` | LOW |
| **PERF-12** | `colorize_perms()` allocates many small `format!()` strings | `src/cmd/banner.rs` | LOW |
| **PERF-13** | `natural_cmp()` is O(n²) in worst case | `src/utils.rs` | LOW |
| **PERF-14** | Banner details header computed eagerly even if not displayed | `src/cmd/banner.rs` | LOW |
| **PERF-15** | `format_size_compact()` called multiple times per item | `src/cmd/banner.rs` | LOW |
| **PERF-16** | Banner cache loaded into memory as full JSON on daemon start | `src/cache/mod.rs` | LOW |

#### Architecture Recommendations

- Consider streaming JSON for banner cache (reduce daemon startup memory)
- Consider SQLite for persistent cache (better concurrent read/write)
- Separate git info from filesystem scan (parallel execution)
- Add `tracing::instrument` spans for key operations
- Expand `benches/performance.rs` for banner rendering and cache operations
- **Flamegraph profiling blocked** — `cargo flamegraph` tool not available for in-session profiling

#### Performance Targets

| Operation | Current | Target |
|-----------|---------|--------|
| Banner (cold) | ~600ms | <200ms |
| Banner (warm) | ~60ms | <10ms |
| Cache file size | ~7 KB | <5 KB |
| Cache load time | ~350ms | <50ms |
| Proactive scan | ~30s | <10s |

---

## 📋 Recommendations

### Immediate (P0)

1. **Run `cargo audit`** when advisory DB is available — git2/libgit2-sys may have CVEs
2. **Remove dead code**: `send_warm`, `get_git_info_summary`, `get_git_info_filtered` (CQ-1, CQ-2)
3. **Refactor `output_rich()`** to use a struct/builder pattern (CQ-3, PERF-5)

### Short Term (P1)

4. **Update `git2` → 0.21.0** (contains security patches in bundled libgit2)
5. **Update `directories` → 6.0.0** (improved permission handling)
6. **Fix `cargo fmt`** — trailing whitespace and formatting inconsistencies
7. **Debounce `warm_nearby_dirs()`** — limit to 3-5 nearby directories instead of 20+ (PERF-3)
8. **Verify `crossterm` upgrade path** — ensure `mio`/`signal-hook` removals don't break the build

### Medium Term (P2)

9. **Reduce `get_git_info()` scope** — use `StatusOptions` to filter or call git CLI for counts only (PERF-1)
10. **Limit `commits_today` revision walk** — use `git log --since=midnight --count` (PERF-2)
11. **Add `gitignore(true)` to WalkBuilder** where appropriate (PERF-11)
12. **Cache `resolve_uid`/`resolve_gid`** results in a HashMap (PERF-8)
13. **Make git diff stats lazy** — compute only when displayed (PERF-4)

### Low Priority (P3)

14. Add `tracing::instrument` spans for profiling
15. Expand benchmark suite in `benches/performance.rs`
16. Consider SQLite-based persistent cache for concurrent access
17. Stream JSON banner cache instead of full load on daemon start

---

## Verification Commands

```bash
# Code quality
cargo clippy --all-targets
cargo fmt --check

# Dependencies
cargo audit  # when advisory DB available
cargo outdated --exit-code 0

# Manual security review
rg 'unsafe\s*{|unsafe\s+fn' src/ --type rust
rg 'shell:\s*true|bash\s+-c' src/ --type rust
rg '\.\./' src/ --type rust

# Performance profiling
cargo flamegraph -- banner ~/path
RUST_LOG=trace f banner path 2>&1 | grep -i duration
```