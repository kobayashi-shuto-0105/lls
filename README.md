# lls

**`lls` = `ls` for LLMs**

`lls` は、LLM やエージェントがファイルシステムを探索しやすくするための、**LLM向けディレクトリ要約CLI**です。

通常の `ls` が「何があるか」を表示するのに対して、`lls` は **何が重要か / 何を無視すべきか / 次にどこを見るべきか** を返します。

> 名前を並べるためのコマンドではなく、  
> **探索計画を立てやすくするためのコマンド**を目指しています。

---

## Overview

LLM がリポジトリやディレクトリを探索するとき、普通の `ls` だけでは情報が足りません。

たとえば次のような一覧があったとしても:

```text
src/
lib/
tmp/
data/
main.ts
run.sh
v2_final_really_final.txt
```

LLM にとっては、次の判断が難しいことがあります。
	•	どれが重要か
	•	どこから読むべきか
	•	どれがソースコードか
	•	どれが生成物か
	•	どれが設定ファイルか
	•	どれを無視してよいか

さらに、以下のようなノイズもあります。
	•	node_modules/
	•	.git/
	•	dist/
	•	build/
	•	coverage/
	•	.next/

lls は、こうした問題に対して、構造化された意味付きの一覧を返すことで対応します。

⸻

Why

普通の ls は人間向けです。
lls は、LLM やエージェントが次の行動を選びやすくするためのツールです。

設計の中心にある考え方は次のとおりです。
	•	listing より navigation
	•	names より meaning
	•	exhaustive output より priority + compression
	•	human-first formatting より machine-stable structure

つまり、lls は単なる一覧ツールではなく、探索支援ツールです。

⸻

Goals

lls の主な目標:
	•	構造が壊れにくい出力を返す
	•	ファイルやディレクトリの役割を推定する
	•	探索優先度を付与する
	•	ノイズを識別して下げる
	•	次に読むべき候補を提案する
	•	プロジェクト種別を推定する
	•	トークン消費を抑えやすい出力を作る

⸻

Non-Goals

初期段階では、以下は重視しません。
	•	派手な色付け
	•	過剰に凝ったツリー表示
	•	最初から全ファイルを深く解析すること
	•	IDE の代替になること
	•	曖昧な自然文だけで返すこと

⸻

Features

1. Structured output

LLM が安定して扱えるように、構造化出力を重視します。

想定フォーマット:
	•	JSON
	•	YAML
	•	NDJSON

例:

{
  "path": ".",
  "entries": [
    {
      "name": "src",
      "type": "directory",
      "priority": "high",
      "reason": "application source directory"
    },
    {
      "name": "package.json",
      "type": "file",
      "priority": "critical",
      "reason": "node project manifest"
    }
  ]
}


⸻

2. Role inference

ファイル名を並べるだけでなく、役割を推定します。

例:
	•	README.md → project_overview
	•	package.json → project_manifest
	•	Cargo.toml → rust_package_manifest
	•	src/ → source_code
	•	tests/ → test_code
	•	target/ → build_output

⸻

3. Priority tagging

LLM が「どこから見るべきか」を判断しやすいように、優先度を付与します。

想定ラベル:
	•	critical
	•	high
	•	medium
	•	low
	•	ignore

例:
	•	README.md → critical
	•	Cargo.toml → critical
	•	src/ → high
	•	tests/ → high
	•	target/ → low
	•	.git/ → ignore

⸻

4. Noise detection

LLM が不要な情報に引っ張られないように、ノイズ源を識別します。

例:
	•	.git/
	•	node_modules/
	•	target/
	•	dist/
	•	build/
	•	coverage/
	•	.cache/

重要なのは、消すのではなく、重要度を下げることです。

⸻

5. Recommended next steps

一覧を返すだけで終わらず、次に取るべき行動を提案します。

例:

{
  "recommended_next_steps": [
    "Read README.md",
    "Inspect Cargo.toml",
    "Explore src/"
  ]
}


⸻

6. Project type inference

ディレクトリ全体からプロジェクト種別を推定します。

例:
	•	Rust CLI
	•	Rust library
	•	Node.js application
	•	TypeScript package
	•	Python package
	•	Next.js application
	•	Monorepo

例:

{
  "project_type": {
    "name": "Rust CLI application",
    "confidence": 0.97
  }
}


⸻

7. Directory summarization

ディレクトリの中身を、そのまま全部列挙する代わりに要約できます。

例:

{
  "name": "src",
  "type": "directory",
  "summary": {
    "file_count": 8,
    "languages": ["Rust"],
    "dominant_kinds": ["cli", "parser", "output"]
  }
}


⸻

8. Size / token awareness

巨大ファイルや LLM に不向きなファイルを先に検知します。

例:
	•	size_bytes
	•	line_count
	•	estimated_tokens
	•	binary
	•	text
	•	minified
	•	generated

⸻

9. Sensitive file detection

扱いに注意すべきファイルを識別します。

例:
	•	.env
	•	.pem
	•	id_rsa
	•	credentials.json
	•	secrets.yaml

