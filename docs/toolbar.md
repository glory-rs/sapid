# Toolbar 与 ActionBar

`toolbar(key)` 使用真正的 Toolbar role 和有序具名 Group，每组包含实际 Button。
全栏共享一个 roving Tab 入口；方向键跨组跳过 disabled/loading 项，Home/End
到首尾，默认循环，可关闭。水平支持 Direction::Rtl，垂直使用上下键。
只改变焦点意图，不把键盘导航当作动作激活；实际焦点仍由宿主确认。

```rust
use sapid::{button, toolbar, toolbar_group};
let tools = toolbar("editor").name("Editing tools").group(
    toolbar_group("clipboard").name("Clipboard")
        .button(button().label("Cut").on_press(|| {}))
        .button(button().label("Copy").on_press(|| {})),
);
```

声明最多 32 个非空 group、128 个总动作（含 ActionBar 清空按钮）。iterator
入口最多消费预算加一，不枚举业务列表。group key 显式且不重复，名称借用验证
后接受，最大 256 字节、非空单行 canonical；非法响应更新保留已接受 UI。
无动态业务集合或虚拟来源缓存。按钮自身的 enabled/loading/style/callback 保留，
共享 theme/size/disabled；组间有独立语义，水平窄屏按组换行，键盘沿声明顺序。
显式 class/style 与 unstyled 按既有 Sapid 约定处理。

`action_bar(key).selected_count(count)` 在 count>0 时展示选择动作。可用 visible
源显式决定展示；count=0 时所有动作禁用。`on_clear_selection` 加入真实按钮，
仅提议清空，不修改传入 count；拒绝保持原栏及选择数。隐藏保留子控件身份与
Owner，重新展示恢复；不制造 modal、浮层、隐式固定定位或返回焦点。
可用 Affix 包裹。`clear-selection` 是内部 group key，含清空按钮时请使用其他
业务 group key。要响应动态动作集合应重建该 Toolbar 的 Owner；不宣称任意
Widget 或 ToggleGroup 的自动注册。

`tests/toolbar.rs` 四项验证分组 roving/禁用恢复、受控清空来源静默、百万
iterator 截断/非法名称恢复、观察者与 Owner captures 清理。
`examples/native/tests/toolbar.rs` 一项运行四主题，验证真实布局/分组间距、
typed Navigate 以及选择栏显示切换后的相同 engine identity；typed dispatch
不等于实际 OS 键盘或屏幕阅读器验收。

固定 Web14 的 Chromium **148.0.7778.96** 在 `toolbar-acceptance.js` 实际
**8/8**，errors=[]，四主题各 CSR/hydration。包含真实方向键/Home/End、RTL、
垂直及非循环、动态 disabled、真实 AX Toolbar/Group、拒绝清空再接受、手机
宽度与 Owner 清理。报告 `D:/BuildCache/sapid/toolbar-browser-web14.json`；
原生 `toolbar-native.log`，green-dark hydration 截图已生成。
