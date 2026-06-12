# Documentation agent instructions

このファイルは`docs/`配下を変更するAIエージェント向けの追加規則である。  
リポジトリrootの[`AGENTS.md`](../AGENTS.md)も同時に適用される。

`docs/`には、開発者向け内部文書とHugoで公開する一般向け文書が共存する。両者を混同しないこと。

---

## 1. Directory zones

```txt
docs/
├── AGENTS.md                    # この規則
├── architecture.md              # 内部: 設計文書
├── implementation-plan.md       # 内部: 実装計画
├── implementation-status.md     # 内部: 実装進捗
├── setup-plan.md                # 内部: setup設計
├── hugo-site-plan.md            # 内部: Hugo構築・公開計画
├── hugo-site-status.md          # 内部: Hugo作業進捗
├── adr/                         # 内部: 設計判断
├── config/_default/             # 公開サイト: Hugo設定
├── content/                     # 公開サイト: 一般向けMarkdown
├── static/                      # 公開サイト: 画像・.nojekyll等
├── layouts/                     # 公開サイト: 最小限のtheme override
├── assets/                      # 公開サイト: Hugo Pipes入力
├── archetypes/                  # 公開サイト: content template
├── themes/                      # 原則未使用。Hugo Modulesを優先
├── resources/_gen/              # 生成物。commitしない
└── public/                      # 生成物。gh-pages worktree。mainへcommitしない
```

### Internal documents

次は開発者・AIエージェント向けであり、そのまま一般公開用contentとして扱わない。

- `docs/*.md`
- `docs/adr/**`

### Public documentation

一般向けWebサイトに掲載する文章は`docs/content/**`へ書く。

- READMEをそのままコピーしない
- READMEより平易に書く
- algorithm、tutorial、複数の実行例を追加する
- 内部のtask ID、未確定実装、AI向け指示を一般向けページへ出さない

---

## 2. Source of truth

Hugo関連作業は次の順で読む。

1. [`docs/AGENTS.md`](AGENTS.md)
2. [`docs/hugo-site-plan.md`](hugo-site-plan.md)
3. [`docs/hugo-site-status.md`](hugo-site-status.md)
4. [`docs/adr/0002-hugo-site-and-gh-pages-worktree.md`](adr/0002-hugo-site-and-gh-pages-worktree.md)
5. 選択したthemeの公式documentation
6. Hugo公式documentation

授業第9回の方針を基本とする。

- Hugoを使う
- site rootは`docs/`
- sourceと生成HTMLを分離する
- `docs/public`を`gh-pages` worktreeとして扱う
- GitHub Pagesは`gh-pages` branchのrootから公開する

このリポジトリ固有の安全策は[`hugo-site-plan.md`](hugo-site-plan.md)を優先する。

---

## 3. Current decisions

- SSG: Hugo
- site root: `docs/`
- default theme: Congo v2
- theme installation: Hugo Modules
- public URL: `https://kobayashi-shuto-0105.github.io/lls/`
- generated output: `docs/public/`
- publishing branch: `gh-pages`
- GitHub Pages source: `gh-pages` / `(root)`
- deployment style: 授業資料に合わせたworktreeからの明示的commit/push
- automatic deployment: 初期段階では行わない

theme変更、公開方式変更、site root変更にはADRが必要。

---

## 4. Initialization safety

このリポジトリの`docs/`は既に空ではない。  
そのため、AIは`hugo new site docs`を無確認で実行してはいけない。

初期化taskでは必ず次を行う。

1. dedicated branchで作業する
2. `git status --short`が空であることを確認する
3. 既存`docs/`の一覧を保存する
4. `hugo new site docs --force --format toml`を実行する
5. `git diff -- docs`で既存fileが上書き・削除されていないことを確認する
6. 想定外の変更があればresetして停止する

禁止:

- `rm -rf docs`
- 既存の内部文書をHugo初期化のために削除する
- `docs/`を別directoryへ一括移動する
- cleanでないworking treeで`--force`を使う

---

## 5. Theme rules

Congo v2をHugo Modulesで管理する。

```sh
cd docs
hugo mod init github.com/kobayashi-shuto-0105/lls
```

`config/_default/module.toml`:

```toml
[[imports]]
path = "github.com/jpanther/congo/v2"
```

ルール:

