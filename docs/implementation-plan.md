# lls MVP Implementation Plan

この文書は、`lls` MVPをAIエージェントが小さなPRへ分割して実装するためのロードマップである。  
仕様の正本は [`.github/assets/spec.md`](../.github/assets/spec.md)、設計境界は [`architecture.md`](architecture.md) とする。

---

## 1. 進め方

- task ID単位でPRを作る
- 1 PRで複数taskを扱う場合も、隣接し強く依存するものに限る
- 各PRは単独でbuild/test可能にする
- 後phaseの機能を前倒ししない
- 完了時に [`implementation-status.md`](implementation-status.md) を更新する
- public behaviorを追加するPRではCLI E2E testを追加する

### Task status

```txt
not_started
in_progress
blocked
done
```

### 優先順

```txt
P0: 後続すべての基盤
P1: MVP必須
P2: MVP品質向上
```

---

## 2. Milestone overview

| Milestone | 目的 | 完了条件 |
|---|---|---|
| M0 | 開発基盤 | library構成、CI quality gates、基本error/model |
| M1 | CLIとconfig | 引数、config discovery、schema/semantic validation |
| M2 | filesystem理解 | project root、scanner、project probe |
| M3 |意味付け | attributes、role、priority、sort |
| M4 | 機械向け出力 | recommendation、summary、compact JSON |
| M5 | 人間向け出力 | `--human`, `-l` |
| M6 | local setup | setup誘導、`--without-codex`、atomic write |
| M7 | Codex setup | ChatGPT-only login、read-only process、proposal validation |
| M8 | hardening | E2E、cross-platform、coverage、README |

---

# M0: Development foundation

## M0-01 — Create library/application skeleton

**Priority:** P0  
**Depends on:** none

### Scope

- `src/lib.rs`を追加
- `src/main.rs`をthin entrypointにする
- `src/app.rs`, `src/model.rs`, `src/error.rs`を追加
- module宣言だけで、future moduleの空fileを大量に作らない
- `AppError`とexit code mappingの骨格を作る

### Tests

- exit code mapping unit test
- binaryがbuildできること

### Acceptance

- `main.rs`に業務ロジックがない
- production codeに`unwrap`がない
- format/clippy/test/buildが通る

### Non-goals

- CLI option実装
- filesystem scan
- JSON output

---

## M0-02 — Define domain enums and output models

**Priority:** P0  
**Depends on:** M0-01

### Scope

- `EntryType`
- `Role`
- `Priority`
- `ProjectType`
- `Warning`
- `Recommendation`
- `Summary`
- `OutputDocument`
- serde representation

### Rules

- enum値はspecと完全一致
- `generated` / `sensitive`はboolean
- optional fieldは`skip_serializing_if = "Option::is_none"`
- unknown enum値を暗黙に受理しない

### Tests

- enum serialization
- optional field omit
- top-level required field serialization

### Acceptance

- example documentをserializeできる
- JSONへ`null`が出ない

---

## M0-03 — Add test support and fixture conventions

**Priority:** P0  
**Depends on:** M0-01

### Scope

- `tests/common/`の方針確立
- temporary directory helper
- binary E2E helper
- fixture naming規則
- OS依存testのguard方法

### Acceptance

- 1つのsmoke E2E testがbinaryを起動できる
- test helperがproduction codeへ漏れない

---

# M1: CLI and configuration

## M1-01 — Parse CLI commands and conflicts

**Priority:** P0  
**Depends on:** M0-01

### Scope

- `lls [path]`
- `--json`
- `--human` / `-H`
- `-l`
- `--depth`
- `--sort`
- `--config`
- `--no-config`
- `lls setup`
- setup flags: `--force`, `--yes`, `--without-codex`

### Rules

- output mode複数指定はexit `1`
- `--config`と`--no-config`はconflict
- depthは`0..=8`
- errorはstderr

### Tests

- valid parse table
- conflict table
- depth boundary: `0`, `8`, `9`
- default path `.`

