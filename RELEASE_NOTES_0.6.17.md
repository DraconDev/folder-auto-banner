# folder-auto-banner 0.6.17

## Performance

- **Non-blocking directory-size refresh** — banner responses now return immediately with cached sizes while stale or missing child directory sizes refresh in the background, preventing zoxide/chpwd navigation from blocking on large `du` work.
- **Faster logical size calculation** — displayed directory sizes now use `du -s -b`, which is much faster for normal workspace trees and avoids falling back to the 4 KiB directory inode size for large directories.
- **Warmer size cache prepopulation** — warm precompute requests now schedule background size refreshes so parent and child banners are populated before the next navigation.

## Notes

- The first cold view of a very large directory returns quickly and may show cached placeholders until the background size refresh completes; subsequent warm calls use populated single-digit-millisecond cache entries.
- Warm cache hits remain single-digit milliseconds after pre-warm.
