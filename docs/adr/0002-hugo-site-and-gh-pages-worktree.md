# ADR-0002: Hugo site and gh-pages worktree publication

- Status: accepted
- Date: 2026-06-12
- Deciders: project maintainers

## Context

授業第9回では、一般向けWebサイトをHugoで作成し、GitHub Pagesへ公開する方針が示されている。  
また、Hugoが生成するHTMLと人間が編集するMarkdownを同じbranch historyへ混在させず、`docs/public`を`gh-pages` branchのworktreeとして扱う方法が推奨されている。

このrepositoryでは既に`docs/`配下にarchitecture、implementation plan、ADRなどの内部文書が存在する。そのため、通常の`hugo new site docs`を無確認で実行すると既存文書との衝突や誤削除につながる可能性がある。

さらに、既存のRust CIへHugo buildやdeploymentを追加するとworkflowの責務が曖昧になる。

## Decision

### Site layout

- Hugo site rootを`docs/`とする
- 一般向け公開sourceを`docs/content/`へ置く
- 内部設計文書は既存の`docs/*.md`と`docs/adr/**`へ残す
- generated siteを`docs/public/`へ出力する

### Initialization

- non-empty directory向けのHugo optionを使用する
- cleanな専用branch上で初期化する
- 初期化直後に`docs/`差分をreviewする
- existing internal documentのoverwrite/deleteを許可しない

### Theme

- initial themeはCongo v2
- Hugo Modulesで管理する
- explicit versionを`go.mod`へ固定する
- theme sourceをrepositoryへcopyして直接編集しない

### Publication

- `gh-pages` branchをgenerated output専用branchとする
- `docs/public`を`gh-pages`のlinked worktreeとする
- GitHub Pagesは`gh-pages` branchのrootから公開する
- public URLを`https://kobayashi-shuto-0105.github.io/lls/`とする
- source commitとgenerated output commitを別branch historyへ分離する

### CI

- Hugo build checkは専用`.github/workflows/hugo.yaml`へ置く
- existing `build.yaml`へHugo処理を追加しない
- initial Hugo workflowはbuild validationだけを担当し、deployしない
- automatic deploymentへ変更する場合は新しいADRを必要とする

## Consequences

### Positive

- 授業資料の方法と一致する
- generated HTMLがmain historyのnoiseにならない
- public contentとinternal engineering documentを分離できる
- theme updateをGo moduleとして追跡できる
- Hugo CIとRust CIの責務が明確になる
- GitHub PagesのURLを長期的に維持しやすい

### Negative

- `docs/`配下にinternal文書とHugo project fileが共存する
- linked worktreeの理解が必要になる
- publish時にmainと`gh-pages`の2つのworking treeを扱う
- initial manual deploymentには人間またはpublish agentの明示操作が必要
- theme/moduleの取得にGoとnetworkが必要

## Alternatives considered

### Put Hugo site in another directory

授業資料の`docs/`方針から外れ、repository内のdocument locationが分散するため不採用。

### Commit `docs/public` to main

generated fileとhand-written sourceのhistoryが混在し、授業資料が避けるべきとしているnoiseを生むため不採用。

### Deploy only with GitHub Actions

技術的には可能だが、初期段階は授業資料の`gh-pages` worktree方式を学習・採用する。将来変更する場合は別ADRで扱う。

### Use a git submodule for the theme

授業資料でも選択肢だが、Congo公式がHugo Modulesを推奨しており、version管理と更新が容易なため不採用。

## Compliance

reviewで次を確認する。

- `docs/public`がmain diffへ含まれない
- Hugo初期化でexisting internal docsが変わっていない
- Congo versionが固定されている
- `.nojekyll`がgenerated siteへ含まれる
- `baseURL`がproject Pages URLである
- Hugo workflowが独立fileである
- `build.yaml`にHugo処理がない
- public contentへsecretやinternal AI instructionが含まれない
