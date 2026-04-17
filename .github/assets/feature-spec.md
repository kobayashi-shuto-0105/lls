# lls Feature Spec

このドキュメントは、`lls` にこれから追加していきたい仕様や拡張案をためていくためのメモです。  
`spec.md` がコア要件を整理する文書なのに対して、ここでは「将来こうしたい」をラフに持っておきます。

## この文書の役割

- まだ MVP に入れないが、今後やりたい機能を残す
- 出力スキーマや CLI オプションの候補を先に言語化する
- 実装時に消えやすいアイデアを雑にでも残しておく

## 追加していきたい機能

現時点で将来的に入れたい機能候補は次のあたりです。

- 構造化出力の拡張
  - JSON だけでなく YAML や NDJSON も視野に入れる
- role inference の強化
  - `README.md`、`Cargo.toml`、`package.json`、`src/`、`tests/` などの役割推定を増やす
- priority tagging の精度向上
  - critical / high / medium / low / ignore をより一貫して返せるようにする
- noise detection の拡張
  - `.git/`、`node_modules/`、`target/`、`dist/`、`build/`、`coverage/` などをうまく下げる
- recommended next steps の改善
  - 一覧を返すだけでなく、次に読むべきファイルやディレクトリを提案する
- project type inference
  - Rust CLI、Rust library、Node.js app、Python package、monorepo などの推定
- directory summarization
  - ディレクトリ配下を全部列挙せず、言語や用途を要約する
- size / token awareness
  - 巨大ファイルや minified file を検出し、先に注意を出せるようにする
- sensitive file detection
  - `.env`、秘密鍵、認証情報っぽいファイルを識別する
- git-aware mode
  - modified / staged / untracked / recently_changed のような情報を使えるようにする

## 追加したい出力項目

将来的には、トップレベルで次のような情報を返せるとよいです。

- `path`
- `project_type`
- `summary`
- `entries`
- `recommended_next_steps`

各 entry では、次のような項目を候補にしておきます。

- `name`
- `path`
- `type`
- `role`
- `priority`
- `reason`
- `generated`
- `sensitive`
- `text`
- `binary`
- `size_bytes`
- `line_count`
- `estimated_tokens`
- `git_status`
- `summary`

命名は見た目のかっこよさより、意味が明確で壊れにくいことを優先します。

## 出力イメージ

最終形では、次のような JSON を返せるとよいです。

```json
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
    }
  ],
  "recommended_next_steps": [
    "Read README.md",
    "Inspect Cargo.toml",
    "Explore src/"
  ]
}
```

## CLI の候補

将来的にほしい CLI の使い方メモです。

- `lls`
  - 人間向けの見やすい表示
- `lls --json`
  - LLM や他ツール向けの構造化出力
- `lls --json --compact`
  - 重要な情報を中心に圧縮表示
- `lls --json --agent`
  - 推奨アクション付きの出力
- `lls --json --depth 2`
  - 探索深度を制限
- `lls --json --git-aware`
  - Git 状況を加味

## フェーズ分けの案

### Phase 1

- JSON output
- type classification
- role inference
- priority tagging
- ignore detection
- recommended next steps

### Phase 2

- project type inference
- directory summary
- generated / binary / minified detection
- size / line count / token estimate
- sensitive file detection

### Phase 3

- git-aware mode
- monorepo support
- workspace detection
- action extraction from manifests
- configurable scoring rules

## 設計メモ

実装時に忘れたくない方向性です。

- listing より navigation を優先する
- names より meaning を返す
- exhaustive output より priority と compression を重視する
- human-friendly と agent-friendly を両立したい
- pretty output より machine-stable structure を優先する

## 備考

ここに書いてあるものは確定仕様ではなく、将来の候補です。  
MVP に入れるかどうか、優先順位が高いかどうかは、その時点で見直してよいです。
