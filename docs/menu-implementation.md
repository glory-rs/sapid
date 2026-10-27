# Menu / DropdownMenu

P4-01 的有限基线提供 `menu`、`dropdown_menu`、`menu_item`、
`menu_checkbox`、`menu_radio_group`、`menu_radio_item`、`menu_submenu`。
独立 `Menu` 使用显式语义 anchor；`DropdownMenu` 提供自己的实际 Button 触发器。
菜单面板为真实 `Role::Menu`，动作和子菜单触发器为 Button，勾选项和单选项为
实际 Check/Radio 输入控件。Glory 的 `MenuItemSemantics` 只细化对应内在控件，
并不允许用交互 View 冒充 menuitem。

```rust
dropdown_menu("document-actions")
    .label("Document actions")
    .item(menu_item("new").label("New document").on_activate(|| {}))
    .item(menu_checkbox("grid").label("Show grid").default_checked(true))
    .radio_group(menu_radio_group("density").default_value("comfortable").items([
        menu_radio_item("comfortable", "Comfortable"),
        menu_radio_item("compact", "Compact"),
    ]))
    .submenu(menu_submenu("share").label("Share")
        .item(menu_item("email").label("Email link")))
```

`open`、`checked`、radio `value` 都是只读受控输入；对应回调提出变更。
不提供受控输入时使用本地 default。动作默认关闭整棵菜单，checkbox/radio 默认保持打开；
每项可设置 `close_on_select`。勾选拒绝恢复真实输入 checked；radio 拒绝恢复整组原生
checked，避免浏览器先取消 peer 后仅恢复当前项。菜单控件以 owner menu ID 作为显式
HTML form owner，使其不进入外围业务表单的成功值；radio 的原生 name 保留真实排他组。

每层直接复用 live Collection、RovingFocus、Typeahead。Up/Down、Home/End、跳过禁用项、
循环和标签搜索只访问已挂载集合。方向键只移动焦点，Enter/Space 才提交勾选值。
触发器 Down/Up 打开首/尾项；LTR Right 打开、Left 返回父层，RTL 反转。
Escape 关闭最深层并返回对应触发器。Tab/ShiftTab 请求关闭根菜单，并通过共享
`Action::ExitMenu` 让宿主在真实已挂载文档里寻找触发器之后/之前的 tab stop。
宿主遍历上限 65,536 个物理节点，排除所有 Menu 子树，保留外围 modal 范围，不遍历业务数据。
该动作在同一提交隐藏面板后仍能执行；已销毁节点不能执行。

Menu 使用共享非模态 Layer，包含实际浏览器 top layer、侧向锚定、翻转和视口碰撞处理。
指针边界事件携带明确 origin/related identity；共享 Memory 验证 runtime、活性、物理 origin
及 ancestry，内部子节点切换不产生错误离开。指针离开已打开子菜单触发器时保持子菜单，
因此可跨越定位间隙；进入 sibling、外部点击和键盘命令负责关闭或切换。
菜单只在真实 mouse/pen `PointerPhase::Move` 时改变悬停焦点；布局/top-layer 出现到
静止鼠标下产生的 Enter 不得抢走键盘设置的首/尾项焦点。Chromium 148 实际触发过该路径，
现有单独回归测试及 148/153 双浏览器证据。

`hover_delay_with` 接受应用/宿主提供的本地 Future 工厂，组件不依赖浏览器计时器。
每层最多保留一个 Owner task；leave、sibling、关闭及 Owner 销毁立即取消，单调 ticket
防止晚到完成重新打开菜单。默认无延迟。取消和配置/捕获析构均在注册表借用之外执行。
每层 raw controlled child 状态只接受至多一个打开项；非法外部多开保留最近有效可见项，
不短暂展示两个 sibling。

配置在接收阶段有界：全树 1,024 项、单层 256 项、深度 8 层。批量迭代最多取剩余额度加
一个哨兵，超限后逐项添加也不再累积配置；键最长 64 字节、生成语义 ID 最长 256 字节。
只保留当前挂载配置和注册项，没有访问历史或底层来源副本。相同批次另修正了
CheckboxGroup/Chip 的外部多选输入：借用验证长度和键预算后才 clone；非法百万项来源
不会先复制整份 Vec，也不会从事件提出无效选择。

