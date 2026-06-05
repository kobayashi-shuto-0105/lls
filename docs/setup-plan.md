# lls setup plan

このメモは、`lls` の初回セットアップと設定保存の実装方針を簡単にまとめたもの。

## 目的

- 初回利用時にプロジェクト設定を作る
- 既存設定がある場合は上書きもできる
- セットアップ後は `.lls/config.json` を読んで挙動を決める
- 秘密情報は設定ファイルに保存しない

## 認証の考え方

OpenAI API の標準的な認証は API key ベースで、環境変数や安全な資格情報ストアから読むのが基本。

- API key はプロジェクト設定ファイルではなく、環境変数または OS の安全な保存先を使う
- `.lls/config.json` には、非機密の設定だけを書く
- 初回セットアップでは、認証情報が未設定なら案内を出して終了してよい

参考:

- [OpenAI API authentication](https://platform.openai.com/docs/api-reference/authentication?api-mode=responses)
- [OpenAI libraries quickstart](https://platform.openai.com/docs/libraries)

## 初回フロー

1. `lls` 実行時に `.lls/config.json` がなければ検出する
2. 対話的なら `lls setup` を案内する
3. 非対話的ならデフォルトルールで続行し、warning を出す
4. `lls setup` は設定ファイルを生成し、必要なら `--force` で上書きする
5. 生成後は `lls` が設定ファイルを読み込んで表示・ソート・推奨候補を決める

## 設定に入れるもの

- デフォルト表示モード
- `-l` の既定ソート順
- ignore パターンの追加
- role / priority のオーバーライド
- README tagline や PDF title の表示設定

## 実装メモ

- 設定ファイルの読み込みは起動時に必ず行う
- 設定ファイルが壊れていたら warning にしてデフォルトへフォールバックする
- `setup` の結果はテストしやすいように pure な設定生成処理と I/O を分ける
