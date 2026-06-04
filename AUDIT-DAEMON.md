# fab Daemon Architecture Audit & Optimization Plan

**Date:** 2026-06-01
**Goal:** Close the gap between current performance (198ms warm, 462ms cold) and audit targets (<10ms warm, <200ms cold).
**Key insight:** The daemon IS just serving cached data, yet the warm path is 20x slower than target. The bottleneck is not the cache — it's the IPC layer.

---

## 📊 Baseline Measurements (Profiled with FAB_PROFILE=1)

| Phase | Time | Notes |
|-------|------|-------|
| **Warm (direct binary)** | 96-100ms | `./target/debug/f banner` |
| **Warm (with `cargo run`)** | 282-435ms | Includes cargo startup overhead |
| **Cold (no daemon)** | 462ms | Full scan + render in client process |
| **IPC connect** | 20µs | Negligible (250ms on first connect due to 200ms sleep) |
| **IPC send_recv (JSON)** | **~95ms** | **Dominant cost** |
| **IPC payload size** | 8.4KB | `serde_json` of BannerData |
| **Render output** | ~5ms | Terminal ANSI escape codes |

**The 95ms IPC overhead is 95% of the warm time.** The cache lookup itself is <1ms.

---

## 🔍 Audit Findings by Category

### 1. IPC Layer (src/daemon_client.rs, src/daemon.rs)

| ID | Finding | Impact | Effort |
|----|---------|--------|--------|
| **IPC-1** | JSON serialization dominates warm path (95ms for 8.4KB) | **HIGH** (-90ms) | LOW (1-2h) |
| **IPC-2** | No connection reuse — new UnixStream per request | LOW (-0.5ms) | MEDIUM (2-3h) |
| **IPC-3** | Fragile message framing (shutdown-write for EOF) | LOW (correctness) | LOW (1h) |
| **IPC-4** | Mutex held during cache hit (blocks all clients) | MEDIUM (concurrency) | LOW (1h) |
| **IPC-5** | No request pipelining (one req per connection) | LOW | MEDIUM (2-3h) |
| **IPC-6** | No binary format option (serde_json only) | **HIGH** (-80ms) | LOW (1h) |
| **IPC-7** | BannerData payload 8.4KB (much is git file_statuses) | **HIGH** (-30ms) | LOW (1h) |

### 2. Cache Strategy (src/daemon.rs)

| ID | Finding | Impact | Effort |
|----|---------|--------|--------|
| **CACHE-1** | TTL=300s (5min) flat, no per-path variance | LOW | LOW (1h) |
| **CACHE-2** | inotify missing CLOSE_WRITE (large file writes trigger many events) | LOW (correctness) | LOW (0.5h) |
| **CACHE-3** | Proactive scan unbounded (scans all $HOME up to 3 levels) | MEDIUM (startup) | MEDIUM (2-3h) |
| **CACHE-4** | No LRU eviction — cache grows unbounded | MEDIUM (disk usage) | LOW (1-2h) |
| **CACHE-5** | warm_nearby_dirs(5) spawns thread per banner | LOW (-5ms) | LOW (0.5h) |
| **CACHE-6** | Proactive scan pre-computes banners for ALL level 1+2 dirs | MEDIUM (startup) | MEDIUM (2h) |
| **CACHE-7** | Banner cache saved to disk every 5 min (JSON) | LOW (-10ms during save) | LOW (0.5h) |
| **CACHE-8** | Cache key is canonicalize()'d path (syscall per request) | LOW (-0.1ms) | LOW (0.5h) |

### 3. Daemon Lifecycle (src/daemon.rs, src/daemon_client.rs)

| ID | Finding | Impact | Effort |
|----|---------|--------|--------|
| **LIFE-1** | 200ms hardcoded sleep in ensure_daemon_running | **HIGH** (-150ms cold) | LOW (0.5h) |
| **LIFE-2** | 50ms polling for socket (up to 2s worst case) | MEDIUM (cold start) | LOW (1h) |
| **LIFE-3** | No PID file (race on multiple daemons) | LOW (correctness) | LOW (1h) |
| **LIFE-4** | No signal handling (SIGTERM kills abruptly) | MEDIUM (data loss) | LOW (1h) |
| **LIFE-5** | nice(10) + ionice idle (too aggressive for proactive scan) | LOW | LOW (0.5h) |
| **LIFE-6** | Idle timeout 10 min (could be 5 for battery) | LOW | LOW (0.5h) |
| **LIFE-7** | Stale socket cleanup only on startup (race window) | LOW | LOW (0.5h) |
| **LIFE-8** | Daemon stderr null (errors invisible) | LOW (debuggability) | LOW (0.5h) |

### 4. Payload (fab-lib/src/fs/mod.rs, fab-lib/src/git/mod.rs)

