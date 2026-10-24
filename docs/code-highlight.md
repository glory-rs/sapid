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
fixture is `code.html` (`mount_code`, family 34). The nine-case actual
CSR/hydration/selection/AX suite passed Chromium 148 against fixed `web27b`:
`D:/BuildCache/sapid/code-browser-web27b.json` contains nine reports and no
errors. It verifies all four themes in CSR/hydration, exact rendered selection
including a terminal LF, language changes, retained token identity, real long
line scrolling, dirty editor identity/focus/selection, inert AX gutter,
unstyled syntax ink, mobile containment and focused dirty adoption.

The first Web27a run exposed Chromium dropping a terminal LF from rendered
selection despite an exact DOM Range. An empty zero-width inline box supplies
a following line without adding text; the unchanged strict selection assertion
passed Web27b. JSON disclosure keyboard acceptance also covers the shared
Grid-navigation ancestor fix: boxless Groups are crossed without requesting a
nonexistent DOM Main part. Both initial failure reports remain archived. The
Green/Dark desktop and 390px mobile screenshots were inspected:
`D:/BuildCache/sapid/code-json-green-dark-hydrate.png` and
`D:/BuildCache/sapid/code-json-green-dark-mobile.png`.

`copy_button(sapid::copy_button().label("Copy code").on_result(handler))`
adds an independently configured real CopyButton. Its source is always the
complete accepted code string, excluding line numbers, token markup and
highlight decoration. Invalid incoming code cannot replace its accepted source.
A source change resets stale feedback and revokes an outstanding result. The
optional child accepts theme, status output and host feedback timer configuration;
`unstyled` on the code block also removes its copy child's theme declarations.
Without a copy child the existing display shape is retained. Native physical
clipboard writes remain explicit Unsupported, with Unavailable button feedback.
See [CopyButton](copy-button.md) for host and lifetime rules.

Code, copy-slot and JSON display focused tests passed in
`D:/BuildCache/sapid/code-json-copy-tests.log` (7 + 5 + 8 reports). The original
Code nine browser groups also passed unchanged on fixed Web28, recorded in
`code-browser-web28.json`. Copy-slot browser acceptance is a separate eight-group suite
(`code-copy-acceptance.js`) that passed fixed Web29 with Chromium 148.0.7778.96
(`D:/BuildCache/sapid/code-copy-browser-web29.json`). It verifies real clipboard
write/read round trips for exact source excluding gutter/token markup, then
source/language replacement and complete collapsed JSON. All four themes and
CSR/hydration paths passed. `json-tree-copy-browser-web29.log` records this run
and the nine-group Json Tree/CopyButton regressions. Native interactive text-range selection is not promised by code display
acceptance.
