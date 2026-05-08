# Dryer: Structural Duplicate Detection for Rust and TypeScript

## Purpose

Build `dryer`, a Rust and TypeScript version of `dry4clj`: a command-line tool that finds candidate duplicate code by comparing normalized syntax structure instead of raw text.

The important inheritance from `dry4clj` is the product shape, not the implementation:

- Scan files and directories.
- Extract meaningful code units.
- Normalize away incidental names and literals.
- Fingerprint full units and nested substructures.
- Score similarity with Jaccard similarity.
- Report filename and line ranges for human review, plus machine-readable output for tooling.

The first useful version should be a local CLI that works well on real Rust and TypeScript repositories without requiring language servers, Node project bootstrapping, `cargo check`, or `tsc`.

## Research Inputs

- Upstream reference: <https://github.com/unclebob/dry4clj> at commit `76094cf24573a1daef1af9f9c8a0616f6b9444d2`.
- `dry4clj` README describes structural fingerprinting and Jaccard scoring over normalized Clojure forms.
- `dry4clj` source confirms the implementation is intentionally simple: read source forms, normalize syntax, collect subtree fingerprints, compare candidate pairs, and print text or EDN.
- Tree-sitter is a good parser backbone because it is designed to build concrete syntax trees, parse many languages, and recover useful trees in the presence of syntax errors: <https://tree-sitter.github.io/tree-sitter/>.
- Rust grammar: <https://github.com/tree-sitter/tree-sitter-rust>.
- TypeScript and TSX grammars: <https://github.com/tree-sitter/tree-sitter-typescript>.

## Goals

1. Detect likely duplicate Rust functions, impl methods, trait default methods, TypeScript functions, class methods, object methods, and arrow functions.
2. Treat renames, literal changes, and local variable differences as incidental by default.
3. Preserve enough syntax signal to avoid floods of false positives.
4. Support both Rust and TypeScript in one tool and one output schema.
5. Make the CLI useful before building editor integrations or CI integrations.
6. Keep the implementation small enough to audit and evolve.

## Non-Goals for the First Release

- No semantic equivalence proof.
- No type checking.
- No full language-server integration.
- No automatic refactoring.
- No cross-language duplicate matching by default.
- No dependency on a valid Cargo workspace or TypeScript project config.
- No guarantee that every macro-expanded Rust duplicate can be detected.

## Product Shape

### CLI

```bash
dryer [options] [file-or-directory ...]
```

Default behavior:

- If no paths are supplied, scan the current directory.
- Include `.rs`, `.ts`, and `.tsx`.
- Ignore common generated/build directories: `target`, `node_modules`, `dist`, `build`, `.git`, `.next`, `coverage`.
- Compare candidates within the same language by default.
- Print text output optimized for quick terminal review.

Core options:

```text
--threshold N          Minimum similarity score, default 0.82
--min-lines N          Minimum source lines in a candidate, default 6
--min-nodes N          Minimum normalized syntax nodes, default 35
--language L           rust, typescript, or all; default all
--format F             text, json, or sarif; default text
--json                 Same as --format json
--sarif                Same as --format sarif
--include GLOB         Additional include glob
--exclude GLOB         Additional exclude glob
--cross-language       Experimental Rust/TypeScript comparisons
--max-candidates N     Limit reported candidates after sorting
--config PATH          Load dryer.toml from an explicit path
```

Example text output:

```text
DUPLICATE rust score=0.89
  crates/api/src/billing.rs:42-78 function invoice_summary
  crates/api/src/receipts.rs:18-55 function receipt_summary

DUPLICATE typescript score=0.86
  web/src/orders.ts:91-128 function buildOrderRows
  web/src/invoices.ts:44-83 function buildInvoiceRows
```

Example JSON shape:

```json
{
  "candidates": [
    {
      "score": 0.89,
      "language": "rust",
      "left": {
        "file": "crates/api/src/billing.rs",
        "start_line": 42,
        "end_line": 78,
        "kind": "function",
        "name": "invoice_summary"
      },
      "right": {
        "file": "crates/api/src/receipts.rs",
        "start_line": 18,
        "end_line": 55,
        "kind": "function",
        "name": "receipt_summary"
      },
      "left_nodes": 214,
      "right_nodes": 229,
      "shared_fingerprints": 109,
      "total_fingerprints": 122
    }
  ]
}
```

## Architecture

Use Rust for the implementation and ship one native CLI.

Proposed initial layout:

```text
Cargo.toml
crates/
  dryer-cli/
    src/main.rs
  dryer-core/
    src/
      lib.rs
      config.rs
      files.rs
      language.rs
      parse.rs
      candidates.rs
      normalize.rs
      fingerprints.rs
      match.rs
      output.rs
      report.rs
plans/
  rust-typescript-dryer-plan.md
```

