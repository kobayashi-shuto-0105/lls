# lls 仕様書

このドキュメントは、`lls` の MVP として実装する仕様を定義する。

`lls` は、ディレクトリやリポジトリを走査し、LLM や人間が「次にどのファイルを見るべきか」を判断しやすいように、各エントリへ役割・重要度・理由を付けて出力する CLI ツールである。

将来的な拡張案や MVP 外の機能は [`feature-spec.md`](feature-spec.md) に記載する。

---

## 1. 目的

通常の `ls` は、ファイル名やディレクトリ名を一覧表示するだけである。

一方で、LLM が初めてリポジトリを調べる場合、単なる名前一覧だけでは次の判断が難しい。

`lls` は、次のような判断を支援する。

- どのファイルが重要か
- どのファイルを最初に読むべきか
- どのディレクトリを無視してよいか
- そのプロジェクトが何で作られていそうか
- どのファイルが設定・ソースコード・生成物・依存物なのか

`lls` は完全なファイルブラウザではない。  
目的は、探索の最初の一手を決めやすくすることである。

---

## 2. MVP の概要

MVP では、`lls` は次のことを行う。

1. 対象パスを受け取る
2. 対象ディレクトリまたはファイルのメタデータを取得する
3. 各エントリに役割を付ける
4. 各エントリに重要度を付ける
5. プロジェクト種別をざっくり推定する
6. 次に読むべき候補を返す
7. 人間向け表示と JSON 出力に対応する

特に JSON 出力は、LLM や外部ツールが安定して利用できる形式にする。

---

## 3. ゴール

### 3.1 機能面のゴール

`lls` は次の機能を持つ。

- 対象パス配下のファイル・ディレクトリを一覧化する
- 各エントリの種別を判定する
- 各エントリの役割を推定する
- 各エントリの重要度を判定する
- よくあるノイズディレクトリを検出する
- よくある生成物・ビルド成果物を検出する
- 秘密情報を含みそうなファイルを検出する
- 主要なマニフェストファイルからプロジェクト種別を推定する
- 次に読むべきファイルやディレクトリを提案する
- JSON 形式で安定した出力を返す

### 3.2 設計上のゴール

`lls` は次の方針を優先する。

- 名前の一覧よりも意味を返す
- 網羅性よりも探索しやすさを優先する
- 見た目の綺麗さよりも機械的に扱いやすい構造を優先する
- 複雑な推論よりも決定的なルールを優先する
- 秘密情報を誤って読ませない安全なデフォルトを採用する

---

## 4. 非目標

MVP では次のことは行わない。

- 全ソースコードの深い解析
- LLM を使った内容要約
- IDE のようなファイルブラウジング
- デフォルトでの深い再帰探索
- symlink の再帰的な追跡
- 秘密情報ファイルの中身の読み取り
- リッチな TUI の実装
- すべての言語・フレームワークへの完全対応
- 完璧なプロジェクト種別推定

---

## 5. CLI 仕様

### 5.1 基本実行

```sh
lls
```

これは次と同じ意味である。

```sh
lls .
```

カレントディレクトリを走査し、人間向けの表示を出力する。

---

### 5.2 パス指定

```sh
lls <path>
```

例:

```sh
lls .
lls src
lls ./packages/api
```

`<path>` が省略された場合は `.` を対象にする。

---

### 5.3 JSON 出力

```sh
lls --json
```

構造化された JSON を出力する。

LLM や外部ツールが利用する場合は、この出力を主に使う。

---

### 5.4 探索深度

```sh
lls --depth <number>
```

探索する深さを指定する。

MVP での扱いは次の通り。

| 値 | 挙動 |
|---:|---|
| `0` | 対象パス自体のみを見る |
| `1` | 直接の子要素を見る |
| `2` | 子と孫まで見る |
| `3` 以上 | 指定可能。ただし実装側で上限を設けてもよい |

デフォルト値は次の通り。

```txt
--depth 1
```

MVP では、デフォルトで巨大なリポジトリを深く探索しない。

---

### 5.5 compact JSON

