# CFM Performance Optimization

Implement the high and medium priority performance improvements from AUDIT.md.

## Goals
- Reduce cold banner time from ~600ms to <200ms
- Reduce warm cache time from ~60ms to <10ms  
- Reduce cache file size from ~7KB to <5KB
- Fix IPC serialization overhead

## Checklist

### High Priority
- [ ] Optimize `get_git_info()` - avoid walking entire working tree for status counts
- [ ] Optimize `commits_today` - avoid walking up to 1000 revisions
- [ ] Debounce `warm_nearby_dirs()` - limit IPC requests after banner display
- [ ] Make `diff.stats()` lazy or optional

### Medium Priority
- [ ] Add short TTLs to subprocess checks (TODO, ports, docker, metrics)
- [ ] Cache `ProjectType::detect()` result
- [ ] Cache `resolve_uid()`/`resolve_gid()` lookups per scan
- [ ] Reduce mutex contention in proactive scan
- [ ] Strip unnecessary fields from banner cache entries

### Verification
- [ ] Run benchmarks before/after each change
- [ ] Verify banner output still correct
- [ ] Test cache persistence

## Progress
(Update as items are completed)

## Notes
- Starting with git optimizations as they have highest impact
- Each change should be independently testable