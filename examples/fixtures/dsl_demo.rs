//! One Widget shared by the SSR, browser and native examples.
use glory::{Cage, Widget, view};
use sapid::{Button, TextField, Theme};

fn action(count: Cage<usize>, theme: Theme) -> impl Widget {
    sapid::button()
        .theme(theme)
        .class("dsl-factory")
        .label("Opaque factory")
        .on_press(move || count.revise(|value| *value += 1))
}

fn branch(theme: Theme) -> impl Widget {
    view! {<div class="dsl-branch"><Button theme={theme} label="Shared Widget branch"/></div>}
}

pub fn demo(theme: Theme) -> impl Widget {
    glory::ui::widget(move |scope| {
        let theme = theme.clone();
        let factory_theme = theme.clone();
        let concrete_theme = theme.clone();
        let branch_theme = theme.clone();
        let count = scope.cage(0usize);
        let name = scope.cage("Sapid".to_owned());
        let visible = scope.cage(true);
        let values = scope.cage(vec![1usize, 2]);
        let members = scope.keyed_list(vec![1usize, 2], |id| *id).unwrap();
        view! {
            <sapid::Card theme={theme.clone()} class="dsl-demo">
                <div>
                    <span class="dsl-count">{count}</span>
                    <button class="dsl-raw" on:click={move || count.revise(|value| *value += 1)}>"Raw HTML"</button>
                    <Button theme={theme.clone()} class="dsl-type" label={count.map(|value| format!("Imported type {value}"))}
                        on_press={move || count.revise(|value| *value += 1)}/>
                    <{action(count, factory_theme)}/>
                    <{sapid::button()} theme={concrete_theme} class="dsl-concrete" label="Concrete factory"
                        on_press={move || count.revise(|value| *value += 1)}/>
                    <TextField theme={theme.clone()} class="dsl-editor" value={name}
                        on_value_change={move |next| name.revise(|value| *value = next)}/>
                    <span class="dsl-name">{name}</span>
                    <Button theme={theme.clone()} class="dsl-toggle" label="Toggle branch"
                        on_press={move || visible.revise(|value| *value = !*value)}/>
                    <Show when={move || visible.get()} children={move || branch(branch_theme.clone())}/>
                    <Button class="dsl-reorder" label="Reorder both sources" on_press={move || {
                        values.revise(|rows| rows.swap(0,1)); members.swap(0,1).unwrap();
                    }}/>
                    <div class="dsl-values"><Each each={values} key={|id| *id} children={|row|
                        view!{<span class="dsl-value-row">{row.value}{":"}{row.index}</span>}
                    }/></div>
                    <div class="dsl-members"><Each each={members} children={|row|
                        view!{<span class="dsl-member-row">{*row.key()}</span>}
                    }/></div>
                </div>
            </sapid::Card>
        }.build(scope)
    })
}