```sh
lls --json --compact
```

余分な空白や改行を含まない JSON を出力する。

ツール連携やトークン節約を目的とする。

---

## 6. 入力仕様

### 6.1 対象パス

対象パスには次のいずれかを指定できる。

- ディレクトリ
- ファイル

対象がディレクトリの場合、その配下のエントリを走査する。  
対象がファイルの場合、そのファイルのみを分類する。

---

### 6.2 利用する情報

MVP では、主に次の情報を使って分類する。

- ファイル名
- 拡張子
- ファイル種別
- ファイルサイズ
- ディレクトリ名
- パス
- よくあるマニフェストファイル名
- よくあるノイズディレクトリ名
- よくある生成物・ビルド成果物名

MVP では、すべてのファイル内容を読む必要はない。

---

### 6.3 内容を読んでよいファイル

MVP では、必要に応じて小さなメタデータ系ファイルだけ読んでもよい。

例:

- `Cargo.toml`
- `package.json`
- `pyproject.toml`
- `go.mod`

ただし、MVP ではファイル名ベースの判定だけでも動作すること。

---

### 6.4 symlink の扱い

デフォルトでは次のように扱う。

- symlink は `type: "symlink"` として出力する
- symlink の先を再帰的に辿らない
- 壊れた symlink を検出できる場合は warning に含める

---

### 6.5 hidden file の扱い

hidden file は自動では無視しない。

重要な hidden file の例:

- `.gitignore`
- `.env.example`
- `.github`
- `.npmrc`
- `.tool-versions`

ただし、よくある hidden なノイズディレクトリは `ignore` として扱う。

例:

- `.git`
- `.next`
- `.turbo`
- `.cache`

---

## 7. JSON 出力仕様

### 7.1 トップレベル構造

`lls --json` は次の構造を返す。

```json
{
  "schema_version": "0.1.0",
  "path": ".",
  "project_type": {
    "name": "rust_cli",
    "confidence": 0.9,
    "evidence": ["Cargo.toml", "src/main.rs"]
  },
  "summary": {
    "total_entries": 8,
    "shown_entries": 8,
    "important_entries": 4,
    "ignored_entries": 2
  },
  "entries": [],
  "recommended_next_steps": [],
  "warnings": []
}
```

---

### 7.2 トップレベルフィールド

| フィールド | 型 | 必須 | 説明 |
|---|---|---:|---|
| `schema_version` | string | yes | 出力スキーマのバージョン |
| `path` | string | yes | 対象パス |
| `project_type` | object | yes | 推定されたプロジェクト種別 |
| `summary` | object | yes | 件数などの概要 |
| `entries` | array | yes | 分類済みエントリ一覧 |
| `recommended_next_steps` | array | yes | 次に取るべき行動の候補 |
| `warnings` | array | yes | 致命的ではない警告 |

---

## 8. Entry 仕様

各エントリは次の構造を持つ。

```json
{
  "name": "Cargo.toml",
  "path": "Cargo.toml",
  "type": "file",
  "role": "manifest",
  "priority": "critical",
  "reason": "Rust のマニフェストファイル。パッケージ情報や依存関係の把握に重要",
  "generated": false,
  "sensitive": false,
  "text": true,
  "binary": false,
  "size_bytes": 1024
}
```

---

### 8.1 Entry フィールド

| フィールド | 型 | 必須 | 説明 |
|---|---|---:|---|
| `name` | string | yes | ファイル名またはディレクトリ名 |
| `path` | string | yes | 対象パスからの相対パス |
| `type` | string | yes | ファイルシステム上の種別 |
| `role` | string | yes | 推定された役割 |
| `priority` | string | yes | 探索上の重要度 |
| `reason` | string | yes | 判定理由 |
| `generated` | boolean | yes | 生成物の可能性が高いか |
| `sensitive` | boolean | yes | 秘密情報を含む可能性があるか |
| `text` | boolean | no | テキストファイルと推定されるか |
| `binary` | boolean | no | バイナリファイルと推定されるか |
| `size_bytes` | number | no | ファイルサイズ |

