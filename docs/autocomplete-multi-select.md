# Autocomplete and MultiSelect

P4-07 `Autocomplete` and P4-08 `MultiSelect` extend the editable choice controller.
They use a real single-line Glory Input, intrinsic Button options, a named
Listbox and the host's focus-preserving Layer. Collection/RovingFocus owns only
mounted, bounded declarations. Home, End, Space and ordinary editing remain
native Input behavior; composition freezes filtering and option commands until
the terminal composition event. Pointer/touch selection retains the Input focus.

`autocomplete(key).value(Lotus<String>).on_value_change(...)` controls freely
edited text. `default_value` supplies local initial text. The optional
`selected_key`/`on_option_select` tracks suggestion identity independently.
`select_option(key, label).value(text)` inserts `text`, while the label remains
presentation. `autocomplete_option(key, text)` uses the same bounded text for
presentation and insertion. Unmatched text is valid. The actual Input owns the
successful form name/value and native `required`; there is no hidden Select.
Open Enter accepts a suggestion or commits unmatched text and closes; closed
Enter retains native form submission. `on_commit` observes text commits.

`multi_select(key).value(Lotus<Vec<String>>).on_value_change(...)` controls
selected known keys. `search`/`on_search_change` independently control the editor;
defaults apply only to local sources. Choosing an option toggles its key, proposes
an empty search and keeps the popup open. Accepted values drive real
`aria-selected`, dynamic keyed tags and a real hidden native Select with
`multiple`, submitting each declared business value. Search text has no form
name. A tag remove button returns focus to the Input. Backspace removes the last
tag only when both the host editor and the controller's accepted search are
empty and composition is inactive; a same-tick rejected empty edit cannot delete
a tag. Readonly/disabled state also protects tag removal.

`MultiSelect::on_ready` supplies a weak `MultiSelectHandle`.
`select_all()` appends enabled options matching the current accepted search and
filter, retaining existing selections outside the filter. An empty filter covers
all enabled declarations; disabled options and groups are skipped. A proposal
exceeding 256 keys fails whole before callbacks. `clear_all()` proposes no selected
keys while preserving search. Both commands fail for readonly, disabled,
composing or retired controls. Controlled sources receive proposals only.
The example uses real external buttons rather than injecting extra controls into
the Listbox, whose contents remain options/passive groups.

MultiSelect exposes ARIA required on its focusable editor. Required selection is
validated through Field/FormState (for example, a validator rejecting an empty
selected Vec). Neither the empty search nor the hidden successful-value proxy
has native `required`. The fixture demonstrates this validation and links its
error to the focusable Input.

Both widgets accept `field`, theme, size, class/style, unstyled, label/name,
enabled, readonly, invalid, form_name, open/default_open/on_open_change,
filter (`Contains`, `Prefix`, `None`), empty_message and bounded options/groups.
Listbox naming uses the Field's label relation or explicit accessible name;
typing a query must not replace the listbox's accessible name with editor text.
Controlled proposals never write a supplied Cage. Dirty reconciliation runs
after the whole drained host tick and preserves canonical echo caret/preedit.

Limits are 1024 declared options, 32 groups, 64-byte ASCII keys, 256-byte labels,
4096-byte canonical single-line search/insertion text, and 128 KiB declaration
payload. MultiSelect accepts at most 256 unique known keys. Only accepted selected
keys mount tags; 1024 options plus 256 tags use fewer than 2200 semantic nodes,
below the 4096 restricted live subtree budget. Option receivers consume at most
1025 values before typed rejection. Invalid million-element selected Vecs or
million-byte search/name sources are checked by borrowing before cloning.
Successful-control names have an independent 256-byte budget; arbitrary Field
business values do not inherit the option-value limit. No visited-key history,
external source scan, browser type or DOM query enters component state.

Glory adds finite `ComboboxKind::Autocomplete`/`Multiple` refinements, not new
generic roles. Only intrinsic single-line Inputs accept these kinds. Known
`controls` relations must agree with `listbox_multiple`; unresolved relations
remain legal during construction. Active resolution ignores missing, hidden,
disabled, suspended and retired options. `ChoiceCommand::RemoveLast` is legal
only for Multiple; Memory and direct App delivery validate the same origin and
current owner relation. The host reads only that editor's current value. HTML
full-shape and prepared-field delta lowering agree for mode/required/default
changes and preserve Input values/revisions. HTML remains format v4 and classic
stream envelope v2: these are finite attribute refinements within that schema.

Native Blitz explicitly rejects all combobox/option refinements before facet
registration, in preflight and direct apply, preserving successful prefixes and
query watermarks. These components therefore do not claim native input or AX
support. Browser CDP IME and AX tests establish browser protocol behavior, not OS
IME or screen-reader acceptance. Classic streaming applies finite metadata and
values; host Layer/input behavior begins when WASM owns the application.

Evidence:

- `src/combobox.rs` and `src/combobox_advanced_tests.rs`: 25 focused tests, including
  free values, insertion identity, composition, same-tick controlled rejection,
  required/proxy separation, disabled/readonly, bounded receivers, huge sources,
  max live tags, stable listbox naming and callback destruction outside graph
  borrows after Owner retirement; filtered/unfiltered select-all, disabled skip,
  clear-all, controlled refusal, command policy/Owner checks and whole-proposal
  rejection at the selected-key limit.
- Glory `render/tests/choice.rs`: 11 shared identity, command, owner, placement,
  late relation, hidden/retired and budget tests. HTML `choice_tests`: three
  tests, including the prepared-patch/full-shape differential oracle.
- Glory SSR `choice_stream_retains_focus_preserving_intrinsics_and_live_active_descendant`
  and `GLORY_CHOICE_STREAM_FIXTURE` produce all four finite modes and a real
  multiple Listbox. Native `blitz/tests/choice.rs`: one expanded preflight/direct
  rejection and prefix/query test covering both new modes.
- `examples/web/choice_extras_form.rs`, `choice-extras.html` and
  `choice-extras-acceptance.js`: Web14 Chromium 148.0.7778.96, 12 actual browser
  groups, errors `[]`, report
  `D:/BuildCache/sapid/choice-extras-browser-web14.json`. Four themes each CSR and
  hydration, mouse/keyboard/touch, AX multiple selection, FormData/free required,
  Field validation, tags/focus, source refusal, same-tick edit/Backspace, narrow
  viewport, Owner teardown, dirty adoption identity/caret, real browser IME and
  Rust-produced classic stream. The green/dark hydration screenshot was inspected.
- Final audit verification: isolated `choice-extras-next` contains the listbox
  naming fix and the then-current 22 focused tests' production source. The strengthened AX
  assertion confirms the listbox stays named `Countries` after editing and
  selecting; all 12 actual browser groups pass again with errors `[]` in
  `D:/BuildCache/sapid/choice-extras-browser-choice-extras-next.json`. Build and
  SSR logs are `choice-extras-next-build.log` and `choice-extras-next-ssr.log` in
  the same build root. Root Web14 was kept unchanged. The fix is ready for the
  next unified snapshot.
- Final unified Web15 verification: all 12 actual browser groups pass again,
  errors `[]`, Chromium 148.0.7778.96,
  `D:/BuildCache/sapid/choice-extras-browser-web15.json`. This includes the
  strengthened AX name assertion and real external select-all/clear-all buttons:
  filtered append retains existing outside-filter selection; unfiltered select-all
  skips the disabled group; clear-all preserves search and remains a controlled
  proposal. Successful FormData, tags and selection agree in all four themes,
  CSR and hydration. P4-07/P4-08 now have complete focused model/actual evidence.