The core crate should have no terminal concerns. The CLI crate should only parse arguments, load config, call the core library, and handle process exit codes.

Key dependencies:

- `tree-sitter` for parsing.
- `tree-sitter-rust` for Rust.
- `tree-sitter-typescript` for TypeScript and TSX.
- `ignore` for gitignore-aware file walking.
- `clap` for CLI parsing.
- `serde`, `serde_json`, and optionally `serde_yaml` or `toml` for config/output.
- `rayon` for parallel parsing and fingerprinting after the single-threaded version is stable.
- `similar` or `pretty_assertions` only in tests if needed.

## Processing Pipeline

1. Load configuration.
2. Expand input paths into files using gitignore-aware walking.
3. Detect language from extension.
4. Parse source with the correct Tree-sitter grammar.
5. Extract candidate units from the syntax tree.
6. Normalize each candidate into a compact language-neutral-ish syntax tree.
7. Count normalized nodes and source lines; discard tiny candidates.
8. Generate subtree fingerprints.
9. Compare candidate pairs using Jaccard similarity.
10. Sort by descending score, then stable source location.
11. Emit text, JSON, or SARIF.

## Candidate Extraction

### Rust v1 Candidates

Extract:

- `function_item`
- methods inside `impl_item`
- default method bodies inside trait definitions
- closures only when they exceed the line/node threshold and are assigned to a binding or passed as a named argument-like child

Skip initially:

- macro definitions
- macro invocations without parseable bodies
- generated files detected by path or config
- tiny getters/setters unless the user lowers thresholds

Candidate metadata:

- file path
- start/end line
- node kind
- visible name when available
- containing impl or trait when available
- language = `rust`

### TypeScript v1 Candidates

Extract:

- `function_declaration`
- `method_definition`
- object literal methods
- `arrow_function` assigned to a variable or property
- exported function-like declarations

Skip initially:

- anonymous callbacks below thresholds
- declaration-only `.d.ts` files by default
- generated bundles and build outputs

Candidate metadata:

- file path
- start/end line
- node kind
- visible name when available
- containing class/object when available
- language = `typescript` or `tsx`

## Normalization Design

`dry4clj` can normalize Clojure lists while preserving list heads. Rust and TypeScript need more nuance because syntax carries more role information.

Use a normalized node model:

```rust
enum Norm {
    Node {
        kind: NormKind,
        role: Option<Role>,
        children: Vec<Norm>,
    },
    Token(TokenKind),
}
```

The serialized fingerprint input should be deterministic and compact, not a direct dump of source text.

### Preserve by Default

Preserve:

- AST node kind.
- Control-flow shape: `if`, `match`, `for`, `while`, `loop`, `try`, `catch`, `return`, `await`, `yield`.
- Operators: arithmetic, comparison, boolean, assignment, nullish, optional chaining, Rust range operators.
- Collection shape: arrays, tuples, vectors, objects, maps, structs, enum variants where parseable.
- Call shape.
- Macro name for Rust macro invocations.
- JSX/TSX structural element shape, but not literal text.

### Normalize by Default

Normalize:

- local identifiers
- function parameter names
- literal values
- string contents
- numeric values
- comments
- whitespace
- most type names
- import paths

### Configurable Name Sensitivity

Names are the biggest false-positive/false-negative control, so make this explicit:

```toml
[normalization]
name_mode = "balanced" # loose | balanced | strict
```

Recommended semantics:

- `loose`: normalize nearly all identifiers and property names.
- `balanced`: normalize locals and literals, but preserve externally meaningful call/property names.
- `strict`: preserve most identifiers, useful for near-copy detection.

Default should be `balanced`.

## Fingerprinting and Similarity

Start with the same conceptual scoring as `dry4clj`:

```text
score = shared subtree fingerprints / union of subtree fingerprints
```

Use stable hashes over serialized normalized subtrees instead of storing full strings everywhere.

Important implementation detail: keep both counts and optional debug data. The matcher only needs hashes, but tests and diagnostics benefit from being able to inspect normalized trees.

### Set vs Multiset

`dry4clj` uses sets. For Rust and TypeScript, repeated syntactic shapes are common, so a multiset option may produce better scores.

Recommended path:

1. v1: set-based Jaccard for simplicity and parity with `dry4clj`.
2. v1.1: add weighted/multiset Jaccard behind an experimental config flag.
3. Keep default unchanged until tested on real repositories.

## Matching Strategy

Naive pairwise comparison is fine for small repositories but will become expensive.

Implementation phases:

1. Start with O(n^2) candidate comparison, grouped by language, with clear tests.
2. Add cheap blocking before comparison:
   - language
   - candidate kind
   - rough node-count bucket
   - fingerprint count bucket
