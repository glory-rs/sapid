# TimeInput / TimeValue

Enable the dependency-free optional `dates` feature. `LocalTime` represents a
wall-clock hour/minute/second/nanosecond, independently of a date, timezone or
absolute instant. `TimeInput` edits a bounded draft in one actual text Input;
`TimeValue` displays an optional time as readonly Text.

```rust
use sapid::prelude::*;
let value = scope.cage(Some(LocalTime::new(9, 30, 0)?));
let widget = time_input("meeting-time")
    .name("Meeting time")
    .form_name("meeting_time")
    .value(value)
    .bounds(TimeBounds::new(
        Some(LocalTime::new(9, 0, 0)?),
        Some(LocalTime::new(17, 0, 0)?),
    )?)
    .on_value_change(move |next| value.revise(|old| *old = next));
# Ok::<(), sapid::DateError>(())
```

The default `TimeFormat` is 24-hour English with minute precision (`HH:MM`).
`TimeFormat::new(HourCycle::H12, TimePrecision::Second, DateLocale::ZhCn)` uses
`hh:mm:ss 上午` / `hh:mm:ss 下午`; English uses uppercase `AM` / `PM`. These are
explicit finite display conventions, without OS locale lookup. Midnight and noon
are `12:00 AM` / `12:00 PM`. `TimeValue` uses the same format and an explicit bounded
`empty_label` (at most 64 UTF-8 bytes, no NUL/CR/LF), defaulting to an em dash.

Program values must be exactly representable by the chosen display: nanoseconds
must be zero, and minute precision requires seconds zero. Invalid initial
configuration returns a typed error. An invalid external source update returns
an error from the flush, retains the last accepted UI, and permits silent recovery
when a later valid value is supplied. It never truncates business precision.

`TimeBounds` is an inclusive same-day interval. `TimeBounds::new(Some(23:00),
Some(01:00))` is rejected because the lower bound is after the upper bound; an
application requiring an overnight schedule must explicitly model its two
intervals or use a date-time domain. `contains` and explicit model-only `clamp`
are available. The input does not automatically call `clamp` during editing or
commit, and no arrow key silently rolls the date across midnight.

The draft distinguishes `Empty`, `Incomplete`, `Valid(LocalTime)`, `Invalid`,
`OutOfBounds(LocalTime)` and `TooLong`. Only Empty and Valid propose a new
`Option<LocalTime>`. Partial and invalid/outside drafts remain visible, with the
previous business time intact. An invalid/outside explicit commit calls
`on_reject`; it does not clamp. A valid explicit commit calls `on_commit` and
uses canonical display formatting. `on_draft_change` and `on_value_change` run
before a generic `InputEdit` observer. Controlled sources are read-only: callbacks
must explicitly accept proposals; refusal restores the accepted display after
the whole host tick. Local defaults work without a controlled source.

The actual editor retains native caret, selection and composition. InputEdit is
an explicit opt-in budget of 32 UTF-8 bytes. The host checks length before copying
a rejected input into business payloads; a finite rejection restores the accepted
display. Direct application Commit payloads also check the budget before making
a pending draft copy. Composition does not parse/format preedit, terminal mapping
and its matching echo publish once, and superseded input revisions are discarded.
The controller retains one pending draft and one current echo only; there is no
input history, polling, timer, query or retained owning Scope in callbacks.

Field metadata, required, invalid/error relationships, explicit `form_name`,
readonly, disabled and Owner-scoped refs are supported. Ordinary Form Enter still
commits the input before the Form submit callback. The actual successful control
contains the current display spelling, including a 12-hour suffix or an incomplete
draft. Applications needing canonical 24-hour transport should explicitly serialize
the typed `LocalTime` (and handle validation) in their submit callback. There is no
hidden HTML field or implicit HTTP serialization. `unstyled` preserves default
width and size-dependent minimum height; explicit styles can override them.

Validation completed so far:

- Four pure model tests cover every second of the day in both hour cycles and
  both supported locales, every valid UTF-8 display prefix, all 1,440 minutes,
  midnight/noon, precision loss, bounds and an oversized draft:
  `D:/BuildCache/sapid/time-model-rust.log`.
- Nine component tests cover drafts, bounds/no clamp, callback order, Chinese
  seconds, controlled refusal/invalid source/recovery, same-tick valid then
  incomplete editing, composition echo/stale revisions, readonly/disabled/Owner,
  display Text and direct million-byte Commit rejection:
  `D:/BuildCache/sapid/time-date-input-rust.log` (also includes 11 DateInput tests).

Two actual native tests passed (`D:/BuildCache/sapid/time-input-native-final.log`).
Four themes query real editor bounds, focus, hit testing, raw draft updates,
ASCII selection/caret, stable engine identity and input-background raster pixels.
The second checks twelve-hour Chinese seconds in the actual editor, unstyled
small dimensions, readonly, disabled focus rejection and Owner cleanup. The first
non-ASCII exact-selection attempt is preserved in
`time-input-native-first-selection-boundary.log` as historical evidence. Current
Glory accepts the Chinese UTF-16 range 9..11 with backward direction; the
2026-10-29 consumer reads back the actual range and retains the same editor.

The original browser suite passed **12/12**, errors `[]`, with Chromium
**148.0.7778.96** against fixed FORMAT-10
`http://127.0.0.1:8794/web31`. Eight theme/CSR/hydration cases exercise actual
Field Input/TimeValue, partial/invalid/outside/empty drafts, Form Enter and typed
snapshot order, display FormData, 12-hour AM/PM and Chinese seconds, controlled
refusal, readonly/disabled, native caret/identity, unstyled dimensions and browser
AX. Four additional cases cover same-tick valid then incomplete edits and
million-byte host rejection/recovery, composition/echo/Owner retirement, dirty
SSR takeover and touch editing/submission. Report:
`D:/BuildCache/sapid/time-input-browser-web31.json`. The sequential three-suite
raw runner log is `dates-browser-web31.log`; `time-input-browser-web31.log` is a
copy of that same log. Screenshots are `time-input-{hue}-{scheme}-{csr|hydrate}.png`;
blue light CSR and green dark hydration were visually inspected. Sapid, WASM and
native focused strict checks passed in `time-input-clippy.log`,
`time-input-web-clippy.log`, `time-input-native-clippy.log`; SSR check is
`time-input-ssr-check.log`.

Native exact revision-bound `SelectText` currently rejects
non-ASCII text explicitly. This existing host boundary is separate from ordinary
native Unicode editing and selection queries, and is not advertised as precise
programmatic Chinese range selection. OS IME, virtual keyboard layouts and actual
screen-reader announcements require separate device/client acceptance.

The original TimeInput12 suite also passed, errors `[]`, against immutable
FORMAT-10 Web33 on Chromium 148.0.7778.96:
`D:/BuildCache/sapid/time-input-browser-web33.json`; raw sequential runner log
`time-browser-web33.log`.
