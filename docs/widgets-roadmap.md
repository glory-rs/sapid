# Sapid widgets 对照与逐项实现计划

核对日期：2026-10-24（目录抓取于 10 月 23–24 日）。Mantine 官方文档显示版本 9.7.3；shadcn 使用本次官方组件目录快照。
这是一个冻结的对照快照，后续上游新增项需要显式核对并更新清单，不自动改变目标范围。
本页由 `docs/widget-inventory.json` 生成；修改清单后运行 `python scripts/widget-plan.py`，
运行 `python scripts/widget-plan.py --check` 检查覆盖完整性和文档同步。

## 当前判断

原有九个主要组件为 Button、TextField、Checkbox、Collapsible、Accordion、Tabs、
Dialog、Drawer、Popover；第一批已增加 Field、Label/Description/Error、Fieldset/Legend，
并接入 TextField/Checkbox 的标签及 required/readonly/invalid。已有测试证明相应功能，
不表示与某个上游组件全部功能等价。蓝/绿各有明暗主题，方向/locale/完整 motion 仍待补齐。
第二批已增加 Textarea、PasswordInput、InputGroup/Addon/Section，完成 Field、受控状态、
Web/SSR 与原生引擎验收。Textarea 自动高度依赖浏览器 field-sizing:content，Blitz 明确拒绝；
固定行数可用于原生。后续批次已增加 Form/FormState、成功控件快照、MixedCheckbox/组、
Switch、RadioGroup/SegmentedControl、动作按钮、展示表面、NumberFormatter 和文档文本/列表组件，
对应证据逐项记录在下表。Toggle、Chip、布局、Typography、Grid/比例、Timeline、Spoiler 与导航
已增加实际宿主证据；Menu、Stepper、辅助文本及图像队列继续推进，完成状态以清单为准。
Glory 的布局/图像/窗口列表基础不等于 Sapid 已完成 Grid/DataTable 等组件。

名称需要按语义对照：当前 Dialog 对应 Mantine Modal / shadcn Dialog；Mantine Dialog
是另外的非模态提示需求，单列 FloatingDialog。Sheet 复用 Drawer 的边缘浮层能力，手势和 snap
points 仍需实现。Collapse 与 Collapsible 复用展开模型，动画单独补齐。Mantine 布局 Group
不是 Glory 的 boxless Group。菜单、Tooltip、Combobox 也不能仅改 Popover 的外观就算完成。

当前浮层交互支持 Web，SSR 输出与 hydration 已验证；Blitz 仍明确拒绝 Layer。跨 App 浮层栈、
完整弹层定位、滚动锁、移动 visual viewport、动态集合和更完整的主题都属于待补强内容。

## 目标边界

1. 覆盖两套官方目录中可见的核心 widgets，并纳入 Mantine 日期/图表/日程/编辑器等扩展。
   shadcn 新增的消息、问卷、Trace 和 AI Elements 也保留条目，作为后期可选展示模块。
2. 同义名称和紧密变体共享实现，但清单保留每个来源 URL。149 项工作不是 149 个独立 widgets；
   其中包含已有组件补强、公共基础设施和多个变体的共同实现。每个变体都要有自己的验收证据。
3. 不照搬 React/JSX API、hooks、JS 库依赖和代码分发 CLI。继续使用手写 Rust builder、
   Cage/Bond/Lotus、Glory Widget 与现有所有权/宿主边界；第三方实现不复制进仓库。
4. 社区 Registry、页面 Blocks、完整业务应用和任意第三方插件不是有限可穷尽的 widgets 清单。
   React 表单库绑定不移植，但对应表单能力列入 P1；主题/provider/方向/locale 是基础设施。
5. 默认 Sapid 保持轻依赖。日期、图表、文件、富文本、日程和聊天组件使用可选模块或独立 crate。
   `sapid-dates/charts/files/editor/schedule/chat` 仅是候选拆分，尚未创建，不宣称接口已确定。
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
4. 已完成 MixedCheckbox/组、Switch、RadioGroup、SegmentedControl；Toggle/Chip 与后续展示/布局继续验收。

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