3. Add an inverted fingerprint index:
   - fingerprint -> candidate IDs
   - estimate overlap before exact Jaccard
   - only exactly compare pairs with enough possible overlap to reach the threshold
4. Consider MinHash/LSH only if real repositories need more scale.

The first release should prioritize correctness and intelligible behavior over clever indexing.

## Output Modes

### Text

Default. Human-readable, close to `dry4clj`.

### JSON

Primary machine format. Stable schema, suitable for tests, dashboards, and future editor integrations.

### SARIF

Useful for GitHub code scanning and CI annotations. It can be implemented after JSON stabilizes.

## Configuration

Support `dryer.toml`:

```toml
threshold = 0.82
min_lines = 6
min_nodes = 35
format = "text"
cross_language = false

include = ["**/*.rs", "**/*.ts", "**/*.tsx"]
exclude = [
  "**/target/**",
  "**/node_modules/**",
  "**/dist/**",
  "**/*.generated.ts",
  "**/*.gen.ts"
]

[normalization]
name_mode = "balanced"
preserve_call_names = true
preserve_property_names = true
preserve_type_names = false

[languages.rust]
include_macros = false
include_closures = "large-only"

[languages.typescript]
include_dts = false
include_jsx = true
include_callbacks = "large-only"
```

CLI flags should override config.

## Testing Strategy

Unit tests:

- file discovery and excludes
- language detection
- candidate extraction for Rust
- candidate extraction for TypeScript
- line range calculation
- normalization snapshots
- fingerprint generation
- Jaccard scoring
- output formatting

Fixture tests:

- Rust positive duplicate with renamed locals/literals
- Rust negative near-shape case with different control flow
- TypeScript positive duplicate with renamed params/properties
- TypeScript React/TSX component duplicate
- generated/build path exclusion
- mixed Rust/TypeScript repository scan

CLI tests:

- no candidates
- text output
- JSON output
- invalid option handling
- config override behavior

Regression fixtures should live under `fixtures/` and stay small.

## Release Milestones

### Milestone 0: Skeleton

- Rust workspace.
- CLI parses options.
- Config model exists.
- File walker finds `.rs`, `.ts`, `.tsx`.
- JSON/text output schema exists with empty results.

### Milestone 1: Rust-Only MVP

- Parse Rust via Tree-sitter.
- Extract functions and impl methods.
- Normalize Rust candidates.
- Fingerprint and compare candidates.
- Report text and JSON.
- Fixture tests for obvious positives and negatives.

### Milestone 2: TypeScript MVP

- Parse TypeScript and TSX via Tree-sitter.
- Extract functions, methods, and assigned arrow functions.
- Normalize TypeScript candidates.
- Reuse the same matcher and output schema.
- Add TS/TSX fixture coverage.

### Milestone 3: Usability Pass

- Add `dryer.toml`.
- Add gitignore-aware walking.
- Add default excludes.
- Add stable sorting and max-candidate limiting.
- Improve error messages for parse failures and unreadable files.

### Milestone 4: Scale Pass

- Parallel parsing/fingerprinting.
- Candidate blocking.
- Inverted fingerprint index.
- Benchmark on at least one medium Rust repo and one medium TypeScript repo.

### Milestone 5: CI/Tooling

- SARIF output.
- Exit-code policy:
  - `0`: ran successfully
  - `1`: candidates found when `--fail-on-duplicates` is set
  - `2`: usage/config error
  - `3`: scan/parsing error when not ignored
- Optional baseline file to suppress accepted duplicates.

## Open Design Questions

1. Should the default compare only within the same language, or should cross-language matching be a first-class feature?
2. How strict should `balanced` name preservation be for method/property names?
3. Should Rust macros be ignored entirely at first, or represented as macro-name plus token-tree shape?
4. Should `.tsx` JSX element names be preserved by default?
5. Should duplicate groups be built transitively, or should v1 only report pairs like `dry4clj`?
6. Should the CLI default path be `.` or `src`? For Rust/TypeScript repositories, `.` plus strong excludes is probably more useful.

## Recommended First Implementation Slice

Implement Milestones 0 and 1 together:

1. Create the Rust workspace and CLI.
2. Add gitignore-aware file discovery for `.rs`.
3. Parse Rust functions and impl methods.
4. Normalize enough Rust syntax to detect renamed versions of the same function.
5. Emit text and JSON.
6. Add fixtures proving one positive duplicate and one negative non-duplicate.

This gives us a real tool quickly while keeping the TypeScript parser and cross-language questions out of the critical path. Once the Rust path is working, TypeScript should be mostly another extractor and normalizer feeding the same core pipeline.