### Acceptance

- CLI parse結果がdomain requestへ変換される
- filesystemへ触れない

---

## M1-02 — Define config structs and built-in defaults

**Priority:** P0  
**Depends on:** M0-02

### Scope

- schemaと対応するRust struct
- built-in default
- effective config
- CLI override merge

### Built-in defaults

- output: json
- depth: 1
- include hidden: true
- include ignored: false
- long sort: priority
- Codex auth method: chatgpt

### Tests

- default snapshotではなくfield assertion
- CLI > config > default
- enum parse

---

## M1-03 — Implement config discovery

**Priority:** P0  
**Depends on:** M1-01, M1-02, M2-01

### Scope

解決順:

1. `--config`
2. `<project_root>/.lls/config.json`
3. `--no-config`
4. setup required

### Tests

- explicit path wins
- root config discovery
- no ancestor config merge
- `--no-config`
- missing config -> exit `5`

### Acceptance

- missing configでfileを作らない
- stdoutは空
- setup guidanceはstderr

---

## M1-04 — JSON Schema validation

**Priority:** P0  
**Depends on:** M1-02

### Scope

- schemaをbinaryへ埋め込む
- raw JSON parse
- schema validation
- unknown key拒否
- invalid config -> exit `7`

### Tests

- valid fixture
- missing required field
- unknown field
- invalid enum
- depth `9`
- auth methodがchatgpt以外

### Acceptance

- invalid configへfallbackしない
- error messageにsecret valueを出さない

---

## M1-05 — Semantic config validation and glob compilation

**Priority:** P0  
**Depends on:** M1-04

### Scope

- `*`, `?`, `**`
- leading `/`
- trailing `/`
- `/` separator normalization
- invalid glob拒否
- absolute path拒否
- safety rule検証
- rule order保持

### Tests

- pattern table
- path separator normalization
- first-match order
- duplicate/conflicting rule
- `.git/` safety violation

### Acceptance

- runtimeはcompiled valid configだけを受け取る

---

# M2: Filesystem and project detection

## M2-01 — Resolve target and project root

**Priority:** P0  
**Depends on:** M0-01

### Scope

- file target
- directory target
- ancestor `.git` discovery
- no `.git` fallback
- symlinkを辿らない

### Tests

- nested directory in repo
- file target
- no git root
- missing target -> `2`
- unreadable target -> `3` where testable

---

## M2-02 — Implement scanner depth and metadata

**Priority:** P0  
**Depends on:** M0-02, M2-01

### Scope

- depth `0..=8`
- direct child traversal
- type, size, mtime
- symlink metadata
- child error -> warning
- deterministic raw path normalization準備

### Tests

- depth `0`, `1`, `2`
- file target
- symlink
- partial metadata failure
- empty directory

### Acceptance

- scannerはrole/priorityを決定しない
- directory iteration順を公開結果へ使わない

---

## M2-03 — Implement ignore directory prune

**Priority:** P1  
**Depends on:** M1-05, M2-02

### Scope

- built-in ignore directory
- config ignore pattern
- directory自身はentryへ含める
- `include_ignored: false`でprune
- `include_ignored: true`でtraverse

### Tests

- `target/`
- `.git/`
- nested ignored directory
- user ignore pattern
- include ignored

---

## M2-04 — Implement fixed-path project probe

**Priority:** P0  
**Depends on:** M2-01, M0-02

### Scope

固定path:

```txt
Cargo.toml
src/main.rs
src/lib.rs
package.json
pyproject.toml
setup.py
go.mod
```

- candidate生成
- fixed precedence
- evidence
- multiple type warning

### Tests

- Rust CLI
- Rust library
- Node
- Python
- Go
- unknown
- polyglot
- depth `1`とは独立

### Acceptance

- recursive scanを行わない
- monorepoを返さない

---

# M3: Classification and ordering

## M3-01 — Detect independent attributes

**Priority:** P0  
**Depends on:** M0-02, M1-05

### Scope

