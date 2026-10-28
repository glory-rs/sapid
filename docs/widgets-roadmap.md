# Sapid widgets 对照与逐项实现计划

核对日期：2026-10-24（目录抓取于 10 月 23–24 日）。Mantine 官方文档显示版本 9.7.3；shadcn 使用本次官方组件目录快照。
这是一个冻结的对照快照，后续上游新增项需要显式核对并更新清单，不自动改变目标范围。
本页由 `docs/widget-inventory.json` 生成；修改清单后运行 `python scripts/widget-plan.py`，
运行 `python scripts/widget-plan.py --check` 检查覆盖完整性和文档同步。
最终全量验收、复现命令与宿主边界见 [验证记录](widgets-verification.md)。
Glory + Sapid 构建的首页、组件文档和实际示例见 [展示站](showcase-site.md)，
运行 `./scripts/check-showcase.ps1 -Serve -Browser` 后打开 `http://127.0.0.1:8795/`。

## 当前判断

原有九个主要组件为 Button、TextField、Checkbox、Collapsible、Accordion、Tabs、
Dialog、Drawer、Popover；第一批已增加 Field、Label/Description/Error、Fieldset/Legend，
并接入 TextField/Checkbox 的标签及 required/readonly/invalid。已有测试证明相应功能，
不表示与某个上游组件全部功能等价。蓝/绿各有明暗主题；有限主题 token/provider、方向、locale、
响应式和运动契约已有独立验收，支持范围与明确拒绝见 P0 和逐组件能力记录。
第二批已增加 Textarea、PasswordInput、InputGroup/Addon/Section，完成 Field、受控状态、
Web/SSR 与原生引擎验收。Textarea 自动高度依赖浏览器 field-sizing:content，Blitz 明确拒绝；
固定行数可用于原生。后续批次已增加 Form/FormState、成功控件快照、MixedCheckbox/组、
Switch、RadioGroup/SegmentedControl、动作按钮、展示表面、NumberFormatter 和文档文本/列表组件，
对应证据逐项记录在下表。Toggle、Chip、布局、Typography、Grid/比例、Timeline、Spoiler 与导航
已增加实际宿主证据；Menu、Stepper、辅助文本、图像、提示层、原生选择和应用框架亦已验收，
新批次的完成状态及宿主边界以清单和逐组件证据为准。
DataTable、Tree/FileTree、TreeSelect 和 Cascader 已按有限契约完成实现和宿主验收；
图表、文件、编辑器、日程和聊天模块的当前完成状态以清单及对应证据为准。

名称需要按语义对照：当前 Dialog 对应 Mantine Modal / shadcn Dialog；Mantine Dialog
是另外的非模态提示需求，单列 FloatingDialog。Sheet 已在 Drawer 上增加有限手势与 snap
points；验收和宿主边界见对应证据。Collapse 与 Collapsible 复用展开模型并接入有限运动协议。Mantine 布局 Group
不是 Glory 的 boxless Group。菜单、Tooltip、Combobox 也不能仅改 Popover 的外观就算完成。

Dialog/Popover 已验收 Web/SSR/hydration、Document 跨 App 栈、完整有限锚点定位与滚动锁，
并接通原生 WindowApp 的真实 top layer、焦点/AX、布局和像素。Drawer/Sheet、Menu/Tooltip/Listbox
等其余原生 Layer facet 仍按证据明确拒绝。独立 Portal 与 FocusScope 已完成真实宿主、跨 App 和生命周期验收；
完整主题和其他组件的当前状态均以逐项证据为准。

## 目标边界

1. 覆盖两套官方目录中可见的核心 widgets，并纳入 Mantine 日期/图表/日程/编辑器等扩展。
   shadcn 新增的消息、问卷、Trace 和 AI Elements 也保留条目，作为可选展示模块。
2. 同义名称和紧密变体共享实现，但清单保留每个来源 URL。149 项工作不是 149 个独立 widgets；
   其中包含已有组件补强、公共基础设施和多个变体的共同实现。每个变体都要有自己的验收证据。
3. 不照搬 React/JSX API、hooks、JS 库依赖和代码分发 CLI。继续使用手写 Rust builder、
   Cage/Bond/Vary、Glory Widget 与现有所有权/宿主边界；第三方实现不复制进仓库。
4. 社区 Registry、页面 Blocks、完整业务应用和任意第三方插件不是有限可穷尽的 widgets 清单。
   React 表单库绑定不移植，但对应表单能力列入 P1；主题/provider/方向/locale 是基础设施。
5. 默认 Sapid 保持轻依赖。日期、图表、文件、富文本、日程和聊天组件使用可选模块或独立 crate。
   当前是 Sapid 的 `dates/charts/files/editor/chat/code/json` 可选 features，日程随 `dates`；
   尚未拆为同名独立 crate，也不把可选宿主依赖带入默认组件图。
   图表需要可访问数据表示和跨宿主绘图能力；富文本需要独立文档/编辑协议，不能空壳交付。
6. 实现默认样式、主题 token、尺寸/状态和 unstyled 组合能力；不把黑白原生控件包装当完整设计系统。
   组合部件可表达 Root/Trigger/Content/Label/Description 等角色，但不引入 UI DSL 或任意 HTML API。

## 执行方式与依赖

按下表 P1 → P13 推进，阶段内按任务编号逐个交付。P0 是贯穿所有阶段的补强队列；
依赖 P0 表示先完成当前组件必需的基础，不要求先完成全部原生浮层/动画等工程才开始 Field。
每批实现 1–3 个有共同依赖的组件；复杂项目先把该项拆为可验收子任务，再编码。
未完成当前批的状态/键盘/宿主测试前，不用新增空 builder 的数量代表进展。

首批明确顺序：

1. 已完成 P1-01 Field/InputWrapper，P1-02 Label/Description/Error，P1-03 Fieldset/Legend；
   Glory 的 required/readonly/invalid 和真实标签/分组关联已接通 TextField/Checkbox，证据见对应任务。
2. 已完成 P1-04 Textarea、P1-05 PasswordInput、P1-06 InputGroup；保留组合输入、选区和受控恢复。
3. 已完成 P1-07 FormState/Validation、P1-08 有限提交/重置/序列化契约；不默认接入网络。
4. 已完成 MixedCheckbox/组、Switch、RadioGroup、SegmentedControl、Toggle/Chip；后续批次按有限契约逐项验收。

每完成一项：更新 JSON 状态和实现/测试证据，再重新生成本页。`baseline` 只表示已有本轮之前的
功能基线；`partial` 表示只有部分部件或 Glory 基础；`planned` 表示没有实现。改为 `done` 必须在
该项 `evidence.implementation` 和 `evidence.tests` 中记录实现及验证证据。本轮继续按组件记录实际宿主证据，未完成项保持 partial/planned，不用新增类型数量替代验收。

## Glory 需要随组件补齐的能力

| 使用方 | 应补充的有限语义/宿主能力 |
|---|---|
| Field / Form | 标签激活关联、description/error、required/readonly/invalid、分组、表单值和 submit/reset |
| Switch / Radio / Toggle | 独立内在交互角色、mixed/pressed/checked、组值和实际焦点行为 |
| Select / Menu / Combobox / Tree | menu/listbox/option/treeitem 等有限类型、活动后代、选择/展开/位置、方向导航和 typeahead |
| Tooltip / AlertDialog / Toast | tooltip/alertdialog、live/busy/progress、可撤销宿主计时；延迟回调不得越过退休 Owner |
| Slider / SpinButton / Splitter | 数值范围、步进、值文本、方向、指针捕获、键盘等价操作和拒绝恢复 |
| Layout / Motion / Icon | typed grid/ratio/换行/响应式布局、方向、有限运动/图标/绘图；逐属性更新支持 profile |
| Table / Grid / Schedule | 表头与单元格关联、网格游标、选择与编辑；只遍历挂载窗口，不能扫描大数据源 |
| File / Clipboard / Editor | 受限文件句柄和预算、剪贴板动作及错误、富文档/选择/IME/undo 协议，宿主负责系统访问 |

这些行描述组件所需的能力类别，实际 enum/API 及宿主边界见实现与协议文档。优先扩展共享有限契约、验证、mapper 和实际
适配器；不在 Sapid 内注入 DOM/ARIA/脚本，也不把交互 View 随意改 role 冒充原生控件。
新增语义必须明确 Web、SSR/stream、Blitz、AccessKit 的支持或拒绝；不能默默降级成错误语义。

## 统一完成标准

- Rust API 与文档：受控/非受控状态、外部更新静默、稳定 key/关联、组合插槽、主题/unstyled。
- 交互：相关键盘/指针/触摸行为，disabled/readonly、焦点、IME、受控拒绝、空态与边界值。
- 生命周期：Owner 退休、任务/计时器取消、异步晚到结果、有限队列；不保留访问历史或整份数据副本。
- 呈现：SSR escaping、稳定关联、hydration 身份/草稿/焦点保持及失败无副作用；stream 适用项验证。
- Glory 回归：Memory 事务回滚、target/event 校验、preflight/direct apply、成功前缀和查询水位；
  HTML full-shape/字段 Patch 差分 oracle，元数据变更不重写输入值或选择。
- 实际宿主：浏览器键盘/指针和 AX 树、截图；原生支持项运行真实引擎/AccessKit 测试。
  不能以 cargo check 或 headless 协议证明 OS IME、读屏语音或未运行的设备能力。
- 门禁：`scripts/check.ps1`、相关 Web/Native 检查、fmt/Clippy，以及本次真正需要的领域模型测试。
  所有构建、工具缓存与测试产物位于 D 盘，并核对 cargo metadata 的 target_directory。

## 逐项队列

同一行列出的近义组件或变体共享一个工作项，仍逐个验证。来源缩写：M= Mantine core；
S= shadcn 主目录；D=日期；C=图表；T=日程；H=代码；X=其他 Mantine 扩展；A=AI Elements。
完整原始条目、来源 URL、状态和关联任务在 `widget-inventory.json`，没有遗漏或未归属的条目。

### P0 · 现有组件与共享基础

保留九个组件的已验证基线；按后续组件需求逐步增加公共能力，避免先做一次庞大重写。

