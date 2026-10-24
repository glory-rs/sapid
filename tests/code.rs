#![cfg(feature = "code")]
use glory::render::{NodeId, Props, TextKind};
use glory::ui::App;
use sapid::*;
fn ids(app: &App, class: &str) -> Vec<NodeId> {
    let tree = app.tree();
    let mut todo = tree.roots().to_vec();
    let mut result = Vec::new();
    while let Some(id) = todo.pop() {
        let n = tree.node(id).unwrap();
        if n.descriptor.view().classes.iter().any(|c| c == class) {
            result.push(id);
        }
        todo.extend(n.children.iter().rev());
    }
    result
}
fn source(app: &App) -> String {
    let tree = app.tree();
    ids(app, "sapid-code-token")
        .into_iter()
        .map(|id| {
            let Props::Text(text) = tree.node(id).unwrap().descriptor.view().props else {
                panic!()
            };
            assert_eq!(text.kind, TextKind::Code);
            assert!(text.selectable);
            text.content.clone()
        })
        .collect()
}
#[test]
fn rust_lexer_is_lossless_utf8_nested_comments_raw_strings_and_lifetimes() {
    let source = "pub fn café<'a>(x: &'a str) { /* outer /* inner */ 😀 */ let c = '😀'; let raw = br##\"<script>\\\"\"##; println!(\"a\\\"b\"); // tail\n}\n";
    let d = tokenize_code(source, CodeLanguage::Rust).unwrap();
    assert_eq!(d.source(), source);
    assert_eq!(
        d.tokens()
            .iter()
            .map(|t| &source[t.range.clone()])
            .collect::<String>(),
        source
    );
    assert_eq!(d.line_starts().len(), 3);
    assert!(d.tokens().iter().any(|t| t.kind == CodeTokenKind::Comment
        && &source[t.range.clone()] == "/* outer /* inner */ 😀 */"));
    for literal in ["'😀'", "br##\"<script>\\\"\"##", "\"a\\\"b\""] {
        assert!(d.tokens().iter().any(|t| t.kind == CodeTokenKind::String && &source[t.range.clone()] == literal), "{literal}");
    }
    assert!(
        !d.tokens()
            .iter()
            .any(|t| t.kind == CodeTokenKind::String
                && source[t.range.clone()].starts_with("'a"))
    );
    for text in ["/* incomplete", "\"unterminated\\", "r#\"raw 未完成", "'a"] {
        let d = tokenize_code(text, CodeLanguage::Rust).unwrap();
        assert_eq!(
            d.tokens()
                .iter()
                .map(|t| &text[t.range.clone()])
                .collect::<String>(),
            text
        );
    }
}
#[test]
fn json_plain_and_draft_lexing_do_not_parse_evaluate_or_emit_markup() {
    let text = "{\"key\": [true, null, -12.5e+2, \"😀\\u0026\"]}";
    let d = tokenize_code(text, CodeLanguage::Json).unwrap();
    assert_eq!(
        d.tokens()
            .iter()
            .filter(|t| t.kind == CodeTokenKind::Keyword)
            .count(),
        2
    );
    assert!(
        d.tokens()
            .iter()
            .any(|t| t.kind == CodeTokenKind::Number && &text[t.range.clone()] == "-12.5e+2")
    );
    let d = tokenize_code("<script>😀</script>", CodeLanguage::Plain).unwrap();
    assert_eq!(d.tokens().len(), 1);
    assert_eq!(d.tokens()[0].kind, CodeTokenKind::Plain);
    assert_eq!(
        tokenize_code("", CodeLanguage::Plain)
            .unwrap()
            .line_starts(),
        &[0]
    );
}
#[test]
fn finite_lexer_boundaries_reject_before_copy_or_unbounded_output() {
    assert_eq!(
        tokenize_code(&"x".repeat(MAX_CODE_BYTES + 1), CodeLanguage::Plain),
        Err(CodeError::Bytes)
    );
    assert_eq!(
        tokenize_code(&";".repeat(MAX_CODE_TOKENS + 1), CodeLanguage::Rust),
        Err(CodeError::Tokens)
    );
    assert!(tokenize_code(&";".repeat(MAX_CODE_TOKENS), CodeLanguage::Rust).is_ok());
    assert_eq!(
        tokenize_code(&"\n".repeat(MAX_CODE_LINES), CodeLanguage::Plain),
        Err(CodeError::Lines)
    );
    assert!(tokenize_code(&"\n".repeat(MAX_CODE_LINES - 1), CodeLanguage::Plain).is_ok());
    assert_eq!(
        tokenize_code(&format!("{}x", "/*".repeat(33)), CodeLanguage::Rust),
        Err(CodeError::CommentDepth)
    );
    assert_eq!(
        tokenize_code(&format!("r{}\"x", "#".repeat(17)), CodeLanguage::Rust),
        Err(CodeError::RawStringHashes)
    );
}
#[test]
fn reactive_source_and_language_preserve_token_identity_and_reject_invalid_values() {
    let app = App::memory();
    let draft = app.scope().cage("fn main() {}\n".to_string());
    let language = app.scope().cage(CodeLanguage::Rust);
    app.mount(
        code_highlight()
            .source(draft)
            .language(language)
            .line_numbers(true)
            .highlighted_lines([1]),
    )
    .unwrap();
    let first = ids(&app, "sapid-code-token")[0];
    assert_eq!(source(&app), draft.get());
    draft.revise(|v| *v = "fn other() { let x = \"😀\"; }\n".into());
    app.flush().unwrap();
    assert_eq!(ids(&app, "sapid-code-token")[0], first);
    assert_eq!(source(&app), draft.get());
    language.revise(|v| *v = CodeLanguage::Plain);
    app.flush().unwrap();
    assert_eq!(ids(&app, "sapid-code-token").len(), 1);
    assert_eq!(ids(&app, "sapid-code-token")[0], first);
    let previous = source(&app);
    draft.revise(|v| *v = "x".repeat(MAX_CODE_BYTES + 1));
    assert!(app.flush().is_err());
    assert_eq!(source(&app), previous);
    assert_eq!(draft.with(String::len), MAX_CODE_BYTES + 1);
    draft.revise(|v| *v = "recovered".into());
    app.flush().unwrap();
    assert_eq!(source(&app), "recovered");
}
#[test]
fn themes_overrides_and_unstyled_keep_structural_selection_and_whitespace() {
    for theme in [
        Theme::blue_light(),
        Theme::blue_dark(),
        Theme::green_light(),
        Theme::green_dark(),
    ] {
        let app = App::memory();
        app.mount(
            code_highlight()
                .theme(theme.clone())
                .source("fn café() {\n  \"😀\"\n}\n")
                .line_numbers(true),
        )
        .unwrap();
        let root = app
            .tree()
            .node(ids(&app, "sapid-code-block")[0])
            .unwrap()
            .descriptor
            .view()
            .style
            .clone();
        assert_eq!(root.background, Some(theme.input));
        assert_eq!(root.font_family, Some(vec![glory::style::Family::Mono]));
        let tree = app.tree();
        let gutter = tree.node(ids(&app, "sapid-code-gutter")[0]).unwrap();
        assert!(gutter.descriptor.view().semantics.as_ref().unwrap().inert);
    }
    let app = App::memory();
    app.mount(code_highlight().unstyled().source("fn main() {}"))
        .unwrap();
    let tree = app.tree();
    assert_eq!(
        tree.node(ids(&app, "sapid-code-block")[0])
            .unwrap()
            .descriptor
            .view()
            .style,
        &glory::style::Style::new()
    );
    for id in ids(&app, "sapid-code-token") {
        assert_eq!(
            tree.node(id).unwrap().descriptor.view().style,
            &glory::style::Style::new()
        );
    }
    assert_eq!(
        tree.node(ids(&app, "sapid-code-source")[0])
            .unwrap()
            .descriptor
            .view()
            .style
            .white_space,
        Some(glory::style::WhiteSpace::Pre)
    );
}
#[test]
fn ssr_text_is_escaped_once_normalized_and_never_interpreted() {
    let html = glory::ssr::render(
        code_highlight()
            .source("fn x() {\r\n  \"<script>&\\\"\"\0\r}\n")
            .line_numbers(true),
    )
    .unwrap();
    assert!(html.contains("&lt;script&gt;"));
    assert!(!html.contains("<script>"));
    assert!(!html.contains('\r'));
    assert!(html.contains('�'));
    assert!(html.contains("data-gly-inert"));
}
#[test]
fn highlighted_iterator_is_bounded_and_owner_cleanup_releases_spans() {
    use std::{cell::Cell, rc::Rc};
    let count = Rc::new(Cell::new(0));
    let calls = count.clone();
    let builder = code_highlight().highlighted_lines((0..1_000_000).map(move |_| {
        calls.set(calls.get() + 1);
        1
    }));
    assert_eq!(count.get(), MAX_CODE_LINES + 1);
    let app = App::memory();
    assert!(app.mount(builder).is_err());
    let draft = app.scope().cage("fn main() {}".to_string());
    let mounted = app
        .scope()
        .mount_with(|| code_highlight().source(draft))
        .unwrap();
    app.flush().unwrap();
    let peak = app.runtime().stats().nodes;
    for _ in 0..8 {
        draft.revise(|s| *s = ";".repeat(100));
        app.flush().unwrap();
        draft.revise(|s| *s = "fn main() {}".into());
        app.flush().unwrap();
        assert_eq!(app.runtime().stats().nodes, peak);
    }
    drop(mounted);
    app.flush().unwrap();
    assert!(ids(&app, "sapid-code-token").is_empty());
}
