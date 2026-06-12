# lls Architecture

この文書は、`lls` MVPの実装構造とmodule間の責務を定義する。  
仕様上の振る舞いは [`.github/assets/spec.md`](../.github/assets/spec.md) を正本とし、本書は「どの責務をどこへ実装するか」を固定する。

---

## 1. 設計目標

- 通常実行をローカルかつ決定的にする
- filesystem、process、terminalなどの副作用を境界へ隔離する
- classification、priority、recommendationをpureに近づける
- JSON契約を表示実装から独立させる
- Codex障害が通常の一覧生成へ影響しないようにする
- module単位とCLI全体の両方をtestできるようにする

---

## 2. システム境界

`lls`には2つの独立したuse caseがある。

### Runtime listing

```txt
CLI args
  ↓
config resolution
  ↓
filesystem scan ─────→ project probe
  ↓                         ↓
raw entries            project type
  ↓
attributes / role / priority
  ↓
sort / recommendation
  ↓
JSON, human, or long output
```

- ネットワークを使用しない
- Codexを使用しない
- configとfilesystem stateが同じなら同じ結果を返す

### Setup

```txt
CLI args
  ↓
project root resolution
  ↓
Codex process adapter ── or ── built-in proposal
  ↓
config proposal
  ↓
schema validation
  ↓
safety validation
  ↓
user confirmation
  ↓
atomic config write
```

- Runtime listingとpipelineを共有しすぎない
- Codexは設定案を返すだけ
- 最終判断と書き込みは`lls`が行う

---

## 3. 推奨source tree

```txt
src/
├── main.rs
├── lib.rs
├── app.rs
├── cli.rs
├── model.rs
├── error.rs
├── config/
│   ├── mod.rs
│   ├── model.rs
│   ├── discovery.rs
│   ├── validation.rs
│   ├── defaults.rs
│   └── pattern.rs
├── scanner/
│   ├── mod.rs
│   ├── fs.rs
│   └── path.rs
├── classifier/
│   ├── mod.rs
│   ├── attributes.rs
│   └── role.rs
├── project_probe/
│   └── mod.rs
├── priority/
│   └── mod.rs
├── recommendation/
│   └── mod.rs
├── sorting/
│   └── mod.rs
├── output/
│   ├── mod.rs
│   ├── json.rs
│   ├── human.rs
│   └── long.rs
├── setup/
│   ├── mod.rs
│   ├── proposal.rs
│   ├── safety.rs
│   └── writer.rs
└── codex/
    ├── mod.rs
    ├── command.rs
    └── process.rs
```

初期段階では小さなmoduleを1ファイルに置いてよい。  
責務が増えた時だけsubmoduleへ分割する。空のdirectoryや将来用traitを先に作らない。

---

## 4. Module responsibilities

### `main`

責務:

- `cli`から引数を受け取る
- `app`を呼ぶ
- exit codeをprocessへ反映する

禁止:

- filesystem scan
- JSON組み立て
- classification
- Codex command組み立て

### `lib`

責務:

- crate内moduleの公開
- integration testから必要な最小APIの公開

binary-only logicを避け、主要処理はlibrary crateから呼べるようにする。

### `cli`

責務:

- subcommandとoptionのparse
- option間のconflict検証
- CLI固有のdefault適用
- parsed commandをapplication requestへ変換

CLIはfilesystemへ触れない。

候補型:

```rust
pub enum CommandRequest {
    List(ListRequest),
    Setup(SetupRequest),
}
```

### `app`

責務:

- use caseのorchestration
- runtime listingとsetupの入口
- errorをexit categoryへ変換

業務ルールそのものは下位moduleへ委譲する。

### `model`

責務:

- domain enum/struct
- serialized output model
- module間で共有するimmutable data

候補型:

```rust
EntryType
Role
Priority
RawEntry
Entry
ProjectType
ProjectTypeResult
Recommendation
Warning
Summary
OutputDocument
```

`Role`、`Priority`、`EntryType`をstringで持ち回らない。

### `config`

責務:

- project rootとconfig pathの解決
- JSON parse
- JSON Schema validation
- semantic validation
- built-in defaults
- glob pattern compile
- CLI overrideとのmerge

処理段階:

```txt
bytes
→ parsed config
→ schema-valid config
→ semantic-valid config
→ compiled effective config
```

