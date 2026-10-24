# TableOfContents

`toc_context(scope, key)` owns at most 256 live headings. Pass that context to one
`toc_document(context, key)`, register real `toc_heading(context, key, level)`
nodes inside its content, and render `table_of_contents(context, key)` wherever
navigation belongs. Keys are explicit ASCII namespaces; levels are 1–6 and labels
are nonempty canonical single-line strings of at most 256 bytes.

```rust
use glory::prelude::*;
use sapid::{table_of_contents, toc_context, toc_document, toc_heading};

let guide = widget(|scope| {
    let context = toc_context(scope, "guide")?;
    (
        table_of_contents(context.clone(), "navigation"),
        toc_document(context, "article").content(|context| {
            col().fill((
                toc_heading(context.clone(), "intro", 1).content("Introduction"),
                toc_heading(context, "details", 3).content("Details"),
            ))
        }),
    ).show_in(scope)
});
```

The index stores only live node IDs, labels, heading levels and explicit `order`
values. Equal orders retain registration order. A heading nests below its nearest
preceding lower level; skipped levels create no empty list items. Navigation uses
actual links in nested lists and `Current::Location` (`aria-current="location"`),
with an accent background and stronger type for the current section. Reactive
labels/orders update existing link nodes. Removal retires the corresponding
short Owner; reinsertion creates a new identity.

`context.current(scope)` observes the section nearest the viewport's top plus
`.offset(px)`. `context.entries(scope)` exposes the bounded live metadata.
`context.reveal(&id)` measures the registered heading and scrolls its document,
retaining the caller's actual focus. It revalidates current staged ancestry.
Unknown/retired/outside headings and unsupported layout/scroll hosts return typed
errors. Negative RTL/elastic scroll offsets remain explicitly unsupported by the
shared nonnegative ScrollTo contract. Native document drivers must run host turns
(e.g. `WindowApp::observe`); `App::settle` alone does not poll host tasks.

Measurement starts in an Owner task outside reactive flushes. Refreshes coalesce
into one batch of at most 258 heading/viewport queries, with one repeat flag;
registration changes cancel previous requests. No source collection, previously
visited headings, browser observers or host objects enter component state.
Scroll/resize notifications refresh the index. `context.refresh()` supports
application layout changes that do not resize the registered heading itself.
SSR renders the first ordered live section and performs no physical requests;
hydration starts actual measurement after adoption. `.current(source)` overrides
navigation highlighting as a read-only controlled input; it does not rewrite
that source or replace the observed context section.

The navigation and headings support theme/size/class/style/unstyled overrides.
The scrolling document defaults to 400px height; override its Style explicitly.
This component indexes headings registered through this API, including those
currently mounted by application control flow. It does not scan arbitrary HTML,
a business data source or an unmounted virtual document.

Evidence currently includes:

- `tests/table_of_contents.rs`: nine component tests, including all 24 four-node
  reorder permutations, read-only controlled current, actual staged ancestry,
  live admission limits, bounded deferred queries and retirement of late replies.
- `src/table_of_contents/model.rs`: three pure hierarchy/geometry tests.
- `examples/native/tests/table_of_contents.rs`: four themes with real engine
  geometry, document scrolling, current updates, focus and engine identity,
  CPU pixels and cleanup. Native uses the application's host executor.
- Shared `Current::Location` validation, HTML field/full-shape oracle and native
  AccessKit current mapping; no screen-reader announcement claim.
- `examples/web/table-of-contents-acceptance.js`: all eleven actual browser
  reports pass on Chromium 148.0.7778.96 against the fixed `toc-next` build,
  `D:/BuildCache/sapid/toc-browser-next.json`. Eight theme/CSR/adoption groups
  verify hierarchy, actual scroll/current, link focus, reorder identity and
  remove/reinsert; mobile, dirty adoption and Owner cleanup also pass.

Actual adoption initially caught a mismatch between SSR's first current heading
and a measuring host awaiting its first task. The index now installs the same
ordered construction seed on both hosts until its first successful measurement;
the measuring-host regression verifies that seed before polling any task.
`D:/BuildCache/sapid/toc-browser-next-initial-adoption-failure.log` preserves the
failure. The example also has a four-heading/control completeness SSR regression.
The final example spacing is also verified by the fixed root Web23 actual
eleven reports, `D:/BuildCache/sapid/toc-browser-root23.json`. The green/dark
mobile screenshot was visually inspected.
