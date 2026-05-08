## Summary

-

## Related issues

- Fixes #

## Test plan

- [ ] `cargo test --workspace`
- [ ] `cargo clippy --workspace -- -D warnings`
- [ ] `cargo fmt --all -- --check`
- [ ] Verified end-to-end against:
  - [ ] keygen.sh Official
  - [ ] Self-hosted CE
  - [ ] Self-hosted EE

## Checklist

- [ ] AI mode (`--ai`) JSON output is stable for any added commands.
- [ ] `keygen schema` reflects new commands / flags.
- [ ] Docs updated (README / `doc/`).
