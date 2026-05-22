<p align="center">
	<img src=".github/assets/lls_logo.png" alt="lls logo" width="240">
</p>

<h1 align="center">lls</h1>

<p align="center"><strong><code>lls</code> = <code>ls</code> for LLMs</strong></p>

<p align="center">
  <a href="https://github.com/kobayashi-shuto-0105/lls/actions/workflows/build.yaml"><img src="https://github.com/kobayashi-shuto-0105/lls/actions/workflows/build.yaml/badge.svg" alt="build"></a>
  <a href="https://coveralls.io/github/kobayashi-shuto-0105/lls?branch=main"><img src="https://coveralls.io/repos/github/kobayashi-shuto-0105/lls/badge.svg?branch=main" alt="Coverage Status"></a>
  <a href="https://github.com/kobayashi-shuto-0105/lls/releases/tag/v0.1.0"><img src="https://img.shields.io/badge/Version-0.1.0-blue.svg" alt="Version"></a>
</p>

`lls` は、LLM やエージェントがリポジトリやディレクトリを探索しやすくするための CLI です。  
通常の `ls` が「何があるか」を並べるのに対して、`lls` は「何が重要か」「何を後回しにしてよいか」「次にどこを見るべきか」を返します。

## Usage

```bash
# カレントディレクトリを走査（デフォルトは depth=1）
lls

# パス指定
lls src
lls ./packages/api

# JSON 出力（LLM やツール連携向け）
lls --json

# コンパクト JSON（トークン節約向け）
lls --json --compact

# 探索深度の指定
lls --depth 0  # 対象パスのみ
lls --depth 1  # 直接の子要素（デフォルト）
lls --depth 2  # 子と孫まで
```

## Example Output

### Human-readable

```
$ lls
Project: rust_cli (confidence: 0.9)
Path: .

Recommended next steps:
1. read README.md - プロジェクト概要を把握するため最初に読むべき
2. read Cargo.toml - パッケージのメタデータと依存関係を確認するため
3. inspect src - メインのソースコードが含まれる可能性が高いため
4. inspect .github - CI/CD の設定を確認するため
5. inspect tests - テストコードを確認するため

Entries:
[critical ] Cargo.toml           file         manifest                188 B Rust のマニフェストファイル
[critical ] README.md            file         project_overview       3606 B プロジェクト概要や使い方が書かれている可能性が高い
[high     ] .github              directory    ci_config           CI/CD 設定ディレクトリ
[high     ] src                  directory    source_code         メインのソースコードディレクトリ
[medium   ] .gitignore           file         config                  712 B Git の除外設定ファイル
[medium   ] Cargo.lock           file         lockfile               6483 B 依存関係の固定ファイル [generated]
[medium   ] tests                directory    test_code           テストコードを含むディレクトリ
[low      ] LICENSE              file         license                1077 B ライセンスファイル
[ignore   ] target               directory    build_output        Rust のビルド成果物 [generated]
[ignore   ] .git                 directory    dependency_cache    Git の内部ディレクトリ [generated]
```

### JSON

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
    "total_entries": 10,
    "shown_entries": 10,
    "important_entries": 8,
    "ignored_entries": 2
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
      "size_bytes": 3606
    }
  ],
  "recommended_next_steps": [],
  "warnings": []
}
```

## What `lls` Returns

各エントリには次の情報が付与されます。

| フィールド | 説明 |
|-----------|------|
| `name` | ファイル名またはディレクトリ名 |
| `path` | 対象パスからの相対パス |
| `type` | `file` / `directory` / `symlink` / `other` |
| `role` | 推定された役割（後述） |
| `priority` | 探索上の重要度（`critical` > `high` > `medium` > `low` > `ignore`） |
| `reason` | 判定理由 |
| `generated` | 生成物・ビルド成果物か |
| `sensitive` | 秘密情報を含む可能性があるか |
| `text` / `binary` | テキスト/バイナリ判定 |
| `size_bytes` | ファイルサイズ |

### Role 一覧

| role | 意味 | 例 |
|------|------|-----|
| `project_overview` | プロジェクト概要 | README.md |
| `manifest` | パッケージ定義 | Cargo.toml, package.json |
| `lockfile` | 依存関係固定ファイル | Cargo.lock, package-lock.json |
| `source_code` | メインのソースコード | src/, main.rs |
| `test_code` | テストコード | tests/, *.test.ts |
| `documentation` | ドキュメント | docs/, CHANGELOG.md |
| `config` | 設定ファイル | .gitignore, tsconfig.json |
| `ci_config` | CI/CD 設定 | .github/workflows/ |
| `build_output` | ビルド成果物 | target/, dist/, build/ |
| `dependency_cache` | 依存物・キャッシュ | node_modules/, vendor/ |
| `generated` | 生成ファイル | *.min.js, *.svg |
| `secret_candidate` | 秘密情報候補 | .env, *.pem |
| `license` | ライセンス | LICENSE |
| `unknown` | 不明 | 分類できないもの |

### プロジェクト種別推定

| 種別 | 判定条件 |
|------|---------|
| `rust_cli` | Cargo.toml + src/main.rs |
| `rust_library` | Cargo.toml + src/lib.rs |
| `rust_package` | Cargo.toml + src/ |
| `node_project` | package.json |
| `python_package` | pyproject.toml / setup.py |
| `go_module` | go.mod |
| `monorepo` | pnpm-workspace.yaml / turbo.json / nx.json |
| `unknown` | 判定不能 |

## Warnings

`lls` は次のような警告を出力します。

- 秘密情報候補ファイル（`.env` など）の存在 — 内容を読まないよう促す
- 壊れたシンボリックリンク
- 一部エントリの権限不足など

## Document Roles

このリポジトリでは、ドキュメントの役割を次のように分けます。

- `README.md`: プロジェクトの概要、目的、使い方
- `.github/assets/spec.md`: MVP の仕様定義
- `.github/assets/feature-spec.md`: 将来的な拡張案

## Development

```bash
# ビルド
cargo build --release

# テスト
cargo test

# clippy（CI と同じチェック）
cargo clippy -- -D warnings
```

## License

MIT
