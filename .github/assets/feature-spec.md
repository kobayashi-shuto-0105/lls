# lls Feature Spec

このドキュメントは、`lls` の将来的な機能候補や拡張案を管理するためのメモである。

MVP として実装する仕様は [`spec.md`](spec.md) に定義する。  
このドキュメントに書かれている内容は、初期実装では必須ではない。

---

## 1. この文書の役割

この文書の役割は次の通り。

- MVP に入れない機能候補を残す
- 将来的な出力スキーマの拡張案を整理する
- CLI オプションの候補を管理する
- 実装時に消えやすいアイデアを残す
- コア仕様である `spec.md` を肥大化させない

ここに書かれている機能は、実装が確定しているわけではない。

---

## 2. ステータス

将来機能には、必要に応じて次のステータスを付ける。

| status | 意味 |
|---|---|
| `candidate` | 候補。実装するか未確定 |
| `planned` | 将来的に実装する予定 |
| `blocked` | 他の設計や機能に依存している |
| `rejected` | 実装しない判断をした |
| `done` | 実装済み |

---

## 3. フェーズ案

| Phase | 内容 |
|---|---|
| Phase 1 | MVP。role, priority, JSON, recommendation |
| Phase 2 | 推定精度とメタデータの拡張 |
| Phase 3 | Git や monorepo 対応 |
| Phase 4 | ルールのカスタマイズと拡張性 |

Phase 1 の詳細は `spec.md` に定義する。

---

## 4. 将来機能候補

| 機能 | status | 優先度 | 内容 |
|---|---|---:|---|
| YAML 出力 | candidate | low | `--yaml` で YAML 出力する |
| NDJSON 出力 | candidate | low | 1 行 1 JSON のストリーミング向け出力 |
| ディレクトリ要約 | candidate | high | ディレクトリ配下を全部列挙せず要約する |
| token awareness | candidate | high | LLM が読むには大きすぎるファイルを検出する |
| 行数カウント | candidate | medium | テキストファイルの行数を返す |
| git-aware mode | candidate | high | modified, staged, untracked などを返す |
| monorepo detection | candidate | high | workspace や packages を推定する |
| 設定ファイル対応 | candidate | medium | ユーザー定義ルールを読み込む |
| sensitive detection v2 | candidate | high | 秘密情報候補の検出精度を上げる |
| generated detection v2 | candidate | medium | 生成物判定を強化する |
| action extraction | candidate | medium | manifest から実行可能コマンドを抽出する |
| language detection | candidate | medium | ファイルやディレクトリの言語を推定する |
| entrypoint detection | candidate | medium | main file や起点ファイルを推定する |
| explanation mode | candidate | low | 理由説明の詳細度を切り替える |

---

## 5. 出力スキーマ拡張案

将来的には、トップレベル JSON に次のような項目を追加できる。

```json
{
  "schema_version": "0.2.0",
  "path": ".",
  "workspace": {},
  "project_type": {},
  "summary": {},
  "entries": [],
  "recommended_next_steps": [],
  "actions": [],
  "warnings": [],
  "diagnostics": {}
}
```

---

## 6. 追加候補のトップレベルフィールド

| フィールド | 内容 |
|---|---|
| `workspace` | monorepo や workspace の情報 |
| `actions` | manifest などから抽出した実行可能コマンド |
| `diagnostics` | 判定ルールやスコアなどのデバッグ情報 |
| `config` | 実際に使われた設定 |
| `performance` | 実行時間や走査件数など |

---

## 7. 追加候補の Entry フィールド

将来的に entry に追加する候補は次の通り。

| フィールド | 型 | 内容 |
|---|---|---|
| `line_count` | number | テキストファイルの行数 |
| `estimated_tokens` | number | 推定トークン数 |
| `language` | string | 推定された言語 |
| `languages` | array | ディレクトリ内の主な言語 |
| `git_status` | string | Git 上の状態 |
| `last_modified` | string | 最終更新日時 |
| `children_summary` | object | ディレクトリ配下の要約 |
| `entrypoint` | boolean | 起点ファイルと推定されるか |
| `workspace_member` | boolean | workspace のメンバーか |
| `confidence` | number | role 判定の信頼度 |
| `matched_rules` | array | 適用された判定ルール |

---

## 8. ディレクトリ要約

### 8.1 目的

巨大なディレクトリをすべて列挙しても、LLM にとっては扱いにくい。

例:

- `src/`
- `packages/`
- `apps/`
- `crates/`
- `docs/`

そのため、将来的にはディレクトリ配下を要約できるようにする。

---

### 8.2 出力例

```json
{
  "name": "src",
  "path": "src",
  "type": "directory",
  "role": "source_code",
  "priority": "high",
  "summary": {
    "total_entries": 12,
    "languages": ["Rust"],
    "notable_files": ["main.rs", "cli.rs", "scanner.rs"],
    "dominant_roles": ["source_code"]
  }
}
```

---

## 9. token awareness

### 9.1 目的

LLM が巨大なファイルや低価値なファイルを不用意に読まないようにする。

検出したいものの例:

- minified JS
- 巨大 JSON
- 大きすぎる lockfile
- vendored code
- snapshot file
- binary file

---

### 9.2 出力候補

