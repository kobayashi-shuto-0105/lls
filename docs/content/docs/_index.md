---
title: "ドキュメント"
description: "lls の仕様、設計、実装計画、進捗を辿るための入口。"
---

# ドキュメント

`lls` の仕様や設計メモは、用途ごとに複数の文書へ分割しています。  
このページは、それぞれの文書へ迷わず辿るための入口です。

## まず読む

{{< cards cols="2" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/README.md" title="README" icon="document-text" subtitle="プロジェクト概要、使い方、出力例。" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/.github/assets/spec.md" title="仕様書" icon="document-text" subtitle="MVP の正本仕様。動作・契約・終了コード。" >}}
{{< /cards >}}

## 設計と実装

{{< cards cols="2" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/docs/architecture.md" title="アーキテクチャ" icon="cube-transparent" subtitle="責務分割、依存方向、モジュール境界。" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/docs/implementation-plan.md" title="実装計画" icon="map" subtitle="M0〜M8 の実装タスクと受け入れ条件。" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/docs/implementation-status.md" title="実装状況" icon="chart-bar" subtitle="現在地、完了済みタスク、直近の引き継ぎ。" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/docs/setup-plan.md" title="Setup 設計" icon="document-text" subtitle="`lls setup` と Codex 境界の設計。" >}}
{{< /cards >}}

## 補助資料

{{< cards cols="2" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/docs/adr/README.md" title="ADR 運用" icon="book-open" subtitle="設計判断をどう残すか。" >}}
  {{< card link="https://github.com/kobayashi-shuto-0105/lls/blob/main/docs/content-view-plan.md" title="将来メモ" icon="light-bulb" subtitle="`lls cat` など未実装機能の検討メモ。" >}}
{{< /cards >}}

## 読み方のおすすめ

1. 全体像を掴むなら `README.md`
2. 挙動の正本を確認するなら `spec.md`
3. 実装の責務分割を追うなら `architecture.md`
4. 今どこまで進んでいるかを見るなら `implementation-status.md`
5. これからの作業単位を確認するなら `implementation-plan.md`

## リポジトリ

- [GitHub リポジトリ](https://github.com/kobayashi-shuto-0105/lls)
