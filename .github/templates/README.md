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
  <a href="https://github.com/kobayashi-shuto-0105/lls/releases/tag/v${VERSION}">
    <img src="https://img.shields.io/badge/Version-${VERSION}-blue.svg" alt="Version">
  </a>
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

現状は Rust 製 CLI の最小構成です。

- `Cargo.toml`: Rust パッケージ定義
- `src/main.rs`: エントリポイント

実装はまだこれからで、まずは「どんなツールにするか」を整理してから形にしていく段階です。

## Development

将来的には、次のような使い方を想定しています。

```bash
cargo run
```

実装が進んだら、ここに実行例やオプションを追記します。
