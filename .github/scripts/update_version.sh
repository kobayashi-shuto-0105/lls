#! /bin/sh
#
# リリース用ブランチ名（例: releases/v0.1.0）から取り出した "0.1.0" を受け取り、
# リポジトリ内の「バージョン表記」を同じ値に揃えるためのスクリプト。
#
# 想定する呼び出し元:
# - `.github/workflows/update-version.yaml`
#
# 更新対象:
# - `Cargo.toml` の `version = "..."` 行
# - `.github/templates/README.md` の `${VERSION}` プレースホルダ → `README.md` を生成
#
# NOTE:
# - `README.md` を直接編集するのではなく、テンプレートから生成する形にしておくと
#   バージョン差分が機械的に追えて、更新漏れを減らせる。

set -eu

TO_VERSION="${1:?usage: update_version.sh <version>}"

# 変更前の version を控えておく（ログ・デバッグ用）。必須ではないが、失敗時に原因追跡しやすい。
FROM_VERSION="$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/g' || true)"
echo "update version: ${FROM_VERSION:-unknown} -> ${TO_VERSION}"

# 途中で失敗しても Cargo.toml / README.md を壊さないよう、一時ファイルに書いてから置き換える。
tmp="$(mktemp)"
sed "s/^version = \".*\"/version = \"${TO_VERSION}\"/" Cargo.toml > "${tmp}"
mv "${tmp}" Cargo.toml

tmp="$(mktemp)"
# README テンプレート内の `${VERSION}` を置換して、README.md を生成する。
sed "s/\${VERSION}/${TO_VERSION}/g" .github/templates/README.md > "${tmp}"
mv "${tmp}" README.md
