# Showcase styling and Mantine API review

后续的全量 API/功能核对与实施批次见 [181 项 Mantine 对照报告](mantine-api-comparison.md)，
以及 [完整 Props/本地入口索引](mantine-api-surface.md)。本页保留第一轮样式修复的记录。

Reviewed 2026-10-25 against [Mantine Core](https://mantine.dev/core/package/),
[MultiSelect](https://mantine.dev/core/multi-select/),
[Pill](https://mantine.dev/core/pill/),
[ActionIcon](https://mantine.dev/core/action-icon/) and
[CloseButton](https://mantine.dev/core/close-button/).

The 149-item inventory already covers the component families in the frozen
Mantine catalogue. Family coverage does not establish API parity. This pass
addresses shared visual defects and adds small, portable action APIs.

## Changes

- MultiSelect selected values now have a wrapping row container. Previously
  each selected value was its own block and filled the form width.
- Pill, MultiSelect, TagsInput and PillsInput share tag typography, padding and
  removal dimensions. Medium tags use a 20px square remove action; ordinary
  CloseButton remains a 36px square. Tag size follows the widget's Size.
- Remove buttons reuse the named native CloseButton, with theme, disabled and
  keyboard-focus feedback. Existing proposals, composition guards, duplicate
  handling and focus return remain with their original controllers.
- Icon actions center content with zero padding. `ActionIconVariant` adds
  Default, Filled, Light, Outline and Subtle appearances to ActionIcon and
  CloseButton; CloseButton defaults to Subtle. `CloseButton::icon` allows a
  custom decorative icon. Explicit styles and unstyled behavior are preserved.
- Editable choice inputs receive themed focus borders and disabled feedback.
  Dropdown options align consistently, expose selection with a decorative check,
  and distinguish hover, disabled and keyboard-focus states.
- Shared fixes live in Sapid's typed styles and opt-in stylesheet, so application
  users receive them too. The showcase adds real variant examples and hydration
  documents for actions, pills, MultiSelect and tags.
- Those four public previews now use focused, copyable demos rather than
  acceptance fixtures. Test-only before/after buttons, refusal counters and
  unrelated form surfaces no longer dominate their layout. The previews include
  useful disabled/readonly controls, real selection counts, icon sizes and variants.

## Useful remaining API gaps

| Mantine capability | Sapid state / follow-up |
| --- | --- |
| Icon action variants; replaceable close icon | Implemented in this pass. Gradient/custom-variant registries are outside this finite API. |
| MultiSelect `maxValues` | The existing 256-key safety budget is not an application selection limit. A configurable limit is useful; it should apply consistently to controlled input, picking and atomic select-all. |
| MultiSelect `hidePickedOptions` | Useful for longer choice lists; must also repair cursor state, empty groups and empty-result messaging using accepted selection. |
| MultiSelect clear button | The existing weak handle supports clear-all, and Select already has clearable. A built-in MultiSelect action should preserve input focus and obey readonly/composition/controlled refusal. |
| Loading and option render slots | Useful, but need explicit pending/empty semantics and bounded display-only content, rather than arbitrary callbacks in preflight. |

These follow-ups are recorded as gaps, not implemented or claimed as complete.
No React API, DOM workaround or new browser dependency is introduced.

## Verification

Run `scripts/check.ps1` and `scripts/check-showcase.ps1 -Serve` (pass a compatible
wasm-bindgen executable with `-Bindgen` if needed), then
`node scripts/showcase-style-browser.cjs`. The focused browser matrix covers four
themes × desktop/mobile × CSR/hydration × four widget pages, plus reduced motion.
It checks actual geometry, page overflow, removal/focus, controlled refusal,
disabled variants and keyboard focus. Reports and screenshots are under
`D:/BuildCache/sapid`; these checks do not claim OS input or screen-reader speech.

Final verification on 2026-10-25:

- `scripts/check.ps1` passed: default/all-feature Sapid tests and doctests,
  required Glory crate tests, and strict Clippy. The log records 2,635 passing
  test executions across 412 suites (`style-library-gate.log`).
- Showcase host/wasm32 strict Clippy, 19 Rust tests, WASM and 36 SSR documents
  passed (`style-showcase-build.log`). Inventory and showcase generators pass
  `--check`; copied sources are the modules compiled into these previews.
- Chromium 148.0.7778.96 passed 270 browser groups: 182 existing showcase checks
  (including all 149 routes), 64 focused style cases, 16 Theme Studio checks and
  eight navigation checks. Every report has an empty errors array.
- Final assets are `assets-4401050d66aafd5f2435` on the local port 8795 server.
  Reports are `showcase-style-browser.json`, `showcase-browser.json`,
  `theme-studio-browser.json` and `showcase-navigation-browser.json` under the
  D-drive build root. Inspected screenshots are `style-{multi-select,tags,pills,actions}-{1440,360}.png`.
