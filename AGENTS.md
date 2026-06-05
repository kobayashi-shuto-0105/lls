# AGENTS.md

このリポジトリで作業するエージェント向けの運用メモです。  
Rust の一般的なベストプラクティスに沿って、実装を小さく、安全に、テストしやすく進めるための指針をまとめます。

## まず読むもの

1. [`.github/assets/spec.md`](.github/assets/spec.md)
2. [`docs/setup-plan.md`](docs/setup-plan.md)
3. [`README.md`](README.md)

`spec.md` が仕様の本体、`docs/setup-plan.md` が初回セットアップ周りの実装計画です。

## 実装方針

- 変更は小さく分ける
- 仕様と実装を混ぜない
- 出力ロジック、分類ロジック、走査ロジックを分離する
- ルールベースの処理を優先し、曖昧な推論は避ける
- 迷ったら安全側に倒す
- 実装だけで終わらせず、テストとドキュメントまで含めて完了とみなす

## Rust の基本ルール

- `cargo fmt` を通るコードにする
- `cargo clippy -- -D warnings` を目標にする
- `unwrap()` と `expect()` は原則使わない
- `panic!` はテストや到達不能の明示以外では避ける
- エラーは `Result` で返し、必要なら独自エラー型にまとめる
- 共有状態はできるだけ減らし、関数の入出力を明確にする
- `pub` は必要最小限にする
- 1 モジュール 1 責務を意識する

## 推奨する構成

実装が進んだら、責務を以下に分けると保守しやすいです。

- `cli`: 引数解析
- `config`: 設定ファイルの読み書き
- `scanner`: ファイルシステム走査とメタデータ取得
- `classifier`: `role`、`generated`、`sensitive` などの判定
- `project_type`: プロジェクト種別推定
- `priority`: 優先度付け
- `recommendation`: 次に読む候補の生成
- `output`: JSON / human / long listing の出力
- `error`: エラー型と終了コード

## テスト方針

- 判定ロジックはできるだけ pure 関数にして単体テストしやすくする
- ファイルシステム依存は最小限にする
- `cargo test` で通るテストを増やす
- 仕様変更が起きやすい箇所は回帰テストを置く
- `entries` の並び順、`recommended_next_steps`、設定ファイル読み込みは重点的にテストする
- 可能なら fixture ベースの E2E テストを置く
- 新しい振る舞いを足したら、必ずテストも一緒に足す

## ドキュメント方針

- 仕様を変えたら、まず [`spec.md`](.github/assets/spec.md) を更新する
- 将来候補に移る内容は [`feature-spec.md`](.github/assets/feature-spec.md) に整理する
- 初回セットアップや運用の話は [`docs/`](docs) に短くまとめる
- README は「何をするツールか」が一読で分かる状態に保つ
- 実装に関する前提が変わったら、`AGENTS.md` も同期する

## 出力方針

- デフォルトは compact JSON
- `--human` は人間向けの色付き表示
- `-l` は long listing
- `--json` と `--human` と `-l` が衝突したら、仕様で決めた優先順位に従う

## エラー処理

- 失敗時でも可能な限り部分結果と warning を残す
- 権限不足や読めないファイルがあっても、全体を落としすぎない
- 設定ファイルが壊れている場合はデフォルトにフォールバックする

## 作業前の確認

- 既存の変更を勝手に巻き戻さない
- 仕様変更が必要なら、先に `spec.md` を更新する
- 実装計画が必要なら、`docs/` に短いメモを置く
- 生成物や `target/` はコミット対象にしない
- テストやドキュメントの更新漏れがないか最後に確認する

## このリポジトリ固有の注意

- 仕様の正本は [`spec.md`](.github/assets/spec.md)
- セットアップの実装メモは [`docs/setup-plan.md`](docs/setup-plan.md)
- 初回セットアップは `lls setup` と初回 `lls` の両方を想定する
- 秘密情報は設定ファイルに保存しない
- `feature-spec.md` は MVP 外の候補置き場として扱う