parse前後の型を可能なら分け、未検証configをruntimeへ渡さない。

### `scanner`

責務:

- 指定depthでentryを列挙
- metadataを取得
- symlinkを識別
- ignore directoryをprune
- partial failureをwarningへ変換

scannerの出力は`RawEntry`とwarningであり、roleやpriorityを決定しない。

候補:

```rust
pub struct ScanResult {
    pub entries: Vec<RawEntry>,
    pub warnings: Vec<Warning>,
}
```

### `classifier`

責務:

- `generated`
- `sensitive`
- `text` / `binary`
- `role`
- `reason_code`

入力は`RawEntry`とcompiled rules。  
filesystemやconfig fileを直接読まない。

属性判定とrole判定は分ける。

### `project_probe`

責務:

- project rootから固定パスの存在を確認
- project type candidateを作る
- fixed precedenceを適用
- evidenceとwarningを返す

再帰scanを行わない。scannerのdepthを変更しない。

### `priority`

責務:

- ignore pattern
- user priority override
- built-in priority
- role default
- safety rule

をspecの順で適用する。

priority決定結果には安定した`reason_code`を付ける。

### `recommendation`

責務:

- eligible entryのfilter
- role/priorityに基づくrank
- action決定
- 最大5件へのtruncate

secret、binary、symlink、ignoreの除外はこのmoduleで再確認する。  
上流判定だけを信用しないdefense in depthとする。

### `sorting`

責務:

- canonical order
- name
- mtime
- size
- tie-break

sorting keyを1箇所に集める。  
output moduleが独自にsortしない。

### `output`

責務:

- `OutputDocument`を各表示形式へ変換
- compact JSON末尾改行
- human modeのTTY/color handling
- long listing formatting

禁止:

- roleやpriorityの再判定
- warningの追加判断
- filesystem access

### `setup`

責務:

- setup use case
- proposal validation
- confirmation
- atomic write
- cleanup

Codex process詳細は`codex`へ委譲する。

### `codex`

責務:

- executable discovery
- login状態確認のprocess実行
- `codex login`案内
- safeな`codex exec`引数生成
- timeout
- exit statusとlast message fileの取得

禁止:

- credential fileのread
- environment API keyの利用
- shell invocation
- config fileの最終write

---

## 5. Domain models

### Raw entry

filesystemから得た未分類情報。

```rust
pub struct RawEntry {
    pub name: OsString,
    pub absolute_path: PathBuf,
    pub relative_path: PathBuf,
    pub entry_type: EntryType,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<SystemTime>,
}
```

非UTF-8 pathはoutput modelへ変換する前にwarningとして除外する。

### Classified entry

```rust
pub struct Entry {
    pub name: String,
    pub path: String,
    pub entry_type: EntryType,
    pub role: Role,
    pub priority: Priority,
    pub reason_code: ReasonCode,
    pub reason: String,
    pub generated: bool,
    pub sensitive: bool,
    pub text: Option<bool>,
    pub binary: Option<bool>,
    pub size_bytes: Option<u64>,
    pub modified_at: Option<SystemTime>,
}
```

`modified_at`はinternal sort用で、MVP JSONへ必ず出すfieldではない。

### Reason code

`reason_code`は機械契約である。  
表示文言をtestの主根拠にしない。

候補:

```txt
known_project_overview
known_manifest
known_source_directory
known_test_directory
known_build_output
matched_role_override
matched_priority_override
matched_ignore_pattern
sensitive_name_pattern
generated_name_pattern
fallback_unknown
```

reason code追加はtestとspec例を同期する。

---

## 6. Dependency direction

許可:

```txt
cli -> model
app -> all use-case modules
config -> model, error
scanner -> model, error
classifier -> model, config compiled rules
project_probe -> model, error
priority -> model, config compiled rules
recommendation -> model
sorting -> model
output -> model, cli output options
setup -> config, codex, error
codex -> error
```

避ける:

```txt
model -> filesystem/process/cli
classifier -> scanner implementation
output -> classifier/priority
codex -> setup writer
scanner -> output
config -> cli parser
```

循環依存が生じた場合、共通型を`model`へ移すか、責務分割を見直す。

---

## 7. Configuration pipeline

