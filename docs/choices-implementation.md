# P2 选择控件实现与验收

本批实现 `MixedCheckbox / CheckboxGroup`、`Switch`，补齐 `Radio / RadioGroup`、`SegmentedControl` 与 `Chip / ChipGroup`。

## 原生语义与状态

- `CheckState::{Unchecked, Checked, Mixed}` 完整保留三态。Glory 使用有限
  `CheckMode` 与独立 `checked` 位；Mixed 通过实际 `input.indeterminate` 属性表示，
  并非将中间态压成布尔值。原生操作提出 Checked/Unchecked；程序仍可设置 Mixed。
- Switch 使用 Check 的原生输入驱动和独立 `role=switch`。Web Space/Tab、checked、
  Field 标签/描述/required/readonly/invalid 与普通 Check 共用同一有限协议。
- 受控来源只读；typed callback 先于通用观察者。完整 host tick 结束后协调拒绝的提案。
  Mixed 的 equal mode patch 也会重申 indeterminate，普通 metadata/Keep 不写控制值。
- CheckboxGroup 只持有显式挂载项，稳定 ASCII key，最多 1024 项。全选汇总仅计算可编辑项；
  全选/清空保留禁用项的既有选择。重复/未知/过量项拒绝；非法外部选择保留上一份有效 UI。
  不注册虚拟数据源中的所有项，也不保存访问历史。
- 所有效果、回调与节点引用随 Owner 退休。内部回调只捕获 WeakScope。

## 共享 mapper 与宿主边界

- HTML 全形状与 field-local patch 对 `checked`、`indeterminate`、Switch/Radio type/role
  保持一致，包含 Set/Clear/Keep、相等属性屏障与完整 differential oracle。
- HTML 格式仍为 v3，classic stream 外壳仍为 v1：使用既有有限 boolean property 编辑，
  新增的 `indeterminate` 在 shared JS 白名单中明确允许。SSR 和 CSR 不依 CSS feature 改变形状。
- 纯无 JS 的 HTML 无法编码 `input.indeterminate` 属性。SSR 保留三态元数据；
  Web 完整验证后 adoption 或 classic stream Shell 启动时才设置真实属性。
  不在控件里注入脚本。hydration 不替换控件身份，保留用户已改变的 checked 值。
- classic stream 保留用户脏 checked/indeterminate；Radio 的原生同名邻居隐式取消也进入
  整组保护，避免随后 server patch 擦除用户已作出的单选。
- 新版 Blitz/Glory 支持真实 mixed indeterminate 和互斥 radio。2026-10-29 四主题
  消费端验证 Mixed 转 Checked、Radio/SegmentedControl/单选 Chip 的受控选择、只读
  与物理控件身份；历史拒绝记录已被本次通过的回归取代。OS 输入及读屏另行验收。

## 可复查证据

已运行并通过：

- `tests/choices.rs`：8 项通过，覆盖三态、受控拒绝/接收、Field、全选与禁用保留、
  非法配置、Owner 清理、SSR 实际 Check/fieldset/legend。
- Glory `crates/html/tests/check_modes.rs`：2 项通过；完整 HTML tests 与 full-shape oracle 通过。
- Glory `crates/render/tests/field_patch.rs::check_mode_prepare_swaps_only_mode_and_rolls_back_without_counters`：通过，
  检查大标签缓冲地址、回滚与水位不变。
- Sapid五个聚焦target共27项及严格Clippy通过；Web例子wasm32编译通过。
- Glory `crates/blitz/tests/check_modes.rs`：4 项通过；preflight/direct-prefix、真实 checked 与控制身份。
- Glory `switch_exports_independent_role_actual_engine_checked_and_live_name`：通过，检查 AX Switch、
  原生 checked、动态名称与稳定 AX identity。
- `tests/segmented_control.rs`：2 项通过，覆盖真正radio与无TabPanel、受控拒绝、局部状态、
  自定义类名、横向导航及Owner清理。
- `tests/radio.rs`：7 项通过；Glory `crates/render/tests/radio.rs`：2 项通过，
  覆盖有限组/表单名冲突、单值/Tab、禁用/readonly、拒绝、非法外部值、SSR 与 Owner 退休。

