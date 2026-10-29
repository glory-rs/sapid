# Tooltip / HoverCard

P5-03 Tooltip 与 P5-04 HoverCard 使用同一个有界悬停/焦点控制器和宿主 Layer。
Tooltip 为真实 `Role::Tooltip`，通过实际目标控件的 `described_by` 关系提供描述，
不声明 popup/expanded，不取焦点，不返回焦点，不阻断 Tab。HoverCard 为具名非模态
Dialog，允许交互内容；打开时保留当前焦点，用户可正常 Tab 进入其链接、按钮和输入。
Tooltip 文本为 1–16,384 字节，使用借用验证的最近有效状态，拒绝无内容及超限文本。

```rust
tooltip("save-help")
    .label("Save draft")
    .text("Your draft remains private until published.")

tooltip("search-help")
    .text("Search by title or author.")
    .target(|target| target.input(glory::ui::input().name("Search")))

hover_card("profile")
    .label("About Ada")
    .name("Ada profile")
    .content(|| glory::ui::link().label("Read profile").target("/profile"))
```

默认目标为实际 Button。`FloatingTarget` 另有 button/link/input/check 的有限 typed
adapter，以及保留现有 Semantics 的 `*_with(builder, base)`。它们保留原 builder 的值、
内容、业务激活处理和焦点配置，安装自己的 ID、描述/控制关系、NodeRef 和
PointerWithin/FocusWithin 兴趣。冲突 ID/关系和未使用 adapter 的目标拒绝构建。
目标 helper 使用 WeakScope/Weak 控制器；应用保留 helper 也不会阻止 Owner 退休或保留回调。
同一目标若已有其他描述 ID，应在应用里组合描述；当前不是任意 ID 列表合并器。

`open` 是只读受控输入，`on_open_change` 只提出变更；没有受控源时使用 `default_open`。
源的外部写入不会被订阅成新的 hover 意图并立即反向提出打开。接受/拒绝关闭均保留实际源
的真值。内部 dismiss/open 回调先于同一 dismiss 的普通观察者；Escape 拒绝不重复提出同一
关闭请求。禁用 Tooltip 隐藏浮层并移除其描述关系。

触发器 hover、触发器 focus、面板 hover、面板 focus 分别占一个当前状态位。
结构阶段合并同一宿主 tick 的焦点迁移，因此卡片内链接到按钮的 focusout/focusin 不会
中途卸载内容。`ContentPolicy::Unmount` 默认在关闭时退休子内容 Owner；KeepMounted 保留
隐藏内容及其状态。Tooltip 始终保留稳定文本/面板 shell，以维持描述关系和 SSR adoption。

`open_delay_with` / `close_delay_with` 接受应用提供的本地 Future 工厂；没有工厂时立即执行。
HoverCard 的指针跨定位间隙应配置正的 close delay（示例使用 160ms）。组件不隐式安装浏览器
或 native 计时器。Menu 与本批共享 `OwnerDelay`：每个 live 控制器最多一个 Owner task，
新意图、反向边界和 Owner 销毁立即取消；非回绕 ticket 阻止晚到完成重开。Host::spawn
仅入队，不同步 poll。配置、Future、回调和取消捕获在借用之外析构。

Glory 新增的 `PointerWithin` 是 Group 可订阅的逻辑观察，不创建布局 wrapper。
实际 mouse/pen 原始 Pointer 由 Web 入队；App 各走一次 origin/related 祖先，在共同祖先
之前派生 Within。Memory 的直达路径同样检查 runtime、live generation、物理 origin、
ancestry 和 related 活性。控件内部移动不错误离开卡片。实际 Move 也可恢复 adoption 前
错过的 Enter；状态位已为 true 时不重复写、不重启 delay，Escape 后原地移动不会重开。
没有历史身份表、共享 origin cache 或业务源扫描。

