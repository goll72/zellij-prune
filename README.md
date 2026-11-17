zellij-prune
============

A small utility to delete old [Zellij](https://github.com/zellij-org/zellij)
sessions (alive longer than a configurable age), skipping sessions whose
name have a specific prefix.

## Configuration

The binary accepts the following CLI options:

 - `--max-age`: maximum session age before pruning. Human-readable (`2w`, `30m`, `1d`, `12h`, `45s`) or raw seconds (e.g. `604800`).

   default: `604800` (7 days)

 - `--prefix`: session-name prefix that marks a session as exempt (never pruned).

   default: `@`

 - `--dry-run`: print the sessions that would be pruned without killing them.

## Usage

To build, run the following command inside a devshell:

```bash
cargo build --release
```

You may then run it as such:

```bash
zellij-prune --max-age 2w --prefix keep:
```
