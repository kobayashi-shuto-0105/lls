# Hugo Publishing Runbook

このrunbookは、授業第9回で示された`gh-pages` branchと`git worktree`を使ってHugo siteを公開するための操作手順である。

前提:

- Hugo sourceは`docs/`
- generated siteは`docs/public/`
- `docs/public/`は`gh-pages` branchのlinked worktree
- GitHub Pagesは`gh-pages` branchのrootを公開
- sourceとgenerated outputは別commit history

---

## 1. Preflight

repository rootで実行する。

```sh
git status --short
git branch --show-current
git worktree list
git branch --list gh-pages
git branch -r --list origin/gh-pages
```

次の場合は停止する。

- working treeがcleanでない
- `docs/public`が通常directoryとして存在する
- `docs/public`が別branchのworktreeになっている
- `gh-pages`のlocal/remote状態が判断できない

現在のbranch名を記録しておく。

```sh
current_branch="$(git branch --show-current)"
```

---

## 2. Create gh-pages branch

localにもremoteにも`gh-pages`がない場合のみ実行する。

```sh
git switch --orphan gh-pages
touch index.html
git add index.html
git commit -m "initial revision for gh-pages"
git switch "$current_branch"
```

確認:

```sh
git branch --list gh-pages
git status --short
```

既にbranchがある場合は作り直さない。

---

## 3. Add worktree

```sh
git worktree add docs/public gh-pages
```

確認:

```sh
git worktree list
git -C docs/public branch --show-current
git -C docs/public status --short
```

期待値:

```txt
gh-pages
```

`docs/public`を外す必要がある場合は、linked worktreeとしてremoveする。

```sh
git worktree remove docs/public
```

通常directoryのように扱わない。

---

## 4. Build production site

```sh
cd docs
hugo --minify
cd ..
```

確認:

```sh
test -f docs/public/index.html
test -f docs/public/.nojekyll
git -C docs/public status --short
```

build前後で`docs/public/.git`が存在することを確認する。

---

## 5. Review generated output

```sh
find docs/public -maxdepth 2 -type f | sort | sed -n '1,120p'
git -C docs/public diff --stat
git -C docs/public diff -- index.html
```

最低限確認する。

- top pageが生成されている
- `.nojekyll`がある
- draft pageがない
- secret、token、local absolute pathがない
- internal AI instructionがない
- asset URLが`/lls/`を考慮している
- 想定外の大量削除がない

---

## 6. Commit and push generated output

```sh
git -C docs/public add -A
git -C docs/public commit -m "docs: publish site"
git -C docs/public push origin gh-pages
```

変更がない場合は空commitを作らない。

source側のcommitはmain向けbranchで行い、generated output commitは`gh-pages`で行う。

---

## 7. Configure GitHub Pages

GitHub repositoryで次を設定する。

```txt
Settings
→ Pages
→ Build and deployment
→ Source: Deploy from a branch
→ Branch: gh-pages
→ Folder: /(root)
→ Save
```

公開URL:

```txt
https://kobayashi-shuto-0105.github.io/lls/
```

---

## 8. Deployment verification

公開後に確認する。

- top pageが200で表示される
- CSSとJavaScriptが404にならない
- navigationが動く
- imageが表示される
- internal linkが壊れていない
- page titleとdescriptionが正しい
- public URLがREADMEから辿れる

失敗時はgenerated HTMLを直接修正せず、`docs/content`またはHugo configを修正して再buildする。

---

## 9. Regular publishing flow

```sh
# source側
cd docs
hugo --minify
cd ..

# generated側
git -C docs/public status --short
git -C docs/public add -A
git -C docs/public commit -m "docs: publish site"
git -C docs/public push origin gh-pages
```

毎回、commit前にgenerated diffと公開対象の安全性を確認する。
