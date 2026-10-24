# Avatar and images

`avatar(name).fallback(text)` mounts a named generic group with explicit text.
It accepts an optional `AvatarShape::{Circle,Square}`, shared sizes, four themes,
`unstyled`, custom class/style and a node reference. The default
fallback is `?`; callers supply localized initials instead of relying on guessed
Unicode or locale rules. Names are nonempty and at most 256 UTF-8 bytes, fallback
text nonempty and at most 64 bytes. Reactive invalid values report an error and
retain the previous accepted presentation. Business sources remain read-only.

`avatar_group(namespace).avatar(key, avatar(...))` preserves source order,
requires stable explicit keys, and accepts at most 64 configured members. The
default displays four members; `max_visible(1..=64)` changes that limit. Additional
members produce one independently named `+N` text avatar, with a customizable
`overflow_label` callback. Omitted members mount no nodes. `overlap(0..=24)` sets
the finite pixel overlap. Child theme/size overrides take precedence over group
inheritance. The group and its text avatars have no activation or popup behavior.

Iterator setters consume only the remaining allowance plus one overflow sentinel
and reject excess configuration before creating visible nodes. They never drain
an unbounded source. Dynamic label effects and node references retire with their
Owner. `unstyled` removes the default visual declarations, including overlap;
explicit style overrides still apply.

`source(url)` adds a real decorative Image inside the named avatar. Explicit text
stays visible during loading and after failure. A successful load replaces it;
accepted source changes reset the fallback while the next image loads. Invalid
external source input preserves the last accepted presentation and can recover.
`on_load` and `on_error` receive the request revision after internal presentation
state updates. Resource changes retain the avatar wrapper and actual image node.
A text avatar claims no image role, and the optional image's alternative is
provided by its named wrapper rather than announced twice.

Evidence: `tests/avatar.rs` covers real text/name semantics, all themes and sizes,
explicit overrides, bounded million-item iterator consumption, pre-creation
validation, source rejection/recovery, identity, group overflow and Owner cleanup.
`examples/web/src/avatar_form.rs` is the shared host fixture.

`image(source)` creates an actual semantic Image. It requires `.alt(text)` or
explicit `.decorative()`, supports typed fit and dimensions, custom style/class,
theme tokens, unstyled and node references. `fallback_src(url)` makes at most one
real fallback request per accepted primary selection, including when both URLs
are equal; failure never creates an unbounded retry loop. Source URLs are nonempty
and at most 4096 UTF-8 bytes. Controlled sources remain read-only. External source
changes make no application callbacks by themselves; actual host completion calls
`on_load(revision)` / `on_error(revision)`. Typed callbacks run before generic
ImageStatus observers. Explicit fit/dimensions survive unstyled; custom style
overrides remain last. Default object fit follows the engine's Fill behavior.

`background_image(source).fill(children)` composes a relative container with a
real absolute decorative Image and an independent content slot. It defaults to
Cover, preserves content interaction and uses no HTML-only CSS URL injection.
Its dimensions and fit are explicit and finite. A completely empty background
needs dimensions from its caller to create a visible box.

Both widgets use Glory's revisioned image requests. Old request outcomes cannot
overwrite a later A/B/A selection, and retired Owners receive no late callbacks.
SSR records interests without fabricating image results; hydration observes the
adopted real image. Web uses a request-bound decode Promise with one current
controller per live image. Retiring that waiter does not promise OS/network
cancellation. Native completion follows actual engine consumption and cache
application. The native default installs no transport: hosts supply an explicit
base URL/provider or the opt-in assets loader with its origin/root/budget policy.
The acceptance provider is deterministic test infrastructure, not a production
transport. See Glory's `docs/image-request-observations.md` for the shared contract.

Additional evidence: `tests/image.rs` covers eight semantic, fallback, callback,
source-revision, theme/unstyled, composition and cleanup cases. Avatar's ninth
case checks actual image load/error/source recovery. Native `tests/image.rs`
reuses `image_form` across all four themes with real PNG consumption, decode
failure, alternate resource, Avatar text recovery, stable identity, geometry and
background content activation. The three bundled image assets include an explicit
200-response invalid image for decode failure. Browser `image-acceptance.js`
checks CSR/hydration, AX, responsive geometry, resource transitions and held A/B/A
requests. Chromium 148.0.7778.96 passed all ten Image and eight Avatar browser
reports (CSR/hydration across four themes, plus two delayed-image protocol runs).
The actual run results are `D:/BuildCache/sapid/image-browser.json` and
`D:/BuildCache/sapid/avatar-browser.json`; hydration screenshots are stored beside
them. These runs cover accessibility snapshots, not screen-reader announcements.

`Style::object_fit(ObjectFit::{Fill,Contain,Cover,None,ScaleDown})` now provides
the shared finite image content sizing contract, independently of image box
dimensions. The default is Fill; an absent declaration leaves the stylesheet
in charge. Explicit Fill resets a previous fit. Object position is centered.
HTML/Record profile v7 and native Blitz v8 advertise every value independently.
The optional CSS parser accepts exactly these five keywords and checks each
original declaration before cascade overrides. Native raster tests verify real
decoded image pixels, centered letterboxing, cover cropping, natural-size and
scale-down behavior, resizing, clearing and stable engine identity. This does
not by itself provide application image completion events.
