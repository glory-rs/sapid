# Context, Sources and InlineCitation

The optional `chat` feature exposes explicit application data without a model
catalogue, preview fetcher or credibility service.

`context().usage(ContextUsage { used, capacity, unit })` shows exact integer
usage text and a real read-only percentage ProgressBar. Counts are bounded by
1e12, units are nonblank canonical single-line text of at most 64 bytes, and
a known capacity must be positive. Usage may exceed capacity: the bar saturates
at 100%, while exact text reports both original counts and the excess. Unknown
capacity is indeterminate and has no invented numeric current value.

`sources("docs").records(records)` displays a finite intrinsic list of
`SourceRecord`s with real links, literal application previews and explicit
`SourceTrust` labels. The UI says “Application trust: verified/unverified/
conflicting”; it does not independently verify a claim. At most 128 distinct
1–64 byte ASCII keys, 256-byte nonblank titles, 4096-byte destinations and
8192-byte nonblank previews fit a combined 256 KiB metadata budget. Validation
precedes cloning/publishing; invalid updates retain the accepted snapshot.
Current keyed rows preserve link identity. No source history or fetched preview
cache is installed. An empty snapshot removes all source rows.

Destinations use an explicit finite subset: `http://` or `https://` with an
ASCII DNS/IPv4 host and optional numeric nonzero u16 port, or `#identifier`
with a 1–64 byte ASCII identifier. Whitespace, controls, credentials, ambient
paths, executable/data schemes, empty hosts and unsupported host forms reject.
Internationalized hosts must be supplied as ASCII punycode. This is not a full
URL parser or transport policy; the host/application still owns navigation.

`inline_citation(record).label("[1]")` creates a genuine inline link. Its
accessible name includes the citation label, source title and application trust.
The title/preview remains supplied data; no popup or automatic request is
inferred. `Sources::on_navigate` and `InlineCitation::on_navigate` are explicit
application overrides producing `SourceNavigation { key, url, open }`. Without
an override the host performs ordinary link navigation. Source updates never
execute navigation. Removing the Owner retires callbacks and controls.

`tests/chat_evidence.rs` checks real links, literal previews, stable keys,
application trust, callback ownership, safe destination subsets, total/record
budgets and exact/unknown/over-capacity ranges. The seven combined tests pass
in `chat-evidence-tests.log`, with strict checking in
`chat-evidence-strict.log`. `chat-evidence-native-tests.log` verifies four actual
WindowApp themes with link/text bounds, hit testing, focus, requests, numeric
ranges, stable identities, raster surfaces and cleanup;
`chat-evidence-native-strict.log` denies warnings. Family97 `citations.html`
and the original `chat-evidence-acceptance.js` supply CSR/adoption/browser checks;
immutable FORMAT12 Web40 passes all **20 combined original reports**, errors
`[]`, Chromium148.0.7778.96 (`chat-evidence-browser-web40.json`, raw
`chat-evidence-plots-browser-web40.log`). Four-theme CSR/adoption checks include
exact ranges, unknown/overflow states, literal previews, callback ownership,
stable link identity, true fragment navigation and actual AX citation names,
plus focused SSR adoption, mobile geometry and Owner cleanup. Mobile screenshots
were visually inspected. Projection and
native layout do not establish native AT client or screen-reader acceptance.
