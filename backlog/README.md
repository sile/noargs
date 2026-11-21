# backlog

This directory holds the work items of this repository that are not closed
yet: proposals, bug reports, and other pieces of work. It is managed with
[`nokori`](https://github.com/sile/nokori), a markdown-file-based backlog tool.

## What this directory is for

An item is a **record of work or a proposal**, not a specification. The
authoritative specification is the code and `README.md` of the crate; those
must make sense without this directory.

## Where an item is

An open item is in `open/`, and a closed one is in `closed/`. The directory an
item is in is its state; the file itself has no status field.

## Read more

    nokori explain flow

says how an item is opened and closed, and what the states mean. The commands
are listed by `nokori --help`.
