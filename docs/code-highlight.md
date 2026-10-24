# CodeBlock and CodeHighlight

Enable the optional `code` feature. It adds no dependency to the default graph.

```rust
sapid::code_highlight()
    .language(sapid::CodeLanguage::Rust)
    .source(source)
    .line_numbers(true)
    .start_line(1)
    .highlighted_lines([2, 3]);
```

`code_block()` starts in Plain mode; `code_highlight()` starts in Rust mode.
Both accept a reactive `source` and reactive `language`. Plain, Rust and JSON
share one bounded lexical text model across HTML and native hosts. Tokens are
UTF-8 source ranges. Rust supports keywords, identifiers, nested block comments,
line comments, quoted/byte/C strings, characters, lifetimes, raw strings and
numbers. This is lexical highlighting, not a compiler or syntax validator.
Incomplete strings/comments remain visible through EOF. JSON highlighting also
accepts malformed drafts; use `validate_json` from the separate `json` feature
when syntax validation is required. No source is evaluated or interpreted as
HTML, and there are no HTML or JavaScript tokenizer dependencies.

`tokenize_code` returns an immutable `CodeDocument` with exact source, ordered
contiguous token ranges and line starts. Public model ranges refer to the exact
input. Widget display uses Glory's canonical text rules: CRLF/CR becomes LF and
NUL becomes U+FFFD. Source Cages remain untouched. Over-budget reactive updates
report an error and preserve the last accepted presentation; a subsequent valid
value recovers.

Limits are 65,536 UTF-8 bytes, 8,192 tokens, 2,048 lines, 32 nested Rust block
comments and 16 raw-string hash delimiters. Each highlighted-lines iterator
consumes at most 2,049 values before rejecting overflow. Highlight line values
are relative, one-based values in `1..=2048`; `start_line` changes only displayed
numbers and must be positive. No large-source virtualization is promised: this
component renders the explicitly bounded current source. It retains no visited
files or token history.

The actual `<code>` text spans are selectable, preserve whitespace and share
the source exactly once. A separate inert, nonselectable gutter supplies visual
line numbers. The code viewport scrolls long lines horizontally. Four themes,
custom style/class composition and unstyled ink are supported. Unstyled retains
whitespace and layout required by the code presentation. Generic observers do
not transform the source. Current token positions retain their Owners as source
and language change; removed positions are retired.

Focused Rust evidence: `tests/code.rs` contains seven passing checks for lexical
Unicode/ranges, finite budgets, malformed drafts, reactive identity/rejection,
theme/unstyled, SSR escaping and bounded iterators/Owner retirement. Focused
strict Clippy passed with `--features code`. The native acceptance test loops
over all four themes and passed actual multiline Bounds, long-line scroll
metrics, pixel changes, retained engine identity and editor focus. The Web
fixture is `code.html` (`mount_code`, family 34); its nine-case actual
CSR/hydration/selection/AX suite is pending execution.

Copy/clipboard is not implemented by this batch. It requires an explicit typed
host capability with permission, rejection and terminal completion rather than
an application callback presented as success. The roadmap item remains partial
until that protocol and copy interaction are implemented and verified. Native
interactive text-range selection is not promised by code display acceptance.