- `generated`
- `sensitive`
- `text`
- `binary`
- stable reason code

### Tests

- `.env`
- `*.pem`
- generated source
- build output
- lockfile generated=true
- binary extension

### Acceptance

- attributeがroleを上書きしない
- secret contentを読まない

---

## M3-02 — Assign role with precedence

**Priority:** P0  
**Depends on:** M3-01

### Precedence

1. user role override
2. exact relative path
3. exact basename
4. directory pattern
5. extension pattern
6. unknown

### Tests

- precedence table
- first matching override
- `.env` -> role config + sensitive true
- generated source -> source_code + generated true

---

## M3-03 — Assign priority with precedence

**Priority:** P0  
**Depends on:** M3-02

### Precedence

1. ignore pattern
2. user priority override
3. built-in priority
4. role default
5. low

### Tests

- README critical
- manifest critical
- src high
- tests medium
- lockfile medium
- build output ignore
- ignore vs priority override
- `.git/` safety rule

---

## M3-04 — Implement sorting

**Priority:** P0  
**Depends on:** M3-03

### Scope

- canonical
- name
- mtime
- size
- all tie-break rules
- stable sort

### Tests

- priority rank
- role rank
- path byte order
- equal mtime
- missing mtime
- directory/missing size last
- repeated run equality

---

# M4: Recommendation and JSON output

## M4-01 — Generate recommendations

**Priority:** P0  
**Depends on:** M3-03, M3-04

### Scope

- eligibility filter
- action `read` / `inspect`
- project overview first
- manifest before source
- max 5
- reason code

### Tests

- sensitive excluded
- binary excluded
- symlink excluded
- ignore excluded
- max 5
- deterministic order

---

## M4-02 — Build summary and warnings

**Priority:** P0  
**Depends on:** M2-02, M3-03

### Scope

- total entries
- shown entries
- important entries
- ignored entries
- warning merge

### Tests

- pruned child not counted
- warning order deterministic
- sensitive warning
- non-UTF-8 warning where testable

---

## M4-03 — Emit compact JSON

**Priority:** P0  
**Depends on:** M4-01, M4-02

### Scope

- `OutputDocument`
- one-line JSON
- final newline
- stdout only
- fatal時partial outputなし

### E2E

- Rust CLI fixture
- Node fixture
- unknown fixture
- missing config
- `--no-config`

### Acceptance

- MVP default use caseがend-to-endで動く

---

# M5: Human output

## M5-01 — Human mode

**Priority:** P1  
**Depends on:** M4-03

### Scope

- project type
- next steps
- top entries
- TTY color
- non-TTY plain text

### Tests

- plain output
- color abstraction
- no JSON mixed output

---

## M5-02 — Minimal long listing

**Priority:** P1  
**Depends on:** M3-04

### Scope

- priority
- role
- type
- human-readable size
- path
- generated/sensitive marker
- sort options

### Non-goals

- README tagline
- PDF title
- recent marker
- ignore file parsing

### Tests

- each sort
- unknown size
- directory
- marker display

---

# M6: Local setup

## M6-01 — Setup-required behavior

**Priority:** P0  
**Depends on:** M1-01, M1-03

### Scope

- missing config guidance
- stderr
- exit `5`
- no write

### Tests

- config remains absent
- stdout empty
- exact actionable command included

---

## M6-02 — Built-in config proposal

**Priority:** P1  
**Depends on:** M1-02, M1-04

### Scope

- `lls setup --without-codex`
- deterministic proposal
- schema validation
- human preview
- `--yes`

### Tests

- accept
- decline
- force conflict
- generated config round-trip

---

## M6-03 — Atomic config writer

**Priority:** P0  
**Depends on:** M6-02

### Scope

- `.lls/` create
- temporary file
- flush/sync
- atomic rename
- cleanup
- `--force`

### Tests

- new write
- existing no force
- replace with force
- simulated failure leaves original
- temporary cleanup

---

# M7: Codex-assisted setup

