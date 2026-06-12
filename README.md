<p align="center">
	<img src=".github/assets/lls_logo.png" alt="lls logo" width="240">
</p>

<h1 align="center">lls</h1>

<p align="center"><strong><code>lls</code> = <code>ls</code> for LLMs</strong></p>

<p align="center">
  <a href="https://github.com/kobayashi-shuto-0105/lls/actions/workflows/build.yaml"><img src="https://github.com/kobayashi-shuto-0105/lls/actions/workflows/build.yaml/badge.svg" alt="build"></a>
  <a href="https://coveralls.io/github/kobayashi-shuto-0105/lls?branch=main"><img src="https://coveralls.io/repos/github/kobayashi-shuto-0105/lls/badge.svg?branch=main" alt="Coverage Status"></a>
  <!--
    Version badge:
    - `.github/scripts/update_version.sh` が README 内のバージョン表記を自動更新できるようにするためのものです。
    - このリポジトリでは `${VERSION}` というプレースホルダを置換して README を生成する想定です。
  -->
  <a href="https://github.com/kobayashi-shuto-0105/lls/releases/tag/v0.1.0"><img src="https://img.shields.io/badge/Version-0.1.0-blue.svg" alt="Version"></a>
</p>

`lls` は、LLM やエージェントがリポジトリやディレクトリを探索しやすくするための CLI です。  
通常の `ls` が「何があるか」を並べるのに対して、`lls` は「何が重要か」「何を後回しにしてよいか」「次にどこを見るべきか」を返すことを目指します。

## Overview

リポジトリ探索では、単なるファイル名の一覧だけでは判断材料が足りません。  
特に LLM にとっては、次のような点が最初のボトルネックになります。

- どれが主要なソースコードか
- どれが設定ファイルか
- どれが生成物やノイズか
- どこから読めば全体像をつかみやすいか

`lls` はこの問題に対して、意味付きで優先度のある一覧を返すことで対応します。

## What `lls` Tries To Return

想定している出力の方向性は次のとおりです。

- 重要なファイルやディレクトリの抽出
- 役割の推定
- 優先度の付与
- ノイズの識別
- 次に読むべき候補の提案
- LLM が扱いやすい構造化出力

たとえば `README.md` や `Cargo.toml` は高優先度、`target/` や `.git/` は低優先度または無視対象として扱う、というような出し分けを想定しています。

## Document Roles

このリポジトリでは、ドキュメントの役割を次のように分けます。

- `README.md`: プロジェクトの概要、目的、読み始める人向けの入口
- `.github/assets/spec.md`: 何を作るか、入力と出力は何か、どこまでを最初のスコープにするかを整理する仕様メモ
- `.github/assets/feature-spec.md`: 今後追加したい機能、拡張案、出力スキーマの候補をためていくメモ

詳細な要求整理は [`.github/assets/spec.md`](.github/assets/spec.md) を参照してください。  
将来機能のメモは [`.github/assets/feature-spec.md`](.github/assets/feature-spec.md) に分けて管理します。

## Current Status

**MVP implementation in progress.** All core modules are implemented and tested.

## Installation

```bash
# Build from source
git clone https://github.com/kobayashi-shuto-0105/lls.git
cd lls
cargo build --release
# Binary at target/release/lls
```

Install via `cargo` (once published):
```bash
cargo install lls
```

## Usage

### Basic listing (requires config or `--no-config`)

```bash
# Use built-in defaults (no config file needed)
lls --no-config

# With automatic config discovery
lls setup --without-codex  # create .lls/config.json
lls                         # use discovered config
```

### Output modes

```bash
lls --json       # compact JSON (default)
lls --human      # human-readable text
lls -l           # long listing format
```

### Options

```bash
lls <path>                   # scan a specific path
lls --depth <0-8>            # set scan depth (default: 1)
lls --sort <name|mtime|size|priority>
lls --config <path>          # use explicit config file
lls --no-config              # skip config discovery
```

### Setup

```bash
lls setup                    # generate config with Codex assist
lls setup --without-codex    # generate config from built-in defaults
lls setup --force            # overwrite existing config
lls setup --yes              # skip confirmation prompt
```

## Output Example

```json
{"schema_version":"0.1.0","path":".","project_type":{"name":"rust_cli","confidence":0.95,"evidence":["Cargo.toml","src/main.rs"]},"summary":{"total_entries":7,"shown_entries":7,"important_entries":4,"ignored_entries":2},"entries":[{"name":"Cargo.toml","path":"Cargo.toml","type":"file","role":"manifest","priority":"critical","reason_code":"known_manifest","reason":"マニフェストファイル","generated":false,"sensitive":false,"text":true,"binary":false,"size_bytes":1024},{"name":"README.md","path":"README.md","type":"file","role":"project_overview","priority":"critical","reason_code":"project_overview","reason":"プロジェクト概要","generated":false,"sensitive":false,"text":true,"binary":false,"size_bytes":512},{"name":"src","path":"src","type":"directory","role":"source_code","priority":"high","reason_code":"source_code_directory","reason":"ソースコード","generated":false,"sensitive":false,"text":false,"binary":false}],"recommended_next_steps":[{"action":"read","path":"README.md","reason_code":"read_project_overview_first","reason":"プロジェクト概要を把握するため"},{"action":"read","path":"Cargo.toml","reason_code":"read_manifest_first","reason":"プロジェクト構成を理解するため"}],"warnings":[]}
```

## Exit Codes

| Code | Meaning |
|-----:|---------|
| `0` | Success |
| `1` | CLI argument error |
| `2` | Target path not found |
| `3` | Permission denied |
| `4` | Unexpected runtime error |
| `5` | Setup required (no config found) |
| `6` | Codex CLI failure |
| `7` | Invalid configuration |

## Development

```bash
# Run tests
cargo test --all-targets --all-features

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Format
cargo fmt --all -- --check
```
