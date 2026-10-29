# P0-10 存活集合、游走焦点与字符搜索

`src/collection.rs` 提供共享 `Collection / RovingFocus / Typeahead`。RadioGroup 的真实
方向键与字符搜索已迁移到该模型，SegmentedControl 和单选 ChipGroup 的游走导航也随之复用。

## 有界登记与 Owner

`Collection::new(scope, CollectionLimits)` 创建显式集合；`register(item_scope, key,
node_ref, enabled, label)` 只能登记当前挂载 Owner 的控件。key 是至多 64 字节的唯一 ASCII
标识，缺省最多 1024 项、单标签 256 字节；可配置上限分别为 1024 项和 4096 字节。
外部虚拟列表须只登记窗口内控件，不能将整个业务数据源交给本模型。

每次登记分配不回绕的序号和独立集合身份。移除后重新使用相同 key，不会恢复旧 token。
item Owner 清理立即删除登记；集合 Owner 退休清空配置。移出的 key、标签和 reactive
配置均在 registry 借用之外析构，读取 reactive label/enabled 也不持有 registry 借用。
模型保留当前登记项、成员变更 pulse 和当前焦点 token；不保留访问历史或扫描业务 source。
悬空 node_ref 不进入 mounted 结果；跨 Runtime 登记和控制器构造明确拒绝。

`RovingFocus::new(scope, collection, looping)`、`repair(preferred_key)` 和
`destination(from, Navigation)` 统一禁用跳过、循环、首尾及移除/禁用后的 Tab entry 修复。
`accept/observe` 更新游走入口，调用者单独发出 `Action::Focus`；它们不把排队意图当作
实际焦点。RadioGroup 仍用真实 `FocusChanged` 更新实际进入项，保留原生单 Tab stop。

## 显式时间与键盘边界

`Typeahead::new(TypeaheadConfig)` 只保存当前 query 和最后一次时间戳。缺省超时 500ms、
query 64 字节，允许配置 1..=60000ms 和 1..=256 字节。`search(collection, from, text,
at_millis)` 接收一个非控制、非空白 Unicode scalar。时钟为 0..=2^53-1 的毫秒整数；
倒退时钟、超长 query、超预算动态 label、非法字符均拒绝并保留原 query。未知/已移除
起点不回退到首项，也不消费 query。搜索仅遍历有界 mounted registry，用 Unicode
lowercase 前缀比较；单字符重复循环，连续不同字符追加前缀，超时清空旧 query。
这不是拼音、模糊搜索、完整 Unicode case-fold 或 IME 输入模型。

共享 Glory `Direction::{Ltr,Rtl}` 只反转横向 Left/Right 的逻辑 Previous/Next；
Home/End 和垂直方向保持原义。`Semantics.navigation_direction/typeahead` 只可用于显式
navigation boundary，HTML `dir`/有限 data 属性的 create、Set/Clear 和 patch oracle 一致。
HTML v3 / classic stream v1 无新增 wire 操作；仍使用既有有限 attribute edits。

`Event::Typeahead { from, text, at_millis }` 及 EventKind 是共享有限事件；render 同时校验
字符、时钟、Runtime 和真实物理起点。Web 只在 opt-in navigation item 上消费合法字符，
保留编辑控件、组合态、Ctrl/Meta/Alt 和 Space。RadioGroup 用 `.typeahead(config)` 显式
启用，`.direction(Direction::Rtl)` 控制方向。箭头/Home/End 会清除当前字符 query。
搜索提案仍只读受控来源；拒绝时重申整组真实 checked，同时保留用户实际移动的焦点。

## 平台与运行证据

- `tests/collection.rs`：4 项实际通过，覆盖 100 次临时 Owner 删除/重登记、key/token
  复用、容量/重复/跨 Runtime 拒绝、禁用/退休修复、有界 Unicode 前缀、repeat、timeout、
  时钟/query/动态 label 拒绝，以及 RadioGroup 实际共享控制与受控拒绝。
- 选择控件五组测试合计 27 项通过；对应 lib/tests 严格 Clippy 通过。
- Glory `crates/render/tests/typeahead.rs`：2 项通过，校验物理起点、挂起、有限字符/时钟、
  metadata 前置条件及 RTL/编辑/组合/修饰键；HTML lib/tests 与 full-shape oracle 通过。
- Glory `native_horizontal_rtl_driver_reverses_arrows_and_preserves_home_end`：真实 native
  key driver 1 项通过。原生当前没有字符搜索输入驱动，因此 typeahead descriptor/patch
  preflight、direct apply 和 listen 均明确拒绝。`crates/apace/tests/check_modes.rs` 最新
  4 项通过，包含该拒绝的成功前缀和 Query applied 水位；不以沉默接受代替功能支持。
- `examples/web/choice-acceptance.js` 最新实际浏览器 10 份报告全部通过、errors=[]，包含
  8 种配色/CSR/hydration 的真实 RTL、prefix、repeat、timeout、拒绝/原生邻居恢复、
  单 Tab stop、禁用、窄视口和 Owner 清理，外加脏 adoption 与 classic stream。
  报告 `D:/BuildCache/sapid/choices-browser-results.json`，截图同目录 `choices-*.png`。

浏览器协议与 AX tree 验收不等于 OS IME 或真实读屏朗读验收。Menu、Listbox 和 Combobox
等后续控件仍须自行接入真实角色、展开层与虚拟窗口生命周期，不能据此宣称已实现。