| ID | Finding | Impact | Effort |
|----|---------|--------|--------|
| **PAY-1** | DirEntry has 12 fields, many skippable | MEDIUM (-20ms) | LOW (1h) |
| **PAY-2** | DirEntry.path duplicates name for top-level | LOW (-5ms) | LOW (0.5h) |
| **PAY-3** | GitInfo.file_statuses is the biggest field | **HIGH** (-30ms) | LOW (1h) |
| **PAY-4** | last_commit_msg is unbounded | LOW (-2ms) | LOW (0.5h) |
| **PAY-5** | GitInfo has 18 fields, ~8 skippable | LOW (-5ms) | LOW (1h) |
| **PAY-6** | BannerData.dir_sizes is always empty (dead) | LOW (-1ms) | LOW (0.5h) |
| **PAY-7** | BannerData.path is redundant | LOW (-0.5ms) | LOW (0.5h) |
| **PAY-8** | BannerData.cached_at rarely used | LOW (-0.5ms) | LOW (0.5h) |
| **PAY-9** | DirSummary has 5 Option fields (always present) | LOW | LOW (0.5h) |
| **PAY-10** | No field-level versioning (schema breaks cached banners) | LOW (correctness) | MEDIUM (2h) |

### 5. Cold Start (fab-lib/src/fs/mod.rs, etc.)

| ID | Finding | Impact | Effort |
|----|---------|--------|--------|
| **COLD-1** | 5 subprocess checks run sequentially | MEDIUM (-100ms) | MEDIUM (2-3h, needs rayon) |
| **COLD-2** | code_metrics walks all files (slowest check) | MEDIUM (-50ms) | MEDIUM (2h) |
| **COLD-3** | scan_todos reads all source files | MEDIUM (-30ms) | LOW (0.5h, env var exists) |
| **COLD-4** | get_git_info_filtered slow for large repos | MEDIUM (-200ms) | MEDIUM (3h, use git CLI) |
| **COLD-5** | ProjectType::detect walks 10 ancestors | LOW (-5ms) | LOW (already limited) |
| **COLD-6** | No direct-scan caching | HIGH (-300ms) | MEDIUM (3h, add local disk cache) |
| **COLD-7** | No minimum-viable banner fallback | LOW | LOW (1h) |

---

## 🎯 Prioritized Optimization Plan

### Quick Wins (P0 — <1h each, >50ms impact)

| # | Action | Expected Impact | Effort | Dependencies |
|---|--------|----------------|--------|--------------|
| 1 | **IPC-1+IPC-6: Add bincode feature flag** | **-80ms warm** | 1-2h | None |
| 2 | **PAY-3: Make file_statuses opt-in** | **-30ms warm** | 1h | None |
| 3 | **LIFE-1: Reduce sleep to 50ms** | **-150ms cold** | 0.5h | None |
| 4 | **PAY-1: Skip non-displayed DirEntry fields** | -20ms warm | 1h | None |
| 5 | **PAY-4: Truncate last_commit_msg to 80 chars** | -2ms warm | 0.5h | None |
| 6 | **PAY-6/7/8: Remove dead/redundant fields** | -2ms warm | 0.5h | None |
| 7 | **CACHE-5: Reduce warm_nearby_dirs to 3** | -5ms warm | 0.5h | None |
| 8 | **LIFE-4: Add signal handling for graceful shutdown** | Data safety | 1h | None |
| 9 | **CACHE-4: Add LRU eviction (max 1000 entries)** | Disk safety | 1-2h | None |
| 10 | **LIFE-3: Add PID file** | Correctness | 1h | None |

**Subtotal P0: ~-140ms warm, ~-150ms cold, ~5h effort**

### Short-term (P1 — 1-3h each, 50-200ms impact)

| # | Action | Expected Impact | Effort |
|---|--------|----------------|--------|
| 11 | **COLD-1: Parallelize subprocess checks (rayon)** | -100ms cold | 2-3h |
| 12 | **COLD-6: Add direct-scan disk cache** | -300ms cold (warm restart) | 3h |
| 13 | **COLD-3: Default scan_todos to disabled** | -30ms cold | 0.5h |
| 14 | **COLD-4: Use git CLI for status counts** | -200ms cold (large repos) | 3h |
| 15 | **CACHE-2: Add CLOSE_WRITE to inotify** | Correctness | 0.5h |
| 16 | **CACHE-3: Bound proactive scan (max 50 dirs, 30s timeout)** | -30s startup | 2-3h |
| 17 | **IPC-2: Reuse connection (one connect per banner)** | -0.5ms warm | 2h |
| 18 | **IPC-4: Use RwLock instead of Mutex on cache** | Concurrency | 1h |
| 19 | **PAY-10: Add field-level versioning to BannerData** | Correctness | 2h |

