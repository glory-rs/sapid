# ModelSelector

`model_selector(key).models(records)` consumes an application-provided catalog.
`ModelRecord` separates stable selection `key`, displayed `name`, provider group,
successful form `value` and `available`. There is no bundled service/provider,
price list, network loader or credential handling. The widget uses a real editable
Combobox, named Listbox provider groups and a real successful Select proxy.
`.field(context)` preserves the Field label; an independent widget defaults to
`Choose model`.

`.value(source).on_change(...)` controls the accepted model key. The callback
proposes only the chosen record; it never writes that source. The current accepted
model/provider is displayed independently from the search editor. `.search`
and `.on_search_change` control the query separately. A local accepted query may
show a proposed model's name even when the selected key is refused; the explicit
accepted-model description and FormData retain the accepted model. Real editing
can propose clearing a mismatched key. Choosing a refused option does not also
emit a spurious clear proposal. Search matches declared model names, including
normal browser composition/keyboard behavior from the shared Combobox.

At most 1,024 declared models and 32 provider groups are admitted. Stable keys
are at most 64 bytes; provider/name/business values are nonempty canonical
single-line strings up to 256 bytes. Total catalog text is charged to a 128 KiB
budget before creating controls. Duplicate/unknown keys and oversized metadata
reject; million-item iterators consume only the first 1,025 entries. Only the
selected record is cloned for a callback. Immutable declared metadata is bounded;
no catalog/history cache is populated by searching.

Native explicitly rejects the editable Choice facet. It does not replace model
selection with unrelated buttons. `D:/BuildCache/sapid/chat-actions-native-tests.log`
tests that rejection while preserving and querying an earlier accepted text
prefix. `chat-actions-tests.log` includes three model tests: real grouped option
metadata/business values, controlled refusal/availability, and declaration/value
budgets. `chat-model-refusal-baseline.log` records the extra-clear proposal before
the shared correction; `chat-model-combobox-regressions.log` passes all 26
Combobox regressions, including independent controlled query/key rejection,
composition, readonly, multiple values and the new one-selection-proposal test.
Immutable FORMAT13 Web43 passes all twelve combined actual browser cases without
page/console errors (`model-suggestions-browser-web43-accepted.json`, Chromium
148; SHA256 `8dfe17b48db2cf351a40555ebd02947677fe451772475d1e236d62023f5aeeec`).
They cover four themes in CSR/adoption, genuine provider groups, native/AX disabled
options, refused and accepted model keys, independent accepted/refused search,
exact successful business values, browser IME, current selection display, mobile
geometry, AX and retirement/remount. Two test-only assumptions were corrected on
the unchanged artifact: native `disabled` and actual AX state replace a redundant
`aria-disabled` attribute assumption; keyboard focus assertions wait for the
shared asynchronous host frame. Original failure reports and focused probes remain.
