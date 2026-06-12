# lls Implementation Status

この文書は、AIエージェント間で現在地を共有するための進捗台帳である。  
各実装PRは、開始時と完了時に該当taskを更新する。

詳細なtask定義は [`implementation-plan.md`](implementation-plan.md) を参照する。

---

## 1. Current state

- **Current milestone:** M0 — Development foundation
- **Next task:** M0-01 — Create library/application skeleton
- **MVP implementation:** not started
- **Blocking issues:** none
- **Last updated:** 2026-06-12

現状のproduction codeは`src/main.rs`の`Hello, world!`のみ。  
仕様、config schema、architecture、implementation planは作成済み。

---

## 2. Task board

| Task | Status | PR | Notes |
|---|---|---|---|
| M0-01 | not_started | - | library/application skeleton |
| M0-02 | not_started | - | domain enums and output models |
| M0-03 | not_started | - | test support and fixtures |
| M1-01 | not_started | - | CLI commands and conflicts |
| M1-02 | not_started | - | config structs and defaults |
| M1-03 | not_started | - | config discovery |
| M1-04 | not_started | - | JSON Schema validation |
| M1-05 | not_started | - | semantic validation and glob |
| M2-01 | not_started | - | target/project root |
| M2-02 | not_started | - | scanner depth and metadata |
| M2-03 | not_started | - | ignore prune |
| M2-04 | not_started | - | fixed-path project probe |
| M3-01 | not_started | - | independent attributes |
| M3-02 | not_started | - | role precedence |
| M3-03 | not_started | - | priority precedence |
| M3-04 | not_started | - | sorting |
| M4-01 | not_started | - | recommendations |
| M4-02 | not_started | - | summary and warnings |
| M4-03 | not_started | - | compact JSON |
| M5-01 | not_started | - | human mode |
| M5-02 | not_started | - | long listing |
| M6-01 | not_started | - | setup-required behavior |
| M6-02 | not_started | - | built-in setup proposal |
| M6-03 | not_started | - | atomic config writer |
| M7-01 | not_started | - | Codex executable/login adapter |
| M7-02 | not_started | - | safe Codex exec request |
| M7-03 | not_started | - | Codex proposal validation |
| M7-04 | not_started | - | integrated Codex setup |
| M8-01 | not_started | - | E2E fixture matrix |
| M8-02 | not_started | - | cross-platform behavior |
| M8-03 | not_started | - | docs and examples |
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

### 2026-06-12 — Planning baseline

- MVP仕様の矛盾解消済み
- agent contract作成
- architecture作成
- implementation plan作成
- 実装は未着手
- 次はM0-01
