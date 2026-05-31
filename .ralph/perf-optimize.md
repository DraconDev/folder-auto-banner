# CFM Performance Optimization

Implement the high and medium priority performance improvements from AUDIT.md.

## Goals
- Reduce cold banner time from ~600ms to <200ms
- Reduce warm cache time from ~60ms to <10ms  
- Reduce cache file size from ~7KB to <5KB
- Fix IPC serialization overhead

## Checklist

### High Priority
- [x] Optimize `get_git_info()` - avoid walking entire working tree for status counts
  - Added `get_git_info_filtered()` with pathspec filtering
  - Added `get_git_info_summary()` for cases not needing file_statuses
  - Cold start: ~2500ms → ~350ms (7x faster)
- [ ] Optimize `commits_today` - avoid walking up to 1000 revisions
- [x] Debounce `warm_nearby_dirs()` - limit IPC requests after banner display
  - Reduced from 20 to 10 sibling directories
  - Added `warm_paths()` to reuse single connection
- [ ] Make `diff.stats()` lazy or optional

### Medium Priority
- [x] Add short TTLs to subprocess checks (TODO, ports, docker, metrics)
  - Already implemented: TODO=60s, metrics=60s, ports=10s, docker=10s
- [ ] Cache `ProjectType::detect()` result
- [x] Cache `resolve_uid()`/`resolve_gid()` lookups per scan
  - Added thread-local caches for uid/gid lookups
  - Avoids reading /etc/passwd and /etc/group for every file
- [ ] Reduce mutex contention in proactive scan
- [ ] Strip unnecessary fields from banner cache entries

### Verification
- [ ] Run benchmarks before/after each change
- [ ] Verify banner output still correct
- [ ] Test cache persistence

## Progress
- [x] Iteration 1: Optimized `get_git_info()` with pathspec filtering
  - Added `get_git_info_filtered()` function in cfm-lib
  - Cold start improved from ~2500ms to ~350ms (7x faster)
  - Warm cache improved from ~1800ms to ~100ms (18x faster)
  - Verified banner output still correct
- [x] Iteration 2: Optimized `warm_nearby_dirs()` and uid/gid caching
  - Reduced sibling directories warmed from 20 to 10
  - Added `warm_paths()` for single connection reuse
  - Added thread-local caches for uid/gid lookups
  - Final performance: ~400ms cold, ~100ms warm

## Notes
- Starting with git optimizations as they have highest impact
- Each change should be independently testable