⸻

10. Git-aware mode

コード修正やレビュー用途のために Git 状況を加味できるようにします。

例:
	•	modified
	•	staged
	•	untracked
	•	recently_changed

⸻

Example

{
  "path": ".",
  "project_type": {
    "name": "Rust CLI application",
    "confidence": 0.97
  },
  "summary": {
    "total_entries": 10,
    "important_entries": 5,
    "ignored_entries": 2
  },
  "entries": [
    {
      "name": "README.md",
      "type": "file",
      "role": "project_overview",
      "priority": "critical",
      "reason": "likely contains high-level documentation",
      "size_bytes": 2911,
      "text": true
    },
    {
      "name": "Cargo.toml",
      "type": "file",
      "role": "rust_package_manifest",
      "priority": "critical",
      "reason": "defines package metadata and dependencies",
      "text": true
    },
    {
      "name": "src",
      "type": "directory",
      "role": "source_code",
      "priority": "high",
      "summary": {
        "languages": ["Rust"],
        "dominant_kinds": ["cli", "parser", "output"]
      }
    },
    {
      "name": "target",
      "type": "directory",
      "role": "build_output",
      "priority": "ignore",
      "reason": "generated build artifacts"
    }
  ],
  "recommended_next_steps": [
    "Read README.md",
    "Inspect Cargo.toml",
    "Explore src/"
  ]
}


⸻

CLI Sketch

Basic

lls

人間向けの見やすい出力。

JSON

lls --json

LLM や他ツール向けの構造化出力。

Compact

lls --json --compact

重要な情報を中心に圧縮表示。

Agent mode

lls --json --agent

推奨アクション付きの出力。

Depth limit

lls --json --depth 2

探索深度を制限。

Git-aware

lls --json --git-aware

Git 状況を加味。

⸻

Output Schema Direction

現時点で想定している代表的なキー:

Top-level
	•	path
	•	project_type
	•	summary
	•	entries
	•	recommended_next_steps

Entry fields
	•	name
	•	path
	•	type (file | directory | symlink)
	•	role
	•	priority (critical | high | medium | low | ignore)
	•	reason
	•	generated
	•	sensitive
	•	text
	•	binary
	•	size_bytes
	•	line_count
	•	estimated_tokens
	•	git_status
	•	summary

曖昧なフィールド名ではなく、意味が明確で安定した名前を優先します。

⸻

Rust

lls は Rust で実装する想定です。

Rust を選ぶ理由:
	•	単一バイナリで配布しやすい
	•	CLI ツールとして高速
	•	ファイルシステム走査との相性がよい
	•	構造化データの扱いがしやすい
	•	型で出力スキーマを安定させやすい

⸻

Development

Build

cargo build

Run

cargo run -- --json

Release build

cargo build --release

Example

cargo run -- --json --compact --agent


⸻

Installation

将来的には以下のような導線を想定しています。

From source

git clone <repo-url>
cd lls
cargo install --path .

Local development

cargo run -- --help

まだ公開方法が未確定なら、ここは後で更新すれば十分です。

⸻

MVP

最初の実装で優先する範囲:
	•	JSON 出力
	•	type 判定
	•	role 推定
	•	priority 推定
	•	ignore 候補の識別
	•	recommended_next_steps 生成

この 5〜6 点だけでも、通常の ls よりかなり LLM フレンドリーになります。

⸻

Roadmap

Phase 1
	•	JSON output
	•	type classification
	•	role inference
	•	priority tagging
	•	ignore detection
	•	recommended next steps

Phase 2
	•	project type inference
	•	directory summary
	•	generated/binary/minified detection
	•	size / line count / token estimate
	•	sensitive file detection

Phase 3
	•	git-aware mode
	•	monorepo support
	•	workspace detection
	•	action extraction from manifests
	•	configurable scoring rules

⸻

Comparison

ls
	•	人間向け
	•	名前を並べる
	•	アルファベット順中心
	•	意味づけが弱い
	•	LLM にはノイズが多い

lls
	•	LLM / agent 向け
	•	重要度と役割を返す
	•	次の行動につながる
	•	ノイズを識別する
	•	探索計画を立てやすい
	•	トークン節約を意識する

⸻

Design Principles

1. Listing is not enough

lls は「何があるか」だけではなく、
何が重要か / 何を無視すべきか / 次に何をすべきか を返します。

2. Meaning over names

ファイル名の列挙より、意味づけを優先します。

3. Compression over exhaustiveness

最初から全部を出すのではなく、要約と優先度付けを重視します。

4. Stable structure over pretty output

見た目よりも、壊れにくく機械処理しやすい出力を優先します。

5. Human-friendly and agent-friendly

人間向け表示とエージェント向け表示の両立を目指します。

⸻

Status

設計・初期実装フェーズ。

現在の主眼は次の 3 点です。
	•	CLI の表面仕様を固める
	•	出力スキーマを安定させる
	•	MVP を小さく実装する

⸻

License

MIT