```json
{
  "size_bytes": 1200000,
  "line_count": 30000,
  "estimated_tokens": 180000,
  "read_risk": "too_large"
}
```

---

### 9.3 read_risk 候補

| 値 | 意味 |
|---|---|
| `safe` | 読んでも問題なさそう |
| `large` | 必要な場合のみ読むべき |
| `too_large` | 全文読み取りは避けるべき |
| `binary` | テキストとして読むのに向かない |
| `sensitive` | 秘密情報を含む可能性がある |
| `generated` | 生成物であり読む優先度が低い |

---

## 10. git-aware mode

### 10.1 CLI 案

```sh
lls --json --git-aware
```

---

### 10.2 目的

作業中のリポジトリでは、最近変更されたファイルや未追跡ファイルが重要なことが多い。

git-aware mode では、次の情報を entry に追加する。

- modified
- staged
- untracked
- deleted
- renamed
- clean

---

### 10.3 出力例

```json
{
  "name": "src/main.rs",
  "path": "src/main.rs",
  "git_status": "modified"
}
```

---

### 10.4 git_status 候補

| 値 | 意味 |
|---|---|
| `modified` | 変更済み |
| `staged` | staging 済み |
| `untracked` | Git 管理外 |
| `deleted` | 削除済み |
| `renamed` | リネーム済み |
| `clean` | 変更なし |
| `unknown` | Git 状態を取得できない |

---

## 11. monorepo 対応

### 11.1 目的

monorepo は単一パッケージのリポジトリと探索方法が異なる。

よくある構成:

```txt
apps/
packages/
crates/
services/
libs/
```

---

### 11.2 判定材料

| シグナル | 意味 |
|---|---|
| `pnpm-workspace.yaml` | pnpm workspace |
| `turbo.json` | Turborepo |
| `nx.json` | Nx workspace |
| workspace 設定を含む `Cargo.toml` | Rust workspace |
| 複数の `package.json` | Node.js 系 monorepo の可能性 |

---

### 11.3 出力例

```json
{
  "workspace": {
    "type": "pnpm_workspace",
    "members": ["apps/web", "packages/ui", "packages/api"],
    "confidence": 0.9
  }
}
```

---

## 12. action extraction

### 12.1 目的

LLM エージェントが次の行動を決めるには、実行可能なコマンドが分かると便利である。

例:

- `npm run test`
- `cargo test`
- `cargo build`
- `pnpm dev`

---

### 12.2 出力例

```json
{
  "actions": [
    {
      "name": "test",
      "command": "cargo test",
      "source": "Cargo.toml"
    },
    {
      "name": "build",
      "command": "cargo build",
      "source": "Cargo.toml"
    }
  ]
}
```

---

## 13. 設定ファイル対応

### 13.1 目的

プロジェクトによって重要なファイルは異なる。

例:

- ドキュメント中心のプロジェクトでは `docs/` が重要
- アプリケーションでは `src/` が重要
- 生成コード中心のリポジトリでは generated file を完全には無視できない

そのため、将来的には設定ファイルでルールを変更できるようにする。

---

### 13.2 設定ファイル名候補

```txt
.lls.toml
lls.toml
```

---

### 13.3 設定例

```toml
[priority]
"docs/" = "critical"
"examples/" = "high"

[ignore]
patterns = ["tmp/", "*.snapshot"]

[sensitive]
patterns = ["*.secret.json"]
```

---

## 14. CLI オプション候補

| オプション | 内容 |
|---|---|
| `--yaml` | YAML 出力 |
| `--ndjson` | NDJSON 出力 |
| `--max-entries 100` | 最大出力件数を制限 |
| `--include-ignored` | ignore 対象も詳細に出す |
| `--no-sensitive-warning` | sensitive warning を抑制 |
| `--git-aware` | Git 状態を含める |
| `--tokens` | 推定トークン数を含める |
| `--explain` | 判定ルールやスコアを含める |
| `--config <path>` | 明示的に設定ファイルを指定 |
| `--no-config` | 設定ファイルを無視する |

---

## 15. スコアリング方式の候補

MVP では単純なルールベースでよい。

将来的には、スコアリングによって priority を決めてもよい。

例:

```txt
README.md          +100
manifest file      +90
src/               +80
tests/             +40
docs/              +30
build output       -100
dependency cache   -100
sensitive file     read recommendation から除外
```

出力例:

```json
{
  "priority_score": 95,
  "matched_rules": [
    "name:README.md",
    "role:project_overview"
  ]
}
```

---

## 16. 慎重に扱うべき案

次の案は便利だが、注意して扱う。

| 案 | 懸念 |
|---|---|
| デフォルトで全ソースファイルを読む | 遅く、ノイズが多い |
| `lls` 内部で LLM を使う | 非決定的でテストしにくい |
| symlink をデフォルトで辿る | ループや想定外の探索が起きる |
| sensitive file を完全に隠す | 存在を把握できなくなる |
| pretty output を主 API にする | 機械的に扱いづらくなる |

---

## 17. 設計原則

将来機能を追加する場合も、次の原則を守る。

- `lls` は完全理解ではなく、探索支援を目的にする
- JSON 出力は安定させる
- sensitive file は存在を表示するが、読む候補にはしない
- generated file や dependency cache は原則として優先度を下げる
- ルールはできるだけ決定的にする
- fixture を使ってテストできる設計にする