浏览器实际最新执行：10 项报告通过，`errors=[]`，单独结果位于
`D:/BuildCache/sapid/choices-browser-results.json`。覆盖 Radio 单 Tab stop、
ArrowDown 跳过禁用项、Home/End、受控拒绝后恢复旧项及独立 AX radio checked；
SegmentedControl 原生单值、ArrowRight禁用跳过、受控拒绝与无Tab面板语义；
Mixed/Switch/Radio 的脏 hydration 身份保留，classic stream 同名Radio组整组脏值保护。
同时检查 Switch 真实轨道样式、窄视口与明暗主题截图。

浏览器复查入口：`examples/web/choice-acceptance.js`，覆盖 8 种配色/CSR/hydration、
脏 adoption、真实 classic stream Shell/Patch/End。所有生成截图、结果与缓存均位于
`D:/BuildCache/sapid`；实际运行结果由集成 gate 的 `browser-results.json` 保存。
浏览器协议和 AX tree 检查不等于 OS IME 或实际读屏朗读验收。

## Radio 分组与表单名称

RadioGroup 具有真实 `radiogroup` 结构角色，子项为真实 `input[type=radio]`；
不用 Tabs 的面板导航语义。空字符串表示未选；选择值必须是组内唯一且稳定的 key。
单 Tab stop、方向导航、禁用跳过和受控拒绝由单值控制器统一维护。
原生 radio 会无事件地取消旧项，因此拒绝提案后会重申整组存活 checked 属性。

`check_group` 是有限原生关联标识；HTML 本身将 native `name` 同时用于单选互斥和
successful-control name。本批 `form_name` 接受有界 ASCII 标识，并在各项设置真实 key
作为 successful value。未显式提供时，使用稳定命名空间生成的 native name。
若另提供 `FormControl.name`，它必须与 `check_group` 一致，否则明确拒绝。
这不影响 RadioGroup 自己的语义 id 或 Field 标签关系。表单传输仍由应用处理。

## SegmentedControl

`SegmentedControl` 使用同一个 RadioGroup 单值控制器及真实 native radio；
`segment(value)` 只定义值、标签、禁用与外观。默认横向，具名组/成功值、Field、
方向键、单 Tab stop、受控拒绝的整组恢复和 Owner 清理均沿用 RadioGroup。
它不构造 TabList/TabPanel；可选 CSS 仅将仍可聚焦的原生 radio 显示为分段外观。
`unstyled` 保留普通原生 radio 外观与所有语义。

## Chip 与复选组 Field

`chip()` 是真实 Checkbox 的胶囊外观；独立的 bool 值使用
`checked / default_checked / on_checked_change`，受控来源仍只读。
`ChipGroup::mode(ChipMode::Single)` 复用 RadioGroup 的单值、真实 radio 与单 Tab stop；
`Multiple` 复用 CheckboxGroup 的独立多选，成员由 `chip_item(key, label)` 定义。
单选的 `value / default_value / on_value_change` 与多选的
`values / default_values / on_values_change` 不得混用；配置不匹配在建树前拒绝。
不使用 pressed 按钮或 Tabs 冒充选择值。

多选 Field 的 required 表示至少一项可编辑选择。未选时，仅首个 enabled 原生 checkbox
持有 required；任一可编辑项选中后清除它，不能将整个组误变成每项均必选。
readonly/invalid 传到各个真实 Check；Fieldset 保留组的 id、标签、描述、错误关系和
全局 aria-invalid，required/readonly 不伪装成 Fieldset 的宿主属性。
禁用项保留既有选择但不满足“可编辑选择”要求。

独立 Chip 和多选 ChipGroup 在原生继续使用真实 Checkbox；单选 ChipGroup 与
RadioGroup/SegmentedControl 共用当前明确的 native Radio 拒绝边界。
`examples/native/tests/choices.rs` 已实际运行 2 项通过：检查 native checked、Field、
控制身份、readonly，以及四种不支持变体的拒绝后清理。
`tests/chip.rs` 已运行 6 项通过；加上 Collection 本批五个 Sapid 聚焦 target 共 27 项与严格 Clippy 通过。
Chip 浏览器检查已实际通过同一 choices suite 的 10 项报告：包含原生 required-any 的
checkValidity、独立/单选/多选受控拒绝、Space/ArrowRight、readonly、disabled、
暗色/窄视口和脏 hydration。报告与截图位置同上，errors=[]。

RadioGroup 的真实方向导航已迁移到 P0-10 共享存活集合，新增显式 RTL 和 opt-in Typeahead。
真实字符、重复循环、前缀、超时及受控拒绝的浏览器验证通过。实现、预算与平台边界见
[collection-implementation.md](collection-implementation.md)。
