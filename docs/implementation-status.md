# lls Implementation Status

この文書は、AIエージェント間で現在地を共有するための進捗台帳である。  
各実装PRは、開始時と完了時に該当taskを更新する。

詳細なtask定義は [`implementation-plan.md`](implementation-plan.md) を参照する。

---

## 1. Current state

- **Current milestone:** M0–M8 — Foundation through hardening
- **Next task:** M8-02 — Cross-platform behavior
- **MVP implementation:** substantially complete (M0-M7 done, M8 hardening in progress)
- **Blocking issues:** none
- **Last updated:** 2026-06-12

実装状況:

- M0: Library skeleton、domain models 完了
- M1: CLI parsing、config structs、discovery、validation、glob 完了
- M2: Target/project root、scanner、prune、project probe 完了
- M3: Attributes、role、priority、sorting 完了
- M4: Recommendations、summary、compact JSON 完了
- M5: Human output、long listing 完了
- M6: Setup-required、built-in proposal、atomic writer 完了
- M7: Codex command builder、process runner、proposal validation、integrated setup 完了
- M8: E2E fixture matrix (23 tests)、README docs 完了

---

## 2. Task board

| Task | Status | PR | Notes |
|---|---|---|---|
| M0-01 | done | #9 | library/application skeleton |
| M0-02 | done | #9 | domain enums and output models |
| M0-03 | done | #9 | test support and fixtures |
| M1-01 | done | #9 | CLI commands and conflicts |
| M1-02 | done | #9 | config structs and defaults |
| M1-03 | done | #9 | config discovery |
| M1-04 | done | #9 | JSON Schema validation |
| M1-05 | done | #9 | semantic validation and glob |
| M2-01 | done | #9 | target/project root |
| M2-02 | done | #9 | scanner depth and metadata |
| M2-03 | done | #9 | ignore prune |
| M2-04 | done | #9 | fixed-path project probe |
| M3-01 | done | #9 | independent attributes |
| M3-02 | done | #9 | role precedence |
| M3-03 | done | #9 | priority precedence |
| M3-04 | done | #9 | sorting |
| M4-01 | done | #9 | recommendations |
| M4-02 | done | #9 | summary and warnings |
| M4-03 | done | #9 | compact JSON |
| M5-01 | done | #9 | human mode |
| M5-02 | done | #9 | long listing |
| M6-01 | done | #9 | setup-required behavior |
| M6-02 | done | #9 | built-in setup proposal |
| M6-03 | done | #9 | atomic config writer |
| M7-01 | done | #9 | Codex executable/login adapter |
| M7-02 | done | #9 | safe Codex exec request |
| M7-03 | done | #9 | Codex proposal validation |
| M7-04 | done | #9 | integrated Codex setup |
| M8-01 | done | #9 | E2E fixture matrix (23 tests) |
| M8-02 | not_started | - | cross-platform behavior |
| M8-03 | done | #9 | docs and examples (README) |
| M8-04 | not_started | - | coverage/release gate |

---

## 3. How to update

### Beginning a task

- taskを`in_progress`へ変更
- PR欄へbranch名またはdraft PR番号を入れる
- Current stateのNext taskを更新する
- blockerがあれば明記する

### Completing a task

- taskを`done`へ変更
- PR番号を入れる
- 次のunblocked taskをNext taskへ設定する
- 新しいdesign decisionがあればADRを追加する
- Last updatedを更新する

### Blocking a task

- taskを`blocked`へ変更
- Notesへ依存taskまたは未決定仕様を書く
- 曖昧なまま別taskで代替実装しない

---

## 4. Active decisions

| Decision | Source |
|---|---|
| Runtime listing is deterministic and local | `spec.md`, ADR-0001 |
| Codex is setup-only | `spec.md`, ADR-0001 |
| Codex auth is ChatGPT-only | `spec.md`, `setup-plan.md` |
| Config is `.lls/config.json` | `spec.md`, `config.schema.json` |
| Invalid config is fatal | `spec.md` |
| Scan depth and project probe are separate | `spec.md`, `architecture.md` |
| Monorepo is post-MVP | `feature-spec.md` |

---

## 5. Known risks

| Risk | Mitigation |
|---|---|
| AIがscopeを広げる | task ID単位のPRとNon-goals記載 |
| 文書と実装が乖離する | PRごとにstatus/spec同期を確認 |
| classification precedenceが分散する | classifier/priority moduleへ集約 |
| OSごとに順序が変わる | normalized pathと明示sort key |
| Codex認証情報を誤処理する | process境界へ委譲しcredential fileを読まない |
| config生成中の破損 | validation後のatomic write |
| snapshot更新で契約破壊を隠す | field assertionとspec review |

---

## 6. Handoff log

新しいentryは上へ追加する。

### 2026-06-12 — M5-M8 hardening (PR #9)

- Fixed CI: moved E2E tests from `src/main.rs` to `tests/e2e.rs` using `CARGO_BIN_EXE_lls`
- Created `setup/codex.rs` module: Codex proposal validation + integrated Codex setup flow
- Updated `run_setup` to support both `--without-codex` and Codex-assisted paths
- Created `tests/common/mod.rs` with fixture creation helpers (Rust CLI/lib, Node, Python, Go, polyglot, sensitive, unknown, invalid config)
- Created 23 fixture-based E2E tests covering all project types, recommendations, depth, sort, human/long output, JSON format, config handling
- Updated README with installation, usage, output example, exit codes
- Updated implementation-status.md with completed tasks
- 150 tests passing (124 unit + 3 integration E2E + 23 fixture E2E)
- fmt, clippy, release build all clean

### 2026-06-12 — M0-M4 foundation implementation (PR #1)

- Library skeleton、domain models、test support
- CLI parsing with clap
- Config structs, schema validation, glob patterns, discovery
- Scanner with depth control, prune, project root resolution
- Classifier with attributes and role precedence
- Priority assignment with built-in rules and safety invariants
- Sorting (canonical, name, mtime, size)
- Recommendations, summary, compact JSON output
- Setup flow (built-in proposal, safety check, atomic write)
- Codex process adapter (command builder + process runner abstraction)
- Human and long listing output formatters
- 117 tests passing, clippy clean, fmt clean
- Binary builds and runs with `--no-config`

### 2026-06-12 — Planning baseline

- MVP仕様の矛盾解消済み
- agent contract作成
- architecture作成
- implementation plan作成
- 実装は未着手
- 次はM0-01
