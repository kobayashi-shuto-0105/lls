# Hugo Documentation Site Plan

この文書は、授業第9回の方針に沿って`lls`の一般向けWebサイトをHugoとGitHub Pagesで構築するためのtask計画である。

## Course-aligned decisions

- SSGはHugo
- Hugo site rootは`docs/`
- themeはCongo v2
- themeはHugo Modulesで管理
- public contentは`docs/content/`
- static assetsは`docs/static/`
- generated siteは`docs/public/`
- `docs/public/`は`gh-pages` branchのlinked worktree
- GitHub Pagesは`gh-pages` branchのrootを公開
- public URLは`https://kobayashi-shuto-0105.github.io/lls/`
- sourceと生成HTMLを同じbranchへ混在させない
- Hugo CIは既存workflowへ追加せず、専用`hugo.yaml`へ分ける

詳細な作業規則は[`docs/AGENTS.md`](AGENTS.md)、公開手順は[`hugo-publishing-runbook.md`](hugo-publishing-runbook.md)を参照する。

---

## Existing repository constraint

授業資料ではrepository rootで次を実行する。

```sh
hugo new site docs
```

しかし、このrepositoryの`docs/`には既にarchitectureやimplementation planが存在する。初期化では既存文書を削除せず、cleanな専用branch上で次を使う。

```sh
hugo new site docs --force --format toml
```

実行直後に`git diff -- docs`を確認し、既存文書のoverwriteやdeleteがあればcommitせず停止する。

---

## Target structure

```txt
docs/
├── AGENTS.md
├── *.md                         # internal engineering documents
├── adr/                         # internal architecture decisions
├── archetypes/
├── assets/
├── config/_default/
│   ├── hugo.toml
│   ├── languages.ja.toml
│   ├── markup.toml
│   ├── menus.ja.toml
│   ├── module.toml
│   └── params.toml
├── content/                     # public documentation source
├── layouts/
├── static/
│   └── .nojekyll
├── go.mod
├── go.sum
└── public/                      # gh-pages worktree
```

---

# HUGO-00: Preflight and inventory

**Depends on:** none

```sh
git status --short
git branch --show-current
hugo version
go version
git version
find docs -maxdepth 2 -type f | sort
```

Acceptance:

- dedicated branchである
- working treeがclean
- Hugo ExtendedとGoが利用できる
- 既存`docs/` file一覧をPR本文へ記録した
- current Hugo/Congo requirementsを公式documentで確認した

---

# HUGO-01: Initialize Hugo scaffold safely

**Depends on:** HUGO-00

```sh
hugo new site docs --force --format toml
git status --short
git diff -- docs
```

Acceptance:

- existing `docs/*.md`と`docs/adr/**`が変わっていない
- Hugo skeletonだけが追加された
- unexpected overwriteがない
- `docs/`全体を作り直していない

---

# HUGO-02: Install and pin Congo

**Depends on:** HUGO-01

```sh
cd docs
hugo mod init github.com/kobayashi-shuto-0105/lls
```

`config/_default/module.toml`:

```toml
[[imports]]
path = "github.com/jpanther/congo/v2"
```

implementation時点のstable versionへ固定し、`go.mod`と`go.sum`をcommitする。floating versionをrelease branchへ残さない。

Acceptance:

- `hugo mod graph`が成功する
- CongoがHugo Moduleとして解決される
- theme cacheや`themes/congo`をcommitしていない

---

# HUGO-03: Configure site and navigation

**Depends on:** HUGO-02

最低限のsite設定:

```toml
baseURL = "https://kobayashi-shuto-0105.github.io/lls/"
title = "lls"
defaultContentLanguage = "ja"
enableRobotsTXT = true
```

Required navigation:

1. What is lls?
2. Installation
3. Quick start
4. Usage
5. Configuration
6. Output format
7. Setup
8. Tutorials
9. Examples
10. Concepts and algorithm
11. Security
12. CLI reference
13. FAQ

Acceptance:

- project siteの`/lls/`配下でasset pathが壊れない
- required pageへmenuから到達できる
- URLを頻繁に変更しない構造になっている
- `docs/static/.nojekyll`がある

---

# HUGO-04: Write public content

**Depends on:** HUGO-03, verified CLI output for examples

Required source:

```txt
content/
├── _index.md
├── getting-started/
├── guide/
├── tutorials/
├── examples/
├── concepts/
├── security.md
├── cli-reference.md
└── faq.md
```

授業資料が求めるREADME以上の内容として、次を必須にする。

- algorithm explanation
- end-to-end tutorial
- RustとNodeなど複数projectの実行例
- configuration details
- output format
- errorとexit code

Rules:

- READMEをそのままcopyしない
- 一般利用者向けに平易に書く
- command outputは実装済みbinaryから生成する
- 未実装pageは`draft = true`
- internal taskやAI指示を公開しない

---

# HUGO-05: Add isolated Hugo CI

**Depends on:** HUGO-03

追加するworkflow:

```txt
.github/workflows/hugo.yaml
```

Responsibility:

- pull requestでHugo buildを確認
- docs source関連pathだけで起動
- `hugo --minify`を実行
- deploymentは行わない
- permissionsは`contents: read`

既存の`build.yaml`、`format.yaml`、`rust-tests.yaml`にはHugo処理を追加しない。

---

# HUGO-06: Prepare gh-pages worktree

**Depends on:** HUGO-03

授業資料に沿って`gh-pages` branchを用意し、次の形にする。

```txt
docs/public -> gh-pages branch worktree
```

preflight:

```sh
git status --short
git worktree list
git branch --list gh-pages
git branch -r --list origin/gh-pages
```

branch作成とworktree追加の具体的なcommandは[`hugo-publishing-runbook.md`](hugo-publishing-runbook.md)を参照する。

Acceptance:

- `git -C docs/public branch --show-current`が`gh-pages`
- main側のstatusに生成HTMLが出ない
- root `.gitignore`に`/docs/public/`がある

---

# HUGO-07: Verify and publish

**Depends on:** HUGO-04, HUGO-05, HUGO-06

```sh
cd docs
hugo --minify
```

確認:

- `index.html`と`.nojekyll`が生成される
- CSS/JSが`/lls/`配下で解決される
- draftがproduction buildへ含まれない
- secretやinternal AI instructionが公開されない
- dedicated Hugo CIが成功する

公開操作は[`hugo-publishing-runbook.md`](hugo-publishing-runbook.md)に従う。

---

## Definition of Done

- [ ] site rootが`docs/`
- [ ] existing internal docsを保持
- [ ] Congo v2をHugo Modulesでversion固定
- [ ] `config/_default`へ設定を分割
- [ ] READMEより詳しいpublic contentがある
- [ ] algorithm、tutorial、複数例がある
- [ ] `hugo --minify`が成功
- [ ] Hugo CIが独立workflowで成功
- [ ] `docs/public`が`gh-pages` worktree
- [ ] Pages sourceが`gh-pages` root
- [ ] public URLで表示可能
- [ ] sourceとgenerated historyが分離
- [ ] sensitive dataが公開されていない
