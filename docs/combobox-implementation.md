# Combobox

P4-06 `combobox(key)` 保留真实单行 Input 的编辑、revision、UTF-16 选区和 composition 协议，使用 `ComboboxKind::Editable` 与真正的 Listbox/Option。Collection、RovingFocus、Preserve Layer、Option 所有权和 RevealOption 沿用 Select 的共享基础，不用修改普通 View 的交互角色。

```rust
combobox("country")
    .label("Country")
    .search(search_text)
    .value(selected_key)
    .form_name("country")
    .option(select_option("ca", "Canada").value("CA"))
    .option(select_option("fr", "France").value("FR"))
    .on_search_change(move |text| search_text.revise(|value| *value = text))
    .on_value_change(move |key| selected_key.revise(|value| *value = key));
```

搜索文字、已选 key 和提交业务 value 独立。`.default_search`、`.default_value`、`.default_open` 提供本地状态；`.search`、`.value`、`.open` 提供只读受控输入。选择建议同时提出 key 和显示 label，调用方可分别接受或拒绝。手动编辑不同于当前选项的文字时提出清空 key 的建议，输入恰好等于某个 label 不会自动选中。程序修改 key 不强制覆盖独立搜索文字。未显式配置初始搜索时，默认显示初选项 label。

`.on_search_change` 包括 composition 中的输入；`.on_search_commit` 是原生 Input Commit，和选项选择回调分开。内部更新和 typed 回调先于通用观察者。所有同 revision 的宿主 tick 编辑处理完再协调真实 value；拒绝的搜索恢复当前源，相等回显保持原生选区。Owner 退休移除集合成员、控件、Effects、callbacks 和引用，用户 capture 的析构不在图借用内。

过滤策略为有限枚举 `ComboboxFilter::{Contains,Prefix,None}`，默认 Contains；采用确定性的 Unicode lowercase，不调用业务过滤闭包。只读取最多 1024 个本控件声明，不扫描外部大来源。分组、disabled 和 key/value/label 的限制与 NativeSelect 相同：32 个不嵌套 group、64 字节 ASCII key、256 字节 label、4096 字节业务 value、128KiB 总声明。搜索最多 `MAX_COMBOBOX_SEARCH_BYTES`（4096 UTF-8 字节），必须是无 NUL/CR/LF 的单行文本。iterator 接收端最多消费预算加一；受控 String/Option 先借用验证再克隆，非法程序更新保持最后有效 UI。

过滤通过有限 Option.hidden 和 enabled 实际隐藏选项；普通 Button 仍拒绝 hidden。分组及空结果使用真正 View 可见性。Option 的最近 Listbox 祖先和原有 4096 live-node 边界预算不因隐藏而失效。Memory.active 包括自身/祖先 hidden；resolver、事件和 Reveal 同样排除失活目标。Binding 尚未执行时，选项处理器也重查当前 visible 与 mounted enabled，保证同 tick 的过滤/关闭后晚到指针或 Commit 不选择旧项。

ArrowUp/Down 导航匹配且启用的选项，Enter 提交当前活动项，Escape 关闭，Tab 保留宿主焦点导航。Home/End/Space 保留文本编辑；关闭态 Enter 保留原生表单提交。焦点始终留在 Input，Option pointerdown 抑制抢焦点。点击、触屏和键盘共享有限选择命令与相同 owner 校验。

组合输入期间冻结过滤、方向导航和选项提交，并延迟受控 Input 值回写，避免重置 preedit/选区。compositionend 后通过已有 ValueChanged/Composition 协议协调源和过滤，不增加专用宿主事件。真实 OS IME 与屏幕阅读器公告仍需独立平台验收；浏览器协议驱动不等于这些验收。

Field 关联真实 Input，保留 label/description/error/required/invalid/readonly/disabled。原生 Input required 只能检查文字非空；“必须选择一个有效 key”属于应用 FormState/Field 校验策略。隐藏 NativeSelect 代理仅提交已选项业务 value，不承载 required/readonly/invalid 约束；搜索文字没有额外表单 name，不会冒充成功选择。readonly 保留焦点和程序写入；disabled 同时禁用 editor 与成功值代理。

当前模型证据：`combobox::tests` 11 项，覆盖真实 Input/SSR/Field、过滤活动/选中分离、composition 冻结/恢复、受控三个源拒绝、百万来源入口与恢复、同 revision 整 tick、多事件过滤后旧项拒绝、closed Commit/同值同步不重新开启、readonly 中 composition 清理，以及 Owner capture 析构。共享 render Choice 9 项、HTML Choice 2 项、SSR Choice producer 1 项已过；Blitz Choice 1 项验证 Select/Editable 两分支均在 facet 注册前 preflight/direct 拒绝且保留成功前缀/查询水位。

浏览器入口 `examples/web/combobox.html`，脚本 `combobox-acceptance.js`；真实 producer 沿用 `GLORY_CHOICE_STREAM_FIXTURE` / `choice_stream`，含 editable Input、active-descendant 和 filtered hidden Option，classic 保持 HTML v4/envelope v2。Sapid lib/tests 与 WASM fixture 严格 Clippy 均通过。

隔离 `web/combobox-next` 产物中 Chromium **148.0.7778.96** 实际 **12/12**，`errors=[]`，报告 `D:/BuildCache/sapid/combobox-browser-combobox-next.json`。包含四主题各 CSR/hydration、真实 keyboard/pointer/touch、AX、Field/FormData、受控恢复、readonly/disabled、UTF-16 Home/End/Space、closed Enter 真实 submit、Owner、dirty adoption 身份/光标、classic filtered Option 和 dirty text/caret；CDP `Input.imeSetComposition` / `Input.insertText` 驱动实际浏览器 preedit，验证冻结过滤/组合 Enter 拒绝/结束后过滤，不等价 OS IME 验收。green-dark-hydrate 截图已目视检查。

最终统一 **Web12** 在相同 Chromium **148.0.7778.96** 重跑 **12/12**，`errors=[]`，报告 `D:/BuildCache/sapid/combobox-browser-web12.json`；验证使用根 8794 统一 WASM/SSR/stream 产物，未以隔离目录替代最终验收。
