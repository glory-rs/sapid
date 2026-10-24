use glory::prelude::*;

pub fn demo(theme: sapid::Theme) -> impl Widget {
    fixture::json_tree_example(theme)
}

#[allow(dead_code, unused_imports)]
mod fixture {
    use glory::prelude::*;
    pub const SOURCE: &str =
        "{\"😀/branch~\":{\"needle\":\"😀 exact <script>\",\"list\":[1,2]},\"version\":1}";
    pub fn json_tree_example(theme: sapid::Theme) -> impl Widget {
        widget(move |scope| {
            let source = scope.cage(SOURCE.to_string());
            let query = scope.cage(String::new());
            let paths = scope.cage(vec![String::new()]);
            let accept = scope.cage(true);
            let proposals = scope.cage(0usize);
            sapid::stack().gap(10.).fill((
   sapid::text_field().theme(theme.clone()).name("Search JSON tree").value(query).on_value_change(move|value|query.revise(|v|*v=value)),
   sapid::json_viewer("code-tree").theme(theme.clone()).class("json-tree-demo").tree_navigation(true).source(source).search(query).expanded(paths).copy_button(sapid::copy_button().theme(theme.clone()).class("tree-json-copy").label("Copy JSON tree source")).on_expanded_change(move|next|{proposals.revise(|n|*n+=1);if accept.get(){paths.revise(|v|*v=next)}}),
   sapid::button().theme(theme.clone()).label("Toggle JSON tree acceptance").on_press(move||accept.revise(|v|*v = !*v)),
   sapid::button().theme(theme.clone()).label("Reorder JSON tree").on_press(move||source.revise(|v|*v="{\"version\":1,\"😀/branch~\":{\"needle\":\"😀 exact <script>\",\"list\":[1,2]}}".into())),
   glory::ui::text().class("json-tree-proposals").content(proposals.map(|n|format!("Tree proposals: {n}"))),
   glory::ui::text().class("json-tree-paths").content(paths.map(|v|format!("Tree expansion paths: {v:?}"))),
  )).show_in(scope)
        })
    }
}
