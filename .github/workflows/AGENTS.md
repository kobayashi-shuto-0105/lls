# Workflow instructions

このdirectoryのGitHub Actions workflowを変更するAIエージェント向けの追加規則。

## 基本方針

- 既存workflowへ無関係なcheckを追加しない
- `build.yaml`はbuild、Clippy、coverageの既存責務を維持する
- 新しいCI目的は専用workflow fileとして追加する
- 1 workflow fileは1つの主要目的を持つ
- workflow間で同じ重い処理を不用意に重複させない
- CI追加は、対応する実装または品質上の必要性が生じた時点で行う
- 将来必要そうという理由だけでworkflowを先行追加しない

## File naming

目的が分かる名前にする。

```txt
build.yaml
format.yaml
rust-tests.yaml
docs.yaml
security.yaml
release.yaml
```

`ci.yaml`や`checks.yaml`のように責務が際限なく増える名前は避ける。

## 現在の最小構成

- `build.yaml`: 既存のcross-platform build、Clippy、coverage
- `format.yaml`: rustfmt check
- `rust-tests.yaml`: pull requestのRust test

追加のlint、docs、security、release workflowは、その機能を実装するPRで必要になった時に別fileとして追加する。

## 変更時の確認

- workflowの目的が既存fileと重複していない
- 権限は最小限
- secretを不要に参照していない
- fork PRで秘密情報を必要としない
- shell scriptへuser-controlled valueを直接埋め込んでいない
- OS matrixが本当に必要か確認した
- localで実行できるcommandを使用している
- `AGENTS.md`とPR templateのquality gateと矛盾しない

## 禁止事項

- unrelated PRで`build.yaml`を整理・改名する
- format、test、release、docs deployを1つのworkflowへ集約する
- `pull_request_target`を安易に使う
- write permissionを理由なく付与する
- external codeをcommit SHA固定なしで新規導入する
- CIを通すためにtestやClippyを弱める
