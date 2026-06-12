# Hugo Documentation Site Status

この文書はHugo documentation site作業の進捗台帳である。  
詳細は[`hugo-site-plan.md`](hugo-site-plan.md)を参照する。

## Current state

- Current task: `HUGO-00 — Preflight and inventory`
- Site scaffold: not started
- Theme: Congo v2 (decided, not installed)
- Hugo Modules: not initialized
- Public content: not started
- Hugo CI: not added
- `gh-pages` worktree: not prepared
- GitHub Pages: not configured
- Blockers: none
- Last updated: 2026-06-12

## Task board

| Task | Status | PR | Notes |
|---|---|---|---|
| HUGO-00 | not_started | - | Hugo/Go/git確認、既存docs inventory |
| HUGO-01 | not_started | - | non-empty `docs/`へ安全にscaffold追加 |
| HUGO-02 | not_started | - | Congo v2をHugo Modulesで固定 |
| HUGO-03 | not_started | - | baseURL、language、navigation、`.nojekyll` |
| HUGO-04 | not_started | - | READMEより詳しいpublic content |
| HUGO-05 | not_started | - | 独立した`.github/workflows/hugo.yaml` |
| HUGO-06 | not_started | - | `gh-pages` branchと`docs/public` worktree |
| HUGO-07 | not_started | - | build、review、publish、公開確認 |

## Fixed decisions

| Item | Decision |
|---|---|
| Site generator | Hugo |
| Site root | `docs/` |
| Theme | Congo v2 |
| Theme install | Hugo Modules |
| Public source | `docs/content/` |
| Generated output | `docs/public/` |
| Publishing branch | `gh-pages` |
| Pages source | `gh-pages` / root |
| URL | `https://kobayashi-shuto-0105.github.io/lls/` |
| Deployment | Initial manual worktree publish |
| Build CI | Separate `hugo.yaml`, no deploy |

## Update rules

Task開始時:

- statusを`in_progress`へ変更
- PR番号またはbranch名を記録
- preflight結果をNotesまたはPR本文へ残す

Task完了時:

- statusを`done`へ変更
- PR番号を記録
- Current taskを次のunblocked taskへ更新
- design decision変更時はADRを追加

Blocked時:

- statusを`blocked`へ変更
- 具体的な不足tool、version、branch状態、未決定事項を書く
- workaroundを勝手に正式方針へしない

## Handoff log

### 2026-06-12 — Planning baseline

- 授業第9回に沿ったHugo構築方針を整理
- non-empty `docs/`向け安全策を追加
- Congo v2 / Hugo Modulesを初期方針に決定
- `gh-pages` worktree publicationを採用
- implementationは未着手
- next taskはHUGO-00
