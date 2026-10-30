# Sapid in Glory's DSL

Sapid builders, custom widgets and Glory `view!` values implement the same
`Widget` interface. There is no component registry, second props interface or
Sapid-specific renderer. Enable Glory's optional `dsl` feature in the application;
Sapid's default dependency graph does not include the macro compiler.

```toml
glory = { path = "../glory/crates/glory", default-features = false, features = ["dsl", "ssr"] }
sapid = { path = "../sapid" }
```

Select `web` / `hydrate`, `stream`, or the native host features in the application
as needed. The paths above assume a sibling application directory.

```rust
use glory::{Widget, view};
use sapid::Button;

fn save(label: &str) -> impl Widget {
    sapid::button().label(label.to_owned())
}

fn demo() -> impl Widget {
    view! {
        <sapid::Card>
            <div>
                <button>"HTML"</button>
                <Button label="Imported Widget type"/>
                <{sapid::button()} label="Concrete factory"/>
                <{save("Opaque Widget factory")}/>
            </div>
        </sapid::Card>
    }
}
```

| Syntax | Resolution |
|---|---|
| `<button>` | Finite raw HTML template, regardless of imported functions |
| `<Button>` | Imported Rust type: `Default` plus typed builder setters |
| `<sapid::Button>` | Explicit Rust type path; ordinary imports/aliases also work |
| `<{sapid::button()} label="Save"/>` | Rust expression returning a concrete builder with visible setters |
| `<{save("Save")}/>` | Ordinary function returning `impl Widget`; pass configuration as arguments |
| `<Type init={factory(args)} .../>` | Explicit initializer for a type without a suitable `Default` |

An expression tag with nested children closes with `</>`, for example
`<{sapid::card()}><Button label="Child"/></>`. Nested widgets use the builder's
existing `.fill` and `glory::ui::Children`. A return type declared only as
`impl Widget` hides concrete `.label` / `.fill` methods; configure it inside the
factory or pass arguments. The DSL does not guess methods or choose between crates.

Create owned reactive state with `glory::ui::widget(move |scope| { ... })` and
build the `view!` result into that scope. `Cage`, `Bond`, `Vary`, typed callbacks,
`Each` and `Show` retain their usual ownership. `#[glory::component]` is optional
syntax for generating a typed Widget builder from a function, not another
component contract. Existing handwritten builder examples use this same interface.

The complete [shared example](../examples/fixtures/dsl_demo.rs) covers raw HTML,
type and factory tags, controlled input, reactive labels and conditional children.
It contains no browser, SSR or native state. The browser and native fixtures
compile this exact file; the simple SSR example renders it too:

```powershell
cargo run --example dsl > D:/BuildCache/sapid/dsl.html
./scripts/check-dsl.ps1 -Browser -Native
# Optional isolated outputs, including the native target and compiler temporaries:
./scripts/check-dsl.ps1 -Browser -Native -BuildRoot D:/BuildCache/sapid/dsl-check
```

The script uses verified D-drive caches and owns its temporary browser/server.
`-BuildRoot` defaults to `D:/BuildCache/sapid`; portable/native targets, DSL output
and compiler temporaries use its `target`, `native-target`, `dsl` and `tmp`
directories. Each stage creates its directories before invoking Cargo, including
standalone native checks. An existing D-drive `CARGO_HOME` is preserved.
Browser outputs are `D:/BuildCache/sapid/dsl/dsl.html` and `dsl-hydrate.html`.
Checks exercise real CSR, Rust SSR adoption with dirty input/selection retained,
all four button spellings, controlled editing, branch replacement and cleanup.
The native check exercises actual engine layout, pointer events and CPU painting.

Use the standard `glory::web` host for portable mixed trees. Synchronous
`glory::ssr::snapshot`, incremental `glory::ssr::stream_with` and native hosts
accept the same Widget. Stream consumers must serve the matching
`glory::web::STREAM_SCRIPT` and hydrate the final matching state; factories need
no stream-specific implementation. Glory's `scripts/build-mixed-stream.ps1 -Verify`
checks real incremental responses including Sapid controls, keyed moves,
suspension, dirty editors and final hydration.

Raw HTML remains a finite profile; use Sapid/Glory Widget controls for controlled
editors and their typed semantics. The browser-only sparse `glory::display::App`
is an optional host optimization. Different host adapters create, adopt, serialize
or paint physical nodes; they do not introduce a second Widget interface.

## One Each for vectors and KeyedList

Both namespaces export the same Each Widget. Handwritten code uses
`glory::ui::each(source).key(key).children(factory)`; the DSL uses
`<Each each={source} key={key} children={factory}/>`.
Pass a Vec, Cage, Bond or Vary vector directly and provide an explicit stable,
unique key. For KeyedList omit `key`, since the list already owns its identities.

Vector factories receive a row with `value` and `index` cages. KeyedList factories
receive the existing member handle, with no cloned value/index mirror. Both
expose `key()`, `field(...)`, `update(...)` and `try_with_untracked(...)`.
Vector row values retain the former row-Cage snapshot behavior; publish collection
edits through the source vector. KeyedList member fields edit the list directly.
Row factories run once for retained keys; reordering preserves nodes and local
Widget state. Deletion retires the row and reinsertion creates fresh state.

The shared example's **Reorder both sources** action exercises both forms and
keeps their physical nodes intact. For initial-value-only rows use
`.seed(factory)` or `seed={factory}` instead of `children`; this mode omits the
vector's item/index sources and extra Owner. The old three-argument `each` and
the separate `each_seed` function have been removed.