依赖：现有 Glory/Sapid 基线。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P0-01 | Button | 已完成并记录证据 | 五档尺寸、左右插槽与独立动态标签、typed disabled颜色和busy/loading、受控静默与Owner存活回调顺序已补齐；四主题Web/SSR接管与原生实际布局/身份和AX busy已验收。 | [M:Button](https://mantine.dev/core/button/)、[M:UnstyledButton](https://mantine.dev/core/unstyled-button/)、[S:Button](https://ui.shadcn.com/docs/components/base/button) |
| P0-02 | TextField | 已完成并记录证据 | 受控/本地状态、Field/Fieldset元数据、表单成功值、版本/组合输入及typed disabled主题已补齐；真实Web/SSR/dirty hydration与native身份、输入事件已验收。 | [M:TextInput](https://mantine.dev/core/text-input/)、[S:Input](https://ui.shadcn.com/docs/components/base/input) |
| P0-03 | Checkbox | 已完成并记录证据 | 布尔受控/本地状态、Field/Fieldset与表单值、独立readonly/invalid及typed disabled主题已补齐；mixed与分组通过独立组件提供，Web/native边界已有证据。 | [M:Checkbox](https://mantine.dev/core/checkbox/)、[S:Checkbox](https://ui.shadcn.com/docs/components/base/checkbox) |
| P0-04 | Collapsible | 已完成并记录证据 | Collapsible 有限 intrinsic height 动画、反向/内容增长、匹配 revision 后的 Unmount、KeepMounted、Clear/0/Reduce 已补齐。Web 四主题 CSR/采用及 Chromium 12 场景原证据保留；Blitz 现接通真实原生几何/CPU 插值、2x 文字重排、嵌套、隐藏/恢复/清理和单次完成回执，实际四主题双内容策略消费端验证编辑器身份、焦点返回、旧回执拒绝与 Owner 清理。OS 输入、读屏和真机呈现仍单独验收。 | [M:Collapse](https://mantine.dev/core/collapse/)、[S:Collapsible](https://ui.shadcn.com/docs/components/base/collapsible) |
| P0-05 | Accordion | 已完成并记录证据 | 有界动态 items(key source, factory) 与独立 rich trigger_content 插槽已实现。每个当前 key 短 Owner 工厂一次、重排保留编辑/触发器/panel 身份、移除重插新身份、受控 absent 意图与本地选择修复、当前顺序导航和实际聚焦成员移除交接已验收。此项不包含另行跟踪的 Collapsible 高度 motion。 | [M:Accordion](https://mantine.dev/core/accordion/)、[S:Accordion](https://ui.shadcn.com/docs/components/base/accordion) |
| P0-06 | Tabs | 已完成并记录证据 | 动态稳定key成员与短Owner、复用trigger/panel工厂、受控缺失选择保留及本地修复、LTR/RTL roving和自动/手动激活已补齐。10模型及旧30回归、四主题原生真实几何/身份/焦点与Chromium148实际9项通过；受拒绝Focus不会吞掉后续用户激活。 | [M:Tabs](https://mantine.dev/core/tabs/)、[S:Tabs](https://ui.shadcn.com/docs/components/base/tabs) |
| P0-07 | Dialog | 已完成并记录证据 | Dialog原生独立top layer布局/命中/paint、Manage焦点/AX、嵌套和非模态关联、滚动隔离、真正Button slots与Document跨App栈均已补齐；Web26矩阵71/675、独立layer12/placement13、原生Layer8/AX1/Sapid消费者7与真fork全验收通过。 | [M:Modal](https://mantine.dev/core/modal/)、[S:Dialog](https://ui.shadcn.com/docs/components/base/dialog) |
| P0-08 | Drawer | 已完成并记录证据 | 四侧 Drawer、默认 Unmount/KeepMounted、稳定 panel、Modal Layer 焦点循环/返回、Escape/outside 和 Owner 清理已验收；P5-02 补齐有限四侧握柄手势、snap、移动 viewport、滚动隔离和动态 reduced motion。原生Drawer Edge/Sheet facet仍明确拒绝；Dialog/Popover原生与跨App能力已在P0-07/P0-09单独验收。 | [M:Drawer](https://mantine.dev/core/drawer/)、[S:Drawer](https://ui.shadcn.com/docs/components/base/drawer) |
| P0-09 | Popover | 已完成并记录证据 | Popover完整12 placement/flip/shift/gap/padding、锚点失效与恢复、真Button trigger/close slots已完成，Web实际13及跨App12、原生12位置/焦点/paint/AX与ColorInput消费者均通过；默认非模态Manage，Preserve明确不抢焦点。 | [M:Popover](https://mantine.dev/core/popover/)、[M:ComboboxPopover](https://mantine.dev/core/combobox-popover/)、[S:Popover](https://ui.shadcn.com/docs/components/base/popover) |
| P0-10 | Collection / RovingFocus / Typeahead | 已完成并记录证据 | 有界存活Owner/key注册、roving禁用/循环/首尾/RTL及有限字符搜索已实现；Radio接入并完成Web/SSR验收，原生RTL已验证、字符搜索驱动明确拒绝，不保留访问历史。 | Sapid/Glory 共享能力 |
| P0-11 | FocusScope / Layer / Portal | 已完成并记录证据 | Layer、Portal 与独立 FocusScope 已闭环：真实 Web/SSR/原生焦点、跨 App/嵌套返回、有限声明与 placement 预算、Owner 生命周期和经典流协议。保持原生其余 Layer facet 的明确能力边界。 | [M:FocusTrap](https://mantine.dev/core/focus-trap/)、[M:Portal](https://mantine.dev/core/portal/) |
| P0-12 | Theme / Direction / Locale / Motion | 已完成并记录证据 | 显式 Theme/Direction/Locale factory context，有限不可变 spacing/control/typography/radius tokens 被真实 Button/Input/Check/Text/Title 消费；typed inherited dir/lang 与 Native AX language，Locale 12 条显式消息，Reduce 被真实 Collapsible 过渡消费。宿主时钟和 Owner 取消复用既有能力，不宣称隐式全控件翻译。 | [S:Direction](https://ui.shadcn.com/docs/components/base/direction)、[M:Transition](https://mantine.dev/core/transition/) |
| P0-13 | VisuallyHidden | 已完成并记录证据 | 纯文本绝对定位零尺寸与双轴裁剪，不使用hidden/display:none；动态文字及稳定描述关系、四主题真实Web/native布局和AX名称/描述已验收。 | [M:VisuallyHidden](https://mantine.dev/core/visually-hidden/) |
| P0-14 | Gallery / Adapter capability matrix | 已完成并记录证据 | Glory + Sapid Rust 展示站：首页、149 工作项文档路由、搜索、明暗主题、实际示例、源码和 Button 参数面板；另保留显式149任务×5能力轴账本及静态审计 Gallery。独立 CSR 站点与20个代表 SSR/adoption 页面分别验收。 | Sapid/Glory 共享能力 |

### P1 · 字段与表单基础

先完成 Field 关联，再处理 required/readonly/invalid、分组、校验、提交与重置。

依赖：P0。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P1-01 | Field / InputWrapper | 已完成并记录证据 | 关联 Label/Description/Error；统一 required、readonly、invalid、disabled、命名优先级和稳定 ID；输入控件本身仍保持内在角色。 | [M:Input](https://mantine.dev/core/input/)、[S:Field](https://ui.shadcn.com/docs/components/base/field) |
| P1-02 | Label / FieldDescription / FieldError | 已完成并记录证据 | 真实标签激活、控件聚焦、错误描述与多控件分组，区分名称和内容。 | [S:Label](https://ui.shadcn.com/docs/components/base/label) |
| P1-03 | Fieldset / Legend | 已完成并记录证据 | 组级禁用和可访问命名，补齐 Glory 的分组控件契约。 | [M:Fieldset](https://mantine.dev/core/fieldset/) |
| P1-04 | Textarea | 已完成并记录证据 | 独立 Textarea、Field、正整数 rows/min_rows/max_rows 与 typed FieldSizing 自动高度；支持软换行/缩回/上限滚动，保留 caret/IME。Web/SSR 已验收；Blitz 支持固定行，明确拒绝自动高度；其他浏览器需支持 field-sizing:content。 | [M:Textarea](https://mantine.dev/core/textarea/)、[S:Textarea](https://ui.shadcn.com/docs/components/base/textarea) |
| P1-05 | PasswordInput | 已完成并记录证据 | 独立 PasswordInput、具名显隐动作、受控/本地可见性和 Field 禁用；Purpose 更新保持节点、值版本、UTF-16 选区及组合协议。原生引擎身份/原始值已验收，实际 OS IME/读屏仍待平台验收。 | [M:PasswordInput](https://mantine.dev/core/password-input/) |
| P1-06 | InputGroup / Addon / InputSection | 已完成并记录证据 | InputGroup 前/后插槽、文本 Addon 和可含原生按钮的 InputSection；Field 只关联主输入，保留名称与自然 Tab 顺序；分组禁用复用 Fieldset。 | [S:Input Group](https://ui.shadcn.com/docs/components/base/input-group) |
| P1-07 | Form / FormState / Validation | 已完成并记录证据 | Owner范围FormState与Form，值留在Cage；同步/异步校验、touched/dirty/errors、取消、ABA版本、reset原子观察与submit token均有回归；Web/原生请求不触发默认网络。 | Sapid/Glory 共享能力 |
| P1-08 | Form submission / serialization | 已完成并记录证据 | 有限FormControl name/value与真实submit/reset按钮；注册key顺序的有界显式成功控件快照，缺失/disabled/重复名/多值/错误/取消明确；文件值明确拒绝，宿主/应用负责传输，不声明DOM FormData全等。 | Sapid/Glory 共享能力 |

### P2 · 常用选择与按钮

补齐最常用的布尔、单选、多按钮选择交互；状态与实际焦点分离。

依赖：P1。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P2-01 | MixedCheckbox / CheckboxGroup | 已完成并记录证据 | MixedCheckbox/CheckboxGroup 保留真实三态与有限全选/禁用策略；新版 Glory 原生已支持 indeterminate。四主题消费端验证 Mixed 转 Checked、真实引擎属性与 Owner 清理。 | Sapid/Glory 共享能力 |
| P2-02 | Switch | 已完成并记录证据 | 真实Switch角色/checked、Field、受控拒绝/外部静默、键盘与主题轨道；Web四主题和原生checked/AX Switch身份已验证。 | [M:Switch](https://mantine.dev/core/switch/)、[S:Switch](https://ui.shadcn.com/docs/components/base/switch) |
| P2-03 | Radio / RadioGroup | 已完成并记录证据 | 真实 RadioGroup、单 Tab、方向/Home/End、整组受控恢复与 Web/adoption/stream 已实现；新版 Glory 原生 Radio 已连接，四主题消费端验证实际 checked、选择、只读和物理身份。 | [M:Radio](https://mantine.dev/core/radio/)、[S:Radio Group](https://ui.shadcn.com/docs/components/base/radio-group) |
| P2-04 | Toggle / ToggleGroup | 已完成并记录证据 | 真实Button pressed、单/多选清空、roving焦点与箭头不改值，12项组件测试、四主题原生及9组浏览器验收通过。 | [M:Toggle](https://mantine.dev/core/toggle/)、[S:Toggle](https://ui.shadcn.com/docs/components/base/toggle)、[S:Toggle Group](https://ui.shadcn.com/docs/components/base/toggle-group) |
| P2-05 | SegmentedControl | 已完成并记录证据 | SegmentedControl 复用真实 RadioGroup 与单值控制器；四主题原生受控选择、只读及物理身份已验证，不引入 Tabs 面板语义。 | [M:SegmentedControl](https://mantine.dev/core/segmented-control/) |
| P2-06 | Chip / ChipGroup | 已完成并记录证据 | Chip 使用真实 Checkbox，ChipGroup 按单选 Radio/多选 Checkbox 分别实现；Web/SSR 与原生均有消费端证据，新增四主题单选受控选择、只读与物理身份。 | [M:Chip](https://mantine.dev/core/chip/) |
| P2-07 | ButtonGroup | 已完成并记录证据 | 自然Tab的ButtonGroup与可选真实Toolbar roving均已实现；禁用/loading跳过、首尾/循环/RTL/垂直焦点导航不激活，Web与原生实际焦点/身份已验收。 | [S:Button Group](https://ui.shadcn.com/docs/components/base/button-group) |
| P2-08 | ActionIcon / CloseButton | 已完成并记录证据 | ActionIcon与CloseButton复用真实Button激活，强制非空名称、主题尺寸/unstyled与disabled；浏览器AX名称和原生布局已验证。新增五种 ActionIconVariant、CloseButton 自定义 icon、居中零 padding 和动态 disabled 配色；视觉复核见 showcase-style-review。 | [M:ActionIcon](https://mantine.dev/core/action-icon/)、[M:CloseButton](https://mantine.dev/core/close-button/) |
| P2-09 | Burger | 已完成并记录证据 | Burger真实六命令三bar矢量路径，expanded/pressed与受控或本地布尔同源；有限GraphicTransition支持打断、动态Reduce/Clear、隐藏/退休取消，不写每帧Cage。四主题native像素/受控拒绝及Web15九项真实键鼠/AX/动画/classic通过，不宣称OS偏好或屏幕阅读器公告。 | [M:Burger](https://mantine.dev/core/burger/) |

### P3 · 展示、排版与基础布局

统一主题和组合 API，为后续菜单、导航、数据展示提供稳定外观。

依赖：P0。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P3-01 | Box / Flex / Stack / Group | 已完成并记录证据 | Box/Flex/Stack/布局Group使用真实View，支持gap/对齐/换行与结构unstyled；Rust/SSR、四主题原生geometry和8组布局浏览器验收通过。 | [M:Box](https://mantine.dev/core/box/)、[M:Flex](https://mantine.dev/core/flex/)、[M:Stack](https://mantine.dev/core/stack/)、[M:Group](https://mantine.dev/core/group/) |
| P3-02 | Center / Container / Space | 已完成并记录证据 | 已完成原有真实布局及 ResponsiveRoot/Context：显式 SSR 初宽、五档有序断点、一个初始 Bounds + coalesced Resize、Owner 取消；断点 Style 被 Center/Container/Space/Grid/SimpleGrid 真正消费，更新几何保持节点身份。列数仍由调用方显式配置，不暗中扫描内容。 | [M:Center](https://mantine.dev/core/center/)、[M:Container](https://mantine.dev/core/container/)、[M:Space](https://mantine.dev/core/space/) |
| P3-03 | Grid / SimpleGrid | 已完成并记录证据 | 已完成原有真实布局及 ResponsiveRoot/Context：显式 SSR 初宽、五档有序断点、一个初始 Bounds + coalesced Resize、Owner 取消；断点 Style 被 Center/Container/Space/Grid/SimpleGrid 真正消费，更新几何保持节点身份。列数仍由调用方显式配置，不暗中扫描内容。 | [M:Grid](https://mantine.dev/core/grid/)、[M:SimpleGrid](https://mantine.dev/core/simple-grid/) |
| P3-04 | AspectRatio | 已完成并记录证据 | 有限正比例/Auto与尺寸约束已接入共享typed Style；非法值先拒绝，源恢复和节点identity由Rust、四主题Web/native真实布局验证。 | [M:AspectRatio](https://mantine.dev/core/aspect-ratio/)、[S:Aspect Ratio](https://ui.shadcn.com/docs/components/base/aspect-ratio) |
| P3-05 | Text / Title / Typography | 已完成并记录证据 | Text/Title/Typography统一变体、主题字号/行高/颜色、语义段落/标题与富内容组合，typed ellipsis在Web真实验收、native显式拒绝；17项联合测试及四主题宿主证据已齐。 | [M:Text](https://mantine.dev/core/text/)、[M:Title](https://mantine.dev/core/title/)、[M:Typography](https://mantine.dev/core/typography/)、[S:Typography](https://ui.shadcn.com/docs/components/base/typography) |
| P3-06 | Anchor | 已完成并记录证据 | Anchor使用真正Link.target、名称和受控导航请求，读取已接受语义；NewContext输出_blank/noopener/noreferrer，native创建与patch明确拒绝，Current导航已验收。 | [M:Anchor](https://mantine.dev/core/anchor/) |
| P3-07 | List | 已完成并记录证据 | 有序/无序/嵌套TextList与Item真实ol/ul/li结构，区别于虚拟List；透明Group放置内容规则、原生角色与四主题Web/native已验证。 | [M:List](https://mantine.dev/core/list/) |
| P3-08 | Blockquote | 已完成并记录证据 | Blockquote含引文/出处插槽，真实blockquote标签与AccessKit角色、主题/unstyled，SSR转义和Web/native布局已验证。 | [M:Blockquote](https://mantine.dev/core/blockquote/) |
| P3-09 | Code / Mark / Highlight | 已完成并记录证据 | Code/Mark真实行内语义；Highlight为有预算的挂载时精确子串切分，保留原文本与跨Mark选择。Code反应更新保持元素/文本身份；不声明动态富文档range重建。 | [M:Code](https://mantine.dev/core/code/)、[M:Mark](https://mantine.dev/core/mark/)、[M:Highlight](https://mantine.dev/core/highlight/) |
| P3-10 | Badge / Pill | 已完成并记录证据 | Badge被动展示与可移除Pill分别建模；关闭动作独立名称，受控拒绝/本地接受、禁用、稳定shell与Owner清理均已验证。 | [M:Badge](https://mantine.dev/core/badge/)、[M:Pill](https://mantine.dev/core/pill/)、[S:Badge](https://ui.shadcn.com/docs/components/base/badge) |
| P3-11 | Card / Paper | 已完成并记录证据 | Card/Paper插槽/边框/主题、typed单层outset shadow/None及三级主题token已实现；四主题浏览器实际阴影与原生CPU像素证据通过。 | [M:Card](https://mantine.dev/core/card/)、[M:Paper](https://mantine.dev/core/paper/)、[S:Card](https://ui.shadcn.com/docs/components/base/card) |
| P3-12 | Separator | 已完成并记录证据 | 横/纵装饰分隔与静态可访问Separator，禁用无效焦点/action/children，HTML aria-orientation与native AX Splitter一致；四主题真实几何通过。 | [M:Divider](https://mantine.dev/core/divider/)、[S:Separator](https://ui.shadcn.com/docs/components/base/separator) |
| P3-13 | Avatar / AvatarGroup | 已完成并记录证据 | 真实Image source与有界AvatarGroup、圆/方五档尺寸、叠放/更多人数及失败文本回退已实现；版本化加载与旧结果/Owner退休控制、四主题Web/SSR/native实际几何及身份已验收。 | [M:Avatar](https://mantine.dev/core/avatar/)、[S:Avatar](https://ui.shadcn.com/docs/components/base/avatar) |
| P3-14 | Image / BackgroundImage | 已完成并记录证据 | 具名/装饰Image、五值ObjectFit、占位尺寸、单次fallback及BackgroundImage内容叠放已实现；Glory版本化Source/ImageStatus与Web decode/native请求代数、旧A/B/A结果/卸载、实际资源和布局已验收。宿主拥有资源加载。 | [M:Image](https://mantine.dev/core/image/)、[M:BackgroundImage](https://mantine.dev/core/background-image/) |
| P3-15 | Indicator / ThemeIcon | 已完成并记录证据 | 有界Indicator九种实际锚定与四种finite ThemeIcon已实现；Graphic纯typed路径、HTML真实SVG/namespace、native CPU矢量与clip/opacity/patch已验收，具名/装饰语义和四主题Web/SSR/stream通过。 | [M:Indicator](https://mantine.dev/core/indicator/)、[M:ThemeIcon](https://mantine.dev/core/theme-icon/) |
| P3-16 | Kbd | 已完成并记录证据 | Kbd真实kbd与Windows/Mac/Linux键名；显示提示不注册快捷键，Web与原生布局/标签已验证。 | [M:Kbd](https://mantine.dev/core/kbd/)、[S:Kbd](https://ui.shadcn.com/docs/components/base/kbd) |
| P3-17 | EmptyState | 已完成并记录证据 | EmptyState提供标题/描述/图示/body/actions插槽，无自动alert或loading推断，真实heading/button、主题与Web/native已验证。 | [M:EmptyState](https://mantine.dev/core/empty-state/)、[S:Empty](https://ui.shadcn.com/docs/components/base/empty) |
| P3-18 | Item / DataList | 已完成并记录证据 | Item展示title/description/media/body/actions无隐式选择；DataList为真实dl/dt/dd直系结构，分组/主题/动作与选择语义分离。 | [M:DataList](https://mantine.dev/core/data-list/)、[S:Item](https://ui.shadcn.com/docs/components/base/item) |
| P3-19 | Timeline | 已完成并记录证据 | 有界稳定key的真实有序列表、节点标记、进度/当前文本替代与children；四主题SSR/Web/native验证布局及子输入identity。 | [M:Timeline](https://mantine.dev/core/timeline/) |
| P3-20 | NumberFormatter | 已完成并记录证据 | NumberFormatter明确有限EnUs/DeDe/ZhCn、Usd/Eur/Cny与decimal/currency/percent，18位小数预算；非有限/溢出拒绝后可恢复，不写数值来源；SSR/四主题Web/native已验证。 | [M:NumberFormatter](https://mantine.dev/core/number-formatter/) |
| P3-21 | Spoiler | 已完成并记录证据 | 有界纯文本与真实高度测量后截断，稳定展开/收起按钮及controls关系；内容缩短时焦点交接、受控拒绝/恢复/退休与四主题Web/native几何验证通过。 | [M:Spoiler](https://mantine.dev/core/spoiler/) |

### P4 · 菜单与选择器

先共享 Collection/Listbox，再实现菜单与 Combobox 家族，复用定位与键盘模型。

依赖：P1、P2、P3。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P4-01 | Menu / DropdownMenu | 已完成并记录证据 | 真实Menu/MenuItem/勾选单选、Collection/RTL/Typeahead、层级/指针延迟/Tab退出与受控拒绝已有实现；原生Menu Layer/输入/AX现已连接。焦点与显式显露在可见绑定后执行，关闭取消旧意图；四主题实际消费者含长菜单滚动点击通过，OS输入/读屏另验收。 | [M:Menu](https://mantine.dev/core/menu/)、[S:Dropdown Menu](https://ui.shadcn.com/docs/components/base/dropdown-menu) |
| P4-02 | ContextMenu | 已完成并记录证据 | ContextMenu region、有限viewport point、最近边界、Native/Override编辑器策略及Menu模型/子菜单/受控关闭已实现；Glory原生ContextPoint Layer/输入及上下文区域Tab边界已连接，四主题实际消费端通过；触屏长按保持宿主策略，真实OS输入/读屏另验收。 | [S:Context Menu](https://ui.shadcn.com/docs/components/base/context-menu) |
| P4-03 | Menubar | 已完成并记录证据 | Menubar/MenuItemAction根语义、互斥受控根菜单、水平/RTL/typeahead和嵌套菜单/Tab边界已实现；Glory原生协议和AX关系现已连接，四主题实际消费端及原有浏览器证据保留；系统菜单栏/OS读屏分开验收。 | [M:Menubar](https://mantine.dev/core/menubar/)、[S:Menubar](https://ui.shadcn.com/docs/components/base/menubar) |
| P4-04 | NativeSelect | 已完成并记录证据 | 真实select/option/optgroup、单/多选独立key/value、Field/Form/required和受控整tick恢复已实现；HTMLv4/classicv2版本升级及旧版本无突变拒绝、dirty adoption/stream已验收。readonly与native Select明确拒绝，不伪装Input。 | [M:NativeSelect](https://mantine.dev/core/native-select/)、[S:Native Select](https://ui.shadcn.com/docs/components/base/native-select) |
| P4-05 | Select | 已完成并记录证据 | 有限Listbox/Option、独立活动项与受控值、清空/分组/disabled/readonly与Field成功值代理已完成；真实Web11字符循环、同tick按键、触摸/dirty接管/AX和typed透明主题通过；native显式拒绝，classic仅有限DOM传递、不伪造pre-WASM Layer驱动。 | [M:Select](https://mantine.dev/core/select/)、[S:Select](https://ui.shadcn.com/docs/components/base/select) |
| P4-06 | Combobox | 已完成并记录证据 | 真实Input editable Combobox，active-descendant、有限过滤、composition冻结/结束协调、独立搜索/选中/open受控状态与Field/FormData已实现。四主题CSR/hydration、dirty adoption、classic stream、真实键鼠触摸/AX及CDP组合输入通过；不宣称OS IME/屏幕阅读器验收，native Choice facet明确拒绝。 | [M:Combobox](https://mantine.dev/core/combobox/)、[S:Combobox](https://ui.shadcn.com/docs/components/base/combobox) |
| P4-07 | Autocomplete | 已完成并记录证据 | 真实单行Input支持自由文字、建议value/label/key分离、过滤活动高亮、composition冻结、Field/native required与实际成功控件值；百万来源预算和受控拒绝恢复、四主题CSR/hydration/dirty/触屏/IME协议/classic通过。共享Choice native明确拒绝；不宣称OS IME。 | [M:Autocomplete](https://mantine.dev/core/autocomplete/) |
| P4-08 | MultiSelect | 已完成并记录证据 | 真实 Multiple Input、独立受控搜索与 known-key Vec、动态 tags/移除、IME 保护、multiple 成功值 proxy 和有界选项已实现。弱 Owner handle 支持当前过滤且 enabled 选项的原子全选及清空；受控拒绝、只读/组合输入/退休拒绝与四主题实际浏览器已验收。 新增 max_values、hide_picked_options、clearable/clear_label 与真实 placeholder；新增边界和宿主证据见 Mantine 全量 API 报告。 | [M:MultiSelect](https://mantine.dev/core/multi-select/) |
| P4-09 | TagsInput / PillsInput | 已完成并记录证据 | 自由 TagsInput/PillsInput 的原子受控 state、重复策略、有界分隔/粘贴/删除、IME 保护、成功值 proxy 与清理已实现；Glory 原生 host 协议和 AX textbox 现已连接，五项四主题实际消费端含 256 tags/UTF-16/拒绝恢复通过。平台剪贴板服务与 OS 输入/读屏另验收；原 Web16 14 项证据保留。 | [M:TagsInput](https://mantine.dev/core/tags-input/)、[M:PillsInput](https://mantine.dev/core/pills-input/) |
| P4-10 | Command / Spotlight | 已完成并记录证据 | Command/Spotlight真实Dialog+editable Input+有界Listbox/分组、显式Owner Palette快捷键、受控搜索/打开及异步结果取消已实现；Glory原生Choice/Palette连接后，四主题实际中文搜索/提交/模态恢复及同批菜单导航隔离通过，OS输入/读屏另验收。 | [S:Command](https://ui.shadcn.com/docs/components/base/command)、[X:Spotlight](https://mantine.dev/x/spotlight/) |

### P5 · 浮层与反馈

补齐现有浮层差距，再增加提示、确认、通知、进度与集中管理。

依赖：P0、P2、P3、P4。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P5-01 | AlertDialog | 已完成并记录证据 | 独立AlertDialog角色与描述、明确确认/取消和默认安全Cancel初始焦点、默认禁用outside、受控拒绝/confirm_enabled/回调退休已实现；复用host Modal Layer，四主题Web/SSR真实焦点/AX与原生四主题safe-focus/真实像素/受控确认拒绝/触发器焦点返回已验收。 | [S:Alert Dialog](https://ui.shadcn.com/docs/components/base/alert-dialog) |
| P5-02 | Sheet / Drawer gestures | 已完成并记录证据 | Drawer有限Sheet snap config与单调revision、四edge握柄capture/触屏拖动和受控snap/close拒绝回弹已实现。真实viewport变化/失焦/inert取消、动态reducedmotion、有限settle中间几何与classic End同DOM水合通过；原生Layer/sheet facet明确拒绝，未宣称OS触屏/AT公告。 | [S:Sheet](https://ui.shadcn.com/docs/components/base/sheet) |
| P5-03 | Tooltip | 已完成并记录证据 | 被动Tooltip角色与稳定描述、hover/focus、可取消Owner延迟及Escape已实现；不抢焦点/禁止交互后代，Web实际AX、dirty接管与classicstream通过，原生 Tooltip Layer 仍明确拒绝；新版 Glory 的共享 PointerWithin 已接通，不能把 Layer 拒绝等同于缺少指针边界驱动。 | [M:Tooltip](https://mantine.dev/core/tooltip/)、[S:Tooltip](https://ui.shadcn.com/docs/components/base/tooltip) |
| P5-04 | HoverCard | 已完成并记录证据 | 交互HoverCard复用非模态Preserve焦点Layer、真实PointerWithin与FocusWithin联合区间、可取消开闭延迟、受控拒绝及稳定内容策略已实现；四主题Web/SSR及实际焦点/Tab通过，原生Layer明确拒绝。 | [M:HoverCard](https://mantine.dev/core/hover-card/)、[S:Hover Card](https://ui.shadcn.com/docs/components/base/hover-card) |
| P5-05 | Alert | 已完成并记录证据 | Info/Success/Warning/Danger与Off/Polite/Assertive公告策略独立，具名真实status/alert和受控/本地关闭、动态文本接受/恢复已实现；四主题Web真实AX与原生布局/身份已验收，不声明读屏语音。 | [M:Alert](https://mantine.dev/core/alert/)、[S:Alert](https://ui.shadcn.com/docs/components/base/alert) |
| P5-06 | Notification / Toast system | 已完成并记录证据 | Notification/Toaster有限去重FIFO、原子更新/替换、动作与关闭、Owner清理和显式Clock共享单timer已实现。精确暂停/过早完成/满队列/析构重入模型10项、四主题原生2项及Web13真实浏览器10项通过；默认永久显示，原生hover与OS公告未宣称。 | [M:Notification](https://mantine.dev/core/notification/)、[X:Notifications system](https://mantine.dev/x/notifications/)、[S:Toast](https://ui.shadcn.com/docs/components/base/toast) |
| P5-07 | Spinner / Loader | 已完成并记录证据 | 加载名称/busy与decorative模式、有限旋转Graphic动画、动态reduced-motion及原生共享时钟已实现；四主题真实Web旋转矩阵/零逐帧DOM写与原生像素、隐藏暂停/Resume/Owner清理通过。 | [M:Loader](https://mantine.dev/core/loader/)、[S:Spinner](https://ui.shadcn.com/docs/components/base/spinner) |
| P5-08 | Progress / Ring / SemiCircle | 已完成并记录证据 | 只读ProgressBar有限min/max/current与本地化value_text、indeterminate不伪造0、typed line与有界圆弧Ring/SemiCircle已实现；非法源保留接受状态，四主题原生真实arc/track像素与AX、Web/SSR/stream通过。 | [M:Progress](https://mantine.dev/core/progress/)、[M:RingProgress](https://mantine.dev/core/ring-progress/)、[M:SemiCircleProgress](https://mantine.dev/core/semi-circle-progress/)、[S:Progress](https://ui.shadcn.com/docs/components/base/progress) |
| P5-09 | NavigationProgress | 已完成并记录证据 | 应用显式开始/数值/结束/取消、单当前操作及不回绕token已实现；替换撤销过期结果，Owner退休无Scope保留，真实Web八主题/adoption场景与native四主题宽度/身份通过。 | [X:NavigationProgress](https://mantine.dev/x/nprogress/) |
| P5-10 | Skeleton | 已完成并记录证据 | 有限像素/百分比占位尺寸、circle/radius、独立busy区域与decorative Graphic已实现；loading隐藏保持identity，不启动计时或动画，四主题Web/SSR及native paint实际geometry/身份已验收。 | [M:Skeleton](https://mantine.dev/core/skeleton/)、[S:Skeleton](https://ui.shadcn.com/docs/components/base/skeleton) |
| P5-11 | Overlay / LoadingOverlay | 已完成并记录证据 | Overlay保留真正绘制的装饰遮罩及pointer透传；LoadingOverlay以busy+inert抑制区域输入/焦点/AX，保留原控件身份并阻止已有top-layer逃逸。四主题CSR/hydration、classic stream/adoption及原生像素/命中/AX通过；不冒用全局modal或通用滚轮锁。 | [M:Overlay](https://mantine.dev/core/overlay/)、[M:LoadingOverlay](https://mantine.dev/core/loading-overlay/) |
| P5-12 | FloatingDialog | 已完成并记录证据 | 独立FloatingDialog为具名居中非模态notice，Preserve焦点、不trap Tab、背景可交互、默认无outside关闭，受控/本地及保留草稿策略已实现；Web/SSR真实行为通过，原生Preserve真实布局/背景焦点/Owner清理已验收。 | [M:Dialog](https://mantine.dev/core/dialog/) |
| P5-13 | ModalsManager | 已完成并记录证据 | 应用内有限FIFO打开/更新/关闭和确认队列、单active Layer、token去重与Owner清理已实现；复用真实Dialog/AlertDialog，更新保留editor，结果回调在manager Owner无借用重入；Web真实最终焦点恢复/安全取消通过。原生四主题真实top layer、Bounds/命中/像素、editor身份/selection、安全Cancel、FIFO最终触发器焦点恢复及Owner退休已验收；仅继承本次关闭managed层的有效focus fallback，Preserve/失效目标不参与，不保留关闭历史。 | [X:Modals manager](https://mantine.dev/x/modals/) |
| P5-14 | FloatingWindow | 已完成并记录证据 | 真实非模态top layer浮窗、受控有限移动/缩放、标题栏/键盘与mouse/touch capture、RaiseLayer焦点层级已完成；7组件+1模型、原生四主题真实Bounds/像素/编辑器/查询预算2、共享Layer10、Web28实际13条CSR/hydration/受控拒绝/碰撞/退休全过。 | [M:FloatingWindow](https://mantine.dev/core/floating-window/) |
| P5-15 | Tour | 已完成并记录证据 | 有限64步/仅当前mounted目标注册、真实ScrollIntoView后Bounds锚定、失效按episode通知并可恢复、焦点返回/取消和有界query已完成；8组件、原生四主题真实滚动/定位/像素/输入身份/目标退休恢复2、Web29实际12条CSR/hydration/受控拒绝/dirty/Owner全过。 | [M:Tour](https://mantine.dev/core/tour/) |

### P6 · 导航与应用布局

覆盖页面导航、侧栏、响应式壳、滚动和可调整布局。

依赖：P2、P3、P4。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P6-01 | Breadcrumbs | 已完成并记录证据 | Breadcrumbs导航地标、当前位置、真实链接与有界中间折叠已实现，Rust/四主题CSR水合通过；真实四主题原生布局/Current/identity和隐藏卸载回归已通过。 | [M:Breadcrumbs](https://mantine.dev/core/breadcrumbs/)、[S:Breadcrumb](https://ui.shadcn.com/docs/components/base/breadcrumb) |
| P6-02 | Pagination | 已完成并记录证据 | Pagination已支持1-based u64当前页/已知与未知总数、边界/跳页/禁用，恒定35slot预算；Rust与四主题浏览器通过，原生套件待完整重跑。；真实四主题原生布局/Current/identity和隐藏卸载回归已通过。 | [M:Pagination](https://mantine.dev/core/pagination/)、[S:Pagination](https://ui.shadcn.com/docs/components/base/pagination) |
| P6-03 | NavLink / NavigationMenu | 已完成并记录证据 | NavLink与真正Navigation/Link/Button/Group组成的NavigationMenu支持水平/嵌套键盘、hover、typed导航请求、受控拒绝与动态Owner焦点清理。track_focusable适配自定义动态控件；一次性打开父子浮层/resize定位已修复。模型13、原生2、四主题CSR/hydration/dirty浏览器9项通过；native可用NavLink/closed header而Group Layer打开明确拒绝。 | [M:NavLink](https://mantine.dev/core/nav-link/)、[S:Navigation Menu](https://ui.shadcn.com/docs/components/base/navigation-menu) |
| P6-04 | Sidebar | 已完成并记录证据 | 折叠Sidebar、嵌套导航、复用Modal Layer的移动Drawer及同一Owner在桌面/移动容器间移动已实现；应用选择路由/持久化，四主题浏览器焦点/受控拒绝与原生桌面布局/Layer拒绝已验收。 | [S:Sidebar](https://ui.shadcn.com/docs/components/base/sidebar) |
| P6-05 | AppShell | 已完成并记录证据 | 真实Banner/ContentInfo/Main与侧栏布局、局部ShellLayout尺寸/断点context、Resize响应式及skip link实际聚焦已实现；不绑定路由，同一导航Owner/控件身份跨布局保持。 | [M:AppShell](https://mantine.dev/core/app-shell/) |
| P6-06 | Stepper | 已完成并记录证据 | Stepper已实现Current::Step/list/状态说明、可选跳步、错误/完成、受控拒绝、KeepMounted/Unmount与实际焦点交接；FormState同步/可取消异步校验屏障显式集成，四主题CSR/水合及原生证据已通过。 | [M:Stepper](https://mantine.dev/core/stepper/) |
| P6-07 | ScrollArea / Scroller | 已完成并记录证据 | ScrollArea/Scroller使用真实ScrollMetrics与有界顺序intent、viewport自身键盘、嵌套输入保留原生编辑、实际边界和focus行为已实现；Owner query取消/晚到拒绝、四主题Web/SSR/native均已验收，水平动作明确LTR边界。 | [M:ScrollArea](https://mantine.dev/core/scroll-area/)、[M:Scroller](https://mantine.dev/core/scroller/)、[S:Scroll Area](https://ui.shadcn.com/docs/components/base/scroll-area) |
| P6-08 | Resizable / Splitter | 已完成并记录证据 | 双pane守恒比例与bounds、控/本地更新、disabled tabstop、真实Separator numeric语义和queued AX动作、keyboard/mouse/touch capture已实现；单pending几何query可取消，晚到/Owner退休有界，四主题Web/SSR及native真实协议通过。 | [M:Splitter](https://mantine.dev/core/splitter/)、[S:Resizable](https://ui.shadcn.com/docs/components/base/resizable) |
| P6-09 | Affix | 已完成并记录证据 | typed Fixed/right/bottom视口锚定、显式Relative容器Absolute/Sticky与响应式偏移已实现；native实际祖先scroll/viewport resize通过，Sticky仍明确拒绝。 | [M:Affix](https://mantine.dev/core/affix/) |
| P6-10 | Toolbar / ActionBar | 已完成并记录证据 | 有界具名分组Toolbar共享roving入口，方向键/Home/End、RTL/垂直/非循环、动态disabled与身份保留。ActionBar按受控selection count展示、只提议清空、不制造modal或夺取焦点。四项模型、一项含四主题原生实际布局和Web14四主题CSR/hydration八项通过；任意Widget/动态业务集合自动注册不在契约内。 | [M:Toolbar](https://mantine.dev/core/toolbar/)、[M:ActionBar](https://mantine.dev/core/action-bar/) |
| P6-11 | TableOfContents | 已完成并记录证据 | 仅存活256标题的显式注册索引、真实Location当前节链接/嵌套列表、实际Bounds滚动及Owner任务查询合并已完成。9组件+3纯模型、原生四主题geometry/CPU/focus/identity1和私有toc-next实际11项CSR/采用/移动端/dirty/退休全过；SSR与首测前确定性seed一致。 | [M:TableOfContents](https://mantine.dev/core/table-of-contents/) |
| P6-12 | OverflowList | 已完成并记录证据 | 有界当前128项OverflowList已实现，真实布局测量、可控前缀提案、单Owner/节点移入Popover与移回、焦点交接、重排和dirty采用已补齐。10组件+2纯模型、Web实际验收及原生inline/default Popover两项各四主题真实验收通过，含top layer、布局/命中/像素、editor身份/value/selection/focus和Owner清理。genuine Blitz fork c127fd9699a89ff8ecba0c8d8c9eccb3330627af修复隐藏旧owner的移动布局缓存引用，保留内部布局与编辑状态。 | [M:OverflowList](https://mantine.dev/core/overflow-list/) |
| P6-13 | FloatingIndicator | 已完成并记录证据 | 有界128当前target的真实Bounds装饰指示器已实现：单任务/两query、当前staged祖先校验、首次采用静态seed、GraphicTransition移动/打断/Reduce、隐藏退休与无焦点改写。5组件+2模型+共享祖先4、原生四主题像素/motion/focus2、私有layer-next实际9项全过。 | [M:FloatingIndicator](https://mantine.dev/core/floating-indicator/) |
| P6-14 | Marquee / RollingNumber | 已完成并记录证据 | Marquee/RollingNumber已走有限TranslationMotion宿主动画；单静态可访问文本、inert视觉副本、实际测宽/稳定reel、暂停/停止/中断/Reduce，无业务Cage逐帧更新。6组件+NumberFormatter11回归、四theme原生geometry/CPU/focus2、Web26实际组件9与基础9（含Rust经典stream End后动画/退休）全过。 | [M:Marquee](https://mantine.dev/core/marquee/)、[M:RollingNumber](https://mantine.dev/core/rolling-number/) |

### P7 · 数值、范围与颜色输入

先可靠的值域/步进/精度，再做拖动、多滑块和颜色二维交互。

依赖：P1、P2、P3。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P7-01 | NumberInput | 已完成并记录证据 | 精确Decimal单源NumberInput已实现：本地化完整草稿/范围/readonly/disabled/步进、真正SpinButton输入、同tick有序键盘、IME回声收口、Enter先Commit再Form Submit。6纯模型、15组件、原生组件2+Spin driver/AX2及Web22实际12项加强边界验收全过。 | [M:NumberInput](https://mantine.dev/core/number-input/) |
| P7-02 | Slider | 已完成并记录证据 | 真实 Slider 语义、键盘等价、受控 cursor（含 commit-only 同 tick 接受）、捕获指针/触屏、RTL/纵向、Field 和有界 geometry query 已实现；11 模型/2 原生与 Web16 四主题实际 10 项通过。 | [M:Slider](https://mantine.dev/core/slider/)、[S:Slider](https://ui.shadcn.com/docs/components/base/slider) |
| P7-03 | RangeSlider | 已完成并记录证据 | 双具名真实 Slider thumb、固定 Lower/Upper 身份和 Tab 顺序、Clamp/min_gap、闭区间 readonly、单次 geometry query 和受控 commit-only 已实现；10 Rust/2 原生与 Web16 四主题实际 10 项通过。交换策略明确固定为 Clamp，拒绝越过相邻 thumb。 | [M:RangeSlider](https://mantine.dev/core/range-slider/) |
| P7-04 | AngleSlider | 已完成并记录证据 | 复用有界 Slider query/cursor 的真正二维 atan2 AngleSlider、0/360 同位置数值端点与超端点环绕、中心保持、顺时针指针及圆外捕获已实现。8组件/3 mapping/2四主题原生真实几何及旋转像素与 Web18 实际10项通过；标准水平 Slider 键盘 L/R、Page/Home/End 提供等价操作。 | [M:AngleSlider](https://mantine.dev/core/angle-slider/) |
| P7-05 | Rating | 已完成并记录证据 | Whole/Half Rating、真实 RadioGroup、数值成功值、受控/Field 与 display_only 已实现；新版 Glory 原生交互/只读 Radio 支持，四主题实际 checked、稳定身份、半星像素和 direct-apply 水位验证通过。 | [M:Rating](https://mantine.dev/core/rating/) |
| P7-06 | ColorSwatch | 已完成并记录证据 | 有限RGBA preview和真实透明棋盘、可选真实Button/pressed勾选、受控来源静默、颜色文本替代与预算已实现。模型5项、四主题原生alpha像素/命中/identity1项、Web15四主题CSR/hydration/keyboard/AX/touch9项通过。 | [M:ColorSwatch](https://mantine.dev/core/color-swatch/) |
| P7-07 | HueSlider / AlphaSlider | 已完成并记录证据 | Hue/Alpha f64 数值域、真实 Slider 交互/Field、有限采样色带与真实透明棋盘、HSV/hex 纯模型已实现。6模型/7组件/1四主题原生像素通过；unstyled 默认宽高缺失已修，Web18 加强实际尺寸/不重叠断言后10项全过，并保留显式 Auto/custom 样式。 | [M:HueSlider](https://mantine.dev/core/hue-slider/)、[M:AlphaSlider](https://mantine.dev/core/alpha-slider/) |
| P7-08 | ColorPicker / ColorInput | 已完成并记录证据 | ColorPicker/ColorInput与共享DragCapture、二维饱和/亮度及真正Slider键盘替代、精确HSV/hex解析/Field/Popover已实现。11组件/3原生/Web19实际11项通过；目检发现的小plane通道宽度/indicator居中已修，加强原生真实bounds与同源command-next实际11项全部通过。默认原生浮层仍明确Unsupported，Picker与显式hex-only已验证。 | [M:ColorPicker](https://mantine.dev/core/color-picker/)、[M:ColorInput](https://mantine.dev/core/color-input/) |
| P7-09 | PinInput / InputOTP | 已完成并记录证据 | PinInput/InputOTP单真实Input+紧凑连续glyph cells已实现，单String控制/完整粘贴拒绝/IME/accepted-only完成/Field与密码AX保密均补齐。typed Ch最后u64 Value位及独立OneTimeCode/Numeric hints共享实现；6模型+10组件、原生真实字体/掩码/指针caret/布局和Web23实际12项全过。OS autofill与宽间距cell不作承诺。 | [M:PinInput](https://mantine.dev/core/pin-input/)、[S:Input OTP](https://ui.shadcn.com/docs/components/base/input-otp) |
| P7-10 | MaskInput | 已完成并记录证据 | 有限ASCII MaskInput已实现，单真实Input、raw/display UTF16映射、粘贴/字面量删除/同tick草稿、typed InputEdit与SelectText、组合及受控拒绝/超大输入恢复。9模型+9组件+原生2与旧输入36回归、Web26实际12含四themeCSR/采用/物理caret/dirty/IME/touch/退休全过；规范键入实际0次value重写。Unicode掩码与OS IME接受不宣称。 | [M:MaskInput](https://mantine.dev/core/mask-input/) |
| P7-11 | JsonInput | 已完成并记录证据 | 可选 json feature 的真实多行 JsonInput、受控草稿/格式化提案、Field 错误关系与 IME、字节/深度/输出预算已实现；格式化保留重复 key、顺序和精确数字 lexeme。7 组件/4 模型/1 四主题原生与 Web16 实际 9 项通过；默认依赖图无 JSON/serde。 | [M:JsonInput](https://mantine.dev/core/json-input/) |

### P8 · 表格与树形数据

静态语义表格先行，数据模型、选择排序、虚拟化和树导航分步加入。

依赖：P3、P4、P6。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P8-01 | Table | 已完成并记录证据 | 真实Table/caption/行组/th/td组合、显式表头关系及有限行列跨度已完成；3模型+4组件+4共享目标/回滚、HTML差分、原生四主题布局/像素/编辑器身份与跨度2、AX命名1、Web24实际12条CSR/hydration/移动/dirty/stream/退休全过。DataTable业务模型单列后续。 | [M:Table](https://mantine.dev/core/table/)、[S:Table](https://ui.shadcn.com/docs/components/base/table) |
| P8-02 | DataTable | 已完成并记录证据 | 有限本地/权威服务端分页、稳定排序/筛选/选择/列显隐、键控窗口行已完成；5模型+6组件、共享坐标/排序/窗口/HTML差分、原生四主题实际窗口/像素2及AX1、Web27b真实20条CSR/hydration/移动/脏输入/服务端迟到回复/classic stream全过，严格检查通过。相同请求的新鲜度和网络取消由应用负责。 | [S:Data Table](https://ui.shadcn.com/docs/components/base/data-table) |
| P8-03 | Tree | 已完成并记录证据 | 真实Tree/TreeItem、有限层级/同级坐标、受控展开/选择、键盘/RTL/typeahead、显式lazy与FileTree纯数据适配已完成；5模型+10组件、render3/HTML差分1、原生真实键盘AX2与四主题布局/像素/滚动/编辑器2，Web30强化12条CSR/hydration/受控拒绝/dirty/touch/禁用Fieldset全过。仅挂载当前展开的有限2048节点，不宣称任意大源窗口化。 | [M:Tree](https://mantine.dev/core/tree/)、[A:File Tree](https://ui.shadcn.com/docs/ai-elements/base/file-tree) |
| P8-04 | TreeSelect | 已完成并记录证据 | TreeSelect 复用 Tree/Combobox 与完整路径值、多选/半选策略；新版 Glory 原生树弹层已连接，四主题消费端验证真实几何、选择、关闭恢复焦点与触发器身份。 | [M:TreeSelect](https://mantine.dev/core/tree-select/) |
| P8-05 | Cascader | 已完成并记录证据 | Cascader 保留分列路径、展开/返回与懒加载；新版 Glory 原生树弹层已连接，四主题消费端验证真实几何、路径选择、焦点返回与触发器身份。 | [M:Cascader](https://mantine.dev/core/cascader/) |

### P9 · 日期与时间

独立日期模型和国际化策略；日历网格先于弹出式日期输入。

依赖：P1、P4、P7。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P9-01 | Date model / DatesProvider | 已完成并记录证据 | 可选dates无依赖公历模型和显式DatesProvider已完成，civil/local/instant/fixed-offset独立类型、严格ISO解析、locale/week-start/参考日期与today/minmax明确传入；7模型测试含完整400年146097日与9999年所有月份边界、负Unix及偏移溢出，严格通过。不推断IANA/DST或OS时钟。 | Sapid/Glory 共享能力 |
| P9-02 | Calendar / MiniCalendar | 已完成并记录证据 | 有限42格Calendar/MiniCalendar、独立焦点/选择/today、月年跳转、禁用日/边界/locale/RTL、共享Grid键盘协议已完成；6组件+7日期模型、共享目标/HTML差分/原生键盘AX、原生四主题布局像素2、Web27b实际12条CSR/hydration/同tick导航/dirty/退休全过。 | [D:Calendar](https://mantine.dev/dates/calendar/)、[D:MiniCalendar](https://mantine.dev/dates/mini-calendar/)、[S:Calendar](https://ui.shadcn.com/docs/components/base/calendar) |
| P9-03 | DatePicker / DatePickerInput | 已完成并记录证据 | 单日/固定容量32多日/有序范围、Calendar复用、Field成功值及默认真实Dialog输入弹层已完成；5模型+9组件、原生默认popup实际3（四主题布局/像素/焦点/输入身份/退休）、Web28真实12条CSR/hydration/受控拒绝/同tick/容量/dirty/touch全过。 | [D:DatePicker](https://mantine.dev/dates/date-picker/)、[D:DatePickerInput](https://mantine.dev/dates/date-picker-input/)、[S:Date Picker](https://ui.shadcn.com/docs/components/base/date-picker) |
| P9-04 | DateInput | 已完成并记录证据 | 真实单行Input、有限严格ISO/注入parser、独立Empty/Incomplete/Valid/Invalid/OutOfBounds草稿与civil值、composition/程序revision/受控拒绝、默认Calendar Dialog及Field已完成；3模型+10组件、原生四主题输入/caret/像素/popup2、Web30强化12条通过，Calendar选择先发布Valid草稿再value回调。 | [D:DateInput](https://mantine.dev/dates/date-input/) |
| P9-05 | MonthPicker / MonthPickerInput | 已完成并记录证据 | 按月选择及范围，网格键盘导航。 | [D:MonthPicker](https://mantine.dev/dates/month-picker/)、[D:MonthPickerInput](https://mantine.dev/dates/month-picker-input/) |
| P9-06 | YearPicker / YearPickerInput | 已完成并记录证据 | 年份/区间、十年视图和边界。 | [D:YearPicker](https://mantine.dev/dates/year-picker/)、[D:YearPickerInput](https://mantine.dev/dates/year-picker-input/) |
| P9-07 | TimeInput / TimeValue | 已完成并记录证据 | TimeInput/TimeValue 保留小时/分/秒、12/24h、空值与本地化；新版 Glory 原生支持中文 UTF-16 精确选区，真实编辑器、只读/禁用、身份与像素已验证。 | [D:TimeInput](https://mantine.dev/dates/time-input/)、[D:TimeValue](https://mantine.dev/dates/time-value/) |
| P9-08 | TimePicker / TimeGrid | 已完成并记录证据 | 分段时间输入与可选时间网格、步长/禁用值。 | [D:TimePicker](https://mantine.dev/dates/time-picker/)、[D:TimeGrid](https://mantine.dev/dates/time-grid/) |
| P9-09 | DateTimePicker / InlineDateTimePicker | 已完成并记录证据 | DateTimePicker 与 InlineDateTimePicker：显式 civil/fixed-offset/application resolver、fold/gap 策略，部分草稿不默认补齐，受控值/清除/拒绝、真实 Calendar/TimePicker 组合和 Done/Escape/Form 边界。 | [D:DateTimePicker](https://mantine.dev/dates/date-time-picker/)、[D:InlineDateTimePicker](https://mantine.dev/dates/inline-date-time-picker/) |

### P10 · 文件、媒体、代码与编辑器

宿主能力显式授权；文件、剪贴板、富文本与高亮保持可选依赖。

依赖：P1、P4、P5、P6、P8。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P10-01 | FileInput / FileButton | 已完成并记录证据 | FileButton/FileInput 与 Dropzone/Attachment：真实可信 OS chooser/drop、有限非 ambient File 引用/版本/元数据/字节读取、filter/count/bytes 预算、四个跨 App 不可早退 read permits、当前预览 URL 安全解码/撤销、dirty adoption/Owner 清理。显式 blitz-files 已接通原生选择/读取/私有预览及四主题真实消费端；桌面原生投放通道与四主题 Dropzone 真实句柄执行端已通过；移动 driver、实际 OS 拖动/chooser 输入验收仍开放，ambient 路径不支持；上传网络由应用提供。 | [M:FileInput](https://mantine.dev/core/file-input/)、[M:FileButton](https://mantine.dev/core/file-button/) |
| P10-02 | Dropzone / Attachment | 已完成并记录证据 | FileButton/FileInput 与 Dropzone/Attachment：真实可信 OS chooser/drop、有限非 ambient File 引用/版本/元数据/字节读取、filter/count/bytes 预算、四个跨 App 不可早退 read permits、当前预览 URL 安全解码/撤销、dirty adoption/Owner 清理。显式 blitz-files 已接通原生选择/读取/私有预览及四主题真实消费端；桌面原生投放通道与四主题 Dropzone 真实句柄执行端已通过；移动 driver、实际 OS 拖动/chooser 输入验收仍开放，ambient 路径不支持；上传网络由应用提供。 | [X:Dropzone](https://mantine.dev/x/dropzone/)、[S:Attachment](https://ui.shadcn.com/docs/components/base/attachment) |
| P10-03 | CopyButton | 已完成并记录证据 | 显式带revision/request的可信宿主剪贴板action、真实成功/拒绝/不可用/失败、每文档至多一个OS写入、Owner/源替换撤销与注入计时反馈已完成；4组件、共享协议/HTML零输出差分/SSR拒绝、原生默认不可用及四主题像素1；显式可选桌面剪贴板、四主题 CopyButton/Code/Json/Tags 异步消费端和 macOS 系统往返另已通过，Android 安全 JNI/ARM64 前台系统服务独立验收，iOS driver/移动菜单仍开放；Web28实际9组写读/权限/迟到/dirty/退休全过。 | [M:CopyButton](https://mantine.dev/core/copy-button/) |
| P10-04 | Carousel | 已完成并记录证据 | 索引轮播最多七个短 Owner、百万 count 不扫描源；受控页、RTL/Swipe、注入时钟与暂停协议已实现。新版 Glory 原生真实鼠标悬停可取消当前时钟，离开恢复；四主题几何/像素与清理验证保留。 | [X:Carousel](https://mantine.dev/x/carousel/)、[S:Carousel](https://ui.shadcn.com/docs/components/base/carousel) |
| P10-05 | Lightbox | 已完成并记录证据 | 最多128条/256KiB元数据，真实管理Dialog中仅一个Image资源消费者；替代文本/解码失败、受控键值和1..4倍实际滚动缩放、键盘和焦点返回，关闭/Owner释放及宿主资源配置边界；Web/Native四主题实测。 | [X:Lightbox](https://mantine.dev/x/lightbox/) |
| P10-06 | CodeBlock / CodeHighlight | 已完成并记录证据 | 可选code无依赖语言tokenizer、真实可选文本/行号/强调/滚动、字节/token/行预算及accepted source真实复制已完成；7 Code+5组合复制测试、原生四主题布局像素1，原Code9与Web30配置复制8实际通过，精确源不含gutter/token装饰。 | [H:CodeHighlight](https://mantine.dev/code-highlight/code-highlight/)、[S:Code Block](https://ui.shadcn.com/docs/components/base/code-block) |
| P10-07 | JsonViewer | 已完成并记录证据 | 有界无损JSON模型/重复key拒绝、文档disclosure与显式真实Tree模式、路径身份/搜索/深度节点预算及accepted完整源复制已完成；10 Rust、原生四主题两模式实际布局/像素/焦点/重排、原Json9及Web30 Tree9/组合复制8实际通过。Tree模式支持含root64级并在发布前拒更深数据。 | [H:JsonViewer](https://mantine.dev/code-highlight/json-viewer/) |
| P10-08 | RichTextEditor | 已完成并记录证据 | 真实有限富文档/UTF16选区/marks/段落与heading、应用受控拒绝、独立32项2MiB历史、Web composition/source仲裁/真实plain clipboard；FORMAT13共享mapper/SSRstream/oracle，Web43原9+classic11+extra3全过；Native编辑与相关AX明确typed拒绝，不用Textarea冒充。 | [X:Rich text editor](https://mantine.dev/x/tiptap/) |

### P11 · 图表

可选图表模块，先可访问的数据替代表示和绘图协议，再逐图形扩展。

依赖：P3、P5、P7。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P11-01 | Chart / ChartTheme / Legend / Tooltip | 已完成并记录证据 | 可选 charts 无新增依赖；有限 data/domain/frame/Graphic 协议、八色主题、真实 legend/tooltip、逐点操作/受控提案、可访问完整数值表与安全 SVG 几何导出。原生默认 Click/键盘/绘制支持，pointer hover/Tooltip 保持明确拒绝。 | [S:Chart](https://ui.shadcn.com/docs/components/base/chart) |
| P11-02 | AreaChart / LineChart / BarChart / CompositeChart | 已完成并记录证据 | Line/Area/Bar/Composite 实际几何、真实空值断线、正负 bar 堆叠与非负 area 堆叠、有限刻度/显式裁剪、inclusive brush、稳定完整 key、全数据表、受控拒绝及 CSR/adoption/原生四主题验收。 | [C:AreaChart](https://mantine.dev/charts/area-chart/)、[C:LineChart](https://mantine.dev/charts/line-chart/)、[C:BarChart](https://mantine.dev/charts/bar-chart/)、[C:CompositeChart](https://mantine.dev/charts/composite-chart/) |
| P11-03 | PieChart / DonutChart / RadialBarChart | 已完成并记录证据 | 真实 Pie/Donut 角度占比和零值处理、RadialBar 同心弧线；有限键盘逐点与完整数值表，四主题 Web CSR/adoption 和 Native 真实绘制/命中验收。 | [C:PieChart](https://mantine.dev/charts/pie-chart/)、[C:DonutChart](https://mantine.dev/charts/donut-chart/)、[C:RadialBarChart](https://mantine.dev/charts/radial-bar-chart/) |
| P11-04 | ScatterChart / BubbleChart / CandlestickChart | 已完成并记录证据 | 真实 XY/面积尺寸尺度与零尺寸、OHLC 交叉约束/方向、至多128次索引回调的显式采样及来源数量说明；有限逐点/完整表，四主题 Web/Native 实测。 | [C:ScatterChart](https://mantine.dev/charts/scatter-chart/)、[C:BubbleChart](https://mantine.dev/charts/bubble-chart/)、[C:CandlestickChart](https://mantine.dev/charts/candlestick-chart/) |
| P11-05 | RadarChart / FunnelChart / Sparkline | 已完成并记录证据 | Radar 径向轴/缺值断线、Funnel 独立梯形、Sparkline 真正分段路径，保留文字与数值替代、受控交互及四主题 Web/Native 实测。 | [C:RadarChart](https://mantine.dev/charts/radar-chart/)、[C:FunnelChart](https://mantine.dev/charts/funnel-chart/)、[C:Sparkline](https://mantine.dev/charts/sparkline/) |
| P11-06 | Heatmap / MatrixChart / WaffleChart | 已完成并记录证据 | 32×32/1024cells 有限 Heatmap/面积 Matrix、明确五档数值色标和每格原值，Waffle 确定性100格余数分配；数据表分块完整保留，Web/Native 实测。 | [C:Heatmap](https://mantine.dev/charts/heatmap/)、[C:MatrixChart](https://mantine.dev/charts/matrix-chart/)、[C:WaffleChart](https://mantine.dev/charts/waffle-chart/) |
| P11-07 | BarsList / BulletChart / GaugeChart | 已完成并记录证据 | 有符号水平 BarsList、目标/分段 Bullet、半环 Gauge；真实只读 Progress 范围、刻度/目标/精确表值、键盘受控交互，Web/Native 实测。 | [C:BarsList](https://mantine.dev/charts/bars-list/)、[C:BulletChart](https://mantine.dev/charts/bullet-chart/)、[C:GaugeChart](https://mantine.dev/charts/gauge-chart/) |
| P11-08 | Treemap / SunburstChart / SankeyChart | 已完成并记录证据 | 128节点/depth32 当前层级权重/隐藏派生、交替 Treemap 与角度深度 Sunburst、256边有限 DAG Sankey 保持流量端口厚度；稳定key、父子导航和完整表，Web/Native 实测。 | [C:Treemap](https://mantine.dev/charts/treemap/)、[C:SunburstChart](https://mantine.dev/charts/sunburst-chart/)、[C:SankeyChart](https://mantine.dev/charts/sankey-chart/) |

### P12 · 日程

先事件/时区/重复模型和只读视图，再资源视图与拖动编辑。

依赖：P8、P9、P10。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P12-01 | Schedule / Event model / Recurrence | 已完成并记录证据 | Schedule 有限当前请求窗口、事件/资源/冲突模型、daily/weekly/monthly/yearly 重复与例外、COUNT/UNTIL/月末策略、直跳窗口而非无限预展开；civil/fixed-offset/application resolver 显式 fold/gap。Schedule 当前已交付真实 Agenda；其余视图按独立任务验收。 | [T:Schedule](https://mantine.dev/schedule/schedule/)、[T:Recurring events](https://mantine.dev/schedule/recurring-events/)、[T:Events data](https://mantine.dev/schedule/events-data/) |
| P12-02 | DayView / WeekView | 已完成并记录证据 | Day/Week 真实时间 Grid、半开区间几何、独立重叠lane、slot/事件键盘访问、readonly/disabled、受控选择与实际 ScrollIntoView；当前有限窗口/稳定 key/Owner 清理，短日期表头在四主题 CSR/adoption 和 Native 实际布局中验收。 | [T:DayView](https://mantine.dev/schedule/day-view/)、[T:WeekView](https://mantine.dev/schedule/week-view/) |
| P12-03 | MonthView / YearView / MobileMonthView | 已完成并记录证据 | 真实42日Month/12月Year Grid，weekday列标题、预览上限、有限显式详情与Mobile单日展开；受控分页/选择/readonly、首末行箭头不串列、当前禁用focus护栏，四主题Web/Native实际几何/命中/像素/清理通过。 | [T:MonthView](https://mantine.dev/schedule/month-view/)、[T:YearView](https://mantine.dev/schedule/year-view/)、[T:MobileMonthView](https://mantine.dev/schedule/mobile-month-view/) |
| P12-04 | AgendaView | 已完成并记录证据 | 真实按日期/半开请求区间分组的 Agenda List/Heading/事件按钮、受控选择/拒绝、readonly、窗口分页、当前数据稳定 key 更新、空状态与 Owner 清理；四主题 CSR/adoption、360px touch 与原生布局/命中/像素已验证。 | [T:AgendaView](https://mantine.dev/schedule/agenda-view/) |
| P12-05 | Resource schedule views | 已完成并记录证据 | 真实单画布资源日/周/月排程与双轴Grid，当前有限资源/日期窗口和权限，鼠标/CDP触摸/query-backed跨日跨资源移动/端点调整、键盘等价提案、受控拒绝与同tick排序/Owner取消，所有业务授权/冲突/持久化归App；Web42强化12与Native3实际通过。 | [T:ResourcesDayView](https://mantine.dev/schedule/resources-day-view/)、[T:ResourcesWeekView](https://mantine.dev/schedule/resources-week-view/)、[T:ResourcesMonthView](https://mantine.dev/schedule/resources-month-view/)、[T:ResourcesSchedule](https://mantine.dev/schedule/resources-schedule/) |

### P13 · 对话及 AI 应用展示

以传入的数据和动作构造展示组件；不绑定模型服务、网络、工具执行或隐藏推理数据。

依赖：P4、P5、P6、P8、P10。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P13-01 | Message / Bubble / Marker | 已完成并记录证据 | 有限调用方消息数据、真实段落/Blockquote/Anchor、作者时间状态与分组、引用标记和调用方动作；完整模型先验收再更新，literal文本转义、稳定节点身份和Owner清理，四主题Web CSR/adoption及Native真实布局通过。 | [S:Message](https://ui.shadcn.com/docs/components/base/message)、[S:Bubble](https://ui.shadcn.com/docs/components/base/bubble)、[S:Marker](https://ui.shadcn.com/docs/components/base/marker)、[A:Message](https://ui.shadcn.com/docs/ai-elements/base/message) |
| P13-02 | MessageScroller / Chat | 已完成并记录证据 | 真实索引窗口消息列表/Chat组合，最多64live+focused1、256测量；显式accepted append通知/未读、用户离底/回尾与prepend和顶部相交key+within测量锚点；晚到program ack不冒充user，原始40浏览器12全通过。 | [S:Message Scroller](https://ui.shadcn.com/docs/components/base/message-scroller)、[A:Chat](https://ui.shadcn.com/docs/ai-elements/base/chat) |
| P13-03 | PromptInput | 已完成并记录证据 | 真实Form/Textarea/Submit和Cancel、受控提案拒绝、完整host tick后Owner任务回调，浏览器IME与输入法提交门控、有限建议；可选FileController显式文件权限/读取，无自动发送/清空或模型服务；四主题Web与Native实测，Native可选文件明确拒绝。 | [A:Prompt Input](https://ui.shadcn.com/docs/ai-elements/base/prompt-input) |
| P13-04 | Questionnaire | 已完成并记录证据 | 真实typed FormState与Stepper的有限Questionnaire，受控步骤提案/拒绝、KeepMounted答案身份、当前步与最终全步骤重新验证，连续Invalid attempt与异步Back/Owner取消护栏；无JSON schema VM/服务。Web43原始12和四主题Native实测通过。 | [S:Questionnaire](https://ui.shadcn.com/docs/components/base/questionnaire) |
| P13-05 | ModelSelector | 已完成并记录证据 | 有限显式模型/provider catalog、真实 Combobox/Search/Listbox、独立 accepted model/query 与受控拒绝已实现；新版 Glory 原生 editable Choice 已连接，四主题消费端验证业务值、拒绝/接受、编辑器焦点/身份与像素。 | [A:Model Selector](https://ui.shadcn.com/docs/ai-elements/base/model-selector) |
| P13-06 | Suggestion | 已完成并记录证据 | 真实Toolbar/Buttons有限建议组，当前应用label/value、方向/箭头/Home/End单Tab、disabled与完整key/Owner清理，选择仅typed应用回调不自动发送；四主题Web43与Native实际通过。 | [A:Suggestion](https://ui.shadcn.com/docs/ai-elements/base/suggestion) |
| P13-07 | Confirmation / Checkpoint | 已完成并记录证据 | 有限Confirmation/Checkpoint显式App状态与revision，真实Approve/Reject/Restore按钮仅提案，controlled拒绝/source静默/disabled或缺callback/Owner清理均验证；不授予权限或执行回退，Web43原9和四主题Native实际通过。 | [A:Confirmation](https://ui.shadcn.com/docs/ai-elements/base/confirmation)、[A:Checkpoint](https://ui.shadcn.com/docs/ai-elements/base/checkpoint) |
| P13-08 | Plan / Task / Queue | 已完成并记录证据 | 有限128项/128KiB应用Plan/Task/Queue，真实有序列表和精确任务/processed进度、完整key稳定更新/重排、可操作项仅typed应用回调及受控拒绝、真实禁用/Owner清理，无调度/执行/授权服务；四主题Web和Native实际通过。 | [A:Plan](https://ui.shadcn.com/docs/ai-elements/base/plan)、[A:Task](https://ui.shadcn.com/docs/ai-elements/base/task)、[A:Queue](https://ui.shadcn.com/docs/ai-elements/base/queue) |
| P13-09 | Reasoning / ChainOfThought | 已完成并记录证据 | 仅显示应用已提供可公开摘要/有限步骤，真实Collapsible与无序/有序列表，显式状态/耗时、受控拒绝、完整模型先验和稳定key/Owner清理；无隐藏推理访问或模型服务，四主题Web/Native实际通过。 | [A:Reasoning](https://ui.shadcn.com/docs/ai-elements/base/reasoning)、[A:Chain of Thought](https://ui.shadcn.com/docs/ai-elements/base/chain-of-thought) |
| P13-10 | Context / Sources / InlineCitation | 已完成并记录证据 | 显式上下文整数用量/未知容量/超限文字与只读Progress；有限Sources真实List/Link、literal预览与调用方可信度，InlineCitation真实文内链接/默认fragment导航和可选App接管，无自动抓取；四主题Web/Native实际通过。 | [A:Context](https://ui.shadcn.com/docs/ai-elements/base/context)、[A:Sources](https://ui.shadcn.com/docs/ai-elements/base/sources)、[A:Inline Citation](https://ui.shadcn.com/docs/ai-elements/base/inline-citation) |
| P13-11 | Trace / StackTrace / Subagent / Tool | 已完成并记录证据 | sealed有限Trace森林/Stack帧和真实Tool/Subagent Collapsible/ordered list，显式Public/Truncated/无原文Redacted与应用状态/elapsed/Inspect或location回调；完整key/受控展开/hidden与Owner重入释放，严格360px/1100px和函数/位置分行；无工具执行/调度/file访问，Web44原始24和四主题Native2实际通过。 | [S:Trace](https://ui.shadcn.com/docs/components/base/trace)、[A:Stack Trace](https://ui.shadcn.com/docs/ai-elements/base/stack-trace)、[A:Subagent](https://ui.shadcn.com/docs/ai-elements/base/subagent)、[A:Tool](https://ui.shadcn.com/docs/ai-elements/base/tool) |
| P13-12 | Terminal / TestResults | 已完成并记录证据 | 只读Terminal和TestResults，十亿条索引源有界可见行/测量/稀疏索引，显式stdout/stderr/状态/统计、literal控制字符转义与无原文Redacted；真实测试详情/First/Newest/Pause、完整key和受控源、清空时序/Owner回收，无命令或测试执行；Web42原始10与四主题Native实际通过。 | [A:Terminal](https://ui.shadcn.com/docs/ai-elements/base/terminal)、[A:Test Results](https://ui.shadcn.com/docs/ai-elements/base/test-results) |

## 快照覆盖校验

| 目录 | 原始条目数 |
|---|---:|
| mantine-core | 121 |
| shadcn-components | 66 |
| mantine-dates | 15 |
| mantine-charts | 22 |
| mantine-schedule | 13 |
| mantine-code | 2 |
| mantine-extensions | 8 |
| shadcn-ai | 21 |

共 268 个来源条目，归并为 149 项工作：0 项已有基线、0 项部分基础、0 项未实现、149 项按后续完成标准关闭。日程目录包含事件数据与重复规则两项模型文档，基础设施也单独计入，不能把总数解释为缺失组件数量。

## 对照来源

- [mantine-core](https://mantine.dev/core/package/)
- [shadcn-components](https://ui.shadcn.com/docs/components)
- [mantine-extensions](https://mantine.dev/x/extensions/)
- [mantine-dates](https://mantine.dev/dates/getting-started/)
- [mantine-charts](https://mantine.dev/charts/getting-started/)
- [mantine-schedule](https://mantine.dev/schedule/getting-started/)
- [mantine-code](https://mantine.dev/code-highlight/getting-started/)
- [shadcn-ai](https://ui.shadcn.com/docs/ai-elements)

命名差异另核对 [Mantine Dialog](https://mantine.dev/core/dialog/)、[Mantine Modal](https://mantine.dev/core/modal/) 和 [shadcn Drawer](https://ui.shadcn.com/docs/components/base/drawer)。