```txt
ConfigLocation
  ↓
Raw JSON bytes
  ↓
serde_json::Value
  ↓ JSON Schema
ProjectConfig
  ↓ semantic checks
ValidatedConfig
  ↓ pattern compilation + CLI override
EffectiveConfig
```

### Schema validation

- `.github/assets/config.schema.json`をbuild時に`include_str!`してよい
- runtimeでrepository pathを必要としない
- unknown keyを拒否する

### Semantic validation

JSON Schemaだけで表せない次を検証する。

- glob syntax
- absolute path禁止
- `.git/` safety ruleへの反抗
- credentialらしいfield/value禁止
- duplicate/conflicting rule

### Merge

```txt
CLI > project config > built-in default
```

merge後の`EffectiveConfig`だけをruntime pipelineへ渡す。

---

## 8. Filesystem strategy

### Project root

- directory target: target自身から祖先へ
- file target: parentから祖先へ
- 最初の`.git`をroot
- なければtarget directoryまたはfile parent

### Scan

- directory iteration順を信用しない
- entriesを収集後、sorting moduleで必ず順序を決める
- permission errorは対象rootならfatal、子entryならwarning
- symlinkはmetadata取得に`symlink_metadata`を使う
- symlink targetをfollowしない

### Prune

ignore directoryを見つけたら:

1. directory entry自身を`RawEntry`へ入れる
2. child walkを停止する
3. `include_ignored`時だけchildをwalkする

---

## 9. Codex process boundary

process起動は必ずargument arrayで行う。

```rust
Command::new("codex")
    .arg("exec")
    .arg("--ephemeral")
    .arg("--sandbox")
    .arg("read-only")
    // ...
```

### Process abstraction

unit testで実Codexを起動しない。

候補interface:

```rust
pub trait ProcessRunner {
    fn run(&self, request: ProcessRequest) -> Result<ProcessResult, ProcessError>;
}
```

production implementationは`std::process::Command`、testはfakeを使う。

### Output handling

- stdout event streamをconfigとして解釈しない
- `--output-last-message`のtemporary fileだけをproposal候補にする
- file size上限を設ける
- UTF-8でない、空、巨大、schema不一致は失敗
- error messageへraw model outputを出しすぎない

---

## 10. Atomic config write

推奨手順:

1. `.lls/`を作る
2. 同一directoryへrandom temporary filenameでcreate-new
3. permissionを適切に設定
4. 完全なJSONを書く
5. flush
6. `sync_all`
7. 同一filesystem内でrename
8. directory syncが可能なら行う
9. failure時にtemporary fileを削除

既存configは`--force`なしで触らない。  
`--force`時も直接truncateせずatomic replaceする。

---

## 11. Error architecture

内部errorは原因を保持するが、user-facing messageと分ける。

候補:

```rust
pub enum AppError {
    Cli(CliError),
    TargetNotFound { path: PathBuf },
    PermissionDenied { path: PathBuf },
    SetupRequired { root: PathBuf },
    InvalidConfig(ConfigError),
    Codex(CodexError),
    Runtime(RuntimeError),
}
```

`AppError::exit_code()`を1箇所に置く。  
各moduleが任意の整数exit codeを返さない。

warningはerror型ではなくdomain dataとしてoutputへ渡す。

---

## 12. Testing architecture

### Unit

pure function:

- pattern match
- role precedence
- priority precedence
- recommendation eligibility
- sort keys
- summary count
- project type precedence

### Integration

- config discovery
- depth scan
- prune
- non-UTF-8 behavior（対応OSのみ）
- atomic write
- fake process setup

### CLI E2E

- stdout/stderr
- exit code
- compact JSON
- conflicting mode
- missing config
- `--no-config`
- setup decline/accept

fixtureは目的ごとに小さく保つ。

```txt
tests/fixtures/
├── rust_cli/
├── node_project/
├── sensitive/
├── ignored_tree/
├── partial_permission/
└── unknown/
```

---

## 13. Architecture change policy

次の変更はADRを必要とする。

- module責務の大幅な移動
- runtime listingからprocess/networkを呼ぶ変更
- serialized public model変更
- config format変更
- exit code変更
- Codex credential handling変更
- syncからasyncへの変更

小さなfile分割やprivate helper追加はADR不要。
