# NativeSelect

P4-04 NativeSelect 使用 Glory 的真实 `Props::Select`；HTML/SSR/Web 映射到原生 `select`、`option`、`optgroup`，不通过 View 修改角色。

```rust
native_select("plan")
    .label("Subscription plan")
    .form_name("plan")
    .value(selected_key) // Lotus<Option<String>>，键不是业务 value
    .placeholder("Choose a plan")
    .option(select_option("starter", "Starter").value("start-value"))
    .group(select_group("Business")
        .option(select_option("pro", "Professional").value("pro-value")))
    .on_value_change(move |proposal| selected_key.revise(|value| *value = proposal));
```

`.values(Lotus<Vec<String>>)` / `.default_values(iter)` 选择多选模式；`.multiple(true)` 可显式配置空的本地多选。`.default_value` 提供非受控单选初值。`.rows(1..=64)` 使用原生 size。`.field(...)` 保留 label/description/error/required/invalid/disabled 与业务表单名；`.form_name` 可直接提供提交名称。`.name`/`.label` 是可访问名称，和表单名分开。

选项 key 是唯一的 1–64 字节 ASCII token。label 最多 256 UTF-8 字节、业务 value 最多 4096 字节，所有声明合计最多 128KiB；最多 1024 个 option、32 个不嵌套 optgroup。`.options(iter)`、group 的 `.options(iter)` 和逐项接收都只保留预算加一，不先完整收集未知长度来源。配置超限在创建语义节点前拒绝。选项静态配置由 Rc 共享，选择补丁不复制标签/values。动态替换 options、异步数据源、虚拟化、搜索、可编辑输入不属于 NativeSelect 此批范围。

业务 value 默认等于 key，可以重复，也可以显式为空；控制值始终是 key。多选外部 Vec 可以任意顺序，UI 按文档顺序规范化，外部源不会因此被改写。源值先借用验证，超大 Vec/String 不在校验前复制。非法后续程序更新保留最后有效 UI，恢复后可继续。受控事件只提出建议；整个 drained host tick 后统一读取源，显式相等写入修复拒绝的宿主选择。typed callback 在通用观察者前运行，Owner 退休清理控件/订阅/回调。

单选 None 使用内部禁用隐藏占位 option，保证 SSR 解析时不会自动选首项；该项不进入 FormData。真正空业务 value 的正常 option 仍可提交。禁用 option/group 可以由程序选择；用户不能新选禁用项，但可以取消之前的程序选择。HTML select 没有 readonly，`.readonly(true)`（包括 Field）返回明确 Unsupported，未用 disabled 冒充；FormControl.value 对 Select 同样拒绝，因为成功值来自 options。原生键盘、焦点、表单成功值和约束验证遵循 [HTML Select 规范](https://html.spec.whatwg.org/multipage/form-elements.html#the-select-element)。

共享协议为 `SelectSelection { keys, revision }`、`Patch::SelectSelection`、`Event::SelectChanged`。keys 唯一、已知、文档顺序且符合单/多选数量。Keep 忽略 revision，Set/Clear 单调；同版本只能相同 keys。`Scope::set_select_selection` 分配 live node 下一版本，旧事件丢弃，未来事件拒绝，不回绕；元数据更改不替换控件或重写选择。

本批有意升级 HTML/Snapshot **v4**、classic stream envelope **v2**（资源 envelope 仍 v1）。有限 Shape 增加 options，Property 增加 selectedKeys。SSR 输出完整原生结构；Web 通过 typed 子节点创建/真实属性读写，采用先验证整个子树再修复 disabled/inactive，活跃 dirty 选择以正常事件回放。classic stream 同步 selected defaults，同时保留用户实际 dirty 选择。旧 Snapshot v3/stream v1 在采用/启动前明确拒绝，测试确认宿主根未突变。SVG Graphic 的有限 tag/属性同时属于当前 v4 定义。

Blitz 当前缺少原生 Select 编辑/事件实现，App preflight 和 direct apply 均在引擎 facet 注册前明确拒绝；保留已接受操作前缀及 query watermark。本批没有宣称 native Select、OS 原生弹出选择器或屏幕阅读器公告已验收。

证据：

- `tests/native_select.rs` 7 项：受控拒绝/回显/观察顺序、单多选/文档顺序、Field/SSR/FormControl、组禁用与 Owner、百万项接收/百万 Vec/String 入口、非法源恢复，以及同一宿主 tick 两个相同 revision 事件完整处理。
- Glory render `tests/select.rs` 4 项，UI `tests/select.rs` 3 项，HTML full-shape differential oracle 4 项全部通过；keep/reset/默认/相等写入与 live guard 回滚、声明 Rc 共享均覆盖。
- SSR `select_stream_uses_v4_shape_v2_envelope_and_preserves_native_selection_through_end` 通过，`GLORY_SELECT_STREAM_FIXTURE` 生成真实帧。Blitz `tests/select.rs` 1 项验证提前拒绝、无 facet 与 accepted-prefix/query 水位。
- 统一 Web snapshot7 下 Chromium **148.0.7778.96** 实际 **12/12**，`errors=[]`：四主题各 CSR/hydration、键盘/Tab/实际选中值、受控拒绝、原生 FormData/required/disabled、真实 AX combobox/listbox 名称、390px、Owner、dirty adoption/identity/focus、旧 HTML 格式拒绝、真实 Rust classic v2/v4、dirty stream 与旧 envelope 拒绝。报告 `D:/BuildCache/sapid/native-select-browser-web7.json`；八张 `native-select-*.png`，green-dark-hydrate 已目视检查。
- dirty stream 场景在测试服务器响应的原 classic asset 前插入用户 change 的测试触发，然后执行完整原控制器和真实 Rust 帧；不修改生产控制器。多个 parser-blocking script 共用一次网络请求，不能按第三次请求假定暂停点。
- 最终 Web7 已包含非空 `.selected` 初值版本1回归，以及 stream setter keys/order/cardinality 检查；实际浏览器同时验证 unknown/cardinality 非法补丁在选择字段突变前拒绝。Choices10/Menu11/Hints10 在相同 Web7/v4/v2 产物重跑全部通过，报告分别为 `choice-browser-web7.json`、`menu-browser-web7.json`、`hints-browser-web7.json`。

复现：Sapid `cargo test --test native_select`；Glory `cargo test -p glory-render --test select`、`cargo test -p glory-ui --test select`、`cargo test -p glory-html --lib`、`cargo test -p glory-ssr --features stream --lib select_stream`、`cargo test -p glory-blitz --test select`。所有 targets、Cargo home、临时文件均在 D:，浏览器脚本 `examples/web/native-select-acceptance.js`。
