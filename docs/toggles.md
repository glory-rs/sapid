# Toggle / ToggleGroup

Toggle 是具有 `pressed: Option<bool>` 状态的原生 Button。它保持 Button 的 Enter/Space 激活、
焦点和禁用规则；Web/SSR 映射 `aria-pressed`，Apace 的 AccessKit 节点提供 Toggled 状态。
它不使用 Tabs、Switch 或 Radio 的角色冒充按钮选择。

```rust
use sapid::{toggle, toggle_group, toggle_item, ToggleMode};

toggle().label("Bold").pressed(bold).on_pressed_change(move |next| {
    bold.revise(|value| *value = next);
});

toggle_group("format")
    .name("Text styles")
    .mode(ToggleMode::Multiple)
    .value(selected)
    .items([
        toggle_item("bold").label("Bold"),
        toggle_item("italic").label("Italic"),
    ])
    .on_value_change(move |next| selected.revise(|value| *value = next));
```

Standalone Toggle 提供 `.pressed(...)`、`.default_pressed(bool)`、`.on_pressed_change(...)`，
以及统一 theme/size/unstyled/class/style/name/enabled/node_ref/on API。受控来源只读；回调提出
新值，应用决定是否接受。默认值仅在没有受控来源时生效，不受 setter 调用顺序影响。
外部源更新保持静默；拒绝提议时旧 pressed 与样式保持不变。Button 没有浏览器原生勾选副作用，
因此无需伪造乐观状态或写回受控源。

## 组值与焦点

ToggleGroup 的值统一为 `Vec<String>`。`ToggleMode::Single`（默认）允许零或一个 key；点击
已选项清空，点击另一项替换。Multiple 独立切换成员，移除时保留其余顺序，新选项追加末尾。
程序可以明确选中 disabled 项；用户操作不能切换它，多选操作保留它的值。

组提供 `.value(...)`、`.default_value(iter)`、`.on_value_change(...)`、`.items(...)`/`.item(...)`、
`.orientation(...)`、`.loop_focus(bool)` 和统一外观/事件 API。每项支持 label/name/enabled/
class/style/unstyled/node_ref/on，主题及尺寸继承组配置。`.name(...)` 是组的可访问名称。

只有一个 enabled 成员拥有 Tab stop。实际焦点在组中时以实际 FocusChanged 为准；离组后重新
进入时优先当前 enabled 选项，否则使用首个 enabled 成员。禁用项跳过；全部禁用时无 Tab stop。
水平左右、垂直上下、Home/End 与可关闭循环只移动焦点，**不自动改变选值**；Enter/Space 或
鼠标才切换。排队 Focus action 不会被当成已经发生的实际焦点。

内部状态和 typed on_value_change 先执行，随后是 item observers 和 group Activate observers。
Owner 退休后停止后续 observers；任务及闭包不持有自己的拥有 Scope。

成员是显式、有限、静态配置的已挂载按钮：每组 1..=1024 项，key 唯一、最多 64 字节且遵循稳定
ASCII SemanticId。items/default_value 接收阶段最多消费 1025 项；单项追加达到该哨兵后立即释放，
后续批量 iterator 不再被访问。超过预算会拒绝整个配置（包括被受控 source 覆盖的超预算 default），
不会静默截断为成功组件。缺失 key、重复值、多值 Single 或超预算配置明确失败。外部不合法值在正常
flush 返回类型化错误，UI 保留最近有效状态。仅保留当前成员、当前选择和身份索引，不保留访问历史。
动态集合、RTL 方向和更大虚拟数据源并未在该有限 API 中冒充已支持。

## 外观与表单

默认 pressed 使用主题 accent/on_accent/border，未选项使用 control token；显式 style 逐字段覆盖，
保留未覆盖的主题声明。unstyled 删除主题外观，保留按钮、pressed 和导航语义。

Toggle 没有隐式提交成功值，也不会产生隐藏输入。需要表单值时显式用 FormState 注册所选 Cage 并
提供 ControlValue 编码器。它不提供仅属于 Input/Check 的 readonly/required 语义。

## 验证与边界

- `tests/toggles.rs` 15 项通过：百万项 lazy 接收预算、单项哨兵、未使用 captures 释放/析构写入重入，以及本地/受控/拒绝/外部静默、禁用、单多选/清空、未知/重复值、实际焦点与
  roving、导航不选值、typed/item/group 顺序、Owner 释放和 SSR 原生 pressed Button/escaping。
- `examples/native/tests/toggles.rs` 实际运行四套主题，通过真实 Apace box layout、engine identity、
  pressed patch、受控拒绝、actual Focus action 和组成员退休。此测试不等于 OS 键盘/读屏验收。
- `examples/web/src/toggles_form.rs`、`toggles.html`、`toggle-acceptance.js` 提供四主题 CSR/SSR hydration、
  pointer/Enter/Space、水平/垂直箭头与Home/End、禁用跳过、单/多选、受控拒绝、外部静默及 pressed AX 树。
  Chrome 155.0.8059.84 实际完成 9 组报告、错误为空，结果在 `D:/BuildCache/sapid/toggles-browser.json`。
- Glory 的 `native_toggle_button_space_enter_queue_activation_and_keep_pressed_engine_identity` 与
  pressed AccessKit identity/Toggled 回归实际通过，原始日志 `D:/BuildCache/sapid/links-toggle-native.log`。

共享 Glory pressed 验证拒绝非 Button、tab、popup 和 form_action 冲突。真实 OS 输入法、读屏朗读、
跨设备触摸和动态大集合仍须对应平台验收；不从 headless 或引擎布局测试推断这些能力。
