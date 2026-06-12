# AGENTS.md

このファイルは、`lls` を実装・修正するAIエージェント向けの作業契約である。  
単なるコーディング規約ではなく、仕様の優先順位、変更単位、設計境界、テスト条件、引き継ぎ方法を定義する。

このリポジトリで作業するエージェントは、作業開始前に本書を読み、完了前に本書のチェックリストを満たすこと。

---

## 1. 最初に読むもの

次の順に読む。

1. [`AGENTS.md`](AGENTS.md)
2. [`.github/assets/spec.md`](.github/assets/spec.md)
3. [`.github/assets/config.schema.json`](.github/assets/config.schema.json)
4. [`docs/architecture.md`](docs/architecture.md)
5. [`docs/implementation-plan.md`](docs/implementation-plan.md)
6. [`docs/implementation-status.md`](docs/implementation-status.md)
7. [`docs/setup-plan.md`](docs/setup-plan.md)
8. [`.github/assets/feature-spec.md`](.github/assets/feature-spec.md)
9. [`README.md`](README.md)

### 文書の優先順位

矛盾がある場合は次を優先する。

1. 現在のユーザー要求とPRの明示的なスコープ
2. `.github/assets/spec.md`
3. `.github/assets/config.schema.json`
4. `docs/architecture.md`
5. `docs/setup-plan.md`
6. `docs/implementation-plan.md`
7. `docs/implementation-status.md`
8. `.github/assets/feature-spec.md`
9. `README.md`

仕様と実装が矛盾している場合、実装に合わせて勝手に仕様を読み替えない。  
仕様の修正が必要なら、実装変更と同じPRで正本を更新し、理由をPR本文に書く。

---

## 2. 現在の開発方針

`lls` はRust製CLIとして、次の順で完成させる。

1. 決定的なローカルコア
2. 安定したJSON契約
3. 設定ファイルとエラー契約
4. 人間向け表示
5. `lls setup --without-codex`
6. Codex CLIを使うsetup

実装順序は [`docs/implementation-plan.md`](docs/implementation-plan.md) に従う。  
現在の進捗は [`docs/implementation-status.md`](docs/implementation-status.md) を参照する。

未着手の将来機能を先回りして実装しない。

---

## 3. 絶対に崩してはいけない不変条件

以下はMVPのhard invariantである。

### 通常実行

- 通常の`lls`実行ではCodexやネットワークを呼ばない
- 同じfilesystem state、設定、CLI引数なら同じ順序と同じ`reason_code`を返す
- 設定がない通常実行は自動生成せず、setupへ誘導して終了コード`5`
- `--no-config`指定時だけ組み込み既定値で続行する
- 不正な設定へ黙ってフォールバックせず、終了コード`7`
- output modeを複数指定した場合は暗黙に優先せず、CLI引数エラー

### Codex

- Codexは`lls setup`の設定案生成にだけ使う
- 認証はCodex CLIのSign in with ChatGPTだけ
- OpenAI Platform API keyは扱わない
- `OPENAI_API_KEY`や`CODEX_API_KEY`を読まない
- `~/.codex/auth.json`を直接読まない、コピーしない、表示しない
- Codex subprocessはread-only sandboxかつephemeralで実行する
- Codex出力はJSON Schema検証後にのみ採用する
- Codex失敗時にpartial configを残さない

### 分類と安全性

- `role`は意味上の役割を表す
- `generated`と`sensitive`はroleとは独立したboolean属性
- `secret_candidate`と`generated`をroleへ戻さない
- `sensitive: true`は常にrecommendationから除外する
- binaryとsymlinkはrecommendationから除外する
- `.git/`は常に`priority: ignore`
- 秘密情報候補の内容を分類目的で読まない

### 走査とproject probe

- scan depthとproject type probeを混同しない
- project probeは固定パスの存在確認だけを行う
- `--depth 1`でも`src/main.rs`を根拠に`rust_cli`を判定できる
- ignore directory自身はentryへ出し、既定では子孫をpruneする
- symlinkを再帰的に辿らない
- MVPではmonorepo判定を行わない

### 出力

- default JSONは1行のcompact JSON
- stdout末尾には改行を1つ付ける
- optional fieldは取得不能なら省略し、`null`を出さない
- fatal error時にpartial JSONをstdoutへ出さない
- diagnostics、進捗、確認質問はstderrへ出す

