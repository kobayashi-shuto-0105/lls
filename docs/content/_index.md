
---

title: "lls"

description: "LLM やエージェント向けに、重要なファイルと次の探索先を返す CLI。"

---

# `lls`

`lls` は **ls for LLMs** を目指す CLI です。  
単なるファイル一覧ではなく、どこから読めばよいか、何を後回しにしてよいか、どのファイルが重要かを返します。

{{< cards >}}

  {{< card link="/docs/" title="ドキュメント" icon="book-open" subtitle="仕様、設計、実装計画、進捗をまとめて確認。" >}}

  {{< card link="https://github.com/kobayashi-shuto-0105/lls" title="GitHub" icon="github" subtitle="ソースコード、履歴、Issue、PR を確認。" >}}

{{< /cards >}}

## `lls` がやりたいこと

通常の `ls` は「何があるか」を教えます。  
`lls` は、その一歩先として次を返そうとします。

- 何が重要か
- 何を無視してよいか
- 各ファイルやディレクトリの役割は何か
- 次にどこを見ればよいか

## 返したい方向性

~~~txt
README.md        critical   project overview
Cargo.toml       critical   manifest
src/             high       source_code
target/          ignore     build_output
.git/            ignore     dependency_cache
~~~

## 主な入口

- [ドキュメント一覧](/docs)
- [README（GitHub）](https://github.com/kobayashi-shuto-0105/lls/blob/main/README.md)
- [仕様書（GitHub）](https://github.com/kobayashi-shuto-0105/lls/blob/main/.github/assets/spec.md)
