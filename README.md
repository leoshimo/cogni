# cogni

[![Rust](https://github.com/leoshimo/cogni/actions/workflows/rust.yml/badge.svg)](https://github.com/leoshimo/cogni/actions/workflows/rust.yml)

Unix native interface for LLMs optimized for happiness.

## Focus

`cogni` brings language model scripting (prompting) into familiar Unix
environment by focusing on:

- Ergonomics and accessibility in Unix shell
- Composability and interop with other programs - including `cogni` itself
- Ease of language model programming in both ad-hoc and repeatable manner

For example, designing for IO redirection (`stdin`, `stdout`) allows `cogni` to
work with files, editor buffers, clipboards, syslogs, sockets, and many external
tools without bespoke integrations.

## Features

- Unix-minded Design (IO redirection, composability, interop)
- Ad-hoc Language Model Scripting
- 🚧 Repeatable Scripts via Templates
- Flexible input and output formats (Text, JSON, NDJSON)
- Standalone binary - No Python required
- 🚧 Multi-Step Prompting
- 🚧 Tool Augmentation
- 🚧 Output Constraints (in set, in type)
- 🚧 Provided integration with external tools (Emacs, Raycast)

## Non-Features

- Interactive use - instead, invoke `cogni` from within interactive environments (REPLs, emacs, etc) 

## Installation

```sh
# Install from crates.io
$ cargo install cogni

# From source
$ cargo install --path .
```

---

## Tour of cogni

🚧 WIP
