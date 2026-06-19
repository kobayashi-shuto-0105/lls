
---

title: "lls"

description: "ls for LLMs: a smarter directory listing CLI for agents."

---

# `lls`

`lls` is **ls for LLMs**: a directory listing CLI that helps agents understand what matters, what to ignore, and what to inspect next.

{{< cards >}}

  {{< card link="/docs/" title="Read the Docs" icon="book-open" subtitle="Overview, design notes, and implementation plans." >}}

  {{< card link="https://github.com/kobayashi-shuto-0105/lls" title="GitHub" icon="github" subtitle="Source code and development history." >}}

{{< /cards >}}

## Why `lls`?

Normal `ls` tells you **what exists**.

`lls` tries to tell agents:

- what is important

- what can be ignored

- what role each file probably has

- where to inspect next

## Example direction

~~~txt

README.md        high   project overview

Cargo.toml       high   Rust package definition

src/main.rs      high   CLI entry point

target/          low    generated build output

.git/            skip   repository metadata

~~~

## Project Links

- [Documentation](/docs)