- Goのmodule fileでtheme versionを固定する
- `@latest`をrelease branchへ残さない
- `go.mod`と`go.sum`をcommitする
- Hugo module cacheをcommitしない
- theme sourceを直接編集しない
- overrideは`docs/layouts/`へ最小限に置く
- theme更新だけのPRを分ける

Hugoが生成したrootの`docs/hugo.toml`は、`config/_default/hugo.toml`へ設定を移し、build成功を確認してから削除する。

---

## 6. Content rules

### Required public sections

最低限、次を用意する。

```txt
content/
├── _index.md
├── getting-started/
│   ├── _index.md
│   ├── installation.md
│   └── quick-start.md
├── guide/
│   ├── _index.md
│   ├── usage.md
│   ├── configuration.md
│   ├── output.md
│   └── setup.md
├── tutorials/
│   ├── _index.md
│   └── first-repository.md
├── examples/
│   ├── _index.md
│   ├── rust.md
│   └── node.md
├── concepts/
│   ├── _index.md
│   ├── classification.md
│   ├── priority.md
│   └── recommendations.md
├── security.md
├── cli-reference.md
└── faq.md
```

未実装機能のページを完成済みとして公開しない。  
未実装ページが必要なら明確に`draft = true`とする。

### Writing style

- 一般利用者向けの日本語を基本にする
- 1ページ1目的
- 冒頭で「何ができるか」を示す
- commandはcopy可能なcode blockにする
- 出力例は実際のbinaryで再生成する
- error例には終了コードも書く
- 秘密情報、実在credential、local pathを掲載しない
- 更新日時をfront matterまたはGit情報から表示できるようにする

### README relationship

- README: 開発者が短時間で概要と最初の実行を理解する入口
- Hugo site: 一般向けの詳しい説明、tutorial、algorithm、複数例

同じ文章を二重管理しすぎない。READMEから詳細ページへ誘導する。

---

## 7. Local verification

site実装後は次を実行する。

```sh
hugo version
go version
cd docs
hugo server --buildDrafts
```

別terminalまたはserver停止後:

```sh
cd docs
hugo --minify
```

確認項目:

- buildがerrorなしで完了する
- top pageが表示される
- navigationに孤立ページがない
- internal linkが切れていない
- code blockが崩れていない
- project page用`baseURL`でCSS/JS pathが壊れない
- draftがproduction buildへ含まれない
- secretや内部AI指示が公開されない

`docs/public`内を直接修正してbuild errorを直してはいけない。

---

## 8. gh-pages worktree rules

`docs/public`は通常directoryではなく、`gh-pages` branchのlinked worktreeとして扱う。

禁止:

- `docs/public`をmain branchへcommitする
- `docs/public`のHTMLを手編集する
- `rm -rf docs/public`
- main branchで生成物とsourceを同じcommitへ入れる
- worktreeの`.git` fileを削除する

worktreeを外す場合:

```sh
git worktree remove docs/public
```

公開commitは`docs/public`内で行う。

```sh
git -C docs/public status --short
git -C docs/public add -A
git -C docs/public commit -m "docs: publish site"
git -C docs/public push origin gh-pages
```

build前に公開対象へsecretがないことを確認する。

---

## 9. CI policy

Hugo siteのscaffoldとthemeがcommitされるまでは、Hugo用workflowを追加しない。

siteがbuild可能になったPRで、専用の`.github/workflows/hugo.yaml`を追加する。

役割:

- pull requestでHugo buildを検証する
- publishは行わない
- `build.yaml`、`format.yaml`、`rust-tests.yaml`へHugo処理を追加しない

自動deployへ移行する場合は、検証workflowとdeploy workflowを分け、ADRを追加する。

---

## 10. Completion checklist

- [ ] `hugo-site-status.md`のtaskを更新した
- [ ] internal文書とpublic contentを混在させていない
- [ ] Hugo buildが成功した
- [ ] `docs/public`をmainへcommitしていない
- [ ] theme versionを固定した
- [ ] baseURLがproject site URLになっている
- [ ] draftの扱いを確認した
- [ ] 公開内容にsecret・credential・内部AI指示がない
- [ ] READMEとの責務分担を確認した
- [ ] CI変更は専用workflowに分離した
- [ ] publish時は`gh-pages`だけへ生成物をcommitした
