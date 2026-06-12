# ADR-0001: Deterministic core and Codex setup boundary

- Status: accepted
- Date: 2026-06-12
- Deciders: project maintainers

## Context

`lls`はLLMやagent向けのdirectory listing CLIである。  
一方、MVPにはCodex CLIを利用した初回設定生成も含まれる。

Codexを通常のlisting pipelineへ組み込むと、次の問題が起きる。

- 同じ入力で出力が変わる
- testがnetwork/account/model状態へ依存する
- latencyと利用制限が通常実行へ影響する
- credential handlingの範囲が広がる
- failure modeがfilesystem listingと混ざる
- JSON contractの再現性が下がる

ただし、project固有のpriorityやignore設定案を初回に作る用途ではCodexが有用である。

## Decision

`lls`を次の2境界へ分ける。

### Deterministic runtime core

通常の`lls`、`--json`、`--human`、`-l`は次だけを使用する。

- CLI arguments
- validated project config
- local filesystem metadata
- built-in deterministic rules

通常実行はCodex、HTTP、OpenAI SDK、networkを使用しない。

### Codex-assisted setup boundary

Codexは`lls setup`の設定案生成にだけ利用する。

- authenticationはCodex CLIのSign in with ChatGPTへ委譲する
- OpenAI Platform API keyをサポートしない
- credential fileを`lls`が直接読まない
- processはread-only sandboxかつephemeral
- outputはJSON Schemaとsemantic safety checkで検証する
- 最終writeは`lls`がatomicに行う
- Codex失敗時は既存configを変更しない

`lls setup --without-codex`を用意し、Codexなしでも決定的なdefault configを生成できるようにする。

## Consequences

### Positive

- 通常実行の再現性が高い
- unit/E2E testをaccountなしで実行できる
- security boundaryが明確になる
- Codex failureがlistingへ波及しない
- JSON出力を安定させやすい
- coreへHTTP clientやasync runtimeを導入せずに済む

### Negative

- runtimeでproject内容をLLMへ解釈させる高度な分類はできない
- setupとruntimeで一部のproject observation logicが重複する可能性がある
- Codex subprocess adapterとfake process testが必要になる
- userは初回setupまたは`--no-config`を明示する必要がある

## Alternatives considered

### Call Codex on every listing

却下。非決定的、低速、account依存となり、CLIの基本用途を壊す。

### Use OpenAI Platform API directly

却下。API key管理、HTTP client、billing/account差分をMVPへ持ち込むため。

### Do not use Codex at all

却下。project固有設定の初期案を作る用途ではCodexの価値が高く、明確な境界に隔離すればcoreの性質を維持できる。

### Let Codex write `.lls/config.json` directly

却下。write権限、partial file、schema逸脱、意図しないproject変更のriskがある。Codexはproposalのみ返し、`lls`が検証・保存する。

## Compliance

実装とreviewで次を確認する。

- runtime moduleから`codex` moduleを直接呼ばない
- production dependencyにHTTP client/OpenAI SDKを追加しない
- Codex commandに`--ephemeral`と`--sandbox read-only`がある
- bypass sandbox/write権限のflagがない
- credential file readがない
- API key environment variable readがない
- proposal validation後にatomic writeする
- normal runtime testがCodex executableなしで成功する
- fake processでCodex error pathをtestする