---

## 9. type の値

`type` は次のいずれかにする。

| 値 | 意味 |
|---|---|
| `file` | 通常ファイル |
| `directory` | ディレクトリ |
| `symlink` | シンボリックリンク |
| `other` | その他 |

---

## 10. role の値

`role` は次のいずれかを使う。

| role | 意味 | 例 |
|---|---|---|
| `project_overview` | プロジェクト概要 | `README.md` |
| `manifest` | パッケージやプロジェクトの定義 | `Cargo.toml`, `package.json`, `pyproject.toml` |
| `lockfile` | 依存関係の固定ファイル | `Cargo.lock`, `package-lock.json`, `yarn.lock` |
| `source_code` | メインのソースコード | `src/`, `main.rs`, `index.ts` |
| `test_code` | テストコード | `tests/`, `__tests__/`, `*.test.ts` |
| `documentation` | ドキュメント | `docs/`, `CHANGELOG.md` |
| `config` | 設定ファイル | `.gitignore`, `tsconfig.json`, `eslint.config.js` |
| `ci_config` | CI/CD 設定 | `.github/workflows/`, `.gitlab-ci.yml` |
| `build_output` | ビルド成果物 | `dist/`, `build/`, `target/` |
| `dependency_cache` | 依存物やキャッシュ | `node_modules/`, `vendor/` |
| `generated` | 生成されたファイル | generated client, compiled file |
| `secret_candidate` | 秘密情報候補 | `.env`, `*.pem`, `id_rsa` |
| `license` | ライセンス | `LICENSE`, `LICENSE.md` |
| `unknown` | 不明 | 分類できないもの |

---

## 11. priority の値

`priority` は次のいずれかにする。

| priority | 意味 |
|---|---|
| `critical` | 最初に読むべき重要ファイル |
| `high` | 探索や実装理解に重要 |
| `medium` | コア理解後に読むとよい |
| `low` | 初回探索では優先度が低い |
| `ignore` | 初回探索では基本的に無視してよい |

---

## 12. priority 判定ルール

### 12.1 critical

プロジェクト理解に不可欠なファイルは `critical` にする。

例:

| エントリ | role |
|---|---|
| `README.md` | `project_overview` |
| `Cargo.toml` | `manifest` |
| `package.json` | `manifest` |
| `pyproject.toml` | `manifest` |
| `go.mod` | `manifest` |
| `src/main.rs` | `source_code` |
| `src/lib.rs` | `source_code` |

---

### 12.2 high

主要な実装やワークフローに関係するディレクトリは `high` にする。

例:

| エントリ | role |
|---|---|
| `src/` | `source_code` |
| `app/` | `source_code` |
| `lib/` | `source_code` |
| `packages/` | `source_code` |
| `.github/workflows/` | `ci_config` |

---

### 12.3 medium

重要ではあるが、最初に必ず読む必要はないものは `medium` にする。

例:

| エントリ | role |
|---|---|
| `tests/` | `test_code` |
| `docs/` | `documentation` |
| `examples/` | `documentation` |
| `Cargo.lock` | `lockfile` |
| `package-lock.json` | `lockfile` |
| `tsconfig.json` | `config` |

---

### 12.4 low

初回探索では優先度が低いが、後で役立つ可能性があるものは `low` にする。

例:

| エントリ | role |
|---|---|
| `.gitignore` | `config` |
| `LICENSE` | `license` |
| `.editorconfig` | `config` |

---

### 12.5 ignore

依存物、キャッシュ、ビルド成果物などは `ignore` にする。

例:

| エントリ | role |
|---|---|
| `.git/` | `dependency_cache` |
| `node_modules/` | `dependency_cache` |
| `target/` | `build_output` |
| `dist/` | `build_output` |
| `build/` | `build_output` |
| `coverage/` | `build_output` |
| `.next/` | `build_output` |
| `.turbo/` | `build_output` |
| `__pycache__/` | `build_output` |

---

## 13. 秘密情報候補の扱い

秘密情報を含む可能性があるファイルは、次のように扱う。

