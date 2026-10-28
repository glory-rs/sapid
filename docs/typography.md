# Text / Title / Typography

`text()` 默认构造 Paragraph；`.inline(true)` 使用 Inline，`title(level)` 使用真实 Heading 1..=6。
普通文字以 content 提供名称，不将纯段落改成 ARIA 名称；Title 可用 `.name(...)` 显式命名。
内容全部作为文本转义，不解释 HTML、模板或 UI DSL。

```rust
sapid::typography().theme(theme).content(|context| (
    context.title(1).content("Documentation"),
    context.text().variant(sapid::TextVariant::Lead).content("Readable by design."),
    context.text().content("A paragraph with a stable semantic identity."),
    context.anchor().href("/guide").label("Read the guide"),
));
```

Typography 是富内容的语义组合容器。`.content(|context| ...)` 的 context 显式提供继承 Theme、Size、
unstyled 的 text/title/anchor builders；支持 tuple、Vec 或任意 IntoChildren。它不是隐式全局 Theme
provider。`.fill(children)` 接受应用已经配置的任意语义子组件；这些子组件保持自己的显式主题。
可以组合 Code、Kbd、Mark、列表等已有组件，不接受不受限的 HTML 字符串。

TextVariant 提供 Body、Lead、Muted、Small、Caption。Body 使用当前 size 的正文指标；Lead 放大，
Small/Caption 缩小，Muted/Caption 使用语义 muted 颜色。Title 各层级有独立字号和行高。
`.font_size(px)`、`.line_height(px)`、`.weight(number)`、`.color(Color)` 显式覆盖相应指标；
Style 的数值校验仍由 Glory 统一执行。`.style(...)` 最后逐字段覆盖，不重置未声明的主题属性。

Text/Title 支持 theme/size/unstyled/class/style/node_ref/on、可响应式 content 和 selectable。
unstyled 移除主题外观，保留显式 font_size/line_height/color/weight/truncate/style；不会删除文字语义。
外部源更新不发布变更回调，保留节点 identity。动态布局和文字来源仍使用 Cage/Bond/Vary。

Sapid 与 Glory 均导出 text 构造器；混用两个 prelude 时请显式使用 `sapid::text()` 或导入别名。

## 单行截断

`.truncate(true)` 声明 typed `TextOverflow::Ellipsis`、NoWrap、水平 Hidden 和 Block。
应用应通过 `.style(...)` 提供实际 width/max_width。完整字符串仍保留在语义节点和可访问树，
不截短业务源。false 不下发 Clip 或其他额外限制，避免无必要拒绝宿主能力。

Web/SSR/Record 支持此有限样式。当前固定 Blitz 引擎没有相应 ellipsis 布局/paint 路径，
原生 profile v6 对整个 TextOverflow property 明确 UnsupportedStyle；不会只藏掉文字冒充截断。
不使用 truncate 的 Text/Title/Typography 可在原生正常排版。本批不宣称支持多行 line-clamp 或
任意富编辑器文档协议。

## 证据

- `tests/typography.rs` 9 项验证真实 Paragraph/Inline/Heading1..6、变体、数值拒绝、显式样式、unstyled、
  reactive identity、rich context、SSR escaping 和 typed ellipsis。
- `examples/native/tests/typography.rs` 三项测试包括四主题真实排版/engine identity，及原生
  ellipsis/NewContext 初始拒绝、动态外部上下文拒绝后 accepted request 行为。
- `examples/web/typography-acceptance.js` 在 Chrome 155.0.8059.84 实际完成 9 组报告，错误为空：
  四主题 CSR/hydration、真实 CSS ellipsis/完整文本、原生 H1/H2/P、AX 标题、链接及聚焦接管/退休。
  结果位于 `D:/BuildCache/sapid/typography-browser.json`，截图为 `typography-*.png`。

AX 树证据不等于读屏朗读、OS IME 或所有浏览器平台验收。
