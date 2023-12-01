# cogni

[![Rust](https://github.com/leoshimo/cogni/actions/workflows/rust.yml/badge.svg)](https://github.com/leoshimo/cogni/actions/workflows/rust.yml)

Unix-minded interface for interacting with LLMs.

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
- Flexible input and output formats (Text, JSON, NDJSON, 🚧 Transcript)
- Standalone binary - No Python required
- 🚧 Repeatable Scripts via Templates

## Non-Features

- Interactive use - instead, invoke `cogni` from within interactive environments (REPLs, emacs, etc) 

## Installation

```sh
# Install from crates.io
$ cargo install cogni

# From source
$ cargo install --path .
```

## Setup

`cogni` expects an OpenAI API Key to be supplied via `--apikey` option or more
conveniently `OPENAI_API_KEY` environment variable:

```sh
# in shell configuration
export OPENAI_API_KEY=sk-DEADBEEF
```

---

## Basic Usage

See `cogni --help` for documentation

```sh
# Via stdin
$ echo "How do I read from stdin?" | cogni

# Via file
$ echo "What is 50 + 50?" > input.txt
$ cogni input.txt
50 + 50 equals 100.


# Via flags:
#  -s, --system <MSG>            Sets system prompt (Always first)
#  -a, --assistant <MSG>         Appends assistant message
#  -u, --user <MSG>              Appends user message
$ cogni --system "Solve the following math problem" --user "50 + 50"
50 + 50 equals 100.

# Few-shot prompting with multiple messages
$ cogni --system "Solve the following math problem" \
    -u "1 + 1" \
    -a "2" \
    -u "22 + 20" \
    -a "42" \
    -u "50 + 50"
100

# Via flags AND stdin - flag messages come before stdin / file
$ echo "50 + 50" | cogni --system "Solve the following math problem" \
    -u "1 + 1" \
    -a "2" \
    -u "22 + 20" \
    -a "42"
100
```

---

## Tour of cogni

An gallery of examples to get the inspiration flowing

> :warning: `cogni` uses the [OpenAI API](https://openai.com/blog/openai-api), thus *any data fed into program will be sent to OpenAI*.

### In the Shell

```sh
# Creating Summary of Meeting Transcripts
$ cat meeting_saved_chat.txt | cogni -s "Extract the links mentioned in this transcript, and provide a high level summary of the discussion points"

# Narrate Weather Summary
$ curl -s "wttr.in/?1" | cogni -s "Summarize today's weather using the output. Respond in 1 short sentence." | say

# Create a ffmpeg cheatsheet from man page
$ man ffmpeg | cogni -T 300 -s "Create a cheatsheet given a man page. Output should be in Markdown, and should be a set of example usages under headings." > cheatsheet.md

# Catchup after long hiatus from git repo
$ git show -n 3 | cogni -s "What was I working on recently?"
```

### In Emacs

Emacs can use `shell-command-on-region` to pipe buffer regions to `cogni`.

For example, the following defines a command that plumbs region to `cogni`, optionally replacing original contents:

```emacs-lisp
(defun leoshimo/cogni-on-region (start end prompt replace)
  "Run cogni on region. Prefix arg means replace region, instead of separate output buffer"
  (interactive "r\nsPrompt: \nP")
  (shell-command-on-region start end
                           (format "cogni -s \"%s\"" prompt)
                           nil replace))

(global-set-key (kbd "M-c") #'leoshimo/cogni-on-region)
```

This binding is useful across a wide range of tasks, for example:

- Normalizing non-uniform text - e.g. unstructured logs to structured JSON events.
- Editing or organizing text semantically - e.g. rewording or grouping by category.
- Generating summary for an Org Agenda doc.

### In Vim

Vim can run external shell commands on entire buffer or visual selection to
power similar workflows possible from Emacs. See `h :!` in vim.

For example, given a bulleted list of fruits, it an be sorted by color by:

1. Selecting the list of fruits in visual mode
2. Type `:!cogni -s "Sort this list by color"`
