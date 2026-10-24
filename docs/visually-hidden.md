# VisuallyHidden

`visually_hidden().content(text)` retains semantic plain text in an absolute,
zero-sized block with both overflow axes clipped. It does not set `hidden`,
`aria-hidden`, opacity or `display:none`. It consumes no normal-flow space and
has no Tab stop. The plain-text API excludes invisible interactive descendants.
It is not a focus-revealed skip link; AppShell provides that separate behavior.

`semantic_id(id)` provides an explicit target for a label/description relation;
`node_ref` and reactive custom classes retain their ordinary Owner lifetime.
Reactive text updates preserve the original text node. Content is bounded at
64KiB; invalid creation allocates no node, invalid updates preserve accepted
content, and a later valid value recovers. Structural clipping is fixed rather
than overridden by theme or arbitrary style. There is no decorative styling to
disable with `unstyled`.

Four Rust tests cover semantic flags, structure, escaped SSR, bounds, reactive
recovery, identity and retirement. The four-theme native fixture actually
measures a zero-sized hidden box and unchanged neighboring layout, then verifies
engine identity through a text patch. The native AccessKit regression verifies
that clipped text still supplies a button name and live description. Logs:
`D:/BuildCache/sapid/hidden-native.log` and `hidden-ax.log`.

The browser fixture tests all four themes in CSR and hydration, actual zero
dimensions, layout, Tab order, CDP accessibility names/descriptions, text-node
identity and cleanup. Its result is recorded only after an actual run; Chrome
AX snapshots and AccessKit tree checks do not establish screen-reader speech.

The actual 2026-10-24 Chromium 148.0.7778.96 run passed all eight cases with
errors=[]: `D:/BuildCache/sapid/hidden-browser.json`. Four hydrated screenshots
are saved alongside it as `hidden-{hue}-{scheme}-hydrate.png`.