---

## 4. 作業単位

### 1 PR = 1つの検証可能な目的

良い例:

- domain modelとserde定義を追加する
- config discoveryとvalidationを追加する
- scannerのdepth/pruneを実装する
- canonical sortを実装する
- Codex process adapterを追加する

悪い例:

- CLI、scanner、分類、出力、setupを一度に実装する
- MVPと将来機能を同じPRに混ぜる
- リファクタリングと仕様変更を理由なく混ぜる

目安として、1 PRで変更する主要責務は1つ、多くても隣接する2つまでにする。

### 作業開始時

1. `docs/implementation-status.md`で次のunblocked taskを確認する
2. 関連する仕様節と受け入れ条件を列挙する
3. 変更予定ファイルを決める
4. 非目標を明記する
5. テストを先に決める

### 作業完了時

1. 実装とテストを完了する
2. `docs/implementation-status.md`を更新する
3. 新しい設計判断があればADRを追加する
4. READMEや仕様の同期漏れを確認する
5. PRテンプレートを埋める

---

## 5. アーキテクチャ規則

正式な構成は [`docs/architecture.md`](docs/architecture.md) を参照する。

### 依存方向

```txt
main / cli
    ↓
application orchestration
    ↓
domain rules and models
    ↑
adapters: filesystem / process / terminal
```

- domain modelはCLI、filesystem、processへ依存しない
- scannerはclassificationやJSON formattingを行わない
- classifierはfilesystem I/Oを行わない
- outputは分類判断を変更しない
- setupは通常実行のscan pipelineへ入り込まない
- Codex固有処理は`codex`境界へ隔離する

### 推奨モジュール

```txt
src/
├── main.rs
├── lib.rs
├── app.rs
├── cli.rs
├── model.rs
├── error.rs
├── config/
├── scanner/
├── classifier/
├── project_probe/
├── priority/
├── recommendation/
├── sorting/
├── output/
├── setup/
└── codex/
```

実装途中でも、`main.rs`へ業務ロジックを置かない。

### 抽象化の基準

次の境界はtest doubleへ差し替えられるようにする。

- filesystem metadata/scan
- clockが必要な場合の時刻取得
- Codex subprocess
- terminal/TTY判定
- config writer

ただし、将来利用を想像した抽象化を先行させない。  
同じ責務の実装が2つ必要になった時点、またはテスト境界として必要な時点でtraitを導入する。

---

## 6. Rustコーディング規則

- edition 2024を維持する
- `cargo fmt`準拠
- `cargo clippy --all-targets --all-features -- -D warnings`を通す
- production codeで`unwrap()`、`expect()`、`panic!()`を原則使用しない
- `Result`と明示的なerror typeを使う
- `pub`はcrate内部で必要な最小範囲にする
- boolean引数の多用を避け、意味のある型やoptions structを使う
- pathは内部では`Path` / `PathBuf`で扱い、出力境界で正規化する
- shell command文字列を組み立てず、`std::process::Command`へ引数を個別に渡す
- error messageへcredential、file content、Codexの生出力を埋め込まない
- 大きな関数は処理手順ではなく意味で分割する
- コメントは「何をしているか」ではなく「なぜそうするか」を書く

### 依存crate

依存追加にはPR本文で理由を書く。

原則:

- CLI: `clap`
- serialization: `serde`, `serde_json`
- error: `thiserror`
- JSON Schema:成熟したvalidator crateを1つだけ
- glob:仕様の`*`, `?`, `**`を再現できるcrateを1つだけ
- temp/fixture: test用途を優先

禁止または要承認:

- async runtime
- HTTP client
- OpenAI SDK
- telemetry SDK
- global logger初期化
- filesystem watcher
- database

MVP coreにネットワーク依存を追加しない。

---

## 7. エラー処理

終了コードはspecの定義を唯一の正本とする。

- `0`: success
- `1`: CLI argument error
- `2`: target missing
- `3`: permission denied
- `4`: unexpected runtime error
- `5`: setup required
- `6`: Codex/setup failure
- `7`: invalid config

### fatalとwarning

fatal:

- target path自体が存在しない
- target path自体を読めない
- CLI引数不正
- config不正
- setupの保存失敗

