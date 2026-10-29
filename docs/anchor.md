# Anchor

`anchor()` 包装 Glory 的原生 Link。`.href(...)` 是链接地址（底层 API 命名为 Link.target）；
`.label(...)` 是可见文字，`.name(...)` 可独立提供可访问名称。不会用 Button 冒充链接。

```rust
use glory::render::LinkOpen;

sapid::anchor()
    .label("Open documentation")
    .href("https://example.test/docs")
    .open_in(LinkOpen::NewContext)
    .on_navigate(|request| {
        // request.href and request.open describe the currently accepted node.
        // The application owns routing, window policy and transport.
    });
```

`on_navigate` 接收 `AnchorRequest { href: String, open: LinkOpen }`，先于普通 Activate observers。
提供该 typed callback 时，组件声明 `Semantics.link_request`，Web/stream 宿主在原生监听边界
阻止默认导航，再将请求排队到正常 UI turn。回调不是 HTTP 成功通知，也不会自动启动外部浏览器。

**没有 on_navigate 的 Anchor 保留浏览器原生 href 行为**（包括 fragment 和正常新上下文策略）。
普通 `.on(EventKind::Activate, ...)` 是事件观察，不是阻止原生导航的接口。

`LinkOpen::Current` 默认省略 target/rel；NewContext 在 Web/SSR 输出 `target="_blank"` 与
`rel="noopener noreferrer"`。动态清除/切回 Current 会同步删除两者，保持同一个链接节点。
Apace 当前明确拒绝 NewContext；普通 Current 链接及其应用请求在原生有效，不静默执行系统打开动作。

回调用身份校验后的 `Scope::link_request` 读取当前 accepted LinkProps.target 与 link_open，
不直接读取可能尚未绑定或已被宿主拒绝的源值。因此 native NewContext 更新被拒绝、源写入尚未 flush
或应用显式 Patch 链接时，不会发出与当前已接受节点不一致的导航请求。

Anchor 提供统一 theme/size/unstyled/class/style/name/focus/node_ref/on，以及响应式 href、label 和
open_in，以及 `.current(Vary<Option<Current>>)` 当前 page/step 位置语义（参见 `navigation.md`）。
默认采用主题链接颜色，unstyled 保留显式样式和导航语义。外部源变化静默；Owner 退休后
释放回调和引用，不保存访问地址历史。URI 策略、路由、下载、网络与实际跨窗口行为仍归应用及宿主。

## 证据

`tests/anchors.rs` 8 项验证 href/label/name 分离、typed-before-observer、外部静默/identity、target/rel
与 escaping、明确的导航捕获标记、Owner 释放、unstyled、pending source/显式 Patch 的 accepted 请求。

`examples/native/tests/typography.rs` 验证 Current 的真实引擎布局/identity、NewContext 初始原子拒绝，
以及动态 NewContext 拒绝后回调仍获得 Current。共享 Glory 另覆盖 preflight/direct apply 成功前缀和
query watermark，组件层没有兼容性回退。

`examples/web/typography-acceptance.js` 在 Chrome 155.0.8059.84 实际完成 9 组报告，错误为空：
四主题 CSR/hydration、pointer/Enter 请求不导航/不开 popup、无回调原生 fragment、动态 href/context
及 target/rel 清除、AX link、focused hydration identity 和 Owner 退休。报告在
`D:/BuildCache/sapid/typography-browser.json`。
