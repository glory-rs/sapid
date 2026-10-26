# Mantine 全组件 API 与功能差异核实

核实日期：2026-10-25；官方文档版本：9.7.3。
逐项覆盖 **181 个 Mantine 条目**：121 Core、15 Dates、22 Charts、13 Schedule、8 官方扩展、2 Code。
官方来源：[Core 目录](https://mantine.dev/core/package/)、[官方扩展](https://mantine.dev/x/extensions/)、
[完整文档（包含功能示例和 Props 表）](https://mantine.dev/llms-full.txt)。

## 结论与核实边界

旧 inventory 的 done 表示有限功能批次已交付，**不表示 Mantine API 等价**。本报告逐项比对官方文档、
本地公开 Rust 入口、实现和能力矩阵；没有把文件存在、同名方法、近似拼写或 SSR HTML 当成功能验收。
Props 表包括同页子部件的公开属性；继承的原生 HTML 属性和所有 React 类型重载不一定在表内。
功能表是经实现核实的差异与设计决策，附录是完整文档表面的可追溯索引；未映射属性是核实候选，不能直接解释为缺失。
快照记录 4,659 个按文档条目计数的 Props 字段与其类型；不同条目的同名字段重复计数，不是同等数量的缺失功能。
历史宿主证据沿用 capability-matrix，新增功能的当前运行证据单列，未逐个重验所有上游功能组合。

Rust 的 Widget、Cage/Bond/Lotus、Owner 和有限宿主协议保持不变；不移植 React hooks、任意 DOM/ARIA 注入、
polymorphic component、任意 Recharts/Tiptap/Embla 插件或全局 provider。通用尺寸、颜色、圆角等可通过 typed Style/Theme 组合，
不因缺少同名便捷 prop 就重复建立状态。原生支持、浏览器协议和实际 OS/读屏验收分别记账。

部分 theme/size/variant、声明预算与组合策略属于挂载期配置，并非所有 Mantine prop 都可直接传 Lotus。
响应式值/禁用/只读等按各 builder 明确暴露的绑定工作；Style 组合不等于已提供上游同名动态 API，也不以重建节点冒充原位更新。

## 按报告执行的批次

| 批次 | 优先级 | 要补齐的行为 | 归属与状态 |
| --- | --- | --- | --- |
| A1 | P0 | MultiSelect max_values / hide_picked_options / clearable，含受控拒绝、IME、空组与游标恢复 | 已实现并验收：29 项选择控件测试、浏览器四主题 × 两尺寸 × CSR/SSR；受控拒绝/IME/空组/游标/焦点均有证据 |
| A2 | P0 | 真实 input/textarea placeholder：Glory 共享语义/HTML/SSR/stream + Sapid 输入族 | 已实现并验收：render/HTML 差分/SSR stream、9 类 Sapid 输入；Blitz 明确拒绝并保留成功前缀，未声称原生绘制支持 |
| A3 | P1 | Button Light / Outline / Subtle 外观，保留 loading/disabled/自定义 Style | 已实现并验收：新增三种外观；四主题 loading/disabled/显式样式与真实浏览器恢复均通过 |
| A4 | P1 | dropdown 配置、option slot、lineClamp、slider marks、分段 progress、日期 presets、日程统一编辑 | 待后续有限设计；未实现，不承诺整套上游等价 |

## 逐组件核对

| Mantine / 官方文档 | Sapid 现有对应 | 经核实的差异与借鉴决策 | 优先级 |
| --- | --- | --- | --- |
| [AppShell](https://mantine.dev/core/app-shell/) | AppShell：header/sidebar/footer、breakpoint、drawer/rail | 不是所有 Mantine aside/section/offset/hiddenFrom 组合；需补区域配置与独立滚动证据，不能 CSS 编译通过即视为所有布局成立。 | P2 |
| [AspectRatio](https://mantine.dev/core/aspect-ratio/) | AspectRatio：有限宽高比、嵌套布局 | polymorphic 根元素不移植；媒体裁切/替换内容由 Image fit 负责，不把 ratio 当全套图像组件。 | 边界 |
| [Center](https://mantine.dev/core/center/) | Center/Container/Space：居中、fluid/max_width、响应式阈值 | Mantine 命名容器尺寸和 strategy 不是逐项兼容；已能 typed Style 组合，优先补文档示例而非同义参数。 | P2 |
| [Container](https://mantine.dev/core/container/) | Center/Container/Space：居中、fluid/max_width、响应式阈值 | Mantine 命名容器尺寸和 strategy 不是逐项兼容；已能 typed Style 组合，优先补文档示例而非同义参数。 | P2 |
| [Flex](https://mantine.dev/core/flex/) | Box/Flex/Stack/Group：布局、gap、对齐、换行、宽高 | Group grow/preventGrowOverflow 的便捷策略尚缺；响应式值通过 ResponsiveRoot 显式组合。Box 的任意 component/ref API 不移植。 | P2 |
| [Grid](https://mantine.dev/core/grid/) | Grid/SimpleGrid：有限轨道、span/offset、响应式测量 | 与 Mantine 的 12 列/自动填充/容器查询模型不同；auto-fit/minColWidth 等策略需新增有限布局算法，不能仅重命名 columns。 | P2 |
| [Group](https://mantine.dev/core/group/) | Box/Flex/Stack/Group：布局、gap、对齐、换行、宽高 | Group grow/preventGrowOverflow 的便捷策略尚缺；响应式值通过 ResponsiveRoot 显式组合。Box 的任意 component/ref API 不移植。 | P2 |
| [SimpleGrid](https://mantine.dev/core/simple-grid/) | Grid/SimpleGrid：有限轨道、span/offset、响应式测量 | 与 Mantine 的 12 列/自动填充/容器查询模型不同；auto-fit/minColWidth 等策略需新增有限布局算法，不能仅重命名 columns。 | P2 |
| [Space](https://mantine.dev/core/space/) | Center/Container/Space：居中、fluid/max_width、响应式阈值 | Mantine 命名容器尺寸和 strategy 不是逐项兼容；已能 typed Style 组合，优先补文档示例而非同义参数。 | P2 |
| [Splitter](https://mantine.dev/core/splitter/) | Splitter：两面板、范围/step、commit、引用 | 不支持任意多 pane 树、collapsed/expand API、复杂 snap；需有界 panel model 与共享 drag/keyboard 协议后再增加。 | P2 |
| [Stack](https://mantine.dev/core/stack/) | Box/Flex/Stack/Group：布局、gap、对齐、换行、宽高 | Group grow/preventGrowOverflow 的便捷策略尚缺；响应式值通过 ResponsiveRoot 显式组合。Box 的任意 component/ref API 不移植。 | P2 |
| [AlphaSlider](https://mantine.dev/core/alpha-slider/) | HueSlider/AlphaSlider：有限颜色范围、键盘/commit | 缺 onScrubStart/End 便捷回调和任意 root props；可在通用 drag 会话语义上增加，不能以值改变猜测手势边界。 | P2 |
| [AngleSlider](https://mantine.dev/core/angle-slider/) | AngleSlider：圆周映射、键盘、有限值 | 缺 marks/自定义 label/自动 snap；保持角度域和端点策略显式，复用共享 Slider 的能力而非复制输入驱动。 | P2 |
| [Checkbox](https://mantine.dev/core/checkbox/) | Checkbox：原生 Check、受控值、Field；MixedCheckbox 独立 | indeterminate 已由 MixedCheckbox 表达；自定义 icon/labelPosition/多色变体未形成直接 API，先完善视觉插槽，避免改变原生 Check。 | P2 |
| [Chip](https://mantine.dev/core/chip/) | Chip/ChipGroup：单/多选、受控值、真实输入、Field | 缺 checked icon、icon position 和完整 variant；整组的受控默认值/禁用已有，不应重复引入另一套状态。 | P2 |
| [ColorInput](https://mantine.dev/core/color-input/) | ColorPicker/ColorInput：RGBA/hex、色面拖动、受控草稿/Field | 缺 hsl/rgb 字符串格式、swatchesOnly、多组色板和 eyedropper；先保持无损模型，再扩展显式格式，不调用隐式浏览器服务。 | P1 |
| [ColorPicker](https://mantine.dev/core/color-picker/) | ColorPicker/ColorInput：RGBA/hex、色面拖动、受控草稿/Field | 缺 hsl/rgb 字符串格式、swatchesOnly、多组色板和 eyedropper；先保持无损模型，再扩展显式格式，不调用隐式浏览器服务。 | P1 |
| [Fieldset](https://mantine.dev/core/fieldset/) | Fieldset/Legend：原生分组、禁用传播、主题尺寸 | variant/legendPosition 等展示配置可用 Style，但需补充带 legend 的视觉示例；不要把禁用透明度当实际禁用。 | P2 |
| [FileInput](https://mantine.dev/core/file-input/) | FileInput/FileButton：共享 File 协议、Picker、控制器/结果 | 缺目录、多种 value renderer 等完整 Mantine props 组合需依共享配置判断；native picker facet 仍拒绝，不能把 Web 文件测试当本地OS支持。 | P2 |
| [HueSlider](https://mantine.dev/core/hue-slider/) | HueSlider/AlphaSlider：有限颜色范围、键盘/commit | 缺 onScrubStart/End 便捷回调和任意 root props；可在通用 drag 会话语义上增加，不能以值改变猜测手势边界。 | P2 |
| [Input](https://mantine.dev/core/input/) | Field/Label/Description/Error：真实关联与 required/invalid | success 状态、说明/错误/控件顺序、loading/section 宽度尚无统一 API；InputGroup 已可组合左右内容。placeholder 属共享缺口。 | P1 |
| [JsonInput](https://mantine.dev/core/json-input/) | JsonInput：有界解析/format、错误、草稿/只读、真实 placeholder | 本轮补 placeholder；autosize/formatOnBlur 等便捷行为仍需定义组合和失焦次序。 | P2 |
| [MaskInput](https://mantine.dev/core/mask-input/) | MaskInput：有限 mask、raw/display 索引、paste/组合保护 | 不兼容任意第三方 mask 对象、动态 mask 引擎；保留 typed finite schema，并补 display 格式示例。 | 边界 |
| [NativeSelect](https://mantine.dev/core/native-select/) | NativeSelect：真实 select、分组、多选、成功值 | NativeSelect 的左右 sections 需 InputGroup 组合；Blitz select facet 拒绝。不能将浏览器 select 打开状态作为可控 popup。 | P2 |
| [NumberInput](https://mantine.dev/core/number-input/) | NumberInput：Decimal、locale、clamp、step、草稿与拒绝 | 缺完整千分位/前后缀/字符串数值/hold interval API；不能用 f64 替换当前精确小数，也不导入任意 formatter。 | P2 |
| [PasswordInput](https://mantine.dev/core/password-input/) | PasswordInput：显隐/组合保护、Field、真实 placeholder | 本轮补 placeholder；显隐自定义图标/部位仍待补。不能重建输入实现；native 密码值投影与 hint 支持是不同承诺。 | P2 |
| [PinInput](https://mantine.dev/core/pin-input/) | PinInput：长度/字母表、mask、autofill、complete、原子编辑 | 缺一组可单独引用的 input 单元等 UI 细项；现有是共享单 Input 投影，不假装独立多输入。onComplete 已有不能误报。 | P2 |
| [Radio](https://mantine.dev/core/radio/) | RadioGroup：键盘、方向、单选值、required/表单代理 | 缺 Radio.Card/Indicator 展示子部件、图标与标签位置；可以现有组和 slot 组合，需单独验收点击区域与焦点。 | P2 |
| [RangeSlider](https://mantine.dev/core/range-slider/) | RangeSlider：双 thumb、collision 策略和受控值 | 缺 minRange/maxRange 与 marks-only 选择，thumb 顺序和交叉必须有明确定义；单 Slider 两次发布不可冒充原子范围编辑。 | P1 |
| [Rating](https://mantine.dev/core/rating/) | Rating：精度、display_only、clearable、键盘/Field | 缺任意 empty/full symbol renderer、hover preview 回调；图标表现与真实 radio 选择分离，避免 hover 修改受控值。 | P2 |
| [SegmentedControl](https://mantine.dev/core/segmented-control/) | SegmentedControl：真实 radios、方向、受控值 | 缺独立指示器动画、fullWidth/equal-width 的成套配置；不得以动画代替选中语义，布局可用 Style。 | P2 |
| [Slider](https://mantine.dev/core/slider/) | Slider：有限 range、方向/步长、commit、值文本 | 缺 marks、snap 到 marks、浮动 label、自定义 thumb；先扩展共享绘制/标记模型，不在 widget 注入 DOM tooltip。 | P1 |
| [Switch](https://mantine.dev/core/switch/) | Switch：真实 CheckMode::Switch、受控值、Field | 缺 on/offLabel、thumbIcon、labelPosition、颜色/尺寸的完整视觉组合；保留真实 checkbox 键盘行为。 | P2 |
| [Textarea](https://mantine.dev/core/textarea/) | Textarea：固定行数、min/max rows、浏览器 autosize、真实 placeholder | 本轮补多行 placeholder；autosize 与 placeholder 在 native 各有拒绝边界，固定无 hint 行数仍支持。resize 不是 intrinsic sizing。 | P2 |
| [TextInput](https://mantine.dev/core/text-input/) | TextField：value/commit、Purpose、Field、readonly/invalid、真实 placeholder | 本轮补有界响应式 placeholder，独立于名称/值/版本；native 对该可选 hint 明确拒绝。任意原生属性转发不移植。 | P2 |
| [Autocomplete](https://mantine.dev/core/autocomplete/) | Autocomplete：自由值/建议 key 分离、commit、真实 placeholder | 本轮补 placeholder；limit、autoSelectOnBlur/renderOption 仍缺。模糊搜索应为显式纯 filter，不引入 JS 引擎。 | P1 |
| [Cascader](https://mantine.dev/core/cascader/) | Cascader：完整路径、分列、懒加载、clearable | 缺 changeOnSelect/showSearch/multiple 等组合；并非所有 Select props 可直接转发，需保持路径/叶选择策略与原子受控提议。 | P2 |
| [Combobox](https://mantine.dev/core/combobox/) | Combobox：Input/listbox、过滤/键盘/IME、真实 placeholder | 本轮补 placeholder；分部、自定义 filter、结果 limit/远程 loading 仍是缺口。有界声明不等于虚拟化万项。 | P1 |
| [ComboboxPopover](https://mantine.dev/core/combobox-popover/) | Popover：锚点定位、碰撞处理、焦点/关闭策略 | 缺 width=target、箭头、统一下拉尺寸/样式参数与组合子部件 API；定位应继续由 Glory 宿主执行。 | P1 |
| [MultiSelect](https://mantine.dev/core/multi-select/) | MultiSelect：独立搜索/受控值、max_values、隐藏已选、内建清空、placeholder | 本轮补 maxValues/hidePickedOptions/clearable/hint；renderOption/renderPill、拖拽重排、loading、统一下拉配置仍未提供，不称完整等价。 | P1 |
| [Pill](https://mantine.dev/core/pill/) | Badge/Pill：三种 badge variant、受控移除、紧凑标签 | 上一轮修复尺寸与 close action；Badge 的 dot/左右 section/gradient、Pill 的拖拽排序仍未完整提供。先补展示 slot，再考虑有界排序。 | P2 |
| [PillsInput](https://mantine.dev/core/pills-input/) | TagsInput/PillsInput：原子 state、重复/分隔/粘贴/移除、max_tags、placeholder | 本轮补 placeholder；clearable、建议、onBlur add 和 reorder 仍缺。max_tags 已有，不能误报 maxTags 缺失。 | P1 |
| [Select](https://mantine.dev/core/select/) | Select：受控选择、默认值、clearable、popup 和 Field | 与 Mantine searchable Select 不同：搜索由 Combobox 承担；应明确组合而非重名别名。缺 option renderer/虚拟结果/统一 dropdown config。 | P1 |
| [TagsInput](https://mantine.dev/core/tags-input/) | TagsInput/PillsInput：原子 state、重复/分隔/粘贴/移除、max_tags、placeholder | 本轮补 placeholder；clearable、建议、onBlur add 和 reorder 仍缺。max_tags 已有，不能误报 maxTags 缺失。 | P1 |
| [TreeSelect](https://mantine.dev/core/tree-select/) | TreeSelect：路径身份、multiple/半选/叶策略、受控 | 缺纯文本搜索、hidePicked、完整 renderValue/下拉配置；先共用 Combobox 搜索模型，再扩展树过滤，native 仍拒绝组合 facet。 | P1 |
| [ActionIcon](https://mantine.dev/core/action-icon/) | ActionIcon/CloseButton：五种 variant、尺寸、自定义 icon、禁用 | 上一轮已补主要外观与 close icon；loading/custom-loader 仍缺。任意 component、gradient 和全局 variant registry 不移植。 | P2 |
| [Button](https://mantine.dev/core/button/) | Button：六种外观、受控禁用/loading、图文插槽和按钮组 | 本轮补 Light/Outline/Subtle；gradient/任意 variant resolver 不移植。fullWidth/justify 可用 Style，loading 自定义显示仍可后续完善。 | P2 |
| [CloseButton](https://mantine.dev/core/close-button/) | ActionIcon/CloseButton：五种 variant、尺寸、自定义 icon、禁用 | 上一轮已补主要外观与 close icon；loading/custom-loader 仍缺。任意 component、gradient 和全局 variant registry 不移植。 | P2 |
| [CopyButton](https://mantine.dev/core/copy-button/) | CopyButton：真实 Clipboard action、结果/feedback/labels | 不把未授权环境/SSR当复制成功；任意 render-prop 可由 Widget 组合替代，多个复制控件的状态仍分开。 | 边界 |
| [FileButton](https://mantine.dev/core/file-button/) | FileInput/FileButton：共享 File 协议、Picker、控制器/结果 | 缺目录、多种 value renderer 等完整 Mantine props 组合需依共享配置判断；native picker facet 仍拒绝，不能把 Web 文件测试当本地OS支持。 | P2 |
| [Toggle](https://mantine.dev/core/toggle/) | Toggle/ToggleGroup：单/多选、pressed、方向、键盘 | 缺完整外观/图标位置配置。Mantine Toggle 与 checkbox 语义不等同；使用真正 pressed Button，不用 Check 伪装。 | P2 |
| [UnstyledButton](https://mantine.dev/core/unstyled-button/) | Button：六种外观、受控禁用/loading、图文插槽和按钮组 | 本轮补 Light/Outline/Subtle；gradient/任意 variant resolver 不移植。fullWidth/justify 可用 Style，loading 自定义显示仍可后续完善。 | P2 |
| [Anchor](https://mantine.dev/core/anchor/) | Anchor：真实 href、current、宿主导航和目标上下文 | 缺 underline 策略便捷 API；Style/CSS 可组合。new context 原生明确拒绝，非路由/下载协议不隐式加入。 | P2 |
| [Breadcrumbs](https://mantine.dev/core/breadcrumbs/) | Breadcrumbs：真实 links/current、有限条目 | 缺自定义 separator/图标 slot 等便捷 API；自动折叠应复用 OverflowList，不把静态省略符当可导航隐藏内容。 | P2 |
| [Burger](https://mantine.dev/core/burger/) | Burger：展开、名称、controls、有限动画和时长 | 缺自定义线粗/颜色便捷参数；typed Style 已可覆盖颜色。真实展开与动画状态已独立，不需要复制 DOM click 驱动。 | P2 |
| [NavLink](https://mantine.dev/core/nav-link/) | NavLink：href/current、展开 children；NavigationMenu 独立 | 缺 right/left section、autoContrast、description 的统一便捷参数；禁止通过 label 或 role 变更重置节点输入状态。 | P2 |
| [Pagination](https://mantine.dev/core/pagination/) | Pagination：边界/兄弟页、受控值、未知下一页 | 缺完整 control 图标/edge slots 与响应式策略；现有数据窗口不等价远程数据缓存，分页请求应由应用管理。 | P2 |
| [Stepper](https://mantine.dev/core/stepper/) | Stepper：active/complete、可跳步、可选步骤、内容策略 | 缺 loading/自定义 completed/progress icon 和 iconPosition；控件选择与业务校验分离，不自动把 step click 当验证成功。 | P2 |
| [TableOfContents](https://mantine.dev/core/table-of-contents/) | TableOfContents：显式 heading 注册、current/reveal/offset | 不扫描任意 DOM selector；auto headings/文字自动取样与样式 slots 有差异。显式上下文更适合多宿主与虚拟列表。 | 边界 |
| [Tabs](https://mantine.dev/core/tabs/) | Tabs：受控/默认值、方向、自动/手动激活、动态页和保留策略 | 缺 inverted/variant/颜色/独立 tab 图标部位等便捷配置；allowTabDeactivation/reorder 需先定义空选中与键盘行为。 | P2 |
| [Tree](https://mantine.dev/core/tree/) | Tree：有界节点、懒加载、check/多选、键盘、render_label | 不是任意 DOM renderer/tree hook；拖拽重排/批量展开 API 待明确，绝不扫描底层大列表来导出 AX 树。 | P1 |
| [Alert](https://mantine.dev/core/alert/) | Alert：severity、announcement、可关闭与内容 slot | 缺多种 variant/icon/close 部位便捷 API；announcement 与静态 alert 需区分，不让每条说明自动打断读屏。 | P2 |
| [EmptyState](https://mantine.dev/core/empty-state/) | EmptyState：title/description/illustration/actions | 缺尺寸预设/多种 media 布局；插槽足以组合，优先改示例与默认留白，避免引入业务空状态。 | P2 |
| [Loader](https://mantine.dev/core/loader/) | Loader：有限 Graphic、尺寸/线宽、可停动画 | Mantine 多种 loader 类型/自定义 registry 未提供；自定义 Graphic 可组合，仍需尊重 reduced motion 和静态 fallback。 | P2 |
| [Notification](https://mantine.dev/core/notification/) | Notification/系统：有界队列、更新/关闭、计时与 hover 暂停 | 缺分区域 placement、任意 transition 和完整 loader/icon slots；队列有界与 app-local context 优于无界全局管理器。 | P2 |
| [Progress](https://mantine.dev/core/progress/) | Progress/Ring/SemiCircle：真实 range/progress 语义与几何 | 缺分段 Progress.Section/多彩 ring sections 和完整标签 slot；单 value 不等于多段，值得补充有界 segments 模型。 | P1 |
| [RingProgress](https://mantine.dev/core/ring-progress/) | Progress/Ring/SemiCircle：真实 range/progress 语义与几何 | 缺分段 Progress.Section/多彩 ring sections 和完整标签 slot；单 value 不等于多段，值得补充有界 segments 模型。 | P1 |
| [SemiCircleProgress](https://mantine.dev/core/semi-circle-progress/) | Progress/Ring/SemiCircle：真实 range/progress 语义与几何 | 缺分段 Progress.Section/多彩 ring sections 和完整标签 slot；单 value 不等于多段，值得补充有界 segments 模型。 | P1 |
| [Skeleton](https://mantine.dev/core/skeleton/) | Skeleton：loading、圆/矩形、有限宽高 | 缺默认 shimmer/animate 配置与内容保留的完整组合；应优先复用 MotionPolicy，而非插入独立无限计时器。 | P2 |
| [ActionBar](https://mantine.dev/core/action-bar/) | Toolbar/ActionBar：有界组、roving、selected_count/clear | Mantine ActionBar placement/transition/portal 配置不同；不得把横排 Toolbar 当完整底部浮条。可用 shared Layer 进一步补齐。 | P2 |
| [Affix](https://mantine.dev/core/affix/) | Affix：显式 viewport/container、offset、sticky | Native typed Sticky 仍拒绝；任意 zIndex/portalTarget 不自动移植。遵守已测试的视口定位与原生不同能力。 | 边界 |
| [Dialog](https://mantine.dev/core/dialog/) | FloatingDialog 对应 Mantine Dialog：非模态提示 | 非 Modal：不隐式焦点圈闭/外点关闭。placement/close/transition 细项仍不同，使用共享 Layer 不另造驱动。 | P2 |
| [Drawer](https://mantine.dev/core/drawer/) | Drawer：侧边 Layer、内容策略；Sheet 提供手势和 snap | 缺位置相关的自动尺寸、完整 close/overlay 插槽和过渡配置；snap 已有有限语义，不能宣称任意 CSS 动画。 | P2 |
| [FloatingIndicator](https://mantine.dev/core/floating-indicator/) | FloatingIndicator：活 target/geometry、有限运动 | 缺自由 transform transition 与多个 targets 的任意 ref API；使用非 owning 节点引用，测量之后仍不宣称已显示帧。 | P2 |
| [FloatingWindow](https://mantine.dev/core/floating-window/) | FloatingWindow：真实 bounds/raise、拖拽约束、键盘 | 缺 minimize/maximize/persist storage 和完整标题栏 slot；尺寸与 viewport 约束已存在，不能只用 absolute div 代表窗口协议。 | P2 |
| [HoverCard](https://mantine.dev/core/hover-card/) | HoverCard：共享 Hint/Layer 与延迟、保留策略 | 缺 HoverCard.Group 协调延迟和更细 focus 交互配置；复杂内容仍须共享 Layer 栈处理 Escape/外点。 | P2 |
| [LoadingOverlay](https://mantine.dev/core/loading-overlay/) | Overlay/LoadingOverlay：显式遮罩、透明度、内容状态 | 缺 blur、loaderProps/自定义 loader slot 和 zIndex 便利项；遮罩与真实 inert/焦点阻断必须分开验收。 | P2 |
| [Menu](https://mantine.dev/core/menu/) | Menu：真实菜单、子菜单、radio/check、roving/typeahead | 缺所有 Mantine trigger/transition/arrow 配置；现有 HoverDelay/placement 不等于完整 compound API。native 子菜单等仍按矩阵拒绝。 | P2 |
| [Menubar](https://mantine.dev/core/menubar/) | Menubar：多菜单、方向、键盘与菜单状态 | 桌面 native Menubar facet 仍拒绝；Web 的触发/过渡细项与 Mantine 不完全一致，不写为跨平台已完成。 | 边界 |
| [Modal](https://mantine.dev/core/modal/) | Dialog 对应 Mantine Modal：共享 Layer、焦点返回、栈、关闭策略 | 缺 centered/fullScreen/滚动区域与过渡细项的成套 builder；尺寸可由 Style 表达。不要把 Mantine Dialog 当模态框。 | P1 |
| [Overlay](https://mantine.dev/core/overlay/) | Overlay/LoadingOverlay：显式遮罩、透明度、内容状态 | 缺 blur、loaderProps/自定义 loader slot 和 zIndex 便利项；遮罩与真实 inert/焦点阻断必须分开验收。 | P2 |
| [Popover](https://mantine.dev/core/popover/) | Popover：锚点定位、碰撞处理、焦点/关闭策略 | 缺 width=target、箭头、统一下拉尺寸/样式参数与组合子部件 API；定位应继续由 Glory 宿主执行。 | P1 |
| [Tooltip](https://mantine.dev/core/tooltip/) | Tooltip：hover/focus/touch、有界延迟、placement | 缺 Group 延迟共享/完整 arrow/多行布局选项；不能用 title 属性当同一功能，native 浮层仍遵守其拒绝边界。 | P2 |
| [Tour](https://mantine.dev/core/tour/) | Tour：显式目标、步骤、导航/完成/取消、失效恢复 | 缺自由 renderStep/spotlight 视觉配置、完整 overlay 点击策略；目标必须是活的语义节点，不扫描虚拟列表源数据。 | P2 |
| [Accordion](https://mantine.dev/core/accordion/) | Accordion：多开/不可全关、方向、键盘、heading_level、动态项 | 缺完整 chevron/旋转/图标位置和外观变体配置；可先用 trigger_content 组合，动画时长应复用 Collapse 协议。 | P2 |
| [Avatar](https://mantine.dev/core/avatar/) | Avatar/AvatarGroup：fallback、max_visible、重叠、名称 | 缺自动姓名取色/initials 算法和完整 group overflow slot；不是所有异步图片配置都等价，需沿用 request revision 与资源预算。 | P2 |
| [BackgroundImage](https://mantine.dev/core/background-image/) | Image/BackgroundImage：fit、fallback、load/error、资源身份 | 缺 srcSet/responsive sizes 和加载策略；必须扩展共享图像语义/宿主预算，不能在组件里注入 DOM 属性。 | P1 |
| [Badge](https://mantine.dev/core/badge/) | Badge/Pill：三种 badge variant、受控移除、紧凑标签 | 上一轮修复尺寸与 close action；Badge 的 dot/左右 section/gradient、Pill 的拖拽排序仍未完整提供。先补展示 slot，再考虑有界排序。 | P2 |
| [Card](https://mantine.dev/core/card/) | Card/Paper：header/body/footer、border、有限阴影 | Card.Section 的负边距和继承 padding 规则不是任意容器已有；优先明确 section 边界。颜色/圆角已可 Style 覆盖。 | P2 |
| [ColorSwatch](https://mantine.dev/core/color-swatch/) | ColorSwatch：静态颜色或真实交互、checked、尺寸 | 缺自动 contrast/check 图标/叠加 slot；色彩透明棋盘与可访问名称已是不同责任，不把纯色 div 当交互。 | P2 |
| [DataList](https://mantine.dev/core/data-list/) | DataList：term/description 与 Item 媒体/动作 | Mantine 的横/纵布局和列对齐配置仍有差异；现有语义与布局可组合，需补长标签/窄屏验收。 | P2 |
| [Image](https://mantine.dev/core/image/) | Image/BackgroundImage：fit、fallback、load/error、资源身份 | 缺 srcSet/responsive sizes 和加载策略；必须扩展共享图像语义/宿主预算，不能在组件里注入 DOM 属性。 | P1 |
| [Indicator](https://mantine.dev/core/indicator/) | Indicator/ThemeIcon：位置、颜色、标签、尺寸、有限 variant | 缺 Indicator processing 动画/offset 和完整 ThemeIcon gradient/自动对比；先补静态布局，再复用有限运动。 | P2 |
| [Kbd](https://mantine.dev/core/kbd/) | Kbd：键组合、平台差异、文档语义 | 不支持任意 component；自定义快捷键显示/尺寸可组合，显示快捷键不等于全局键盘注册。 | 边界 |
| [NumberFormatter](https://mantine.dev/core/number-formatter/) | NumberFormatter：显式 locale、货币/百分比与精度 | 不是完整 Intl/NumberFormat：缺 compact/engineering/unit、任意前后缀与自定义 grouping；保留有限支持并拒绝未知格式。 | P2 |
| [OverflowList](https://mantine.dev/core/overflow-list/) | OverflowList：真实尺寸/有界动态项、overflow slot | 缺任意 DOM children 测量与 collapseFrom 的完整策略；复用当前窗口/注册节点，禁止保留访问过的业务项。 | P2 |
| [RollingNumber](https://mantine.dev/core/rolling-number/) | Marquee/RollingNumber：有限 translation、暂停、格式化 | 缺 hover/reduced-motion 组合和完整 glyph 翻滚动画；数值文本过渡不是完整逐字符动画，需如实区分。 | P2 |
| [Spoiler](https://mantine.dev/core/spoiler/) | Spoiler：测量高度、展开、标签、引用 | 缺完整过渡参数/尺寸同步配置；需要沿用真实测量与宿主帧，不应定时猜测溢出。 | P2 |
| [ThemeIcon](https://mantine.dev/core/theme-icon/) | Indicator/ThemeIcon：位置、颜色、标签、尺寸、有限 variant | 缺 Indicator processing 动画/offset 和完整 ThemeIcon gradient/自动对比；先补静态布局，再复用有限运动。 | P2 |
| [Timeline](https://mantine.dev/core/timeline/) | Timeline：active、marker、标题/描述和进度 | 缺 reverseActive、lineVariant、bullet 对齐等完整策略；用显式数据顺序，不能反转显示却改变访问顺序。 | P2 |
| [Blockquote](https://mantine.dev/core/blockquote/) | Blockquote：引用与 citation 的有限语义 | 缺 icon/iconSize 和完整色彩变体；可组合显示 slot，引用来源语义不应变为任意 HTML。 | P2 |
| [Code](https://mantine.dev/core/code/) | Code/Mark/Highlight：有限文档部位、查询高亮、转义 | 缺多关键字/重叠匹配/autoContrast 等便捷策略；Code 的高亮语言属于可选 CodeHighlight，不把 plain Code 标为完整语法高亮。 | P2 |
| [Highlight](https://mantine.dev/core/highlight/) | Code/Mark/Highlight：有限文档部位、查询高亮、转义 | 缺多关键字/重叠匹配/autoContrast 等便捷策略；Code 的高亮语言属于可选 CodeHighlight，不把 plain Code 标为完整语法高亮。 | P2 |
| [List](https://mantine.dev/core/list/) | List：真实有序/无序列表和条目 | 缺 list icon、withPadding/center 等便捷 API；图标列表需保留原生列表语义，不能用普通 Row 模拟。 | P2 |
| [Mark](https://mantine.dev/core/mark/) | Code/Mark/Highlight：有限文档部位、查询高亮、转义 | 缺多关键字/重叠匹配/autoContrast 等便捷策略；Code 的高亮语言属于可选 CodeHighlight，不把 plain Code 标为完整语法高亮。 | P2 |
| [Table](https://mantine.dev/core/table/) | Table：真实表格、span、caption/head/body/foot、stripe/border | 缺 sticky header/captionSide/高亮行/滚动容器的统一配置；sticky 在 native 受限。不要把纯表格当完整 DataTable。 | P2 |
| [Text](https://mantine.dev/core/text/) | Text/Title/Typography：语义级别、截断、字重/行高、主题 | 缺 lineClamp、多行截断、gradient 和任意 HTML 排版容器；lineClamp 值得扩展 Glory 样式，不能由单行 ellipsis 冒充。 | P1 |
| [Title](https://mantine.dev/core/title/) | Text/Title/Typography：语义级别、截断、字重/行高、主题 | 缺 lineClamp、多行截断、gradient 和任意 HTML 排版容器；lineClamp 值得扩展 Glory 样式，不能由单行 ellipsis 冒充。 | P1 |
| [Typography](https://mantine.dev/core/typography/) | Text/Title/Typography：语义级别、截断、字重/行高、主题 | 缺 lineClamp、多行截断、gradient 和任意 HTML 排版容器；lineClamp 值得扩展 Glory 样式，不能由单行 ellipsis 冒充。 | P1 |
| [Box](https://mantine.dev/core/box/) | Box/Flex/Stack/Group：布局、gap、对齐、换行、宽高 | Group grow/preventGrowOverflow 的便捷策略尚缺；响应式值通过 ResponsiveRoot 显式组合。Box 的任意 component/ref API 不移植。 | P2 |
| [Collapse](https://mantine.dev/core/collapse/) | Collapsible：展开、内容保留、有限高度过渡 | 与 Collapse 共享逻辑；任意 transition 名称/回调、水平折叠仍不是完整契约，不将只设置 CSS 算宿主动画支持。 | P2 |
| [Divider](https://mantine.dev/core/divider/) | Separator：水平/垂直、可访问名称、长度与粗细 | 缺可见 label/标签位置与 dashed/dotted 便捷参数；名称不是可见标签。边框样式可组合，标签分隔须保留分隔语义。 | P2 |
| [FocusTrap](https://mantine.dev/core/focus-trap/) | Portal/FocusScope：显式目标、失效回调、焦点范围与生命周期 | 对应 FocusScope 的 auto_focus/loop_tab/contain/return_focus；不采用 DOM selector 或 React child ref。需分别看 Web 与 native 的实际焦点证据。 | P2 |
| [Marquee](https://mantine.dev/core/marquee/) | Marquee/RollingNumber：有限 translation、暂停、格式化 | 缺 hover/reduced-motion 组合和完整 glyph 翻滚动画；数值文本过渡不是完整逐字符动画，需如实区分。 | P2 |
| [Paper](https://mantine.dev/core/paper/) | Card/Paper：header/body/footer、border、有限阴影 | Card.Section 的负边距和继承 padding 规则不是任意容器已有；优先明确 section 边界。颜色/圆角已可 Style 覆盖。 | P2 |
| [Portal](https://mantine.dev/core/portal/) | Portal/FocusScope：显式目标、失效回调、焦点范围与生命周期 | 不采用任意 DOM selector、ref 和 React child-clone API。FocusTrap 对应 FocusScope；任意 transition 样式注册仍待设计。 | P2 |
| [ScrollArea](https://mantine.dev/core/scroll-area/) | ScrollArea/Scroller：真实 scroll/query、轴、keyboard 和 metrics | 缺完整自绘滚动条/scrollHideDelay/overscroll 策略；浏览器/原生 scrollbar 外观本就不同，不应伪造滚动数据。 | P2 |
| [Scroller](https://mantine.dev/core/scroller/) | ScrollArea/Scroller：真实 scroll/query、轴、keyboard 和 metrics | 缺完整自绘滚动条/scrollHideDelay/overscroll 策略；浏览器/原生 scrollbar 外观本就不同，不应伪造滚动数据。 | P2 |
| [Toolbar](https://mantine.dev/core/toolbar/) | Toolbar/ActionBar：有界组、roving、selected_count/clear | Mantine ActionBar placement/transition/portal 配置不同；不得把横排 Toolbar 当完整底部浮条。可用 shared Layer 进一步补齐。 | P2 |
| [Transition](https://mantine.dev/core/transition/) | Theme/locale/direction/motion：app-local 主题与 typed Style | 不移植全局 MantineProvider/React hooks 或默认浏览器 locale；保留显式 app/request 边界。默认 props registry/任意样式 resolver 暂不采纳。 | 边界 |
| [VisuallyHidden](https://mantine.dev/core/visually-hidden/) | VisuallyHidden：语义文字、固定隐藏样式和引用 | 不支持 polymorphic component；可访问名称/描述与真正视觉隐藏保留，不应通过 display:none 替代。 | 边界 |
| [DatePicker](https://mantine.dev/dates/date-picker/) | DatePicker/Input：单/多/范围、弹层、排除日、受控 | 缺 presets、多个月份、任意 valueFormat、level controls；保留 CivilDate 和显式限额，range 不完整态已有独立模型。 | P1 |
| [DatePickerInput](https://mantine.dev/dates/date-picker-input/) | DatePicker/Input：单/多/范围、弹层、排除日、受控 | 缺 presets、多个月份、任意 valueFormat、level controls；保留 CivilDate 和显式限额，range 不完整态已有独立模型。 | P1 |
| [DateTimePicker](https://mantine.dev/dates/date-time-picker/) | DateTimePicker/Inline：日期+时间草稿、显式 zone resolver | 缺输入型弹层/presets/format 和完整 clearable；时区转换保持显式 provider，不引入全局本地时区假设。 | P1 |
| [InlineDateTimePicker](https://mantine.dev/dates/inline-date-time-picker/) | DateTimePicker/Inline：日期+时间草稿、显式 zone resolver | 缺输入型弹层/presets/format 和完整 clearable；时区转换保持显式 provider，不引入全局本地时区假设。 | P1 |
| [DateInput](https://mantine.dev/dates/date-input/) | DateInput：ISO 草稿、自定义 parser、picker、reject/commit | 缺完整 date format/locale parse/placeholder；模糊日期解析不是显示格式，保持 typed parser 和明确错误恢复。 | P1 |
| [MonthPicker](https://mantine.dev/dates/month-picker/) | MonthPicker/Input：YearMonth、单/多/范围、排除值、Grid | 缺月份文本自定义、多列年份跨度、presets/标签格式；不把 picker 的 locale 参数转发视为所有日历格式。 | P2 |
| [MonthPickerInput](https://mantine.dev/dates/month-picker-input/) | MonthPicker/Input：YearMonth、单/多/范围、排除值、Grid | 缺月份文本自定义、多列年份跨度、presets/标签格式；不把 picker 的 locale 参数转发视为所有日历格式。 | P2 |
| [YearPicker](https://mantine.dev/dates/year-picker/) | YearPicker/Input：CivilYear、有限十年、范围/多选、Grid | 缺多年份面板/decadeLabelFormat/预设值；同族 Month API 不意味着全部 Year API 同名。 | P2 |
| [YearPickerInput](https://mantine.dev/dates/year-picker-input/) | YearPicker/Input：CivilYear、有限十年、范围/多选、Grid | 缺多年份面板/decadeLabelFormat/预设值；同族 Month API 不意味着全部 Year API 同名。 | P2 |
| [TimeInput](https://mantine.dev/dates/time-input/) | TimeInput/Value：LocalTime、12/24h、精度、草稿/边界 | 不支持任意 dayjs format/自动时区；本地墙上时间和瞬时时刻分开。placeholder 与更多 locale 是可借鉴项。 | P1 |
| [TimePicker](https://mantine.dev/dates/time-picker/) | TimePicker/Grid：分段 SpinButton、step/排除、12/24h、草稿 | 缺 presets/groups、duration 模式、clearable 和完整下拉配置；不把下拉 TimeGrid 当完整分段输入，当前两者都有真实语义。 | P1 |
| [TimeGrid](https://mantine.dev/dates/time-grid/) | TimePicker/Grid：分段 SpinButton、step/排除、12/24h、草稿 | 缺 presets/groups、duration 模式、clearable 和完整下拉配置；不把下拉 TimeGrid 当完整分段输入，当前两者都有真实语义。 | P1 |
| [TimeValue](https://mantine.dev/dates/time-value/) | TimeInput/Value：LocalTime、12/24h、精度、草稿/边界 | 不支持任意 dayjs format/自动时区；本地墙上时间和瞬时时刻分开。placeholder 与更多 locale 是可借鉴项。 | P1 |
| [MiniCalendar](https://mantine.dev/dates/mini-calendar/) | Calendar/MiniCalendar：CivilDate、月份、排除/周起点 | 缺丰富 day renderer、withWeekNumbers/multiple months/任意 locale 文本格式；不得用普通按钮网格代替真实 Grid 键盘语义。 | P1 |
| [Calendar](https://mantine.dev/dates/calendar/) | Calendar/MiniCalendar：CivilDate、月份、排除/周起点 | 缺丰富 day renderer、withWeekNumbers/multiple months/任意 locale 文本格式；不得用普通按钮网格代替真实 Grid 键盘语义。 | P1 |
| [AreaChart](https://mantine.dev/charts/area-chart/) | Area/Line/Bar/Composite：typed 数据/轴/domain/legend/selection | 缺 Recharts arbitrary props、brush/reference areas/复杂 tooltips、动画与多个轴的全部组合；优先有限 reference/轴格式，不暴露第三方引擎对象。 | P1 |
| [BarChart](https://mantine.dev/charts/bar-chart/) | Area/Line/Bar/Composite：typed 数据/轴/domain/legend/selection | 缺 Recharts arbitrary props、brush/reference areas/复杂 tooltips、动画与多个轴的全部组合；优先有限 reference/轴格式，不暴露第三方引擎对象。 | P1 |
| [LineChart](https://mantine.dev/charts/line-chart/) | Area/Line/Bar/Composite：typed 数据/轴/domain/legend/selection | 缺 Recharts arbitrary props、brush/reference areas/复杂 tooltips、动画与多个轴的全部组合；优先有限 reference/轴格式，不暴露第三方引擎对象。 | P1 |
| [CompositeChart](https://mantine.dev/charts/composite-chart/) | Area/Line/Bar/Composite：typed 数据/轴/domain/legend/selection | 缺 Recharts arbitrary props、brush/reference areas/复杂 tooltips、动画与多个轴的全部组合；优先有限 reference/轴格式，不暴露第三方引擎对象。 | P1 |
| [CandlestickChart](https://mantine.dev/charts/candlestick-chart/) | Scatter/Bubble/Candlestick：有界数据、采样、选择 | 缺 brush/多轴/自定义 symbol 与完整 tooltip formatting；采样必须报告源数量，不能默默把全量数据当已绘制。 | P2 |
| [DonutChart](https://mantine.dev/charts/donut-chart/) | Pie/Donut/Radial：有限 Plot、legend/selection、可访问数据 | 缺任意分段 label renderer/动画/复杂 ring/tooltip；静态图形、数据表与真实选择各有证据，不能靠图形相似推断 API 等价。 | P2 |
| [PieChart](https://mantine.dev/charts/pie-chart/) | Pie/Donut/Radial：有限 Plot、legend/selection、可访问数据 | 缺任意分段 label renderer/动画/复杂 ring/tooltip；静态图形、数据表与真实选择各有证据，不能靠图形相似推断 API 等价。 | P2 |
| [FunnelChart](https://mantine.dev/charts/funnel-chart/) | Radar/Funnel/Sparkline：有限 Plot/options 与语义数据 | 缺多 radar 叠加和各类插值/label/tooltip 完整配置；Sparkline 的有限模型不同于任意 Recharts 属性。 | P2 |
| [RadarChart](https://mantine.dev/charts/radar-chart/) | Radar/Funnel/Sparkline：有限 Plot/options 与语义数据 | 缺多 radar 叠加和各类插值/label/tooltip 完整配置；Sparkline 的有限模型不同于任意 Recharts 属性。 | P2 |
| [ScatterChart](https://mantine.dev/charts/scatter-chart/) | Scatter/Bubble/Candlestick：有界数据、采样、选择 | 缺 brush/多轴/自定义 symbol 与完整 tooltip formatting；采样必须报告源数量，不能默默把全量数据当已绘制。 | P2 |
| [BubbleChart](https://mantine.dev/charts/bubble-chart/) | Scatter/Bubble/Candlestick：有界数据、采样、选择 | 缺 brush/多轴/自定义 symbol 与完整 tooltip formatting；采样必须报告源数量，不能默默把全量数据当已绘制。 | P2 |
| [RadialBarChart](https://mantine.dev/charts/radial-bar-chart/) | Pie/Donut/Radial：有限 Plot、legend/selection、可访问数据 | 缺任意分段 label renderer/动画/复杂 ring/tooltip；静态图形、数据表与真实选择各有证据，不能靠图形相似推断 API 等价。 | P2 |
| [Sparkline](https://mantine.dev/charts/sparkline/) | Radar/Funnel/Sparkline：有限 Plot/options 与语义数据 | 缺多 radar 叠加和各类插值/label/tooltip 完整配置；Sparkline 的有限模型不同于任意 Recharts 属性。 | P2 |
| [Heatmap](https://mantine.dev/charts/heatmap/) | Heatmap/Matrix/Waffle：离散数据、色阶与可访问表 | 缺完整日期热图布局、hover renderer、多色 cell shape 和任意动画；需明确 missing value 和有限颜色映射。 | P2 |
| [BarsList](https://mantine.dev/charts/bars-list/) | BarsList/Bullet/Gauge：有限形状与数据标签 | 缺目标/区间段/动态格式的完整组合；优先 typed thresholds/labels，不能把 progress 换名当 gauge/bullet。 | P2 |
| [BulletChart](https://mantine.dev/charts/bullet-chart/) | BarsList/Bullet/Gauge：有限形状与数据标签 | 缺目标/区间段/动态格式的完整组合；优先 typed thresholds/labels，不能把 progress 换名当 gauge/bullet。 | P2 |
| [GaugeChart](https://mantine.dev/charts/gauge-chart/) | BarsList/Bullet/Gauge：有限形状与数据标签 | 缺目标/区间段/动态格式的完整组合；优先 typed thresholds/labels，不能把 progress 换名当 gauge/bullet。 | P2 |
| [WaffleChart](https://mantine.dev/charts/waffle-chart/) | Heatmap/Matrix/Waffle：离散数据、色阶与可访问表 | 缺完整日期热图布局、hover renderer、多色 cell shape 和任意动画；需明确 missing value 和有限颜色映射。 | P2 |
| [Treemap](https://mantine.dev/charts/treemap/) | Treemap/Sunburst/Sankey：有界层级/流、导航和数据表 | 缺任意布局算法/节点链接 render/动态图过渡；需先校验层级/环/预算，不能导入任意图形引擎配置。 | P2 |
| [SunburstChart](https://mantine.dev/charts/sunburst-chart/) | Treemap/Sunburst/Sankey：有界层级/流、导航和数据表 | 缺任意布局算法/节点链接 render/动态图过渡；需先校验层级/环/预算，不能导入任意图形引擎配置。 | P2 |
| [SankeyChart](https://mantine.dev/charts/sankey-chart/) | Treemap/Sunburst/Sankey：有界层级/流、导航和数据表 | 缺任意布局算法/节点链接 render/动态图过渡；需先校验层级/环/预算，不能导入任意图形引擎配置。 | P2 |
| [MatrixChart](https://mantine.dev/charts/matrix-chart/) | Heatmap/Matrix/Waffle：离散数据、色阶与可访问表 | 缺完整日期热图布局、hover renderer、多色 cell shape 和任意动画；需明确 missing value 和有限颜色映射。 | P2 |
| [Schedule](https://mantine.dev/schedule/schedule/) | Schedule/事件/recurrence：有限模型、显式 zone/provider、窗口 | 缺整套 header/view control、通用 DnD/resize/外部事件/overlap callbacks；已有资源编辑不等于普通 Schedule 全部拖拽，优先补统一编辑提议。 | P1 |
| [DayView](https://mantine.dev/schedule/day-view/) | Day/Week：有界时间窗口、真实 Grid、布局/选择 | 缺完整 slot renderer、外部拖放、event resize/交互背景事件；需共享编辑协议和目标/权限/冲突校验。 | P1 |
| [MonthView](https://mantine.dev/schedule/month-view/) | Month/Year/MobileMonth：有限日历视图、选择、窗口 | 缺完整日期跳转/自定义日单元和拖拽编辑；同名 view 并不等价 desktop/mobile 全部交互。 | P1 |
| [WeekView](https://mantine.dev/schedule/week-view/) | Day/Week：有界时间窗口、真实 Grid、布局/选择 | 缺完整 slot renderer、外部拖放、event resize/交互背景事件；需共享编辑协议和目标/权限/冲突校验。 | P1 |
| [YearView](https://mantine.dev/schedule/year-view/) | Month/Year/MobileMonth：有限日历视图、选择、窗口 | 缺完整日期跳转/自定义日单元和拖拽编辑；同名 view 并不等价 desktop/mobile 全部交互。 | P1 |
| [MobileMonthView](https://mantine.dev/schedule/mobile-month-view/) | Month/Year/MobileMonth：有限日历视图、选择、窗口 | 缺完整日期跳转/自定义日单元和拖拽编辑；同名 view 并不等价 desktop/mobile 全部交互。 | P1 |
| [AgendaView](https://mantine.dev/schedule/agenda-view/) | AgendaView：有界窗口事件展示/选择、locale | 缺分组 header/空日策略/完整事件 body renderer；应用负责数据加载，不建立访问日期的永久事件缓存。 | P2 |
| [ResourcesDayView](https://mantine.dev/schedule/resources-day-view/) | Resource views：资源窗口、分页、有界 Grid 与编辑提议 | 缺普通 Schedule 对应的全部视图和跨资源外拖/resize组合；对资源页、时间窗、DST、权限和 stale commit 分别验收。 | P1 |
| [ResourcesWeekView](https://mantine.dev/schedule/resources-week-view/) | Resource views：资源窗口、分页、有界 Grid 与编辑提议 | 缺普通 Schedule 对应的全部视图和跨资源外拖/resize组合；对资源页、时间窗、DST、权限和 stale commit 分别验收。 | P1 |
| [ResourcesMonthView](https://mantine.dev/schedule/resources-month-view/) | Resource views：资源窗口、分页、有界 Grid 与编辑提议 | 缺普通 Schedule 对应的全部视图和跨资源外拖/resize组合；对资源页、时间窗、DST、权限和 stale commit 分别验收。 | P1 |
| [ResourcesSchedule](https://mantine.dev/schedule/resources-schedule/) | Resource views：资源窗口、分页、有界 Grid 与编辑提议 | 缺普通 Schedule 对应的全部视图和跨资源外拖/resize组合；对资源页、时间窗、DST、权限和 stale commit 分别验收。 | P1 |
| [Recurring events](https://mantine.dev/schedule/recurring-events/) | Schedule/事件/recurrence：有限模型、显式 zone/provider、窗口 | 缺整套 header/view control、通用 DnD/resize/外部事件/overlap callbacks；已有资源编辑不等于普通 Schedule 全部拖拽，优先补统一编辑提议。 | P1 |
| [Events data](https://mantine.dev/schedule/events-data/) | Schedule/事件/recurrence：有限模型、显式 zone/provider、窗口 | 缺整套 header/view control、通用 DnD/resize/外部事件/overlap callbacks；已有资源编辑不等于普通 Schedule 全部拖拽，优先补统一编辑提议。 | P1 |
| [CodeHighlight](https://mantine.dev/code-highlight/code-highlight/) | CodeHighlight：有限语言/token、行号/高亮、复制 | 不兼容 Shiki/highlight.js 任意 grammar/themes、多文件 tabs 等；新语言须增加有界 lexer 和错误/大文本预算。 | P2 |
| [JsonViewer](https://mantine.dev/code-highlight/json-viewer/) | JsonViewer：有限 tree、搜索/展开/复制、键盘 | 缺任意 JS object/循环引用/custom renderer；JSON 与 runtime object inspector 不同，保留深度/节点/字节限制。 | 边界 |
| [Notifications system](https://mantine.dev/x/notifications/) | Notification/系统：有界队列、更新/关闭、计时与 hover 暂停 | 缺分区域 placement、任意 transition 和完整 loader/icon slots；队列有界与 app-local context 优于无界全局管理器。 | P2 |
| [Spotlight](https://mantine.dev/x/spotlight/) | Command/Spotlight：搜索、分组、异步结果、显式 shortcut 和 handle | 缺自由渲染结果/完整 actions 分部 API；任意 fuzzy highlight 和 loading slot 待设计。现有命令与快捷键已受 Owner 保护。 | P2 |
| [Carousel](https://mantine.dev/x/carousel/) | Carousel：有界 window/overscan、current、swipe、autoplay | 不是 Embla 全部 options/plugins；缺多 slide 排列/dragFree/细粒度 controls，优先共享窗口逻辑而非引入 JS 引擎。 | P2 |
| [Lightbox](https://mantine.dev/x/lightbox/) | Lightbox：真实图片/标题、有限 view/zoom、Layer | 缺 thumbnails、视频支持和完整 gestures；transform/缩放需要有限语义与预算，不让原图资源永久留在访问缓存。 | P2 |
| [Dropzone](https://mantine.dev/x/dropzone/) | Dropzone：有限接收/预算/拒绝、读取/预览、取消 | 缺全局 window dropzone、更多 reject slots/拖动文案；全局文件访问/读取不隐式加入，保留权限和资源预算。 | P2 |
| [NavigationProgress](https://mantine.dev/x/nprogress/) | NavigationProgress：显式 start/set/finish/cancel handle | 缺自动增量等动画策略；不能让 loader 定时器逃逸 Owner，关闭后不应重新启动。 | P2 |
| [Modals manager](https://mantine.dev/x/modals/) | ModalsManager：app-local 有界队列、confirm/cancel、结果/更新 | 缺任意 React context modal 注册；有限 typed request 可以扩展，不采纳全局 registry/任意组件注入。 | P2 |
| [Rich text editor](https://mantine.dev/x/tiptap/) | RichTextEditor：有限 RichDocument、格式/selection/history | 不是 Tiptap plugin/HTML 编辑器；链接、表格、嵌套列表、媒体和协同编辑未全覆盖，必须先扩展 Glory 文档/编辑协议。 | P1 |

完整 Props、功能章节与本地入口见 [API 表面索引](mantine-api-surface.md)；机器事实见 [快照](mantine-api-audit.json)。

## 本轮验证与复现

- 官方快照：181 条组件/工具目录记录、4,659 个按条目计数的 Props 字段（含同页子部件）及其类型；刷新器会拒绝未知新增目录页或无法解析的 Props 行，排除示例代码里的伪章节。
- `scripts/check.ps1` 完整通过：415 组、2,649 次测试执行及默认/all-features 严格 Clippy（测试次数包含多 feature 重复运行）。日志：`D:/BuildCache/sapid/mantine-library-gate.log`。
- 最终变更另补跑 29 项选择控件、3 项输入 hint、10 项 Button 增强测试，以及 all-features/all-targets 严格 Clippy。日志：`mantine-final-choice.log`、`mantine-final-input-tests.log`、`mantine-final-sapid-clippy.log`。
- Glory：共享 hint 验证/回滚与 HTML prepared-field/full-shape 差分通过；独立 SSR stream 的 hint patch 通过；Blitz descriptor-local preflight/direct apply 拒绝与前缀保留通过，native tests 严格 Clippy 通过。日志：`mantine-glory-placeholder.log`、`mantine-html-placeholder.log`、`mantine-stream-placeholder.log`、`mantine-native-placeholder.log`、`mantine-native-clippy.log`。
- 引擎来源检查与 fork 自有测试全部通过：保留两个 full SHA、六个上游包名及锁文件来源。修正检查器过期的十包断言，改为核对当前包含 opt-in i18n 的完整十一包集合。未修改 fork 代码或依赖 revision。日志：`mantine-engine-final.log`。
- 展示站 host/wasm32 严格 Clippy、19 项 Rust 检查、WASM 与 36 个代表性 SSR 文档通过。日志：`mantine-showcase-final.log`；最终资产 `assets-aec435696a9fb3e4493f`。
- Chromium 148.0.7778.96：350 组检查、errors=[]。包含新增 48 组 API 专项、64 组样式、182 组既有站点检查（含全部 149 路由）、16 组 Theme Studio、32 组当前配色顺序检查、8 组导航检查。报告均在 `D:/BuildCache/sapid/*-browser.json`。
- 新增浏览器专项同时验证真实 Input 的 value setter 未在 hint 更新时调用、节点/UTF-16 选区/预编辑保留，四主题、1440/360px、CSR/真实 SSR 接管均通过。CDP IME 是协议证据，不等于 OS IME 或读屏语音验收。
- 本轮状态仅将 A1–A3 标为完成；A4 与逐项差异表中的其他缺口仍保留为未实现/待设计，没有把目录覆盖写成 API 全兼容。

复现：`python scripts/mantine-api-audit.py`；离线检查：`python scripts/mantine-api-audit.py --check`。
上游事实仅在显式 `--refresh` 时更新；完整文档缓存在 D 盘，仓库只保存属性/章节事实和 SHA256，未复制第三方实现。
文档快照 SHA256：`aea0899df7df45c238b4d7ce7773580784ca7242b7a1a650abf06d8d276e69c4`。
