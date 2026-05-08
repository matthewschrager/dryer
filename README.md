# dryer

`dryer` finds candidate duplicate Rust and TypeScript code by comparing
normalized syntax structure instead of raw text.

It is inspired by [`dry4clj`](https://github.com/unclebob/dry4clj), but uses
Tree-sitter parsers for Rust, TypeScript, and TSX.

## Usage

```bash
cargo run -p dryer -- [options] [file-or-directory ...]
```

Examples:

```bash
cargo run -p dryer -- crates web/src
cargo run -p dryer -- --json --threshold 0.9 src
cargo run -p dryer -- --language rust fixtures/rust
```

Options:

```text
--threshold N          Minimum structural similarity score, default 0.82
--min-lines N          Minimum source lines in a candidate, default 6
--min-nodes N          Minimum normalized syntax nodes, default 35
--language L           all, rust, or typescript; default all
--format F             text, json, or sarif; default text
--json                 Same as --format json
--sarif                Same as --format sarif
--include GLOB         Additional include glob
--exclude GLOB         Additional exclude glob
--cross-language       Experimental Rust/TypeScript comparisons
--max-candidates N     Limit reported candidates after sorting
--config PATH          Load dryer.toml from an explicit path
--fail-on-duplicates   Exit 1 when candidates are found
```

## Development

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```
