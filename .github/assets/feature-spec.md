# lls Feature Spec

このドキュメントは、`lls` の MVP 後に検討する機能候補を管理する。  
MVP の正式仕様は [`spec.md`](spec.md) であり、本書に重複記載しない。

---

## 1. status

| status | 意味 |
|---|---|
| `candidate` | 候補 |
| `planned` | 実装予定 |
| `blocked` | 依存事項あり |
| `rejected` | 実装しない |
| `done` | 実装済みまたは MVP へ移動済み |

---

## 2. phase

| Phase | 内容 |
|---|---|
| Phase 1 | MVP。設定、Codex-assisted setup、role、priority、JSON、recommendation |
| Phase 2 | メタデータ、出力形式、判定精度 |
| Phase 3 | Git、monorepo、workspace |
| Phase 4 | plugin、scoring、内容閲覧 |

設定ファイル `.lls/config.json` と ChatGPT login による Codex-assisted setup は Phase 1 に含まれる。

---

## 3. 機能候補

| 機能 | status | 優先度 | 内容 |
|---|---|---:|---|
| YAML 出力 | candidate | low | `--yaml` |
| NDJSON 出力 | candidate | low | 1 行 1 JSON |
| directory summary | candidate | high | 子孫の要約 |
| token awareness | candidate | high | 巨大ファイル検出 |
| line count | candidate | medium | 行数 |
| git-aware mode | candidate | high | modified / staged / untracked |
| monorepo detection | candidate | high | workspace と member 推定 |
| sensitive detection v2 | candidate | high | 検出精度向上 |
| generated detection v2 | candidate | medium | 生成物判定強化 |
| action extraction | candidate | medium | manifest から command 抽出 |
| language detection | candidate | medium | 言語推定 |
| entrypoint detection | candidate | medium | 起点推定 |
| content view mode | candidate | high | redaction 付き内容閲覧 |
| explanation mode | candidate | low | 判定詳細 |
| README tagline | candidate | low | long listing の補助 |
| PDF metadata title | candidate | low | long listing の補助 |
| ignore file integration | candidate | medium | `.gitignore` / `.dockerignore` |
| recent marker | candidate | low | birth time / ctime による `🆕` |
| scoring-based priority | candidate | medium | rule score |
| plugin rules | candidate | medium | 外部ルール追加 |
| Codex setup refinement | candidate | medium | prompt、schema、再試行戦略の改善 |
| OpenAI Platform API key auth | rejected | - | MVP 方針により対応しない |

---

## 4. monorepo

MVP では `project_type: monorepo` を返さない。  
将来は次を evidence として扱える。

- `pnpm-workspace.yaml`
- `turbo.json`
- `nx.json`
- workspace 設定を含む `Cargo.toml`
- 複数の `package.json`
- `apps/`, `packages/`, `crates/`, `services/`

候補出力:

```json
{
  "workspace": {
    "type": "pnpm_workspace",
    "members": ["apps/web", "packages/ui"],
    "confidence": 0.9
  }
}
```

---

## 5. git-aware mode

```sh
lls --git-aware
```

候補 field:

```json
{
  "git_status": "modified"
}
```

候補値:

```txt
modified
staged
untracked
deleted
renamed
clean
unknown
```

---

## 6. token awareness

候補 field:

```json
{
  "line_count": 30000,
  "estimated_tokens": 180000,
  "read_risk": "too_large"
}
```

候補値:

```txt
safe
large
too_large
binary
sensitive
generated
```

---

## 7. advanced long listing

MVP の `-l` は priority、role、type、size、path、属性だけを表示する。  
将来、次を追加できる。

- README tagline
- PDF metadata title
- ignore file integration
- recent marker
- git status
- line count
- estimated token count

各機能は個別に無効化できることが望ましい。

---

## 8. content view mode

候補:

```sh
lls cat <path>
```

基本方針:

- redaction を既定で有効
- binary と巨大ファイルを拒否または制限
- API key、Bearer token、JWT、private key、`.env` value をマスク
- `--raw` は強い警告を伴う明示 opt-in
- 一覧モードとは別 command とする

---

## 9. scoring

MVP は fixed precedence rule を使う。  
将来は score を diagnostics として追加できる。

```json
{
  "priority_score": 95,
  "matched_rules": [
    "name:README.md",
    "role:project_overview"
  ]
}
```

score を導入しても、次は score より優先する。

- sensitive を recommendation から除外
- `.git/` を ignore
- binary を recommendation から除外

---

## 10. Codex 拡張

MVP の Codex 利用は setup 時の設定案生成だけである。  
将来候補:

- setup prompt versioning
- setup 結果の diagnostics
- retry / repair loop
- 生成案と built-in 案の diff
- model capability negotiation
- Enterprise access token を用いた trusted automation

引き続き、`lls` が Codex の credential file を直接読む設計は採用しない。

---

## 11. 設計原則

- MVP 仕様を本書へ重複させない
- JSON の互換性を優先する
- sensitive は存在を表示しても読む候補にしない
- generated と role を混同しない
- future feature は deterministic core を壊さない
- fixture と fake process でテスト可能にする
