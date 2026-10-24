# Portal

`portal_context(scope, namespace)` owns an explicit same-App namespace. Declare
destinations with `portal_target(&context, key).fill(widget)` and source-owned
content with `portal(&context, target).content(factory)`. The target may be a
reactive bounded key. This uses Glory's ordinary semantic placement; Web, SSR
and native see the same actual parent. There is no DOM-only portal, fake dialog
role or top-layer fallback. Core ownership and callback execution remain with
the source, while semantic ancestry, inherited presentation and physical layout
follow the destination.

Content is constructed once in a short source Owner, inside one boxless Group.
Moving between live destinations preserves the physical editor, focus, caret
and composition protocol. Disabling a source or losing its current destination
parks and suspends its existing range. Re-enabling or registering the current
destination resumes that same range and local state. A retired source disposes
its moved descendants; reconstructing that source invokes a fresh factory.
Suspension is real terminal focus/composition loss, rather than an assertion
that an off-document editor stays actively composing.

At most 128 current target declarations and 1024 current sources are accepted
per context. Keys contain at most 64 ASCII semantic-token bytes. Registrations
reserve capacity before child construction or source factories run. Only live
keys, numeric registration identities and Weak source handles are retained;
empty subscriber buckets are removed. Cross-App contexts, duplicate targets,
cycles and existing restricted content/adapter rules reject explicitly.
Failed movement preserves the accepted semantic/physical prefix.

Target retirement cannot wait for a later reactive Structure callback. Each
target is created directly under the Owner registering its cleanup, and declares
the UI-only `Scope::defer_descendant_disposal` boundary. That blocks a child's
retired-ancestor optimization from deleting the target prematurely. Its cleanup
explicitly enters each still-live source Owner, moves the range back to the
source anchor and suspends it before disposing the target. If the source anchor
also belongs to the disappearing semantic subtree, the range parks as a
suspended root. A new target can then recover it without reconstructing its
editor. Scope/node terminal cleanup reclaims the boundary; there is no retired
marker history. Ordinary nodes keep the empty-set fast path and existing bulk
disposal coalescing.

Evidence under `D:/BuildCache/sapid`:

- `portal-tests.log`: eleven component/SSR tests, including both declaration
  orders, target/source/simultaneous retirement, foreign-owned sources inside
  disappearing targets, cyclic-move refusal, exact budgets and four-theme SSR
  repeatability with initially missing destinations. Failed target child
  construction also parks a still-live foreign source before disposing the target.
- `portal-ui-tests.log` and `portal-ui-strict.log`: three lifecycle regressions,
  including child-before-parent cleanup, capture destruction reentry, query and
  marker reclamation; 2000 ordinary Views still coalesce into one Dispose.
  All 151 Glory UI tests pass in `portal-ui-regressions.log` (30 reports).
- `portal-native-tests.log`: four-theme real layout movement, unchanged engine
  editor identity, actual focus/selection, target retirement/resume and raster
  output. Native geometry and input observations use the real engine.
- `portal-browser-portal-next.json`: all fifteen original browser groups pass
  against the fixed private `portal-next` HTML9 artifact, Chromium148,
  `errors: []`. They cover eight theme/CSR/hydration combinations, real connected
  moves, selection/composition protocol, dirty adoption, source retirement,
  initially missing CSR/hydrated targets and complete App retirement.
- `portal-browser-web31.json`: all fifteen original groups pass again against
  the fixed Web31 HTML10 artifact, including the source with failed-construction
  cleanup coverage; `errors: []`.

These browser checks do not establish OS IME or screen-reader acceptance. This
API introduces no HTML protocol field; the fixed artifact uses HTML9 and classic
stream envelope2.
