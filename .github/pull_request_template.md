## Goal

<!-- このPRが達成する検証可能な目的を1つ書く。 -->

## Task

- Implementation task ID: `M?-??`
- Related specification:
  - `.github/assets/spec.md` section:
  - `docs/implementation-plan.md` task:
- Related ADR:

## Scope

<!-- 変更する責務と主要fileを書く。 -->

- 

## Non-goals

<!-- このPRでは意図的に行わないことを書く。 -->

- 

## Behavior

### Before

<!-- 変更前の状態。 -->

### After

<!-- 変更後に観測できる状態。 -->

## Invariants checked

- [ ] Normal listing remains local and deterministic
- [ ] Codex is used only by `lls setup`
- [ ] No OpenAI Platform API key handling was added
- [ ] Invalid config is not silently ignored
- [ ] `role`, `generated`, and `sensitive` remain separate concerns
- [ ] Scan depth and project probe remain independent
- [ ] Sensitive, binary, ignored, and symlink entries are excluded from recommendations
- [ ] stdout/stderr and exit-code contracts are preserved
- [ ] Public JSON ordering is deterministic

<!-- 関係しない項目は理由を書いてチェックする。無言で未チェックにしない。 -->

## Tests

### Added or changed

- 

### Commands

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

### Results

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] `cargo test --all-targets --all-features`
- [ ] `cargo build --release`

## Error and boundary cases

<!-- 正常系以外に確認したケースを書く。 -->

- Empty input:
- Boundary value:
- Invalid input:
- Partial failure:
- Cross-platform consideration:
- Sensitive-data consideration:

## Documentation

- [ ] `docs/implementation-status.md`を更新した
- [ ] 仕様変更時に`.github/assets/spec.md`を更新した
- [ ] config変更時に`config.schema.json`を更新した
- [ ] 設計判断がある場合にADRを追加した
- [ ] READMEへの影響を確認した

## Dependencies

<!-- 追加dependencyがある場合、必要性と代替案を書く。 -->

- Added dependencies: none
- Rationale:
- Alternatives considered:

## Risks

<!-- regressions、安全性、互換性、未検証事項。 -->

- 

## Handoff

### Completed

- 

### Remaining

- Next task ID:

### Blockers

- none
