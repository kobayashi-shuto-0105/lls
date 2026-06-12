# lls 仕様書

このドキュメントは、`lls` の MVP として実装する仕様を定義する。  
将来機能は [`feature-spec.md`](feature-spec.md) に分離し、本書と矛盾する場合は本書を優先する。

`lls` は、ディレクトリやリポジトリを走査し、LLM や人間が「次にどのファイルを見るべきか」を判断しやすいように、各エントリへ役割・重要度・理由を付けて出力する CLI ツールである。

---

## 0. MVP の決定事項

MVP では、次を確定事項とする。

- デフォルト出力は compact JSON
- `--human` / `-H` は人間向けテキスト
- `-l` は最小限の long listing
- 設定ファイルは `.lls/config.json` のみ
- 初回の通常実行では設定ファイルを自動生成せず、`lls setup` へ誘導する
- `lls setup` は Codex CLI を利用して設定案を生成できる
- Codex 認証は **ChatGPT ログインのみ**をサポートする
- OpenAI Platform の API key 認証はサポートしない
- 通常の一覧生成はローカルの決定的ルールだけで動作し、Codex を呼び出さない
- `role` は意味上の役割、`generated` と `sensitive` は独立した属性として扱う
- 出力用の探索深度と、プロジェクト種別判定用の probe は分離する
- monorepo 判定は MVP に含めない
- 分類・ソート・設定優先順位は本書で固定する

---

## 1. 目的

通常の `ls` は名前と基本メタデータを一覧表示する。  
`lls` は、初見のリポジトリに対して次の判断を支援する。

- どのファイルが重要か
- どのファイルを最初に読むべきか
- どのディレクトリを初回探索で無視してよいか
- そのプロジェクトが何で作られていそうか
- どのエントリが設定、ソースコード、生成物、依存物なのか

`lls` は完全なファイルブラウザではない。  
目的は、探索の最初の一手を決めやすくすることである。

---

## 2. MVP の処理

MVP の通常実行は次の順で処理する。

1. CLI 引数を解析する
2. 対象パスと project root を解決する
3. 設定ファイルを解決して検証する
4. 指定された深度でファイルシステムを走査する
5. 各エントリのメタデータを取得する
6. 各エントリの属性と role を分類する
7. priority を決定する
8. 独立した project probe でプロジェクト種別を推定する
9. `recommended_next_steps` を生成する
10. canonical order または明示された sort で並べる
11. JSON または人間向け表示を stdout に出力する

通常実行では Codex を呼び出さない。

---

## 3. 非目標

MVP では次を行わない。

- 全ソースコードの深い意味解析
- 通常実行時の LLM 呼び出し
- OpenAI Platform API key による認証
- IDE のようなファイルブラウジング
- デフォルトでの深い再帰探索
- symlink の再帰的追跡
- 秘密情報候補の内容読み取り
- リッチな TUI
- monorepo / workspace の判定
- `.gitignore` / `.dockerignore` の完全な解釈
- README tagline 抽出
- PDF metadata title 抽出
- birth time による `🆕` 表示
- 完璧なプロジェクト種別推定

これらの候補は `feature-spec.md` に置く。

---

## 4. CLI 仕様

### 4.1 基本実行

```sh
lls
lls <path>
```

`<path>` を省略した場合は `.` を対象とする。

設定ファイルが見つからない通常実行は一覧を生成せず、stderr に次の行動を表示して終了コード `5` を返す。

```txt
lls: project configuration was not found
Run `lls setup` to create .lls/config.json, or use `--no-config` to run with built-in defaults.
```

通常実行は設定ファイルを自動作成しない。

### 4.2 出力モード

```sh
lls --json
lls --human
lls -H
lls -l
```

- デフォルトは `--json` と同じ compact JSON
- `--human` / `-H` は人間向けテキスト
- `-l` は long listing

複数指定時はエラーとし、暗黙の優先順位で無視しない。終了コードは `1` とする。

### 4.3 探索深度

```sh
lls --depth <number>
```

| 値 | 挙動 |
|---:|---|
| `0` | 対象パス自体のみ |
| `1` | 直接の子要素まで |
| `2` | 子と孫まで |
| `3`〜`8` | 指定深度まで |

デフォルトは `1`、MVP の上限は `8` とする。  
上限を超えた値は CLI 引数エラーとする。

### 4.4 設定関連

```sh
lls --config <path>
lls --no-config
```

