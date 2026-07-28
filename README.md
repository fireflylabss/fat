# ◆ fat

**fat** — a fast, syntax-aware alternative to `cat`, written in Rust.

Built in the spirit of `bat`: files get an informative header, a quiet gutter,
line numbers and terminal-aware colors. It remains intentionally small and
local: no service, account, telemetry or configuration file is required.

```text
── src/main.rs ─────────────────────────────────────────────────
  1 │ use std::io;
  2 │
  3 │ fn main() {
  4 │     println!("hello from fat");
  5 │ }
```

## Install

Requires Rust **1.85+**.

```bash
cargo install --path .
fat src/main.rs
```

From the checkout:

```bash
cargo run --release -- --style=full src/main.rs
```

## Usage

```bash
fat src/main.rs
fat --style=full Cargo.toml README.md
fat -n -r 20:60 src/main.rs
curl -s https://example.com/snippet.rs | fat -l rust -
fat --file-name settings.toml - < config
fat -pp README.md > README.txt
```

| Option | Description |
|---|---|
| `-n`, `--number` | Number-only view, without headers or grid |
| `-p`, `--plain` | Exact plain text without decorations or color |
| `-l`, `--language` | Set syntax when the filename is unknown |
| `-r`, `--line-range N:M` | Show an inclusive range of lines |
| `--style` | `default`, `full`, `header`, `numbers` or `plain` |
| `--paging` / `-P` | Control the built-in `less -R` pager behavior |
| `--file-name` | Name stdin data for the header and syntax detection |
| `-A`, `--show-all` | Show spaces, tabs and line endings |
| `-s`, `--squeeze-blank` | Collapse consecutive blank lines |

The default pager activates only on interactive output that is longer than the
terminal. Color automatically stays out of pipes, `NO_COLOR` is respected, and
`--color=always` is available for integrations.

## Philosophy

fat takes the useful surface of `bat` without trying to recreate every layer:
read files quickly, inspect a range, name data arriving through stdin and keep
the Unix pipeline clean.

## Development

```bash
cargo fmt --check
cargo test
cargo build --release
```

## License

Apache License 2.0 — see [LICENSE](LICENSE).
