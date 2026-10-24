# Optional date model

Enable Sapid's `dates` feature to use the date family. The feature adds no
third-party dependency and does not change the default dependency graph. The
model is a finite proleptic-Gregorian Rust API, with supported civil years
`1..=9999`. It has no global clock, ambient locale, timezone database or serde
integration.

```rust
use sapid::dates::*;

let reference = CivilDate::parse_iso("2024-02-29")?;
let bounds = DateBounds::new(
    Some(CivilDate::parse_iso("2024-01-01")?),
    Some(CivilDate::parse_iso("2025-12-31")?),
)?;
let provider = DatesProvider::new(reference)
    .locale(DateLocale::ZhCn)
    .week_start(Weekday::Sunday)
    .today(Some(reference))
    .bounds(bounds);
```

`DatesProvider` is an explicit copyable context passed to date controls. Its
reference date is required so SSR and client construction share an intentional
initial month. `today` is a separate optional civil value and defaults to absent.
The host clock is never read. Locale and first weekday are independent settings;
the initial finite locales are English and simplified Chinese. There is no
implicit locale negotiation or operating-system format policy.

| Type | Meaning |
|---|---|
| `CivilDate` | A valid calendar date; no clock time or timezone |
| `YearMonth` | A valid year/month, with finite first/last day |
| `LocalTime` | Hour/minute/second/nanosecond wall-clock value; no date or offset |
| `LocalDateTime` | Explicit civil date plus wall-clock time; still no absolute instant |
| `UnixInstant` | Signed Unix seconds plus nonnegative fractional nanoseconds |
| `FixedOffset` | An application-supplied constant minute offset, `-23:59..=+23:59` |
| `DateBounds` | Inclusive optional min/max, checked for reversed bounds |
| `CivilRange` | An ordered inclusive pair of dates |
| `MonthGrid` | Exactly the current 42 day cells; impossible edge dates are `None` |

Civil parsing is strict ASCII `YYYY-MM-DD`; it rejects whitespace, short fields,
invalid days, year zero and trailing timezone text. Leap years follow the 4/100/400
Gregorian rule. `LocalTime::parse_iso` accepts exactly `HH:MM`, `HH:MM:SS` or
`HH:MM:SS.fraction` with one to nine decimal fractional digits. It rejects 24:00,
leap seconds, offsets and excess precision. Formatting emits a canonical date
or a time with seconds and a trimmed nonzero fractional part. It does not retain
an editing draft's spelling or fractional digit count; input controls keep drafts
separately.

Date arithmetic is checked. `add_months` requires the caller to choose
`MonthRollover::Clamp` or `Reject`: January 31 shifted to February either clamps
to its last day or returns an error. Day arithmetic never wraps the supported
range. Ordinal conversion uses bounded year/month comparisons, without a cache
or scan over previous dates. Month grids use a constant 42 cells and explicitly
leave dates before 0001-01-01 or after 9999-12-31 empty.

Converting local values to an instant requires `resolve_fixed(offset)`, and
converting back requires `instant.to_local(offset)`. Negative Unix seconds use
Euclidean day division. Overflow or conversion outside the supported civil range
returns a typed error. `Z` and signed `HH:MM` parse as known fixed offsets;
`-00:00` normalizes to known UTC, without RFC3339's unknown-offset interpretation.
IANA timezone identifiers and daylight-saving ambiguity/gap resolution are not
implemented. Supplying a fixed offset does not infer any daylight-saving rule,
and local date/time values never silently become UTC.

Seven model tests passed through Cargo with the optional feature:
`D:/BuildCache/sapid/dates-model-rust.log`. They cover all 146097 dates of a full
Gregorian cycle, first/last day of every month through year 9999, ISO rejection,
leap boundaries, checked arithmetic, all seven week starts, fixed 42-cell grids,
exact local time fractions, negative Unix timestamps, every fixed-offset extreme,
conversion overflow, inclusive ranges and provider locale/week-start policy.
Strict model checking passed in `dates-model-clippy.log` under the same build
root. Calendar rendering and host acceptance are documented separately once
their component implementation is complete.