```json
{
  "role": "secret_candidate",
  "sensitive": true
}
```

例:

| パターン | 理由 |
|---|---|
| `.env` | 環境変数や秘密情報を含む可能性がある |
| `.env.local` | ローカルの秘密情報を含む可能性がある |
| `*.pem` | 秘密鍵や証明書の可能性がある |
| `*.key` | 秘密鍵の可能性がある |
| `id_rsa` | SSH 秘密鍵の可能性がある |
| `id_ed25519` | SSH 秘密鍵の可能性がある |

秘密情報候補は `recommended_next_steps` に含めない。

ただし、存在自体は `entries` に表示する。  
これは、LLM やユーザーが「そのようなファイルがある」と認識できるようにするためである。

---

## 14. 生成物・ビルド成果物の扱い

生成物と推定されるものは、次のように扱う。

```json
{
  "generated": true
}
```

よくある例:

| パターン | role | priority |
|---|---|---|
| `dist/` | `build_output` | `ignore` |
| `build/` | `build_output` | `ignore` |
| `target/` | `build_output` | `ignore` |
| `coverage/` | `build_output` | `ignore` |
| `*.min.js` | `generated` | `ignore` |
| `*.lock` | `lockfile` | `medium` |

lockfile は生成物に近いが、依存関係の再現性に重要なため、原則として `ignore` にはしない。

---

## 15. プロジェクト種別推定

`project_type.name` は次のいずれかにする。

| name | 判定ルール |
|---|---|
| `rust_cli` | `Cargo.toml` と `src/main.rs` が存在する |
| `rust_library` | `Cargo.toml` と `src/lib.rs` が存在する |
| `node_project` | `package.json` が存在する |
| `python_package` | `pyproject.toml` または `setup.py` が存在する |
| `go_module` | `go.mod` が存在する |
| `monorepo` | 複数のパッケージ・ワークスペースらしき構造がある |
| `unknown` | 強い判定材料がない |

---

### 15.1 confidence

`confidence` は `0.0` から `1.0` の数値にする。

| 値 | 意味 |
|---:|---|
| `0.9` - `1.0` | 非常に強い根拠がある |
| `0.7` - `0.89` | ある程度強い根拠がある |
| `0.4` - `0.69` | 弱い根拠がある |
| `0.0` - `0.39` | 判定不能または信頼できない |

例:

```json
{
  "name": "rust_cli",
  "confidence": 0.95,
  "evidence": ["Cargo.toml", "src/main.rs"]
}
```

```json
{
  "name": "unknown",
  "confidence": 0.0,
  "evidence": []
}
```

---

## 16. recommended_next_steps

`recommended_next_steps` は、次に取るべき行動を示す。

各要素は次の構造にする。

```json
{
  "action": "read",
  "path": "README.md",
  "reason": "プロジェクト概要を把握するため最初に読むべき"
}
```

---

### 16.1 フィールド

| フィールド | 型 | 必須 | 説明 |
|---|---|---:|---|
| `action` | string | yes | 推奨する行動 |
| `path` | string | yes | 対象パス |
| `reason` | string | yes | その行動を推奨する理由 |

---

### 16.2 action の値

MVP では次の値を使う。

| action | 意味 |
|---|---|
| `read` | ファイルを読む |
| `inspect` | ディレクトリを調べる |
| `skip` | 初回探索では避ける |

---

### 16.3 推奨ルール

`recommended_next_steps` は次のルールで作る。

1. `critical` を優先する
2. `README.md` などの概要ファイルを優先する
3. マニフェストファイルをソースディレクトリより先にする
4. `priority: "ignore"` のエントリは除外する
5. `sensitive: true` のエントリは除外する
6. バイナリファイルは除外する
7. 最大 5 件までにする

推奨順序の目安:

1. `README.md`
2. マニフェストファイル
3. メインのソースコード
4. テスト
5. 設定や CI

例:

