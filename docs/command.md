# Command / Spotlight

P4-10 is implemented and validated in the fixed root Web20 snapshot.

The palette uses a real modal Dialog and a real single-line Input, with editable
Combobox metadata controlling a mounted Listbox. Results are intrinsic Button
Options; application `on_action(key)` handlers perform commands. There is no
hidden Select value, form submission or persistent selected business value.
Keyboard navigation keeps actual input focus and uses active-descendant and the
shared finite RevealOption action. Composition freezes navigation and execution.

Open and search are independent controlled proposals. A weak Owner handle opens,
closes or toggles the palette. Registering Ctrl/Cmd+K requires one explicit live
physical ancestor through the handle; no global listener or business callback is
installed. Host editing, composition, modifier and repeat guards remain in force.

Static declarations, a stamped `CommandResponse { query, results }` controlled
results source, and `load_with(query)` are
mutually exclusive. The asynchronous path uses Owner resources: new accepted
queries, closing, disabling and Owner disposal cancel obsolete requests. Pending
results cannot activate old mounted options, even before structural work runs.
Late results cannot publish to a cancelled lifetime. `CommandHandle::cancel`
revokes the current request and `refresh` explicitly restarts it. A single
non-wrapping request generation also blocks stale actions immediately on refresh,
before the next resource effect. Loading, empty, error and cancelled states are
distinct visible content. Closed asynchronous palettes use the same inert shape
in SSR and CSR. Cold SSR without an execution host renders the pending state;
the browser starts its own request lifetime after adoption.
Static declarations match bounded label/description/keywords. Stamped controlled
and loader result sets already answer their query and are displayed in the
supplied order within each declared group; they are not discarded by an additional
literal substring filter. Ungrouped actions precede groups in declaration order.

The limit is 256 actions, 32 groups and 64 KiB total declared text. Keys use at
most 64 ASCII bytes, labels 256 bytes, description/keywords 1024 bytes each and
search 4096 bytes. Palette, action and group labels must contain non-whitespace
text. Borrowed validation precedes cloning a controlled result set;
iterator admission stops after the budget plus one rejection sentinel. Current
keyed rows are bounded well below the shared 4096-node restricted subtree limit.
No visited commands, old results or external source scan is retained. Per-option
highlight bindings compare one current mounted member token; they do not each
scan the result collection.

The dedicated `EventKind::Palette` / `Event::Palette { from }` only observes the
explicit Ctrl/Cmd+K chord and routes to the nearest live listener. It does not
subscribe to general Search/Help commands. The physical non-editor origin,
inert/disabled/retired boundaries and direct App/reference delivery are shared.
No generic key string or unbounded shortcut registry is added.

Current evidence: eleven Sapid model tests include controlled refusal, grouped
intrinsic roles, canonical input echo, stale stamped results, composition,
million-element admission, application capture cleanup, cold SSR/pending shape
equality, lazy host polling, new-query/refresh/disable/close/Owner cancellation,
nearest shortcut-scope retirement, blank/Unicode-whitespace name rejection before
node allocation and the maximum live/scrollable result set. Render's Palette test covers finite
origin constraints and suspension; native `apace/tests/palette.rs` admits the
interest while preserving the accepted Bounds watermark. The native driver now
connects Choice, editable Input, Listbox and Palette facets. Ctrl/Meta+K only
observes its nearest explicitly registered non-editor physical scope; editors,
composition, modified/repeated chords and owner retirement retain their guards.
The actual four-theme Command consumer opens a modal, searches Chinese labels,
commits through Choice and restores focus. Native/AX/CPU and mobile SDK evidence
are bounded separately from actual platform input or reader acceptance; see
[native menus](../../glory/docs/native-menus.md).
`ssr/src/stream/command_tests.rs` produces real Dialog/Input/Listbox/Option
Shell/Patch/End, originally verified with HTML v5/classic v2 on Web20. Current
producers follow HTML v13/classic v2, without a pre-WASM input driver.

The Web fixture is `examples/web/src/command_form.rs` and its independent suite is
`examples/web/command-acceptance.js`. Its original fourteen groups pass on
Chromium 148 against the fixed root Web20 snapshot, with no recorded errors:
`D:/BuildCache/sapid/command-browser-root20.json`. This includes eight theme and
CSR/hydration combinations, actual keyboard/pointer/touch/AX, asynchronous
replacement/cancel/refresh, IME, controlled refusal, oversized-search recovery,
maximum-results scrolling, dirty adoption and classic stream handoff.


当前 Apple 主机复验：既有 Menu/Menubar/ContextMenu/Command 四组浏览器脚本共
49 个报告通过（含四主题 CSR/水合）。Home 验证默认事件未被拦截，光标行首使用
平台快捷键，避免把某一浏览器宿主的 Home 位置当作所有系统的保证。当前报告为
`/Volumes/Data/Caches/glory/native-menus/browser-results.json`，保留此前失败诊断。
