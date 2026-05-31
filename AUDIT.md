# CFM Performance Audit

## ✅ Fixed

- [x] **Broken `file_statuses` filter in `compute_banner_data()`** — The old filter used `keep.contains(&first_component)` which matched ALL files under top-level directories (e.g., `target/debug/build/...` matched because `"target"` was in `top_items`). This caused 39,425 git status entries per project and 62.5 MB cache files. Fixed by limiting to depth 0 or 1 only.
- [x] **Daemon cache not persisted on shutdown** — The `Shutdown` handler called `std::process::exit(0)` without saving the banner cache first. Added `save_banner_cache()` call before exit.

## 🔍 Investigation Notes

### Root Cause
The banner took ~2.5s cold and ~1.8s warm because:
1. `compute_banner_data()` stored ALL git file statuses (39K+ entries for a single project)
2. These were serialized/deserialized over IPC on every banner request
3. The 62.5 MB cache file itself took 387ms just to parse

### After Fix
- Cold start: ~600ms (from ~2500ms)
- Warm cache: ~60ms (from ~1800ms)
- Cache size: ~7 KB (from 62.5 MB)

---

## 📋 Remaining Tasks

### High Priority

- [ ] **`get_git_info()` still walks entire working tree** — `repo.statuses(None)` gets ALL file statuses even though we only need top_items. Consider using `StatusOptions` to filter or calling git CLI for status counts only.
- [ ] **`commits_today` walks up to 1000 revisions** — For large repos with many daily commits, this is expensive. Could cache this or use `git log --since=midnight --count`.
- [ ] **`warm_nearby_dirs()` spawns 20+ IPC requests after every banner** — This fires `send_warm()` for parent + all siblings. Should be debounced or limited to fewer directories.
- [ ] **`diff.stats()` computes full diff for lines added/deleted** — `repo.diff_tree_to_workdir()` can be slow for large repos. Consider making this lazy or optional.

### Medium Priority

- [ ] **`DirSummary::scan_with_options()` runs multiple subprocess checks** — TODO scan, port detection, Docker check, and code metrics all run on every cache miss. These should have shorter TTLs or be lazy-loaded.
- [ ] **`ProjectType::detect()` walks up ancestors** — Reads directory contents at each level looking for project markers. Should cache result or limit ancestor depth.
- [ ] **`resolve_uid()`/`resolve_gid()` reads `/etc/passwd` per file** — Could be cached in a HashMap for the duration of the scan.
- [ ] **Proactive scan blocks banner requests during `du` batches** — The `dir_sizes` mutex is held during batch inserts. Should use a lock-free approach or reduce lock contention.
- [ ] **Banner cache JSON is huge even after fix** — Each entry still contains full `DirSummary` with all metadata. Consider stripping unnecessary fields before caching.

### Low Priority

- [ ] **`WalkBuilder` doesn't use `.gitignore()`** — Currently `ignore(false)` means it walks through `target/`, `node_modules/`, etc. for project type detection. Consider `.gitignore(true)` where appropriate.
- [ ] **`colorize_perms()` allocates many small strings** — Uses `format!()` for each character. Could use a single pre-allocated buffer.
- [ ] **`output_rich()` has too many parameters** — The function takes 20+ parameters. Should use a builder pattern or options struct.
- [ ] **`natural_cmp()` is O(n²) in worst case** — Could be optimized with a single-pass comparison.
- [ ] **No lazy loading of banner details** — The header (branch, stats, TODOs, code metrics) is computed eagerly even if not displayed. Could be deferred.
- [ ] **`test_cache::TestResults::load()` runs on every banner** — Reads test results from disk even when not needed. Should be cached with a short TTL.
- [ ] **`format_size_compact()` called multiple times per item** — Called for both display and width calculation. Could cache the formatted string.

### Architecture

- [ ] **Consider streaming JSON for banner cache** — The entire cache is loaded into memory on daemon start. For 100+ entries, this could be significant.
- [ ] **Consider Redis/SQLite for cache** — JSON files don't support concurrent reads/writes well. SQLite would be more robust.
- [ ] **Separate git info from filesystem scan** — These are independent operations that could run in parallel.
- [ ] **Add performance benchmarks to CI** — The existing `benches/performance.rs` should be expanded to cover banner rendering and cache operations.
- [ ] **Add tracing spans for performance profiling** — Instrument key operations with `tracing::instrument` to identify bottlenecks in production.

---

## 📊 Performance Targets

| Operation | Current | Target | Notes |
|-----------|---------|--------|-------|
| Banner (cold) | ~600ms | <200ms | Reduce git/filesystem work |
| Banner (warm) | ~60ms | <10ms | Optimize IPC serialization |
| Cache file size | ~7 KB | <5 KB | Strip unnecessary fields |
| Cache load time | ~350ms | <50ms | Use binary format or streaming |
| Proactive scan | ~30s | <10s | Parallelize, reduce scope |

---

## 🔧 How to Investigate Further

1. **Add timing instrumentation**:
   ```rust
   let start = std::time::Instant::now();
   // ... operation ...
   tracing::debug!("operation took {:?}", start.elapsed());
   ```

2. **Profile with `cargo flamegraph`**:
   ```bash
   cargo install flamegraph
   cargo flamegraph -- banner ~/Dev/cli-file-manager
   ```

3. **Measure IPC overhead**:
   ```bash
   # Compare daemon vs direct scan
   time f banner path  # with daemon
   time f banner path  # without daemon (kills daemon first)
   ```

4. **Check cache hit rates**:
   ```bash
   RUST_LOG=info f banner path 2>&1 | grep -i cache
   ```