**Subtotal P1: ~-130ms cold, ~2-3ms warm, ~15h effort**

### Medium-term (P2 — architectural changes)

| # | Action | Expected Impact | Effort |
|---|--------|----------------|--------|
| 20 | **IPC-5: Length-prefixed binary protocol** | Enables pipelining | 2-3h |
| 21 | **COLD-2: Make code_metrics lazy/async** | -50ms cold | 2h |
| 22 | **CACHE-6: Smart proactive scan (only frequently-accessed dirs)** | -20s startup | 2h |
| 23 | **COLD-7: Minimum-viable banner (skip all subprocess checks)** | -150ms cold (worst case) | 1h |
| 24 | **PAY-9: Flatten DirSummary Option fields** | -5ms warm | 0.5h |
| 25 | **LIFE-2: Use inotify on socket dir instead of polling** | -2s cold (worst case) | 1h |

**Subtotal P2: ~-200ms cold, ~-5ms warm, ~10h effort**

---

## 🏆 Recommended Implementation Order

### Phase 1: "Make the daemon fast" (target: warm <30ms)
**Estimated effort: 5-6h**
1. **IPC-1+IPC-6**: Add bincode feature flag → 95ms → 15ms
2. **PAY-3**: Make file_statuses opt-in → 15ms → 12ms
3. **PAY-1, PAY-4, PAY-6/7/8**: Trim payload → 12ms → 10ms

### Phase 2: "Make the cold start fast" (target: cold <200ms)
**Estimated effort: 4-5h**
1. **LIFE-1**: Reduce sleep to 50ms → 462ms → 312ms
2. **COLD-1**: Parallelize subprocess checks → 312ms → 212ms
3. **COLD-3**: Default scan_todos off → 212ms → 182ms

### Phase 3: "Make the cold restart fast" (target: cold <100ms)
**Estimated effort: 3-4h**
1. **COLD-6**: Add direct-scan disk cache → 182ms → 50ms (warm restart)

### Phase 4: "Harden the daemon" (correctness + safety)
**Estimated effort: 5-6h**
1. **LIFE-4**: Signal handling
2. **LIFE-3**: PID file
3. **CACHE-2**: CLOSE_WRITE inotify
4. **CACHE-4**: LRU eviction
5. **PAY-10**: Schema versioning

---

## 📈 Expected Final Performance

| Metric | Current | After P0 | After P1 | After P2 | Target |
|--------|---------|----------|----------|----------|--------|
| Warm (direct binary) | 96-100ms | **~10ms** | ~10ms | ~10ms | <10ms ✅ |
| Cold (no daemon) | 462ms | ~312ms | ~182ms | ~50ms | <200ms ✅ |
| IPC payload | 8.4KB | ~3KB | ~3KB | ~3KB | <5KB ✅ |
| Cache hit rate | unknown | unchanged | +10% | +20% | >80% |

---

## 🔧 Key Implementation Notes

### Bincode migration (IPC-1+IPC-6)
- Add `bincode` as optional dependency, feature-gated
- Use `#[cfg(feature = "bincode")]` in `send_and_recv` and `send_response`
- Keep JSON as default for backward compatibility
- Bincode is ~5-10x faster than serde_json for the same data
- Expected: 95ms → 10-15ms for IPC

### file_statuses opt-in (PAY-3)
- Add `#[serde(skip)]` or `#[serde(skip_serializing_if = "HashMap::is_empty")]`
- Currently the field is populated for every banner even when not displayed
- Most banners only show aggregate counts (staged, modified, untracked)
- Expected: 8.4KB → 3KB payload, 95ms → 65ms IPC

### Parallel subprocess checks (COLD-1)
- Use `rayon::scope` or `tokio::join!` to run the 5 checks concurrently
- Currently: TODO scan + port detection + Docker check + code metrics + build status run sequentially
- Expected: -100ms cold (4 parallel checks → 1 sequential)

### Direct-scan disk cache (COLD-6)
- Write BannerData to `~/.cache/fab/cold_cache.json` on each direct scan
- On startup, check if cache exists and is <5min old
- Replaces the first cold start after daemon crash
- Expected: 182ms → 50ms (warm restart)

---

## 🎯 Success Criteria

After implementing P0 + P1:
- [ ] Warm banner render <30ms (currently 96ms)
- [ ] Cold banner render <250ms (currently 462ms)
- [ ] IPC payload <5KB (currently 8.4KB)
- [ ] Zero clippy warnings
- [ ] Zero fmt diffs
- [ ] All existing tests pass
- [ ] New benchmarks show improvement

After implementing P2:
- [ ] Cold banner render <100ms (warm restart)
- [ ] Warm banner render <15ms
- [ ] Daemon survives SIGTERM gracefully
- [ ] Cache stays bounded under heavy use