`LayerFocus::{Manage,Preserve}` 是单一有限焦点策略。Dialog/Menu 默认 Manage；
Tooltip 必须为非模态 Preserve、FocusMode::Auto、无 initial/return_focus。HoverCard 使用
非模态 Preserve。宿主跳过其 autofocus/return，保留外围 modal 的实际 Tab/focus scope；
Escape 可来自本 App 触发器或控件，无焦点时也可来自当前实际 hover，但不吞其他 App 或
外围无关 DOM 按钮的键盘事件。共享层处理定位、视口碰撞和 top layer。

Tooltip 的被动内容限制不仅属于 builder：Plan、普通创建/Place、Group move/resume、
Role/Focus/Semantics patch 都执行共享约束，拒绝交互控件、可聚焦节点和交互复合元数据；
Role 改成 Tooltip 时检查其现有 live 子树。普通移动仍只走祖先，只有受限上下文检查被移动
live 子树；不检查业务列表源。Tooltip 自身的 Focus action 在排队/能力判断之前明确拒绝。

## 验收证据

- `tests/hints.rs`：7 项，覆盖真实角色/描述、hover/focus 联合、受控真值/回调顺序、实际
  子节点 Within、typed target 冲突与 Owner 捕获释放、单 task 开/关延迟和晚到唤醒、
  missed Enter/Move、Escape 重新激活、文本预算及稳定 SSR 形状。
- Glory `render/tests/hints.rs`：2 项，涵盖 Tooltip 焦点/内容/patch/Plan/Group Move/Resume
  事务拒绝以及 PointerWithin 直达身份/相关源/退休；`ui/tests/pointer_within.rs`：2 项，
  Within-only Group、共同祖先和早先回调退休 origin 后的重新验证。
- HTML mapper 四项 lib 测试通过，保留 full-shape differential oracle，并加入
  Tooltip/Preserve Layer 及默认重置组合。Glory render/ui 全部 tests 通过。
- SSR `tooltip_stream_preserves_description_relation_and_passive_layer_through_end`：1 项。
  `GLORY_HINTS_STREAM_FIXTURE` 输出真实 Shell/Patch/End；HTML v3 有限文本/属性编辑保留
  description、Tooltip role、hidden reset，End 执行完整 rekey。
- `examples/web/hints-acceptance.js`：Chromium 148.0.7778.96，10/10 报告、errors=[]。
  八组 blue/green × light/dark × CSR/hydrate，加脏输入 adoption 和真实 classic stream。
  检查真实焦点、AX Tooltip/trigger description、计时事件到 hidden 变化的实际时长、
  关闭取消及跨间隙、focus 联合、交互子内容实际 Tab/Enter、foreign Escape、受控拒绝、
  输入值/UTF-16 选区、卸载、Owner 取消和 390px 碰撞。隔离产物
  `D:/BuildCache/sapid/web/hints-next`，报告 `D:/BuildCache/sapid/hints-browser-148.json`，
  八份 `D:/BuildCache/sapid/hints-*.png`；green/dark/hydrate 已人工检查。
- Native `apace/tests/hints.rs`：1 项，descriptor/patch preflight 与 direct apply 明确拒绝
  Layer/PointerWithin，同时保留成功前缀和 query 水位。AccessKit
  `passive_tooltip_model_exports_real_role_description_and_no_focus_action`：1 项，支持的
  无 Layer 模型输出真实 Tooltip、trigger description，且不广告 Focus action。
- Sapid hints/menu 与 Glory render/ui strict Clippy 通过；所有构建/临时产物均在 D 盘缓存。

## 当前边界

Native 仍明确拒绝实际 Layer 和 PointerWithin 输入；AX 模型证据不代表原生浮层可交互，
也不替代屏幕阅读器朗读或 OS 提示行为。Tooltip 只接受有限文本，不提供交互内容槽位。
延迟由应用/宿主提供 Future，默认零延迟；触屏长按不是 hover 的隐式替代策略。
HoverCard 的 content 是实际 Widget factory，组件不代替应用处理其链接导航、异步数据或业务值。
