# lls setup plan

この文書は、MVP の `lls setup` と Codex 認証の実装方針を定義する。  
正式な全体仕様は [`.github/assets/spec.md`](../.github/assets/spec.md) を参照する。

---

## 1. 目的

- `.lls/config.json` を明示的に生成する
- Codex CLI を使い、プロジェクト固有の設定案を作る
- 認証情報を `lls` が保持しない
- Codex の出力を schema validation してから保存する
- setup 失敗時に partial file を残さない

---

## 2. 認証方針

MVP は Codex CLI の **Sign in with ChatGPT** だけをサポートする。

対応しない:

- OpenAI Platform API key
- `OPENAI_API_KEY`
- `CODEX_API_KEY`
- API key の入力フォーム
- `.lls/config.json` への token 保存
- `~/.codex/auth.json` の直接読み取り
- auth cache のコピー

Codex CLI が資格情報を自身の credential store に保存・更新する。  
`lls` は `codex` process の成功・失敗だけを扱う。

通常は browser login の `codex login` を使う。  
headless 環境では `codex login --device-auth` を案内する。

参考:

- [Codex authentication](https://developers.openai.com/codex/auth)
- [Codex non-interactive mode](https://developers.openai.com/codex/noninteractive)
- [Codex CLI reference](https://developers.openai.com/codex/cli/reference)

---

## 3. 初回通常実行

`.lls/config.json` が見つからない `lls` は設定を生成しない。

処理:

1. stderr に setup 誘導を表示
2. stdout は空
3. exit code `5`

ユーザーは次のどちらかを明示する。

```sh
lls setup
lls --no-config
```

これにより、通常実行が暗黙にファイルを書き換えることを防ぐ。

---

## 4. setup flow

### 4.1 Codex を使う既定 flow

1. project root を解決
2. `.lls/config.json` の存在を確認
3. 既存設定があれば `--force` なしでは終了
4. `codex` executable の存在を確認
5. Codex の ChatGPT session が使えるか確認
6. 未認証なら `codex login` を案内または起動
7. embedded config schema を一時ファイルへ書く
8. Codex を read-only / ephemeral で実行
9. final JSON を一時ファイルへ出力
10. `lls` 側でも JSON Schema validation
11. built-in safety rule を再適用
12. 生成案を表示
13. `--yes` がなければ確認
14. `.lls/` を作成
15. temporary file に書く
16. fsync 後に atomic rename で `.lls/config.json` へ置換
17. 一時ファイルを削除

### 4.2 Codex invocation

概念 command:

```sh
codex exec \
  --ephemeral \
  --sandbox read-only \
  --ignore-user-config \
  --ignore-rules \
  --output-schema <temporary-schema.json> \
  --output-last-message <temporary-result.json> \
  --cd <project-root> \
  "<setup-prompt>"
```

制約:

- `danger-full-access` を使わない
- `workspace-write` を使わない
- project file を Codex に変更させない
- shell command の実行結果を信用しない
- final JSON 以外を設定として採用しない
- timeout を設ける
- child process の stdout / stderr を setup 用ログとして扱い、通常 JSON 出力へ混ぜない

### 4.3 `--without-codex`

```sh
lls setup --without-codex
```

Codex CLI や login を要求せず、組み込み既定値から設定を生成する。  
同じ JSON Schema で検証し、同じ atomic write path を使う。

---

## 5. setup prompt contract

Codex へ渡す prompt は次を明示する。

- project を read-only で観察する
- file content を必要以上に読まない
- secret candidate の内容を読まない
- output は schema に一致する JSON のみ
- API key や token を出力しない
- user-specific path を設定へ保存しない
- priority / role override は最小限にする
- ignore pattern は build output、cache、vendor に限定する
- sensitive pattern は安全側へ倒す
- normal runtime は Codex を使わない

prompt には version を付け、テストで固定する。

```txt
lls_setup_prompt_version=0.1.0
```

---

## 6. generated config

正式 schema:

```txt
.github/assets/config.schema.json
```

主な field:

- `schema_version`
- `default_output`
- `scan.depth`
- `scan.include_hidden`
- `scan.include_ignored`
- `long_listing.sort`
- `rules.priority_overrides`
- `rules.role_overrides`
- `rules.ignore_patterns`
- `rules.sensitive_patterns`
- `codex.enabled`
- `codex.auth_method`
- `codex.use_for_setup`

`codex.auth_method` は `"chatgpt"` だけを許可する。

---

## 7. validation と safety normalization

Codex 出力を schema validation した後、次を再確認する。

- unknown key がない
- glob が parse できる
- `scan.depth` が `0..=8`
- priority / role が enum 内
- `.git/` を非 ignore にする override がない
- sensitive rule を弱める設定がない
- API key、token、credential path を含まない
- absolute path を pattern に含まない
- duplicate rule は先頭だけ残すか validation error とする

安全性に関わる矛盾は自動修正せず、validation error として設定案を破棄する。

---

## 8. error handling

| 状況 | exit |
|---|---:|
| 成功 | `0` |
| 既存設定があり `--force` なし | `1` |
| Codex CLI なし | `6` |
| ChatGPT login 不可 | `6` |
| Codex timeout / non-zero exit | `6` |
| Codex output invalid | `6` |
| config write failure | `4` |

失敗時:

- 既存 config を変更しない
- partial config を残さない
- credential 内容をログへ出さない
- Codex stdout 全文を error message へ埋め込まない

---

## 9. テスト

### unit

- project root resolution
- existing config と `--force`
- prompt generation
- Codex command arguments
- ChatGPT-only auth policy
- API key environment variableを参照しない
- valid / invalid Codex output
- safety normalization
- atomic write
- temp cleanup

### integration with fake process

- login required
- login succeeds
- codex exec succeeds
- timeout
- non-zero exit
- malformed JSON
- schema mismatch
- user declines
- `--yes`
- `--without-codex`

実 Codex account を必要とする test は通常 CI から分離する。
