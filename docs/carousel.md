# Carousel

`carousel()` is an index-addressed page widget. It renders one visible slide,
real Previous/Next buttons, a numeric position, optional keyboard/swipe input,
and optional application-supplied autoplay. It has a named generic group with a
tab stop; slides are named groups, without invented carousel or interactive roles.

```rust
sapid::carousel()
    .count(1_000_000usize)
    .current(current)
    .overscan(3)
    .on_current_change(move |next| current.revise(|value| *value = next))
    .render_slide(|index| slide_for_index(index))
```

The factory receives a numeric index, not the whole business source. Count is
bounded by `MAX_CAROUSEL_COUNT` (1,000,000); overscan is 0 through 3, so there
are at most seven current slide Owners. Only those indices are visited. A
retained index keeps its actual widget/editor Owner and physical identity.
Leaving the window retires it; returning later builds a new Owner. Offscreen
window members are hidden and inert, and never become extra AX-visible slides.
The caller must give an index stable business meaning while it is retained.

`current(source)` is controlled, independent of setter order. Buttons, root
arrow/Home/End keys and completed swipes propose through `on_current_change`;
the callback can accept or refuse. Internal acceptance and typed callbacks run
before generic observers. External source writes are silent. Uncontrolled
`default_current` follows local proposals and silently clamps when count shrinks.
Controlled invalid count/current pairs fail validation before building another
window. Empty count has no slides and disables page controls. `looping(true)`
explicitly enables wrapping; edge clamping is the default.

`orientation` sets horizontal or vertical input. Horizontal `direction(Rtl)`
mirrors both arrows and swipes. Input fields and other interactive descendants
keep native editing/activation behavior. A swipe uses a real captured pointer
after six pixels establish axis intent; the perpendicular pan remains native.
Release changes a page only beyond 15% of the measured extent, clamped to
24–80px. There is one current gesture and at most one measurement/query task;
resize bursts coalesce, with one final remeasurement when needed. A changed
page while a gesture is active invalidates its local proposal.

The shared `SwipeCapture { axis, revision }` declaration carries its captured
revision in every typed `Event::Swipe`. Equal revisions require equal axis;
regressions and future input reject, and old input is ignored. Clear retains
only the live node's numeric watermark, preventing same-revision ABA; retirement
releases it. Foreign/reused origins, non-finite geometry and invalid targets fail
before callbacks. Hidden/inert/disabled/suspended business input is suppressed,
while live terminal cancellation can finish cleanup. The host owns one current
controller; second fingers, lost capture, blur, viewport changes and declaration
revocation cancel it. Web uses `pan-y pinch-zoom` or `pan-x pinch-zoom`, preserving
orthogonal scrolling and browser zoom.

`autoplay(|| Box::pin(actual_delay()))` creates one revocable Owner-owned wait.
There is no default clock, per-frame Cage, or background timer history. The
caller must provide a genuine asynchronous delay; an immediately ready future
is not a timer. Focus within the carousel, mouse/pen hover, disabled state,
`paused(source)` and the independently named Pause/Resume button revoke the
current wait. Moving inside a hovered boundary keeps it paused. Teardown revokes
late wakes. A synchronous SSR without a task service emits the deterministic
first shape and never constructs the future; `Scope::has_task_host()` observes
only the injected service's presence. An installed closed/limited service still
returns its usual spawn error. A stream with an explicitly installed executor
may run its supplied clock and remains subject to its outer request lifetime.

Theme/size/class/style/name/node-ref/enabled and generic observers follow Common
builder rules. Four themes and unstyled mode share the same semantics. Unstyled
removes theme colors/borders while retaining the finite width/overflow policy;
explicit style overrides remain last.

## Host coverage and evidence

Rust evidence under `D:/BuildCache/sapid`:

- `carousel-tests-final.log`: six widget tests plus three pure model tests,
  including the million-slide/seven-factory budget, retained editor Owners,
  controlled refusal/source silence, callback order, count shrink, invalid inputs,
  autoplay cancellation and synchronous SSR without a future construction.
- `core-task-host-test.log`: service presence is pure and rejects retired Owners
  and dead runtimes.
- `swipe-render-tests-final.log`: four shared declaration/revision/guard/queue
  tests. `swipe-html-tests.log` verifies field patches against the full-shape
  oracle, including Keep, Clear, axis/revision changes and default reset.
- `swipe-native-tests.log`: real native mouse pointer dispatch verifies axis
  intent, release, configuration cancellation and editor exclusion.
- `carousel-native-debug0.log`: two native tests; all four themes use the real native engine and
  CPU pixels, finite window boxes, retained editor identity, physical focus and
  resized viewport bounds. The current second test verifies real native hover cancellation/resume and
  actual injected-clock polling/cancellation through WindowApp.observe. Builds
  use DEV/TEST_DEBUG=0 and INCREMENTAL=0. This does not prove OS touch gestures.
- `carousel-fixture-ssr.log`: fixture first shape has seven main slide bodies,
  deterministic counters and the real finite swipe metadata.

Current Glory delivers native PointerWithin from actual mouse/pen boundaries.
The default hover-pause policy is supported; `pause_on_hover(false)` is an explicit
application choice. The 2026-10-29 native test moves a real engine pointer, rejects
a stale timer wake while hovered, resumes after leaving, and checks explicit
pause cancellation. See `native-final-v2-sync.log`. These headless protocol and
geometry tests do not establish OS touch or assistive-technology announcements.

Web38's original strict run failed the first keyboard assertion, preserved in
`carousel-browser-web38.json` and `carousel-actual-web38.log`. Diagnosis found
missing Web root-navigation translation and Move incorrectly clearing hover.
Both source fixes and regressions are retained. `carousel38-rest-diagnostic2.log`
is explicitly diagnostic-only: it used browser shims to inspect the remaining
original checks and is not completion evidence. Final unmodified acceptance on
immutable Web39 passed all **9** groups with Chromium **148.0.7778.96**:
`carousel-browser-web39.json` and `carousel-actual-web39.log`, with `errors: []`.
The eight four-theme CSR/SSR groups verify actual root keyboard, mouse swipe,
RTL, native editor/UTF-16 selection preservation, controlled refusal/source
silence, seven-factory bounds, autoplay timer/hover/focus/manual pause, disabled,
unstyled, 390px layout and Owner cleanup. The ninth uses real CDP touch input for
axis admission, second-finger cancellation, captured-revision retirement,
orthogonal pan and Clear. No browser shim or reduced assertion was used in the
final run. `carousel-green-dark-hydrate.png` records the final mobile appearance.

HTML format 12 adds the finite swipe axis/revision attributes, pure field delta
and full shape agreement, classic Start/End versioning and bounded lexical guards.
Old format 11 probes reject before adoption/stream mutation. This does not add a
pre-WASM swipe input driver to the classic renderer.