- `--config` は設定ファイルを明示する
- `--no-config` は設定ファイル探索を行わず、組み込み既定値だけで実行する
- 両方を同時に指定した場合は CLI 引数エラー

### 4.5 setup

```sh
lls setup
lls setup --force
lls setup --yes
lls setup --without-codex
```

- `lls setup` は `.lls/config.json` の設定案を生成する
- 既存設定がある場合は上書きしない
- `--force` は既存設定の上書きを許可する
- `--yes` は生成案の確認を省略する
- `--without-codex` は Codex を呼び出さず、組み込み既定値から設定を生成する
- `--force` と `--yes` は別の意味を持つ

setup の詳細は [`docs/setup-plan.md`](../../docs/setup-plan.md) に定義する。

### 4.6 long listing

```sh
lls -l
lls -l --sort priority
lls -l --sort name
lls -l --sort mtime
lls -l --sort size
```

MVP の `-l` は次だけを表示する。

- priority
- role
- type
- human-readable size
- path
- `generated` / `sensitive` の属性

README tagline、PDF title、ignore file 解釈、`🆕` は MVP 外とする。

---

## 5. project root と設定ファイル探索

### 5.1 project root

対象がファイルなら親ディレクトリ、対象がディレクトリなら対象自身から祖先へ向かって探索し、最初に `.git` が見つかったディレクトリを project root とする。

`.git` が見つからない場合は次を project root とする。

- 対象がディレクトリ: 対象ディレクトリ
- 対象がファイル: 対象ファイルの親ディレクトリ

symlink を辿って別 project root を探索しない。

### 5.2 設定ファイルの解決順

設定ファイルは次の順で解決する。

1. `--config <path>`
2. `<project_root>/.lls/config.json`
3. `--no-config` が指定されている場合は組み込み既定値
4. いずれにも該当しない場合は setup required error

複数の祖先にある `.lls/config.json` をマージしない。

### 5.3 設定ファイル形式

設定ファイルは UTF-8 の JSON とする。  
正式な JSON Schema は [`config.schema.json`](config.schema.json) とする。

例:

```json
{
  "schema_version": "0.1.0",
  "default_output": "json",
  "scan": {
    "depth": 1,
    "include_hidden": true,
    "include_ignored": false
  },
  "long_listing": {
    "sort": "priority"
  },
  "rules": {
    "priority_overrides": [
      {
        "pattern": "docs/**",
        "priority": "high"
      }
    ],
    "role_overrides": [
      {
        "pattern": "docs/**",
        "role": "documentation"
      }
    ],
    "ignore_patterns": [
      "tmp/**"
    ],
    "sensitive_patterns": [
      "*.secret.json"
    ]
  },
  "codex": {
    "enabled": true,
    "auth_method": "chatgpt",
    "use_for_setup": true
  }
}
```

未知のキーはエラーとする。壊れた設定へ暗黙にフォールバックしない。

### 5.4 設定値の優先順位

同じ項目が複数箇所で指定された場合は次を優先する。

1. CLI 引数
2. `.lls/config.json`
3. 組み込み既定値

### 5.5 glob パターン

設定内のパターンは project root からの相対パスへ適用する。

- 区切り文字は OS に関係なく `/`
- `*` は `/` を越えない 0 文字以上
- `?` は `/` を越えない 1 文字
- `**` は `/` を越えて 0 個以上のパス要素
- 先頭 `/` は project root への固定を意味する
- 末尾 `/` はディレクトリだけを対象にする
- パターンは case-sensitive
- 不正パターンは設定エラー

---

## 6. Codex 認証と利用範囲

### 6.1 サポートする認証

MVP がサポートするのは、Codex CLI の **Sign in with ChatGPT** のみとする。

サポートしないもの:

- `OPENAI_API_KEY`
- `CODEX_API_KEY`
- OpenAI Platform の API key
- API key を `.lls/config.json` に保存する方式
- Codex の token や `auth.json` を `lls` が直接読む方式

`lls` は資格情報を保持せず、Codex CLI の認証と資格情報ストアへ委譲する。

### 6.2 login

Codex セッションが利用できない場合、`lls setup` は `codex login` を案内または起動する。  
headless 環境では `codex login --device-auth` を案内する。

`lls` は `~/.codex/auth.json` をコピー、解析、表示、コミットしない。

### 6.3 Codex を使う処理

Codex は `lls setup` の設定案生成にだけ利用する。

概念上、次の read-only 実行を行う。

