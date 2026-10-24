# ColorSwatch

`color_swatch(Color(r,g,b,a))` 绘制有限 RGBA 色块。白底、4×4 灰白棋盘和实际
alpha path 分层，透明色不会错误地变成不透明色或同时淡化棋盘。默认无交互；
有意义的独立预览可 `.name("Half transparent red")`，使用具名 View 的 Group
语义；不命名时是装饰内容。`.side(8..=128)`、`.round(true)`、theme/class/style
和 unstyled 可组合。unstyled 保留预览内容、白底和必要的裁切/尺寸。

`.on_press(...)` / `.interactive(true)` 使用真实 Button。没有显式 name 时，
名称为当前 `Color #rrggbbaa`，会随 Color 源更新。`.pressed(source)`、
`.default_pressed(bool)` 或 `.on_pressed_change(...)` 启用 toggle 模型、真正的
pressed metadata 与有限勾选路径。普通 action button 不发 pressed。受控源
只读，回调提议变化；本地默认才内部接受。typed change、press、通用 observer
依次执行并在 Owner 退休后停止。disabled 同时约束宿主与 App 直接投递。
这不是 ColorInput，也不自动添加表单成功值或复制到剪贴板。

例如：

```rust
use glory::style::Color;
use sapid::color_swatch;
let preview = color_swatch(Color(255,0,0,128)).name("Half transparent red");
let choice = color_swatch(Color(0,100,255,255)).default_pressed(false);
```

颜色本身是四个 u8，永远有界；名字借用验证后接受（非空、256 字节、无
NUL/CR/LF）。非法大小拒绝挂载，非法动态名字保留此前接受的 UI。三个固定
Graphic 节点不使用任意 SVG/HTML，勾选不显示时为空路径，遵守 Button 只允许
inline/image/graphic/boxless Group 子内容的公共规则，不插入 View 包裹。
颜色/选择更新保留实际控件和 Graphic 身份；无访问历史、时钟或额外任务。

模型 `tests/color_swatch.rs` 5 项通过；四主题原生
`examples/native/tests/color_swatch.rs` 一项验证真实 checker/alpha CPU pixels、
零 alpha 显示灰白棋盘、PointerEvents::None 让命中返回 Button、更新后的相同
engine identity 与清理。日志 `D:/BuildCache/sapid/color-swatch-native.log`。
Web fixture 与 `color-swatch-acceptance.js` 已准备，实际浏览器证据待下一构建。
