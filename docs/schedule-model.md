# Schedule civil model and bounded request windows

The optional dependency-free `dates` feature exports the schedule model.
`ScheduleInterval` and `ScheduleWindow` use **half-open civil intervals**:
`start <= time < end`. Adjacent intervals do not conflict. Both endpoints are
actual `LocalDateTime` values; start must precede end. All-day intervals use two
explicit midnight boundaries, with an exclusive final date. An end beyond the
representable civil domain is rejected; the final year does not silently wrap.
An event and a request window are each limited to 366 days.

`ScheduleData` contains one authoritative requested window and one explicit
`ScheduleZone`. Civil has no instant. FixedOffset is an explicit offset in
minutes. Application names a bounded application key; it supplies no IANA rules.
`ScheduleContext` can resolve an interval with the same explicit DateTime policy
used by DateTimePicker. Gaps/folds/wrong candidates and reversed instant endpoints
are typed failures. No wall-clock duration is silently converted into UTC time.
Application resolvers must be called outside reactive-value/registry borrows.

`ScheduleEvent` has an immutable bounded key/title, interval, optional resource,
optional recurrence and exception dates. Daily, Weekly, Monthly and Yearly rules
have an explicit nonzero interval and Never/Count/Until stop. Weekly rules specify
both weekday bits and week start. Missing month-end/leap days use explicit Skip
or Clamp; no default rollover is inferred. Count includes rule candidates before
exceptions are removed, so an exception does not extend a finite recurrence.

Expansion examines candidate dates only in the request window plus at most one
bounded event-duration lookback. It never begins walking at an ancient anchor.
Candidate indices are arithmetic; missing-day COUNT calculations examine at most
one Gregorian 400-year pattern (4800 month slots), independent of distance from
the anchor. Repeated occurrences preserve local wall-clock duration. Resolving
their instant endpoints is a separate explicit operation. Results are ordered by
start, end, then current event index; no previously visited window is cached.

Budgets are explicit:

- Events: 512; keys: 1–64 ASCII identifier bytes; titles: at most 256 bytes.
- Exceptions: 128 per event, sorted and deduplicated after bounded admission.
- Current expansion: at most 2048 occurrences and an explicit caller limit.
- Resources: 64; a requested visible resource slice: 1–8 entries.
- Conflict results: 2048 and an explicit caller limit.
- Timeline slots: at most 96; whole minute coordinates, end may equal 1440.

All iterator admission reads at most `limit + 1` values, including duplicate
sentinels, before rejecting the complete configuration. Budget overflow returns
an error rather than silently truncating an event list or recurrence result.
Resources use complete bounded keys; unknown event resource references are
explicitly rejected by `validate_resources`.

`conflicts` lets the application choose SameResource or AllResources. It reports
indices in that deterministic current expansion. `layout_day` clips timed events
to an explicit `ScheduleTimeWindow` and computes overlap clusters and reusable
lanes. Touching intervals reuse a lane; all members of an overlap cluster use its
maximum lane count. All-day events are kept out of timed lanes. A resource key
filters the current column; absence combines resources into one column.

`SchedulePermissions` defaults to selectable, with moving/resizing disabled.
`validate_edit` checks a typed Move/ResizeStart/ResizeEnd proposal against the
actual current occurrence, original interval/resource and permissions. Moves must
preserve civil duration; endpoint resizing preserves the opposite endpoint and
resource. Validation does not mutate events, automatically edit a recurring series
or bypass application authorization. The application decides whether an accepted
occurrence edit becomes an exception, a replacement or a series update.

Current evidence is model/foundation evidence, not completion of Schedule views:

- Eight recurrence/window/zone/budget tests pass in
  `D:/BuildCache/sapid/schedule-model-rust.log`. Monthly/yearly candidate counts
  are independently checked across the complete 9999-year civil domain;
  an anchor in year 1 expands directly into a window in year 9999.
- Resource admission/conflict and permission/stale-edit tests pass in
  `schedule-resource-rust.log` (two tests).
- Timeline clipping/lanes, midnight slot boundaries and resource filtering pass
  in `schedule-layout-rust.log` (two tests).
- Combined model/layout strict Clippy passes in `schedule-foundation-clippy.log`.
  Schedule/Agenda lib+component strict and native focused strict also pass in
  `schedule-clippy.log` / `schedule-native-clippy.log`.

The real current-window Schedule/Agenda composition and native/Web35 evidence are
documented in [schedule.md](schedule.md). Day/Week/Month/Year/MobileMonth and resource
UI remain in progress; the pure lane/edit models do not claim those views exist.