```sh
codex exec \
  --ephemeral \
  --sandbox read-only \
  --ignore-user-config \
  --ignore-rules \
  --output-schema <config-schema-path> \
  -o <temporary-output-path> \
  -C <project-root> \
  "<lls setup prompt>"
```

要件:

- Codex にファイル書き込み権限を与えない
- 最終出力を JSON Schema で制約する
- Codex の出力をそのまま保存せず、`lls` 側で再検証する
- 検証後の設定案だけを表示し、人間の確認後に `lls` が書き込む
- 一時ファイルは成功・失敗にかかわらず削除する
- Codex 失敗時に不完全な設定を残さない

通常の `lls` / `lls --json` / `lls -l` / `lls --human` は Codex を呼び出さない。

---

## 7. 走査仕様

### 7.1 利用する情報

MVP では主に次を使う。

- ファイル名
- 拡張子
- ファイルシステム上の type
- サイズ
- ディレクトリ名
- project root からの相対パス
- 既知の manifest / lockfile / source / test / build output 名

必要に応じて、小さな manifest の存在確認は行ってよい。  
通常の分類で全ファイル内容を読まない。

### 7.2 hidden entry

hidden entry は自動では無視しない。  
`.git`、`.cache`、`.next` など組み込みノイズディレクトリは priority `ignore` とする。

### 7.3 symlink

- `type: "symlink"` として出力する
- symlink の先を再帰的に辿らない
- broken symlink を検出できる場合は warning に含める

### 7.4 ignore / prune / omit

MVP では用語を次のように固定する。

- `priority: "ignore"`: entries には表示するが recommendation から除外する
- prune: ignore 対象のディレクトリ自身は表示し、子孫を走査しない
- omit: MVP では使用しない

`scan.include_ignored: true` または将来の明示オプションがない限り、ignore ディレクトリは prune する。

`.gitignore` や `.dockerignore` の内容は MVP では解釈しない。

### 7.5 非 UTF-8 パス

UTF-8 に変換できないパスは entries へ含めず、warning code `non_utf8_path_skipped` を追加する。  
lossy 変換したパスを安定 API として出力しない。

---

## 8. project probe

出力用 scan depth と project type 判定は分離する。

project probe は project root から次の固定パスの存在だけを確認できる。

```txt
Cargo.toml
src/main.rs
src/lib.rs
package.json
pyproject.toml
setup.py
go.mod
```

project probe は再帰走査を行わず、scan depth と entries の範囲を変更しない。  
したがってデフォルト `--depth 1` でも `src/main.rs` を根拠に `rust_cli` を判定できる。

### 8.1 project type

| name | 判定 |
|---|---|
| `rust_cli` | `Cargo.toml` と `src/main.rs` |
| `rust_library` | `Cargo.toml` と `src/lib.rs` |
| `node_project` | `package.json` |
| `python_package` | `pyproject.toml` または `setup.py` |
| `go_module` | `go.mod` |
| `unknown` | 強い根拠なし |

複数候補が成立する場合は次の順で選ぶ。

```txt
rust_cli > rust_library > node_project > python_package > go_module
```

同時に warning code `multiple_project_types_detected` を出す。  
monorepo は MVP では判定しない。

---

## 9. Entry 仕様

各 entry は次の構造を持つ。

```json
{
  "name": "Cargo.toml",
  "path": "Cargo.toml",
  "type": "file",
  "role": "manifest",
  "priority": "critical",
  "reason_code": "known_manifest",
  "reason": "Rust のマニフェストファイル",
  "generated": false,
  "sensitive": false,
  "text": true,
  "binary": false,
  "size_bytes": 1024
}
```

### 9.1 フィールド

| field | type | required | 説明 |
|---|---|---:|---|
| `name` | string | yes | basename |
| `path` | string | yes | project root からの `/` 区切り相対パス |
| `type` | string | yes | file system type |
| `role` | string | yes | 意味上の役割 |
| `priority` | string | yes | 探索上の重要度 |
| `reason_code` | string | yes | 安定した機械可読の判定理由 |
| `reason` | string | yes | 人間向け説明 |
| `generated` | boolean | yes | 生成物らしいか |
| `sensitive` | boolean | yes | 秘密情報候補か |
| `text` | boolean | no | テキスト推定 |
| `binary` | boolean | no | バイナリ推定 |
| `size_bytes` | integer | no | ファイルサイズ |

optional field は値を取得できない場合に省略する。`null` は出力しない。

### 9.2 type

```txt
file
directory
symlink
other
```

### 9.3 role

