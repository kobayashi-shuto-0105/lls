# Architecture Decision Records

このdirectoryは、`lls`の長期的な設計判断を記録する。

ADRは「実装方法のメモ」ではなく、後から変更すると複数moduleやpublic contractへ影響する判断に使う。

---

## ADRが必要な変更

- public JSON schema
- config formatまたはdiscovery
- exit code
- module責務の大幅な変更
- sync/async方針
- filesystem/process abstraction
- Codex利用範囲または認証
- security invariant
- deterministic behavior
- 大きなdependency導入

次は通常ADR不要。

- private helperの抽出
- file分割
- typo修正
- test fixture追加
- specで既に確定した挙動の実装

---

## Status

```txt
proposed
accepted
superseded
rejected
deprecated
```

accepted ADRを変更する場合、元fileを書き換えて歴史を消さない。  
新しいADRを追加し、元ADRを`superseded`へ変更して後継番号を記載する。

---

## Naming

```txt
NNNN-short-kebab-case-title.md
```

例:

```txt
0001-deterministic-core-and-codex-boundary.md
```

---

## Template

```md
# ADR-NNNN: Title

- Status: proposed
- Date: YYYY-MM-DD
- Deciders: project maintainers

## Context

なぜ判断が必要か。

## Decision

何を採用するか。

## Consequences

### Positive

- 利点

### Negative

- 欠点、制約、コスト

## Alternatives considered

### Alternative

採用しなかった理由。

## Compliance

実装・test・reviewでどう守るか。
```

---

## Index

| ADR | Status | Decision |
|---|---|---|
| [0001](0001-deterministic-core-and-codex-boundary.md) | accepted | 通常実行を決定的なローカルコアとし、Codexをsetup境界へ隔離する |