```json
[
  {
    "action": "read",
    "path": "README.md",
    "reason": "プロジェクト概要を把握するため最初に読むべき"
  },
  {
    "action": "read",
    "path": "Cargo.toml",
    "reason": "Rust パッケージのメタデータと依存関係を確認するため"
  },
  {
    "action": "inspect",
    "path": "src",
    "reason": "メインのソースコードが含まれる可能性が高いため"
  }
]
```

---

## 17. 人間向け出力

デフォルト出力は、人間が素早く読める形式にする。

例:

```txt
Project: rust_cli (confidence: 0.95)
Path: .

Recommended next steps:
1. read README.md - プロジェクト概要を把握するため最初に読むべき
2. read Cargo.toml - Rust パッケージのメタデータと依存関係を確認するため
3. inspect src/ - メインのソースコードが含まれる可能性が高いため

Entries:
[critical] README.md        file       project_overview  プロジェクト概要を含む可能性が高い
[critical] Cargo.toml       file       manifest          Rust のマニフェストファイル
[high]     src/             directory  source_code       メインのソースコードディレクトリ
[medium]   tests/           directory  test_code         テストディレクトリ
[ignore]   target/          directory  build_output      Rust のビルド成果物
```

厳密な余白は固定しなくてよい。  
ただし、含まれる情報は JSON 出力と対応していること。

---

## 18. JSON 出力例

