# cfm tasks

## Cuts (do now)

- [ ] Remove `build_status` check from default banner path (cargo check = 6.7s, blocks prompt)
- [ ] Make build status opt-in: `fm build-status` command only
- [ ] Remove `tokio` from Cargo.toml (zero async code, compile bloat)
- [ ] Replace `atty` with `std::io::IsTerminal` (Rust 1.70+)
- [ ] Fix `ProjectType::detect` to walk ancestor directories (currently shows "Generic" in subdirs)
- [ ] Consolidate duplicate `Session` struct (defined in save_session.rs, load_session.rs, sessions.rs — should live in state/mod.rs)

## Audit (after cuts)

- [ ] Run cargo clippy, cargo test, cargo fmt
- [ ] Verify banner renders instantly on cold cache
- [ ] Check binary size before/after