warning:

- 一部entryのmetadata取得失敗
- broken symlink
- non-UTF-8 path skip
- 複数project type signal
- sensitive candidate検出

warningをerrorへ昇格させたり、errorをwarningへ降格させる場合は仕様変更として扱う。

---

## 8. テスト規則

### テスト層

- pure rule: 各module内のunit test
- module連携: `tests/`のintegration test
- CLI契約: binaryを起動するE2E test
- filesystem case: `tests/fixtures/`
- Codex: fake process adapter

### 必須観点

変更した責務に応じて、最低限次を確認する。

- normal case
- empty input
- boundary value
- invalid input
- deterministic ordering
- OS差分を吸収したpath
- partial failure
- sensitive dataが出力されないこと

### snapshot/golden test

JSON契約には意味のあるfield assertionを優先する。  
巨大なsnapshotだけで正しさを判断しない。

利用する場合:

- schema versionを明示する
- fixtureの目的を書く
- 意図しない大量更新を禁止する

### 実行コマンド

PR作成前に次を実行する。

```sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
cargo build --release
```

ドキュメントのみの変更でも、CI定義やCargo関連ファイルを変更した場合は実行する。

---

## 9. 仕様変更の扱い

実装中に仕様の穴を見つけても、勝手に都合のよい挙動を選ばない。

次の順で対応する。

1. 既存のspec、schema、ADRから解釈できるか確認
2. 解釈できない場合は最小の安全な案を決める
3. specを先に更新する
4. acceptance criteriaとtestを追加する
5. 実装する
6. ADRが必要なら記録する

次はADR対象とする。

- module境界の変更
- public JSON schemaの変更
- exit codeの変更
- dependency追加のうち設計へ影響するもの
- Codex境界や認証方針の変更
- deterministic behaviorへ影響する変更

ADRの運用は [`docs/adr/README.md`](docs/adr/README.md) を参照する。

---

## 10. 禁止事項

- main branchへ直接実装する
- 未確認の仕様を実装で既成事実化する
- `feature-spec.md`のcandidateをMVPへ混ぜる
- API key認証を追加する
- 通常実行からCodexを呼ぶ
- secret candidateの内容を読む
- invalid configを無視する
- platform依存順序をそのままJSONへ出す
- random、現在時刻、hash map iteration順をcanonical orderへ使う
- shell経由でCodexを起動する
- `target/`、coverage生成物、一時configをコミットする
- テストを削ってCIを通す
- warning抑制だけでclippy failureを隠す
- 無関係なformat変更を大量に混ぜる
- 既存変更を勝手に巻き戻す

---

## 11. Definition of Done

変更は次をすべて満たして完了とする。

- [ ] PRの目的が1つに絞られている
- [ ] 関連するspec節を満たしている
- [ ] hard invariantを壊していない
- [ ] 実装に対応するtestがある
- [ ] error pathをtestしている
- [ ] format、clippy、test、release buildが通る
- [ ] `docs/implementation-status.md`を更新した
- [ ] 仕様変更時にspec/schemaを更新した
- [ ] 設計判断時にADRを追加した
- [ ] READMEまたは利用者向け説明の同期を確認した
- [ ] PR本文に非目標、test evidence、riskを書いた

---

## 12. 引き継ぎフォーマット

作業を途中で止める場合、PR本文またはコメントへ次を書く。

```md
## Handoff

### Completed
- 完了した内容

### Changed files
- `path`: 変更理由

### Decisions
- 決めたことと根拠

### Verification
- 実行したコマンドと結果

### Remaining
- 次に行う具体的なtask ID

### Risks / blockers
- 未解決事項
```

「だいたい完成」「あとは微調整」のような曖昧な引き継ぎは禁止する。

---

## 13. 作業停止条件

次の場合は、それ以上の実装を広げず、仕様または状態文書を更新して停止する。

- specとschemaが矛盾している
- security invariantを守れない
- test可能な境界を作れない
- taskの完了に別phaseの機能が必須
- public JSON互換性を壊す必要がある
- credentialやsecretを扱う必要が生じた
- 依存追加なしでは進めないが、その依存が設計へ影響する

停止は失敗ではない。曖昧なまま実装を進める方が失敗である。