```txt
project_overview
manifest
lockfile
source_code
test_code
documentation
config
ci_config
build_output
dependency_cache
license
data
unknown
```

`generated` と `secret_candidate` は role として使用しない。

例:

- `.env`: `role: "config"`, `sensitive: true`
- `dist/`: `role: "build_output"`, `generated: true`
- `client.generated.rs`: `role: "source_code"`, `generated: true`

### 9.4 priority

```txt
critical
high
medium
low
ignore
```

---

## 10. 分類ルールと衝突解決

### 10.1 属性判定

`generated` と `sensitive` は role より先に独立判定する。  
属性は role override によって消えない。

`sensitive: true` の entry は priority に関係なく recommendation へ含めない。

### 10.2 role の優先順位

role は次の順で最初に一致したものを採用する。

1. `rules.role_overrides` の上から順
2. 組み込みの exact relative path
3. 組み込みの exact basename
4. 組み込みの directory pattern
5. 組み込みの extension pattern
6. `unknown`

複数の user override が一致した場合は、配列で先に書かれたルールを採用する。

### 10.3 priority の優先順位

priority は次の順で決定する。

1. `rules.ignore_patterns` の一致 → `ignore`
2. `rules.priority_overrides` の上から最初の一致
3. 組み込み priority
4. role ごとの既定 priority
5. `low`

ただし次の safety rule は上書きできない。

- `.git/` は常に `ignore`
- `sensitive: true` は recommendation から常に除外
- binary は recommendation から常に除外

### 10.4 組み込み例

| entry | role | priority | attributes |
|---|---|---|---|
| `README.md` | `project_overview` | `critical` | |
| `Cargo.toml` | `manifest` | `critical` | |
| `package.json` | `manifest` | `critical` | |
| `src/` | `source_code` | `high` | |
| `tests/` | `test_code` | `medium` | |
| `docs/` | `documentation` | `medium` | |
| `.github/workflows/` | `ci_config` | `high` | |
| `Cargo.lock` | `lockfile` | `medium` | `generated: true` |
| `.env` | `config` | `medium` | `sensitive: true` |
| `target/` | `build_output` | `ignore` | `generated: true` |
| `node_modules/` | `dependency_cache` | `ignore` | |
| `.git/` | `dependency_cache` | `ignore` | |

lockfile は generated でも priority `ignore` にはしない。

---

## 11. JSON 出力

### 11.1 top-level

```json
{
  "schema_version": "0.1.0",
  "path": ".",
  "project_type": {
    "name": "rust_cli",
    "confidence": 0.95,
    "evidence": ["Cargo.toml", "src/main.rs"]
  },
  "summary": {
    "total_entries": 6,
    "shown_entries": 6,
    "important_entries": 3,
    "ignored_entries": 1
  },
  "entries": [],
  "recommended_next_steps": [],
  "warnings": []
}
```

すべての top-level field は必須とする。

### 11.2 summary

- `total_entries`: 実際に発見した entry 数。prune された子孫は数えない
- `shown_entries`: `entries.length`
- `important_entries`: priority が `critical` または `high` の数
- `ignored_entries`: priority が `ignore` の数

### 11.3 warning

```json
{
  "code": "sensitive_candidate_detected",
  "path": ".env",
  "message": "秘密情報候補を検出した"
}
```

| field | required |
|---|---:|
| `code` | yes |
| `path` | no |
| `message` | yes |

### 11.4 文字列と改行

- JSON は UTF-8
- default / `--json` は 1 行の compact JSON
- stdout の最後に改行を 1 つ付ける
- JSON key の順序へ意味を持たせない
- entry path は `/` 区切り
- optional field に `null` を使わない

---

## 12. recommended_next_steps

各要素:

```json
{
  "action": "read",
  "path": "README.md",
  "reason_code": "read_project_overview_first",
  "reason": "プロジェクト概要を把握するため"
}
```

MVP の action:

```txt
read
inspect
```

`skip` は MVP では使用しない。

生成ルール:

1. `critical` を優先
2. `project_overview` を最優先
3. manifest を source より先にする
4. `ignore` を除外
5. `sensitive: true` を除外
6. binary を除外
7. symlink を除外
8. 最大 5 件

---

## 13. ソート

### 13.1 canonical order

JSON と `--sort priority` は次の完全な順序を使う。

1. priority rank
2. role rank
3. normalized path の byte-wise 昇順

priority rank:

```txt
critical > high > medium > low > ignore
```

role rank:

```txt
project_overview
manifest
source_code
test_code
documentation
ci_config
config
lockfile
license
data
build_output
dependency_cache
unknown
```