`Semantics.popup: Option<PopupKind>` 是唯一 popup 真值，目前为 Dialog/Menu。
旧布尔字段已删除。HTML full-shape differential oracle 覆盖 MenuItem/Radio/Layer/Role
和默认 reset，SSR/CSR 属性形状相同。经典 stream 仍使用 HTML v3 的有限属性/属性值编辑，
验证 PopupKind Set/Clear、checked property、hidden reset、侧锚点和 End rekey。
Hydration 全子树验证通过后才修复 inactive/disabled Check 的脏 checked；活动可交互输入
仍由正常语义事件接受用户编辑。隐藏菜单不会因被过滤的脏事件残留 DOM/ARIA 不一致。

## 验收证据

- `tests/menu.rs`：8 项，覆盖真实控件、共享导航/搜索、层级/关闭、受控拒绝、非法 sibling
  外部状态、Owner 捕获释放、单 task 取消/晚到唤醒、接收预算及稳定 SSR shell。
- `tests/choices_budget.rs`：2 项，百万迭代器/重复单项添加，以及百万外部多选来源的构建和
  事件拒绝路径。
- Glory `render/tests/menu.rs`：3 项；`html/tests/menu.rs`：1 项；HTML mapper lib oracle：4 项。
  UI `menu_exit_survives_the_same_commit_hiding_its_retained_panel`：1 项。
- Glory SSR `menu_stream_retains_intrinsic_roles_and_lowers_popup_layer_and_controlled_properties`：
  1 项，真实 Shell/Patch/End producer；设置 `GLORY_MENU_STREAM_FIXTURE` 可输出实际浏览器页面。
- `examples/web/menu-acceptance.js`：11/11 报告通过、errors=[]。八组 blue/green × light/dark ×
  CSR/hydrate，另含真实 RTL、脏 hydration 和经典 stream。覆盖实际焦点、三层键盘、原生
  Enter/Space、FormData、AX role/checked、侧向 bounds、延迟取消、指针间隙、受控拒绝、
  Tab/ShiftTab、外部点击和待完成计时器上的 Owner 销毁。2026-10-24 隔离产物位于
  `D:/BuildCache/sapid/web/menu-next`，报告 `D:/BuildCache/sapid/menu-browser-results.json`，
  八份截图 `D:/BuildCache/sapid/menu-*.png`；green/dark/hydrate 已人工查看。
  静止鼠标回归修复后，隔离 `web/menu-race` 在 Chromium 148.0.7778.96 和
  153.0.8010.12 各 11/11 报告通过、errors=[]，证据分别为
  `D:/BuildCache/sapid/menu-browser-148.json` 与 `menu-browser-153.json`。
- Native `blitz/tests/menu.rs` 验证当前 Menu Layer 准入、后续拒绝的成功前缀及 query 水位；
  原生输入/CPU/AX 回归验证实际 Menu/Item/Check/Radio、菜单栏关系、模态边界和关闭后动作退休。
- Sapid lib/Menu/budget strict Clippy、Glory SSR stream strict Clippy 通过。

## 当前边界

Glory 原生后端已连接 Menu/Menubar、RTL/Typeahead、勾选 Enter、最近的 ContextMenu region、
Palette 快捷键、ContextPoint 定位和有界 Tab 退出。当前键盘焦点请求只保留一个 NodeId，
在可见性绑定提交后请求 Focus 与显式 ScrollIntoView；不改变 Focus 的 prevent-scroll 契约。
关闭/Tab 会取消旧焦点意图，包括受控关闭拒绝，避免稍后的显露动作夺回焦点。
选项显露只经过当前 Layer 内的物理滚动容器，不扫描业务数据。

`/Volumes/Data/Caches/glory/native-menu-consumer` 的实际消费端覆盖四主题菜单、菜单栏、
ContextMenu、Command/Spotlight、初始子菜单、受控拒绝、长列表显露与滚动后点击；
另验证同一宿主批次中菜单导航与新 Palette modal 的焦点隔离。实现和宿主证据见
[Glory native menus](../../glory/docs/native-menus.md)。历史浏览器记录继续保留，
协议/CPU 和 AX 模型检查不等于真实 OS 菜单键、屏幕阅读器或系统菜单栏验收。

菜单仍是有限显式配置，不提供动态列表来源/虚拟菜单、触屏长按策略、任意宿主定时器或
任意 DOM 插槽。单选/勾选菜单偏好不会作为 Field 表单值提交。Cocoa/Win32 等系统
菜单栏、托盘服务与这套自绘语义菜单分开记录。