## M7-01 — Codex executable and login status adapter

**Priority:** P1  
**Depends on:** M0-01

### Scope

- process runner abstraction
- executable missing
- login status
- ChatGPT-only policy
- device auth guidance

### Tests

- fake executable missing
- logged in
- logged out
- API key env is ignored

### Acceptance

- credential fileを直接読まない

---

## M7-02 — Build safe Codex exec request

**Priority:** P0  
**Depends on:** M7-01, M1-04

### Required args

- `exec`
- `--ephemeral`
- `--sandbox read-only`
- `--ignore-user-config`
- `--ignore-rules`
- `--output-schema`
- `--output-last-message`
- `--cd`

### Tests

- argument vector exactness
- no shell
- no writable/add-dir/bypass flags
- prompt version

---

## M7-03 — Validate Codex proposal

**Priority:** P0  
**Depends on:** M7-02, M1-05

### Scope

- timeout
- exit status
- output size limit
- UTF-8
- JSON parse
- schema validation
- semantic/safety validation
- temp cleanup

### Tests

- success
- timeout
- non-zero
- empty output
- malformed JSON
- schema invalid
- unsafe override
- oversized output

---

## M7-04 — Integrate Codex setup flow

**Priority:** P1  
**Depends on:** M6-03, M7-03

### Scope

- proposal preview
- confirmation
- `--yes`
- atomic write
- exit `6` mapping
- stderr discipline

### Acceptance

- normal listing pathにCodex依存がない
- failure時にconfigを残さない

---

# M8: Hardening and release readiness

## M8-01 — Complete E2E fixture matrix

**Priority:** P1  
**Depends on:** M4-03, M5-02, M7-04

Cases:

- Rust CLI
- Rust library
- Node
- Python
- Go
- unknown
- polyglot
- sensitive
- ignored tree
- broken symlink
- invalid config
- missing config

---

## M8-02 — Cross-platform behavior

**Priority:** P1  
**Depends on:** M8-01

### Scope

- path separator
- case sensitivity expectation
- Windows symlink test guard
- mtime availability
- permission test guard
- atomic rename differences

---

## M8-03 — Documentation and examples

**Priority:** P1  
**Depends on:** M8-01

### Scope

- README installation
- usage examples
- JSON example
- setup example
- security notes
- exit codes
- current status更新

---

## M8-04 — Coverage and release gate

**Priority:** P2  
**Depends on:** M8-01

### Scope

- coverage blind spot確認
- critical precedence/safety branchを重点化
- release build
- version behavior
- changelog/release preparation

Coverage percentageだけを目標にしない。  
security invariantとerror pathがtestされていることを重視する。

---

## 3. Recommended first PR sequence

最初の実装PRは次の順が安全である。

1. M0-01 library/application skeleton
2. M0-02 domain models
3. M1-01 CLI parsing
4. M2-01 project root
5. M1-02 config model/defaults
6. M1-04 schema validation
7. M1-05 semantic validation/glob
8. M1-03 config discovery
9. M2-02 scanner
10. M2-04 project probe
11. M2-03 prune
12. M3-01〜M3-04 classification/order
13. M4-01〜M4-03 default JSON MVP
14. M5 human output
15. M6 local setup
16. M7 Codex setup
17. M8 hardening

M1-03はproject rootへ依存するため、番号順ではなく上記sequenceで進める。

---

## 4. MVP completion checklist

- [ ] default compact JSON
- [ ] `--no-config`
- [ ] setup required behavior
- [ ] config schema/semantic validation
- [ ] depth scan
- [ ] ignore prune
- [ ] fixed-path project probe
- [ ] independent attributes and role
- [ ] priority precedence
- [ ] canonical sorting
- [ ] recommendation
- [ ] warning and summary
- [ ] human output
- [ ] long listing
- [ ] local setup
- [ ] Codex-assisted setup
- [ ] ChatGPT-only auth boundary
- [ ] atomic config write
- [ ] cross-platform CI
- [ ] README usage