这些是分阶段设计目标，不是现在已经增加的 enum/API。优先扩展共享有限契约、验证、mapper 和实际
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
| P0-02 | TextField | 已有基线，需补强 | 已有受控/本地文本、版本和组合输入；通过 P1 接入完整 Field 语义。 | [M:TextInput](https://mantine.dev/core/text-input/)、[S:Input](https://ui.shadcn.com/docs/components/base/input) |
| P0-03 | Checkbox | 已有基线，需补强 | 已有布尔状态；P2 补 mixed、分组及表单值。 | [M:Checkbox](https://mantine.dev/core/checkbox/)、[S:Checkbox](https://ui.shadcn.com/docs/components/base/checkbox) |
| P0-04 | Collapsible | 已有基线，需补强 | 已有展开和内容策略；补独立触发器/内容插槽与可选高度过渡。 | [M:Collapse](https://mantine.dev/core/collapse/)、[S:Collapsible](https://ui.shadcn.com/docs/components/base/collapsible) |
| P0-05 | Accordion | 已有基线，需补强 | 已有单/多开和方向键；共享集合升级后增加动态成员与更多组合插槽。 | [M:Accordion](https://mantine.dev/core/accordion/)、[S:Accordion](https://ui.shadcn.com/docs/components/base/accordion) |
| P0-06 | Tabs | 已有基线，需补强 | 已有自动/手动激活与 roving focus；补动态成员、方向和国际化。 | [M:Tabs](https://mantine.dev/core/tabs/)、[S:Tabs](https://ui.shadcn.com/docs/components/base/tabs) |
| P0-07 | Dialog | 已有基线，需补强 | 已有 Web modal/focus/Escape/outside；仍需原生浮层、滚动锁、组合插槽与跨 App 策略。 | [M:Modal](https://mantine.dev/core/modal/)、[S:Dialog](https://ui.shadcn.com/docs/components/base/dialog) |
| P0-08 | Drawer | 已有基线，需补强 | 已有四侧贴边 Web 浮层；P5 补手势、snap points、移动 viewport 和滚动协调。 | [M:Drawer](https://mantine.dev/core/drawer/)、[S:Drawer](https://ui.shadcn.com/docs/components/base/drawer) |
| P0-09 | Popover | 已有基线，需补强 | 已有交互式弹层及底部定位；补完整 placement/collision、锚点失效与可组合触发器。 | [M:Popover](https://mantine.dev/core/popover/)、[M:ComboboxPopover](https://mantine.dev/core/combobox-popover/)、[S:Popover](https://ui.shadcn.com/docs/components/base/popover) |
| P0-10 | Collection / RovingFocus / Typeahead | 已完成并记录证据 | 有界存活Owner/key注册、roving禁用/循环/首尾/RTL及有限字符搜索已实现；Radio接入并完成Web/SSR验收，原生RTL已验证、字符搜索驱动明确拒绝，不保留访问历史。 | Sapid/Glory 共享能力 |
| P0-11 | FocusScope / Layer / Portal | 部分基础 | 现有宿主浮层已有范围管理；补通用焦点范围、嵌套模态和非模态关系、原生 top layer，明确 Portal 目标和 Owner 归属。 | [M:FocusTrap](https://mantine.dev/core/focus-trap/)、[M:Portal](https://mantine.dev/core/portal/) |
| P0-12 | Theme / Direction / Locale / Motion | 部分基础 | 已完成蓝/绿各明暗主题、十阶色板、独立语义色和正文/按钮对比度检查、可选 focus/hover 样式、reduced-motion；主题为挂载配置。方向/本地化、完整 spacing/typography/radius token 系统、宿主时钟和过渡生命周期仍待实现。 | [S:Direction](https://ui.shadcn.com/docs/components/base/direction)、[M:Transition](https://mantine.dev/core/transition/) |
| P0-13 | VisuallyHidden | 已完成并记录证据 | 纯文本绝对定位零尺寸与双轴裁剪，不使用hidden/display:none；动态文字及稳定描述关系、四主题真实Web/native布局和AX名称/描述已验收。 | [M:VisuallyHidden](https://mantine.dev/core/visually-hidden/) |
| P0-14 | Gallery / Adapter capability matrix | 部分基础 | 已更新文档展示站：导航/搜索、四套主题、交互预览、API 表、响应式布局；基础/组合/浮层页均保留当前页切换主题，跨页保留配色，24 组 CSR/SSR 主题实例已验收。Field 的 Web/SSR/hydration 与原生语义已有证据。完整逐组件适配器矩阵、其他新组件及真实 OS/AT 验收仍需持续补充。 | Sapid/Glory 共享能力 |

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
| P2-01 | MixedCheckbox / CheckboxGroup | 已完成并记录证据 | 真实三态indeterminate与有限CheckboxGroup全选/禁用保留；Web脏接管/stream已验证。纯无JS SSR记录三态；Blitz明确拒绝Mixed，保留成功前缀与查询水位。 | Sapid/Glory 共享能力 |
| P2-02 | Switch | 已完成并记录证据 | 真实Switch角色/checked、Field、受控拒绝/外部静默、键盘与主题轨道；Web四主题和原生checked/AX Switch身份已验证。 | [M:Switch](https://mantine.dev/core/switch/)、[S:Switch](https://ui.shadcn.com/docs/components/base/switch) |
| P2-03 | Radio / RadioGroup | 已完成并记录证据 | 真实Radio/RadioGroup、single Tab、方向/Home/End、禁用与整组受控恢复；native name与successful name须一致；Web/adoption/stream保护脏同组，Blitz明确拒绝Radio。 | [M:Radio](https://mantine.dev/core/radio/)、[S:Radio Group](https://ui.shadcn.com/docs/components/base/radio-group) |
| P2-04 | Toggle / ToggleGroup | 已完成并记录证据 | 真实Button pressed、单/多选清空、roving焦点与箭头不改值，12项组件测试、四主题原生及9组浏览器验收通过。 | [M:Toggle](https://mantine.dev/core/toggle/)、[S:Toggle](https://ui.shadcn.com/docs/components/base/toggle)、[S:Toggle Group](https://ui.shadcn.com/docs/components/base/toggle-group) |
| P2-05 | SegmentedControl | 已完成并记录证据 | SegmentedControl复用真实单选值模型，水平默认/有限items、单Tab与受控拒绝；未冒用Tabs面板语义，原生沿用Radio拒绝。 | [M:SegmentedControl](https://mantine.dev/core/segmented-control/) |
| P2-06 | Chip / ChipGroup | 已完成并记录证据 | Chip真实Checkbox、ChipGroup单选Radio/多选Checkbox，接入受控值、Field、禁用与有限集合；Web/SSR验收通过，原生Radio明确拒绝而Checkbox可用。 | [M:Chip](https://mantine.dev/core/chip/) |
| P2-07 | ButtonGroup | 已完成并记录证据 | 自然Tab的ButtonGroup与可选真实Toolbar roving均已实现；禁用/loading跳过、首尾/循环/RTL/垂直焦点导航不激活，Web与原生实际焦点/身份已验收。 | [S:Button Group](https://ui.shadcn.com/docs/components/base/button-group) |
| P2-08 | ActionIcon / CloseButton | 已完成并记录证据 | ActionIcon与CloseButton复用真实Button激活，强制非空名称、主题尺寸/unstyled与disabled；浏览器AX名称和原生布局已验证。 | [M:ActionIcon](https://mantine.dev/core/action-icon/)、[M:CloseButton](https://mantine.dev/core/close-button/) |
| P2-09 | Burger | 部分基础 | Burger已有具名展开控制、本地/受控状态、拒绝恢复、controls/expanded和稳定身份，Web/原生已验证；pressed与bar动画仍待补齐。 | [M:Burger](https://mantine.dev/core/burger/) |

### P3 · 展示、排版与基础布局

统一主题和组合 API，为后续菜单、导航、数据展示提供稳定外观。

依赖：P0。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P3-01 | Box / Flex / Stack / Group | 已完成并记录证据 | Box/Flex/Stack/布局Group使用真实View，支持gap/对齐/换行与结构unstyled；Rust/SSR、四主题原生geometry和8组布局浏览器验收通过。 | [M:Box](https://mantine.dev/core/box/)、[M:Flex](https://mantine.dev/core/flex/)、[M:Stack](https://mantine.dev/core/stack/)、[M:Group](https://mantine.dev/core/group/) |
| P3-02 | Center / Container / Space | 部分基础 | Center/Container/Space已实现双轴居中、居中内框/最大宽度及显式px/percent spacing；四主题SSR/Web/native通过，全局breakpoint token仍待补。 | [M:Center](https://mantine.dev/core/center/)、[M:Container](https://mantine.dev/core/container/)、[M:Space](https://mantine.dev/core/space/) |
| P3-03 | Grid / SimpleGrid | 部分基础 | 真实typed Grid/SimpleGrid、有限轨道/跨度/offset/自动排布、独立profile/CSS拒绝与SSR/Web/native已验证；响应式断点provider仍待补。 | [M:Grid](https://mantine.dev/core/grid/)、[M:SimpleGrid](https://mantine.dev/core/simple-grid/) |
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
| P3-15 | Indicator / ThemeIcon | 未实现 | 状态标记、装饰图标和可访问名称，设计跨宿主矢量/图标协议。 | [M:Indicator](https://mantine.dev/core/indicator/)、[M:ThemeIcon](https://mantine.dev/core/theme-icon/) |
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
| P4-01 | Menu / DropdownMenu | 已完成并记录证据 | 真实Menu/MenuItem/原生勾选单选、共享Collection/RTL/Typeahead、层级键盘与指针延迟/间隙、Tab退出和受控拒绝已实现；浏览器11组及SSR stream通过，原生Layer/输入明确拒绝并验证前缀水位。 | [M:Menu](https://mantine.dev/core/menu/)、[S:Dropdown Menu](https://ui.shadcn.com/docs/components/base/dropdown-menu) |
| P4-02 | ContextMenu | 未实现 | 右键/键盘菜单键、坐标锚点，触屏另设明确策略。 | [S:Context Menu](https://ui.shadcn.com/docs/components/base/context-menu) |
| P4-03 | Menubar | 未实现 | 横向根导航、嵌套菜单及焦点恢复，复用 Menu 模型。 | [M:Menubar](https://mantine.dev/core/menubar/)、[S:Menubar](https://ui.shadcn.com/docs/components/base/menubar) |
| P4-04 | NativeSelect | 未实现 | 扩展 Glory 原生选项/分组/选中值及 Change 协议，不能用文本 Input 冒充。 | [M:NativeSelect](https://mantine.dev/core/native-select/)、[S:Native Select](https://ui.shadcn.com/docs/components/base/native-select) |
| P4-05 | Select | 未实现 | Listbox/Option、值与活动项、空值、分组、disabled 和受控恢复。 | [M:Select](https://mantine.dev/core/select/)、[S:Select](https://ui.shadcn.com/docs/components/base/select) |
| P4-06 | Combobox | 未实现 | editable combobox、active-descendant、搜索与输入法、弹层和输入焦点协调。 | [M:Combobox](https://mantine.dev/core/combobox/)、[S:Combobox](https://ui.shadcn.com/docs/components/base/combobox) |
| P4-07 | Autocomplete | 未实现 | 允许自由输入，过滤/高亮/选择与输入值分离。 | [M:Autocomplete](https://mantine.dev/core/autocomplete/) |
| P4-08 | MultiSelect | 未实现 | 多选值、选中标签、全选/移除、键盘和大数据选项预算。 | [M:MultiSelect](https://mantine.dev/core/multi-select/) |
| P4-09 | TagsInput / PillsInput | 未实现 | 自由标签、重复策略、分隔/粘贴/删除，保留组合输入。 | [M:TagsInput](https://mantine.dev/core/tags-input/)、[M:PillsInput](https://mantine.dev/core/pills-input/) |
| P4-10 | Command / Spotlight | 未实现 | 搜索分组动作、快捷键入口、空态/异步结果及取消；命令动作由应用提供。 | [S:Command](https://ui.shadcn.com/docs/components/base/command)、[X:Spotlight](https://mantine.dev/x/spotlight/) |

### P5 · 浮层与反馈

补齐现有浮层差距，再增加提示、确认、通知、进度与集中管理。

依赖：P0、P2、P3、P4。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P5-01 | AlertDialog | 未实现 | 独立 alertdialog 角色、描述、确认/取消、初始安全焦点及关闭策略；复用 Dialog。 | [S:Alert Dialog](https://ui.shadcn.com/docs/components/base/alert-dialog) |
| P5-02 | Sheet / Drawer gestures | 未实现 | 复用 Drawer 四侧布局；补 drag-to-dismiss、snap points、滚动冲突和 reduced-motion。 | [S:Sheet](https://ui.shadcn.com/docs/components/base/sheet) |
| P5-03 | Tooltip | 已完成并记录证据 | 被动Tooltip角色与稳定描述、hover/focus、可取消Owner延迟及Escape已实现；不抢焦点/禁止交互后代，Web实际AX、dirty接管与classicstream通过，原生Layer和PointerWithin驱动明确拒绝。 | [M:Tooltip](https://mantine.dev/core/tooltip/)、[S:Tooltip](https://ui.shadcn.com/docs/components/base/tooltip) |
| P5-04 | HoverCard | 已完成并记录证据 | 交互HoverCard复用非模态Preserve焦点Layer、真实PointerWithin与FocusWithin联合区间、可取消开闭延迟、受控拒绝及稳定内容策略已实现；四主题Web/SSR及实际焦点/Tab通过，原生Layer明确拒绝。 | [M:HoverCard](https://mantine.dev/core/hover-card/)、[S:Hover Card](https://ui.shadcn.com/docs/components/base/hover-card) |
| P5-05 | Alert | 未实现 | status/alert 与严重级别，关闭按钮与可选播报策略。 | [M:Alert](https://mantine.dev/core/alert/)、[S:Alert](https://ui.shadcn.com/docs/components/base/alert) |
| P5-06 | Notification / Toast system | 未实现 | 有界队列、替换/去重、定时/暂停、动作、live region、Owner 清理；不持有全部历史。 | [M:Notification](https://mantine.dev/core/notification/)、[X:Notifications system](https://mantine.dev/x/notifications/)、[S:Toast](https://ui.shadcn.com/docs/components/base/toast) |
| P5-07 | Spinner / Loader | 未实现 | 加载名称、busy 状态和 reduced-motion；纯装饰时不重复播报。 | [M:Loader](https://mantine.dev/core/loader/)、[S:Spinner](https://ui.shadcn.com/docs/components/base/spinner) |
| P5-08 | Progress / Ring / SemiCircle | 未实现 | 共享 finite range/progressbar 值域与文本替代，线性和圆形视图。 | [M:Progress](https://mantine.dev/core/progress/)、[M:RingProgress](https://mantine.dev/core/ring-progress/)、[M:SemiCircleProgress](https://mantine.dev/core/semi-circle-progress/)、[S:Progress](https://ui.shadcn.com/docs/components/base/progress) |
| P5-09 | NavigationProgress | 未实现 | 由应用提交开始/进度/结束，复用 Progress；不劫持网络或路由，取消与 Owner 清理一致。 | [X:NavigationProgress](https://mantine.dev/x/nprogress/) |
| P5-10 | Skeleton | 未实现 | 占位尺寸、busy 隶属和装饰可访问性。 | [M:Skeleton](https://mantine.dev/core/skeleton/)、[S:Skeleton](https://ui.shadcn.com/docs/components/base/skeleton) |
| P5-11 | Overlay / LoadingOverlay | 未实现 | 装饰遮罩与实际阻止交互分别建模，区域 busy，不冒用全局 modal。 | [M:Overlay](https://mantine.dev/core/overlay/)、[M:LoadingOverlay](https://mantine.dev/core/loading-overlay/) |
| P5-12 | FloatingDialog | 未实现 | Mantine Dialog 对应非模态边角提示；以可访问的具名非模态浮层实现，区别于当前 modal Dialog。 | [M:Dialog](https://mantine.dev/core/dialog/) |
| P5-13 | ModalsManager | 未实现 | 应用内集中打开/更新/关闭与确认队列，复用 Layer，不创建第二套真值。 | [X:Modals manager](https://mantine.dev/x/modals/) |
| P5-14 | FloatingWindow | 未实现 | 窗口移动/缩放、标题栏、焦点层级，依赖有限拖动及 Bounds。 | [M:FloatingWindow](https://mantine.dev/core/floating-window/) |
| P5-15 | Tour | 未实现 | 步骤锚点、目标失效/滚动、聚焦和取消，不能扫描未挂载业务列表。 | [M:Tour](https://mantine.dev/core/tour/) |

### P6 · 导航与应用布局

覆盖页面导航、侧栏、响应式壳、滚动和可调整布局。

依赖：P2、P3、P4。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P6-01 | Breadcrumbs | 已完成并记录证据 | Breadcrumbs导航地标、当前位置、真实链接与有界中间折叠已实现，Rust/四主题CSR水合通过；真实四主题原生布局/Current/identity和隐藏卸载回归已通过。 | [M:Breadcrumbs](https://mantine.dev/core/breadcrumbs/)、[S:Breadcrumb](https://ui.shadcn.com/docs/components/base/breadcrumb) |
| P6-02 | Pagination | 已完成并记录证据 | Pagination已支持1-based u64当前页/已知与未知总数、边界/跳页/禁用，恒定35slot预算；Rust与四主题浏览器通过，原生套件待完整重跑。；真实四主题原生布局/Current/identity和隐藏卸载回归已通过。 | [M:Pagination](https://mantine.dev/core/pagination/)、[S:Pagination](https://ui.shadcn.com/docs/components/base/pagination) |
| P6-03 | NavLink / NavigationMenu | 部分基础 | NavLink真实链接与独立展开按钮、稳定panel/Owner政策、Current::Page已实现；浏览器9组及原生2项通过，隐藏卸载引擎修复已固定真实fork提交。NavigationMenu尚待实现。 | [M:NavLink](https://mantine.dev/core/nav-link/)、[S:Navigation Menu](https://ui.shadcn.com/docs/components/base/navigation-menu) |
| P6-04 | Sidebar | 已完成并记录证据 | 折叠Sidebar、嵌套导航、复用Modal Layer的移动Drawer及同一Owner在桌面/移动容器间移动已实现；应用选择路由/持久化，四主题浏览器焦点/受控拒绝与原生桌面布局/Layer拒绝已验收。 | [S:Sidebar](https://ui.shadcn.com/docs/components/base/sidebar) |
| P6-05 | AppShell | 已完成并记录证据 | 真实Banner/ContentInfo/Main与侧栏布局、局部ShellLayout尺寸/断点context、Resize响应式及skip link实际聚焦已实现；不绑定路由，同一导航Owner/控件身份跨布局保持。 | [M:AppShell](https://mantine.dev/core/app-shell/) |
| P6-06 | Stepper | 已完成并记录证据 | Stepper已实现Current::Step/list/状态说明、可选跳步、错误/完成、受控拒绝、KeepMounted/Unmount与实际焦点交接；FormState同步/可取消异步校验屏障显式集成，四主题CSR/水合及原生证据已通过。 | [M:Stepper](https://mantine.dev/core/stepper/) |
| P6-07 | ScrollArea / Scroller | 部分基础 | Glory 有滚动查询/动作；补统一视口/滚动条/可见性/键盘与嵌套滚动。 | [M:ScrollArea](https://mantine.dev/core/scroll-area/)、[M:Scroller](https://mantine.dev/core/scroller/)、[S:Scroll Area](https://ui.shadcn.com/docs/components/base/scroll-area) |
| P6-08 | Resizable / Splitter | 未实现 | 可聚焦 separator、键盘调整、指针捕获、最小/最大和布局守恒。 | [M:Splitter](https://mantine.dev/core/splitter/)、[S:Resizable](https://ui.shadcn.com/docs/components/base/resizable) |
| P6-09 | Affix | 未实现 | 视口/容器锚定、响应式偏移，补 typed fixed/sticky 的实际宿主保证。 | [M:Affix](https://mantine.dev/core/affix/) |
| P6-10 | Toolbar / ActionBar | 未实现 | 工具栏 roving focus、分组动作和选择态操作条。 | [M:Toolbar](https://mantine.dev/core/toolbar/)、[M:ActionBar](https://mantine.dev/core/action-bar/) |
| P6-11 | TableOfContents | 未实现 | 标题注册、当前节、滚动与嵌套结构，索引仅持有存活标题。 | [M:TableOfContents](https://mantine.dev/core/table-of-contents/) |
| P6-12 | OverflowList | 未实现 | 测量容纳项、溢出入口和焦点转移，避免全业务数据预排版。 | [M:OverflowList](https://mantine.dev/core/overflow-list/) |
| P6-13 | FloatingIndicator | 未实现 | 跟随活动项的测量/动画与方向变化，不改变语义焦点。 | [M:FloatingIndicator](https://mantine.dev/core/floating-indicator/) |
| P6-14 | Marquee / RollingNumber | 未实现 | 运动展示、停止/暂停、稳定文本替代和 reduced-motion。 | [M:Marquee](https://mantine.dev/core/marquee/)、[M:RollingNumber](https://mantine.dev/core/rolling-number/) |

### P7 · 数值、范围与颜色输入

先可靠的值域/步进/精度，再做拖动、多滑块和颜色二维交互。

依赖：P1、P2、P3。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P7-01 | NumberInput | 未实现 | spinbutton 语义、decimal 模型/步进/范围、空值/中间输入态、locale，不能只依赖 Purpose::Number。 | [M:NumberInput](https://mantine.dev/core/number-input/) |
| P7-02 | Slider | 未实现 | range/slider 角色、值域、方向键/Page/Home/End、pointer capture 和 Field。 | [M:Slider](https://mantine.dev/core/slider/)、[S:Slider](https://ui.shadcn.com/docs/components/base/slider) |
| P7-03 | RangeSlider | 未实现 | 多个具名滑块、顺序/最小间距、焦点与交换策略。 | [M:RangeSlider](https://mantine.dev/core/range-slider/) |
| P7-04 | AngleSlider | 未实现 | 角度环绕、二维指针到数值、可访问的键盘等价操作。 | [M:AngleSlider](https://mantine.dev/core/angle-slider/) |
| P7-05 | Rating | 未实现 | 离散/半星值、单选语义与只读展示。 | [M:Rating](https://mantine.dev/core/rating/) |
| P7-06 | ColorSwatch | 未实现 | 颜色文本替代、透明背景和可选选择状态。 | [M:ColorSwatch](https://mantine.dev/core/color-swatch/) |
| P7-07 | HueSlider / AlphaSlider | 未实现 | 共享 Slider，颜色域和 alpha 保持精度。 | [M:HueSlider](https://mantine.dev/core/hue-slider/)、[M:AlphaSlider](https://mantine.dev/core/alpha-slider/) |
| P7-08 | ColorPicker / ColorInput | 未实现 | 二维饱和/亮度、色彩空间转换、输入解析和键盘替代，复用浮层。 | [M:ColorPicker](https://mantine.dev/core/color-picker/)、[M:ColorInput](https://mantine.dev/core/color-input/) |
| P7-09 | PinInput / InputOTP | 未实现 | 分段显示、完整粘贴/退格、单次输入模型、自动填充与 IME；不把验证码写日志。 | [M:PinInput](https://mantine.dev/core/pin-input/)、[S:Input OTP](https://ui.shadcn.com/docs/components/base/input-otp) |
| P7-10 | MaskInput | 未实现 | 显示与原始值映射、选择/粘贴/删除/组合输入，不在每个按键后粗暴替换文本。 | [M:MaskInput](https://mantine.dev/core/mask-input/) |
| P7-11 | JsonInput | 未实现 | 文本草稿、解析错误、格式化与预算；解析器可选，不进入默认组件依赖。 | [M:JsonInput](https://mantine.dev/core/json-input/) |

### P8 · 表格与树形数据

静态语义表格先行，数据模型、选择排序、虚拟化和树导航分步加入。

依赖：P3、P4、P6。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P8-01 | Table | 部分基础 | Glory 已有 table/row/cell 角色；补表头关联、caption、列/行和 Sapid 组合 API。 | [M:Table](https://mantine.dev/core/table/)、[S:Table](https://ui.shadcn.com/docs/components/base/table) |
| P8-02 | DataTable | 未实现 | 排序/筛选/选择/列显隐/分页，服务端和本地模型分离，再接 Glory 窗口列表。 | [S:Data Table](https://ui.shadcn.com/docs/components/base/data-table) |
| P8-03 | Tree | 未实现 | tree/treeitem、层级/展开/选择、方向键与懒加载；FileTree 作为数据适配器。 | [M:Tree](https://mantine.dev/core/tree/)、[A:File Tree](https://ui.shadcn.com/docs/ai-elements/base/file-tree) |
| P8-04 | TreeSelect | 未实现 | 复用 Tree 和 Combobox，路径值与多选/半选策略。 | [M:TreeSelect](https://mantine.dev/core/tree-select/) |
| P8-05 | Cascader | 未实现 | 分列路径选择、懒加载、返回/展开导航。 | [M:Cascader](https://mantine.dev/core/cascader/) |

### P9 · 日期与时间

独立日期模型和国际化策略；日历网格先于弹出式日期输入。

依赖：P1、P4、P7。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P9-01 | Date model / DatesProvider | 未实现 | 选择 Rust 日期类型；分开 civil date/local time/instant/timezone，显式 locale/week-start 和解析策略。 | Sapid/Glory 共享能力 |
| P9-02 | Calendar / MiniCalendar | 未实现 | 日历网格、月/年跳转、禁用日、焦点日期与选中值，日历范围算法独立测试。 | [D:Calendar](https://mantine.dev/dates/calendar/)、[D:MiniCalendar](https://mantine.dev/dates/mini-calendar/)、[S:Calendar](https://ui.shadcn.com/docs/components/base/calendar) |
| P9-03 | DatePicker / DatePickerInput | 未实现 | 单日/多日/范围、弹层、Field 和键盘输入，复用 Calendar。 | [D:DatePicker](https://mantine.dev/dates/date-picker/)、[D:DatePickerInput](https://mantine.dev/dates/date-picker-input/)、[S:Date Picker](https://ui.shadcn.com/docs/components/base/date-picker) |
| P9-04 | DateInput | 未实现 | 允许中间文本状态、可自定义解析、非法/越界值诊断。 | [D:DateInput](https://mantine.dev/dates/date-input/) |
| P9-05 | MonthPicker / MonthPickerInput | 未实现 | 按月选择及范围，网格键盘导航。 | [D:MonthPicker](https://mantine.dev/dates/month-picker/)、[D:MonthPickerInput](https://mantine.dev/dates/month-picker-input/) |
| P9-06 | YearPicker / YearPickerInput | 未实现 | 年份/区间、十年视图和边界。 | [D:YearPicker](https://mantine.dev/dates/year-picker/)、[D:YearPickerInput](https://mantine.dev/dates/year-picker-input/) |
| P9-07 | TimeInput / TimeValue | 未实现 | 小时/分/秒、12/24h、空值和本地化展示。 | [D:TimeInput](https://mantine.dev/dates/time-input/)、[D:TimeValue](https://mantine.dev/dates/time-value/) |
| P9-08 | TimePicker / TimeGrid | 未实现 | 分段时间输入与可选时间网格、步长/禁用值。 | [D:TimePicker](https://mantine.dev/dates/time-picker/)、[D:TimeGrid](https://mantine.dev/dates/time-grid/) |
| P9-09 | DateTimePicker / InlineDateTimePicker | 未实现 | 日期时间组合和时区/夏令时歧义的显式策略，不默默变成 UTC。 | [D:DateTimePicker](https://mantine.dev/dates/date-time-picker/)、[D:InlineDateTimePicker](https://mantine.dev/dates/inline-date-time-picker/) |

### P10 · 文件、媒体、代码与编辑器

宿主能力显式授权；文件、剪贴板、富文本与高亮保持可选依赖。

依赖：P1、P4、P5、P6、P8。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P10-01 | FileInput / FileButton | 未实现 | 类型化文件引用、选择取消/多选/过滤；宿主返回非 ambient 路径句柄。 | [M:FileInput](https://mantine.dev/core/file-input/)、[M:FileButton](https://mantine.dev/core/file-button/) |
| P10-02 | Dropzone / Attachment | 未实现 | 文件拖入、容量/类型/数量预算、预览/取消与错误；上传网络由应用提供。 | [X:Dropzone](https://mantine.dev/x/dropzone/)、[S:Attachment](https://ui.shadcn.com/docs/components/base/attachment) |
| P10-03 | CopyButton | 未实现 | 显式剪贴板 action、权限/拒绝、成功短暂反馈和计时取消。 | [M:CopyButton](https://mantine.dev/core/copy-button/) |
| P10-04 | Carousel | 未实现 | 页/slide 状态、按钮/键盘/手势、自动播放暂停和窗口化。 | [X:Carousel](https://mantine.dev/x/carousel/)、[S:Carousel](https://ui.shadcn.com/docs/components/base/carousel) |
| P10-05 | Lightbox | 未实现 | 图片预览、切换/缩放、替代文本和资源预算，复用 modal 层。 | [X:Lightbox](https://mantine.dev/x/lightbox/) |
| P10-06 | CodeBlock / CodeHighlight | 未实现 | 可选语言 tokenizer、行号/复制/选区/大文件预算；HTML 和 native 使用同一文本模型。 | [H:CodeHighlight](https://mantine.dev/code-highlight/code-highlight/)、[S:Code Block](https://ui.shadcn.com/docs/components/base/code-block) |
| P10-07 | JsonViewer | 未实现 | 可折叠结构、搜索、复制和深度/节点预算，复用 Tree。 | [H:JsonViewer](https://mantine.dev/code-highlight/json-viewer/) |
| P10-08 | RichTextEditor | 未实现 | Glory 需 rich editing/document/selection/IME/undo 协议；单独架构和安全内容模型，不能拿 Textarea 冒充。 | [X:Rich text editor](https://mantine.dev/x/tiptap/) |

### P11 · 图表

可选图表模块，先可访问的数据替代表示和绘图协议，再逐图形扩展。

依赖：P3、P5、P7。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P11-01 | Chart / ChartTheme / Legend / Tooltip | 未实现 | 设计有限可选绘图协议、尺度/轴/可访问表格、交互与导出；不要求 Sapid 用户安装 JS 图表库。 | [S:Chart](https://ui.shadcn.com/docs/components/base/chart) |
| P11-02 | AreaChart / LineChart / BarChart / CompositeChart | 未实现 | 先基础 Cartesian 图，覆盖空值、堆叠、刻度、刷选和数据预算。 | [C:AreaChart](https://mantine.dev/charts/area-chart/)、[C:LineChart](https://mantine.dev/charts/line-chart/)、[C:BarChart](https://mantine.dev/charts/bar-chart/)、[C:CompositeChart](https://mantine.dev/charts/composite-chart/) |
| P11-03 | PieChart / DonutChart / RadialBarChart | 未实现 | 角度/占比/零值和键盘数据访问。 | [C:PieChart](https://mantine.dev/charts/pie-chart/)、[C:DonutChart](https://mantine.dev/charts/donut-chart/)、[C:RadialBarChart](https://mantine.dev/charts/radial-bar-chart/) |
| P11-04 | ScatterChart / BubbleChart / CandlestickChart | 未实现 | 二维/尺寸尺度、金融 OHLC、抽样和逐点访问预算。 | [C:ScatterChart](https://mantine.dev/charts/scatter-chart/)、[C:BubbleChart](https://mantine.dev/charts/bubble-chart/)、[C:CandlestickChart](https://mantine.dev/charts/candlestick-chart/) |
| P11-05 | RadarChart / FunnelChart / Sparkline | 未实现 | 各自几何和文本替代，小图也保留数值说明。 | [C:RadarChart](https://mantine.dev/charts/radar-chart/)、[C:FunnelChart](https://mantine.dev/charts/funnel-chart/)、[C:Sparkline](https://mantine.dev/charts/sparkline/) |
| P11-06 | Heatmap / MatrixChart / WaffleChart | 未实现 | 离散矩阵与颜色图例，颜色之外仍可辨识。 | [C:Heatmap](https://mantine.dev/charts/heatmap/)、[C:MatrixChart](https://mantine.dev/charts/matrix-chart/)、[C:WaffleChart](https://mantine.dev/charts/waffle-chart/) |
| P11-07 | BarsList / BulletChart / GaugeChart | 未实现 | 目标值、区间、刻度与进度范围语义。 | [C:BarsList](https://mantine.dev/charts/bars-list/)、[C:BulletChart](https://mantine.dev/charts/bullet-chart/)、[C:GaugeChart](https://mantine.dev/charts/gauge-chart/) |
| P11-08 | Treemap / SunburstChart / SankeyChart | 未实现 | 层级/流图布局、稳定 key、导航和节点/边预算。 | [C:Treemap](https://mantine.dev/charts/treemap/)、[C:SunburstChart](https://mantine.dev/charts/sunburst-chart/)、[C:SankeyChart](https://mantine.dev/charts/sankey-chart/) |

### P12 · 日程

先事件/时区/重复模型和只读视图，再资源视图与拖动编辑。

依赖：P8、P9、P10。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P12-01 | Schedule / Event model / Recurrence | 未实现 | 事件区间、时区、重复/例外、冲突和请求窗口，不预展开无限重复。 | [T:Schedule](https://mantine.dev/schedule/schedule/)、[T:Recurring events](https://mantine.dev/schedule/recurring-events/)、[T:Events data](https://mantine.dev/schedule/events-data/) |
| P12-02 | DayView / WeekView | 未实现 | 时间轴/重叠布局与键盘时段导航。 | [T:DayView](https://mantine.dev/schedule/day-view/)、[T:WeekView](https://mantine.dev/schedule/week-view/) |
| P12-03 | MonthView / YearView / MobileMonthView | 未实现 | 日期汇总、溢出、窄屏呈现与可访问选择。 | [T:MonthView](https://mantine.dev/schedule/month-view/)、[T:YearView](https://mantine.dev/schedule/year-view/)、[T:MobileMonthView](https://mantine.dev/schedule/mobile-month-view/) |
| P12-04 | AgendaView | 未实现 | 按日/区间的只读事件列表及窗口数据。 | [T:AgendaView](https://mantine.dev/schedule/agenda-view/) |
| P12-05 | Resource schedule views | 未实现 | 资源列/行、二维窗口、事件定位、拖动编辑及权限校验由应用决定。 | [T:ResourcesDayView](https://mantine.dev/schedule/resources-day-view/)、[T:ResourcesWeekView](https://mantine.dev/schedule/resources-week-view/)、[T:ResourcesMonthView](https://mantine.dev/schedule/resources-month-view/)、[T:ResourcesSchedule](https://mantine.dev/schedule/resources-schedule/) |

### P13 · 对话及 AI 应用展示

以传入的数据和动作构造展示组件；不绑定模型服务、网络、工具执行或隐藏推理数据。

依赖：P4、P5、P6、P8、P10。

| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |
|---|---|---|---|---|
| P13-01 | Message / Bubble / Marker | 未实现 | 消息内容、作者/时间/状态、分组、引用标记；只显示调用方提供的数据。 | [S:Message](https://ui.shadcn.com/docs/components/base/message)、[S:Bubble](https://ui.shadcn.com/docs/components/base/bubble)、[S:Marker](https://ui.shadcn.com/docs/components/base/marker)、[A:Message](https://ui.shadcn.com/docs/ai-elements/base/message) |
| P13-02 | MessageScroller / Chat | 未实现 | 消息窗口化、底部跟随/阅读锚点、未读提示、prepend 和内容高度变化。 | [S:Message Scroller](https://ui.shadcn.com/docs/components/base/message-scroller)、[A:Chat](https://ui.shadcn.com/docs/ai-elements/base/chat) |
| P13-03 | PromptInput | 未实现 | 多行输入、附件/发送/取消/建议与 IME，复用表单和文件协议。 | [A:Prompt Input](https://ui.shadcn.com/docs/ai-elements/base/prompt-input) |
| P13-04 | Questionnaire | 未实现 | 从 typed schema 组合表单、步骤和答复校验；不绑定 React 表单库。 | [S:Questionnaire](https://ui.shadcn.com/docs/components/base/questionnaire) |
| P13-05 | ModelSelector | 未实现 | 分组模型数据、搜索和选择；不内置过时的服务商/价格清单。 | [A:Model Selector](https://ui.shadcn.com/docs/ai-elements/base/model-selector) |
| P13-06 | Suggestion | 未实现 | 建议动作列表、键盘选择和来源传入。 | [A:Suggestion](https://ui.shadcn.com/docs/ai-elements/base/suggestion) |
| P13-07 | Confirmation / Checkpoint | 未实现 | 确认状态与检查点展示，实际授权/回退动作由应用执行。 | [A:Confirmation](https://ui.shadcn.com/docs/ai-elements/base/confirmation)、[A:Checkpoint](https://ui.shadcn.com/docs/ai-elements/base/checkpoint) |
| P13-08 | Plan / Task / Queue | 未实现 | 进度、任务状态与队列展示，可操作项使用应用回调。 | [A:Plan](https://ui.shadcn.com/docs/ai-elements/base/plan)、[A:Task](https://ui.shadcn.com/docs/ai-elements/base/task)、[A:Queue](https://ui.shadcn.com/docs/ai-elements/base/queue) |
| P13-09 | Reasoning / ChainOfThought | 未实现 | 折叠展示应用已提供、允许公开的说明或摘要，不生成或索取隐藏内部推理。 | [A:Reasoning](https://ui.shadcn.com/docs/ai-elements/base/reasoning)、[A:Chain of Thought](https://ui.shadcn.com/docs/ai-elements/base/chain-of-thought) |
| P13-10 | Context / Sources / InlineCitation | 未实现 | 上下文用量、来源列表和文内引用，链接/预览与可信度由数据模型明确。 | [A:Context](https://ui.shadcn.com/docs/ai-elements/base/context)、[A:Sources](https://ui.shadcn.com/docs/ai-elements/base/sources)、[A:Inline Citation](https://ui.shadcn.com/docs/ai-elements/base/inline-citation) |
| P13-11 | Trace / StackTrace / Subagent / Tool | 未实现 | 调用记录、层级状态、输入输出和耗时展示；截断/脱敏/预算，绝不隐式执行工具。 | [S:Trace](https://ui.shadcn.com/docs/components/base/trace)、[A:Stack Trace](https://ui.shadcn.com/docs/ai-elements/base/stack-trace)、[A:Subagent](https://ui.shadcn.com/docs/ai-elements/base/subagent)、[A:Tool](https://ui.shadcn.com/docs/ai-elements/base/tool) |
| P13-12 | Terminal / TestResults | 未实现 | 只读输出、测试状态与大日志窗口化；可选 ANSI 解析不赋予命令执行能力。 | [A:Terminal](https://ui.shadcn.com/docs/ai-elements/base/terminal)、[A:Test Results](https://ui.shadcn.com/docs/ai-elements/base/test-results) |

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

共 268 个来源条目，归并为 149 项工作：8 项已有基线、9 项部分基础、87 项未实现、45 项按后续完成标准关闭。日程目录包含事件数据与重复规则两项模型文档，基础设施也单独计入，不能把总数解释为缺失组件数量。

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