### 13.2 その他の sort

- `--sort name`: normalized path 昇順
- `--sort mtime`: mtime 降順、取得不能は最後、同値は path 昇順
- `--sort size`: size 降順、ディレクトリと取得不能は最後、同値は path 昇順

sort は安定ソートとする。

---

## 14. stdout / stderr / 終了コード

### 14.1 stdout

成功時の本体だけを出力する。

- JSON mode: JSON のみ
- human mode: 人間向け表示のみ
- long mode: long listing のみ

setup の確認質問や進捗は stdout へ混ぜない。

### 14.2 stderr

次を stderr へ出す。

- setup 誘導
- CLI 引数エラー
- 設定エラー
- Codex login 案内
- fatal error
- setup の進捗と確認質問

fatal error 時は partial JSON を stdout へ出さない。

### 14.3 終了コード

| code | 意味 |
|---:|---|
| `0` | 成功 |
| `1` | CLI 引数不正 |
| `2` | 対象パスなし |
| `3` | 対象パスの権限不足 |
| `4` | 予期しない実行時エラー |
| `5` | setup required |
| `6` | Codex CLI 不在、ChatGPT login 不可、または Codex setup 失敗 |
| `7` | 設定ファイル不正 |

一部 entry の読み取り失敗は fatal にせず warnings に含める。

---

## 15. 受け入れ条件

### 15.1 初回実行

設定なしで `lls` を実行した場合:

- `.lls/config.json` を作らない
- setup の案内を stderr に出す
- stdout は空
- 終了コード `5`

`lls --no-config` は組み込み既定値で成功する。

### 15.2 setup

- ChatGPT login 済み Codex CLI から設定案を生成できる
- API key を要求・参照しない
- Codex は read-only sandbox で実行される
- Codex 出力は schema validation される
- 確認前に設定を書き込まない
- 失敗時に partial config を残さない
- `--without-codex` では決定的な既定設定を生成できる

### 15.3 Rust CLI

`Cargo.toml` と `src/main.rs` が存在する場合:

- scan depth `1` でも `project_type.name` は `rust_cli`
- evidence に両方を含む
- `README.md` と `Cargo.toml` は `critical`
- `src/` は `high`
- `target/` は `ignore`
- target の子孫は既定で prune
- recommendation に README、manifest、src を含む

### 15.4 属性と role

`.env`:

```json
{
  "role": "config",
  "sensitive": true
}
```

`dist/`:

```json
{
  "role": "build_output",
  "generated": true,
  "priority": "ignore"
}
```

### 15.5 deterministic output

同じ filesystem state、設定、CLI 引数では、entry の順序、reason_code、recommendation が同一になる。

---

## 16. 推奨モジュール構成

```txt
cli
config
scanner
classifier
project_probe
priority
recommendation
setup
codex
output
error
model
```

| module | 責務 |
|---|---|
| `cli` | CLI 解析 |
| `config` | schema validation、探索、既定値 |
| `scanner` | depth に従う走査 |
| `classifier` | role と属性判定 |
| `project_probe` | scan と独立した固定パス probe |
| `priority` | priority と衝突解決 |
| `recommendation` | 次の行動候補 |
| `setup` | 初回セットアップ |
| `codex` | Codex CLI subprocess 境界 |
| `output` | JSON / human / long |
| `error` | error と exit code |
| `model` | 共通データ型 |

`codex` module は token や auth cache を扱わない。

---

## 17. テスト要件

最低限、次を自動テストする。

- config JSON Schema の valid / invalid fixture
- config 探索順
- CLI > config > built-in の優先順位
- glob の `*` / `?` / `**` / leading slash / trailing slash
- role override の first-match
- ignore pattern と priority override の衝突
- sensitive safety rule
- generated と role の独立性
- depth `1` と rust_cli project probe
- polyglot warning と固定 precedence
- ignore directory の prune
- canonical order
- name / mtime / size sort の tie-break
- compact JSON と末尾改行
- optional field の omit
- stdout / stderr 分離
- setup required exit code
- Codex subprocess の成功、login failure、invalid output、timeout
- setup atomic write
- fixture を使った E2E

Codex の実ネットワーク呼び出しは通常の unit test では行わず、process adapter を fake に差し替える。

---

## 18. 最終原則

```txt
出力は少なく、優先度は明確に、安全側に倒す
```

設定生成には Codex を利用できるが、通常の一覧生成はローカルで決定的に動作させる。
