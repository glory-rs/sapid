# Select

P4-05 的 `select(key)` 使用真实的 Button combobox、Listbox 和 Button Option 内在语义细化。它复用 Collection/RovingFocus/Typeahead、宿主 Layer 定位以及 NativeSelect 的有限选项声明；不会把 Tabs、普通 View 或 Popover 改名充当选择控件。

```rust
select("region")
    .label("Delivery region")
    .form_name("region")
    .value(selected_key)
    .placeholder("Choose a region")
    .clearable(true)
    .option(select_option("ca", "Canada").value("CA"))
    .group(select_group("Europe")
        .option(select_option("fr", "France").value("FR")))
    .on_value_change(move |proposal| selected_key.revise(|value| *value = proposal));
```

`.default_value(Option<String>)` 和 `.default_open(bool)` 提供本地状态；`.value(Lotus<Option<String>>)`、`.open(Lotus<bool>)` 只提出 typed 回调建议，不写业务源。选中 key、显示 label 和提交 value 独立。活动项由共享 mounted collection 管理，不会在方向键移动时自动提交。开启或关闭被控制器拒绝时，实际 expanded、active-descendant、选中样式和成功值遵从当前业务源。非法程序源先借用验证，再读取有限值；后续非法更新保留最后有效 UI。

单次配置最多 1024 个 option、32 个不嵌套分组、128KiB 声明数据；key 为 1–64 字节 ASCII token，label 最多 256 字节，业务 value 最多 4096 字节。接收 iterator 只消费预算加一，不完整收集未知长度来源。选项配置通过 Rc 共享，移动和 patch 不复制完整配置。此组件挂载全部有限选项，不声称虚拟化或无限数据支持。

触发器保持唯一的选项导航 Tab 站点。ArrowUp/Down 打开并跳过 disabled 项/组，Home/End 到首尾，方向导航循环，字符键使用有显式时钟与查询预算的 Typeahead；Enter/Space 提交，Escape 关闭，Tab 保留浏览器焦点导航并在失焦时关闭。触发器焦点保留，`aria-activedescendant` 表示活动项，`aria-selected` 表示选中值。Option 的真实 pointerdown 抑制默认抢焦点，移动和激活仍通过有限宿主事件。

Listbox 使用非模态 `LayerFocus::Preserve`，不自动 focus、return 或 trap Tab。`Action::RevealOption` 只滚动最近拥有该 Option 的 Listbox，使活动项可见；不移动焦点、不滚动页面、不查询业务数据。排队后的关闭、停用和退休再次检查。共享 Action 的 target 与 scroll capability 在排队前验证，直接向 Option 发 Focus 明确拒绝。

Field 的 label/description/error/required/invalid/disabled 保留在可聚焦触发器。隐藏的真实 NativeSelect 代理仅提供成功表单值；它没有 required、readonly 或 invalid 原生约束，不制造浏览器无法聚焦的 required 错误。业务表单提交前的必填校验由 FormState/Field 策略负责；ARIA required 不等同原生 submit 阻止。None 不提交，clearable 通过正常值建议产生 None；显式空业务值仍可提交。readonly 保留可聚焦触发器并拒绝值交互，disabled 同时禁用触发器和表单代理。

共享有限协议为 `ComboboxKind::{Select,Editable}`、`OptionSemantics { owner }`、`PopupKind::Listbox`、`Role::Listbox`、`Event::Choice` 和 `Action::RevealOption`。本组件使用 Select 分支，Editable 分支留给真实 Input 编辑组件。Listbox 的最近祖先决定 Option 所属；跨边界移动才检查最多 4096 个活节点的受限子树，同一 owner 的普通 Group 重排只走祖先。不保留访问历史、来源副本或额外 Node 计数。未放置声明可先配置，但实际 Place/Resume/Plan attach 必须解析正确 owner。整个嵌套 Listbox 移动保留其独立所有权边界。

active-descendant 与命名关联一样允许晚解析：初建、关闭、Structure 优先卸载以及 Owner 退休不要求 Binding 先清理引用。resolver 只返回当前活跃、可激活、正确 Listbox owner 与祖先关系的 Option；已知错误目标和 owner 明确拒绝。Metadata guard 验证失败或被丢弃不改变模型统计，保持 prepared field rollback 和原有成功前缀约定。

HTML/SSR/Web 共用有限映射和 full-shape differential oracle。该批验收使用 Snapshot HTML v4 / classic envelope v2；当前 producer 随共享协议使用 HTML v13 / envelope v2。该批新增的角色/属性复用既有有限 edits。SSR 输出静态角色、selected、expanded、Layer 和关联，真实 Rust classic producer 验证异步更新及 End。classic 控制器仅应用有限 DOM 编辑，不拥有 WASM Layer 的定位/输入服务，因此 pre-WASM 的手动 popover 不因这些关联属性自动获得交互承诺。Blitz 尚无本族 Choice 输入/Layer driver（独立 Dialog/Popover Layer 已另有原生支持），构造 preflight 和 direct apply 在 facet 注册前明确拒绝 combobox/option，保留成功前缀和 query watermark；未宣称原生或屏幕阅读器公告验收。

已有自动证据：Sapid `select::tests` 9 项（含打开后 readonly 的 Escape、unstyled、非法 live 百万字节源恢复、重复字符循环、同一 drained host tick 连续按键）；Glory render `tests/choice.rs` 8 项（含精确事件来源与退休身份）；HTML lib 6 项（含 Choice 2 项、无 navigation 的 combobox typeahead 宿主属性及差分、既有 full-shape oracle）；UI `option_reveal_uses_scroll_capability_and_drops_closed_or_retired_targets` 1 项；SSR `choice_stream_retains_focus_preserving_intrinsics_and_live_active_descendant` 1 项；Blitz `tests/choice.rs` 1 项。Sapid lib/tests 与 Web example WASM 严格 Clippy 通过。浏览器脚本为 `examples/web/select-acceptance.js`。

统一 Web10 中 Chromium **148.0.7778.96** 实际 **11/11**，`errors=[]`，报告 `D:/BuildCache/sapid/select-browser-web10.json`。八组主题/CSR/采用验证真实 keyboard、pointer、AX combobox/listbox/option、Field、FormData 业务值与 None、disabled/readonly、受控值与开启拒绝、字符循环、同一 JS task 的宿主按键队列、Escape/Tab、390px 和 Owner 退休；另有真实触屏、保留焦点/身份的采用及真实 Rust classic v2/v4 三组。RevealOption 验证仅 listbox scrollTop 改变、活动末项可见、焦点和页面 scrollY 保持。八张 `select-*.png` 已生成，green-dark-hydrate 已目视检查。Web9 首轮暴露的 typeahead 宿主属性缺失和连续按键按旧 DOM 分类均已在 Web10 验收修复；仅作诊断的 Web9 人工属性修补不计入证据。

最终统一 **Web11** 重跑仍 **11/11**，`errors=[]`，报告 `D:/BuildCache/sapid/select-browser-web11.json`。该产物包含 inactive Option 的 typed 透明背景，新增 computed-style 回归确认不再露出浏览器默认按钮灰底；green-dark-hydrate 截图已再次目视检查，活动项与深色面板清晰可辨。
