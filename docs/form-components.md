# Form / SubmitButton / ResetButton

`form(key)` 是 Glory 有限表单语义上的手写 Rust builder。Web/SSR 输出真实 `<form novalidate>`；
提交和重置始终由宿主先捕获并排队，应用回调在正常 UI turn 执行，不由原生 listener 同步重入业务。
`Form` 不自动联网、不保存值副本，也不把异步校验完成当作即时原生 submit 事件。

```rust
use sapid::{form, reset_button, submit_button};

form("profile")
    .name("Profile settings")
    .on_submit(move |submitter| {
        // Optional NodeId identifies an actual activated submit button.
        // Connect FormState::submit here; application code owns transport.
    })
    .on_reset(move || {
        // Connect FormState::reset or another explicit reset policy here.
    })
    .fill((
        submit_button().label("Save profile").entry("intent", "save"),
        reset_button().label("Restore defaults"),
    ));
```

`on_submit(FnMut(Option<NodeId>))` 接收请求；普通单行 Enter 的 submitter 是 None，实际按钮激活
提供其 live NodeId。typed `on_submit`/`on_reset` 先于同 EventKind 的通用 `.on(...)` observers。
回调期间退休 Owner 时，后续 observers 不再执行。FormState 的成功/失败应通过它自己的
`SubmitToken` 和状态观察，不能把 Form 的 `.on(EventKind::Submit, ...)` 当异步成功通知。

Form/按钮保留统一 `theme`、`size`、`unstyled`、`class`、`style`、`name`、`enabled`、`on` 和
`node_ref` API。按钮另有 `label` 和 `.entry(name, value)`。entry 是成功 submitter 元数据，
与可访问 name/label 分开；应用从 submitter 身份选择是否加入自己的提交快照。
`FormState::serialize` 不会隐式遍历 DOM 或自动加入按钮。

Form 内部始终有稳定的无边框 Fieldset，将 `enabled(false)` 传为真实原生分组禁用，
保留全部子节点身份并正确影响焦点和 Tab。Form 自身同时拦截请求。单独按钮 disabled 只影响
该按钮。Form 是结构部件，显式非 Auto focus 会由 Glory 拒绝；它不冒充可聚焦交互控件。

## Field 的成功控件元数据

`.form_name(name)` 声明 Input 或 Check 的成功控件名字；不改变 label、AX 名称、控件值和输入版本。
`.form_value(value)` 提供 Check 的显式值，需同时提供 form_name；普通 Input 的值始终来自
InputProps，额外 form_value 会被 Glory 明确拒绝。

```rust
field("account-email")
    .label("Account email")
    .form_name("email")
    .control(|context| text_field().field(context));

field("updates")
    .label("Product updates")
    .form_name("consent")
    .form_value("yes")
    .control(|context| checkbox().field(context));
```

动态 form_name/form_value 通过既有语义 Patch 更新，Input 的 identity/revision/selection 保持不变。
受控接受/拒绝、readonly、required、invalid 及描述/错误关联继续由现有 Field/控件实现。
名字最多 256 字节且禁 NUL/CR/LF；显式值禁 NUL/CR，未额外添加长度上限。两者先借用校验，再共同发布到 accepted metadata；无效的动态 pair 保留原有 UI。
不开放任意 HTML 属性。

## 宿主与限制

- Web：鼠标、按钮 Enter/Space、单行 Enter、原生 requestSubmit/reset 请求均排队并阻止默认导航/原生值重置。
  Textarea Enter 插入换行，不隐式提交。阻止默认发生在监听边界，应用 reset 在后续 host turn 运行。
- SSR：共享映射输出真实表单、按钮类型和 name/value，并转义用户字符串；不会执行提交。
- hydration：复用既有 DOM/脏输入接管协议；示例验证输入 identity、draft、caret、focus 和提交模型一致。
- 原生：Glory shell 提供语义提交/重置排队；本批模型测试不声称已完成 OS IME 或读屏验收。

嵌套 Form、失效 submitter 和跨表单 origin 由共享 semantic target/placement 规则拒绝。
文件句柄、multipart/urlencoded、实际 HTTP 和完整原生 FormData 等价性仍不属于该组件接口；
文件快照通过 `ControlValue::UnsupportedFile` 明确拒绝，详见 `forms-state.md`。

## 证据

`tests/forms.rs` 的 11 项测试覆盖 typed/observer 顺序、按钮与单行 Enter 默认动作、Textarea 不提交、
稳定禁用 Fieldset、名称/值分离、metadata 更新保持输入版本、显式 reset、Owner 退休和 SSR escaping，另验证百万字节无效名字在控件 factory 前拒绝、
5000 字节合法值可接受及无效 pair 原子保旧/恢复。
FormState 的 22 项独立测试见 `docs/forms-state.md`。

`examples/web/src/forms_form.rs` 显式连接 FormState；`forms.html` 与
`form-acceptance.js` 覆盖四套主题 CSR/SSR hydration、pointer/Enter/Space、错误/caret、受控拒绝、
readonly/disabled 成功控件、原生 reset 不提前改值、外部变化静默、AX form landmark、异步取消/完成及
脏接管/Owner 退休。Chrome 155.0.8059.84 实际运行 10 组报告通过、无页面错误；结果在
`D:/BuildCache/sapid/forms-browser.json`，截图为该目录的 `forms-*.png`。