Rust CLI プロジェクトの場合:

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
  "entries": [
    {
      "name": "README.md",
      "path": "README.md",
      "type": "file",
      "role": "project_overview",
      "priority": "critical",
      "reason": "プロジェクト概要や使い方が書かれている可能性が高い",
      "generated": false,
      "sensitive": false,
      "text": true,
      "binary": false,
      "size_bytes": 2400
    },
    {
      "name": "Cargo.toml",
      "path": "Cargo.toml",
      "type": "file",
      "role": "manifest",
      "priority": "critical",
      "reason": "Rust のマニフェストファイル。パッケージ情報や依存関係の把握に重要",
      "generated": false,
      "sensitive": false,
      "text": true,
      "binary": false,
      "size_bytes": 900
    },
    {
      "name": "src",
      "path": "src",
      "type": "directory",
      "role": "source_code",
      "priority": "high",
      "reason": "メインのソースコードディレクトリ",
      "generated": false,
      "sensitive": false
    },
    {
      "name": "tests",
      "path": "tests",
      "type": "directory",
      "role": "test_code",
      "priority": "medium",
      "reason": "テストコードを含むディレクトリ",
      "generated": false,
      "sensitive": false
    },
    {
      "name": "target",
      "path": "target",
      "type": "directory",
      "role": "build_output",
      "priority": "ignore",
      "reason": "Rust のビルド成果物ディレクトリ",
      "generated": true,
      "sensitive": false
    },
    {
      "name": ".env",
      "path": ".env",
      "type": "file",
      "role": "secret_candidate",
      "priority": "medium",
      "reason": "環境変数や秘密情報を含む可能性がある",
      "generated": false,
      "sensitive": true,
      "text": true,
      "binary": false,
      "size_bytes": 120
    }
  ],
  "recommended_next_steps": [
    {
      "action": "read",
      "path": "README.md",
      "reason": "プロジェクト概要を把握するため最初に読むべき"
    },
    {
      "action": "read",
      "path": "Cargo.toml",
      "reason": "Rust パッケージのメタデータと依存関係を確認するため"
    },
    {
      "action": "inspect",
      "path": "src",
      "reason": "メインのソースコードが含まれる可能性が高いため"
    }
  ],
  "warnings": [
    {
      "path": ".env",
      "message": "秘密情報候補を検出したため、明示的に必要な場合を除き内容を読まないこと"
    }
  ]
}
```

---

## 19. エラーハンドリング

### 19.1 終了コード

| コード | 意味 |
|---:|---|
| `0` | 成功 |
| `1` | CLI 引数の不正 |
| `2` | 対象パスが存在しない |
| `3` | 権限不足 |
| `4` | 予期しない実行時エラー |

---

### 19.2 致命的でないエラー

対象ディレクトリ自体は読めるが、一部のエントリが読めない場合、処理全体は失敗させない。

その場合は `warnings` に含める。

例:

```json
{
  "path": "private-dir",
  "message": "エントリのメタデータ取得時に権限エラーが発生した"
}
```

---

## 20. ソートルール

`entries` は次の順で並べる。

1. `priority`
2. `role` の重要度
3. 名前の昇順

priority の順序は次の通り。

```txt
critical > high > medium > low > ignore
```

---

## 21. 受け入れ条件

MVP は、少なくとも次のケースを満たすこと。

---

### 21.1 Rust CLI リポジトリ

入力例:

```txt
.
├── README.md
├── Cargo.toml
├── Cargo.lock
├── src/
│   └── main.rs
├── tests/
└── target/
```

期待結果:

- `project_type.name` が `rust_cli`
- `README.md` が `critical`
- `Cargo.toml` が `critical`
- `src` が `high`
- `tests` が `medium`
- `target` が `ignore`
- `recommended_next_steps` に次が含まれる
  - `README.md`
  - `Cargo.toml`
  - `src`

---

### 21.2 Node.js プロジェクト

入力例:

```txt
.
├── README.md
├── package.json
├── package-lock.json
├── src/
├── node_modules/
└── dist/
```

期待結果:

- `project_type.name` が `node_project`
- `package.json` が `critical`
- `node_modules` が `ignore`
- `dist` が `ignore`
- `recommended_next_steps` に `node_modules` や `dist` が含まれない

---

### 21.3 秘密情報候補を含むディレクトリ

入力例:

```txt
.
├── README.md
├── .env
└── src/
```

期待結果:

- `.env` の `sensitive` が `true`
- `.env` の `role` が `secret_candidate`
- `.env` が `recommended_next_steps` に含まれない
- `.env` に関する warning が出力される

---

### 21.4 種別不明のディレクトリ

入力例:

```txt
.
├── notes.txt
├── data.csv
└── archive.zip
```

期待結果:

- `project_type.name` が `unknown`
- エントリ一覧は出力される
- バイナリやアーカイブファイルを最初に読む候補にしない
- プロジェクト種別が不明でも処理は失敗しない

---

## 22. 実装方針

MVP はルールベースで決定的に実装する。

処理の流れは次の通り。

1. CLI 引数をパースする
2. 対象パスを解決する
3. ファイルシステムを走査する
4. メタデータを取得する
5. 各エントリを分類する
6. プロジェクト種別を推定する
7. priority を付ける
8. recommended_next_steps を生成する
9. 人間向け表示または JSON を出力する

分類ロジックと出力ロジックは分離すること。  
これにより、分類ルールをテストしやすくする。

---

## 23. 推奨モジュール構成

実装上は、次のような構成にするとよい。

```txt
cli
scanner
classifier
project_type
priority
recommendation
output
error
```

各モジュールの責務:

| モジュール | 責務 |
|---|---|
| `cli` | CLI 引数の解析 |
| `scanner` | ファイルシステム走査とメタデータ取得 |
| `classifier` | role, generated, sensitive, text/binary の判定 |
| `project_type` | プロジェクト種別の推定 |
| `priority` | priority の付与 |
| `recommendation` | 次に読むべき候補の生成 |
| `output` | 人間向け表示と JSON 出力 |
| `error` | エラー型と終了コードの定義 |

---

## 24. テスト要件

MVP では次のテストを用意する。

- role 判定のテスト
- priority 判定のテスト
- プロジェクト種別推定のテスト
- 秘密情報候補検出のテスト
- 生成物・ノイズ検出のテスト
- recommended_next_steps の並び順テスト
- JSON 出力構造のテスト
- fixture ディレクトリを使った E2E テスト

分類と recommendation のロジックは、実際のファイルシステムに依存せず単体テストできるようにする。

---

## 25. 最終原則

`lls` は、すべてを同じ重みで見せるツールではない。

目的は、次に取るべき行動を決めやすくすることである。

迷った場合は、次を優先する。

```txt
出力は少なく、優先度は明確に、安全側に倒す
```
