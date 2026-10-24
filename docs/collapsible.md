# Collapsible

`collapsible(key)` 提供真实 Button 触发器、独立内容区以及 `controls` / `expanded` 关系。键在组件生命周期内稳定；标签与可见触发器内容分开配置。

```rust
collapsible("details")
    .label("More details")
    .trigger_content(|| glory::ui::text().content("Show details"))
    .content(|| glory::ui::text().content("Additional information"))
    .height_transition(240)
    .content_policy(ContentPolicy::Unmount)
```

`.trigger_content(factory)` 与 `.content(factory)` 独立挂载，各自具有实际 Owner。`KeepMounted` 在关闭后保留编辑器与其状态；默认 `Unmount` 在关闭时释放内容，再次展开构建新的内容 Owner。普通未启用动画的模式继续支持原生后端。

`.open(source)` 是受控来源，`.on_open_change` 只提交用户提案。外部来源更新保持静默；拒绝提案不会更改已接受配置或启动动画。内部接受/提案先于通用事件回调。触发器有 disabled 保护，关闭含焦点的内容时尝试返回仍然存活的触发器。

## 可选高度过渡

`.height_transition(millis)` 接受响应式 `u16`，有限范围为 0..=500ms。`.animated(source)` 也启用该能力，未显式设置时长时使用 180ms；false 将当前时长设为零并完成当前目标。相等配置不分配新 revision，只有实际接受的 open/时长改变才推进计数；非回绕耗尽返回类型化错误。非法时长先拒绝，保留最后有效配置。

Web 对真实 intrinsic height 插值。进入、退出、快速反向和内容高度变化使用浏览器当前几何。Unmount 的关闭内容 Owner 保留至匹配 revision 的物理终态；重开使旧关闭结果失效。关闭期间内容立即 inert、从 Chrome AX 中省略，动画盒继续可见。零时长、停用动画、动态 Reduce 或宿主不支持插值时立即完成；能力不足同时关闭物理过渡。宿主仅保留一个当前观察，没有每帧 Cage 或已访问配置历史。

外部裁剪壳必须保持 intrinsic block：不接受自身 height/min/max-height/aspect-ratio、非零 padding/border/shadow、flex-grow 或定位。内容内层可自由布局和装饰。默认主题的间距置于内层，`.unstyled()` 去除外观而保留结构与语义；显式样式仍按这些结构约束校验。

SSR 没有物理完成观察，关闭直接采用逻辑内容策略。Blitz 明确拒绝启用高度过渡这一可选 facet，创建和 patch 都在前缀契约下拒绝；不会以原生 check 通过宣称动画支持。`animated(false)` 仍是已声明的 motion facet，原生应省略该 API。普通折叠原生基线保持可用。

## 协议与证据

高度能力使用共享有限 `CollapseConfig`、`CollapseComplete` 和显式 completion capability，HTML Snapshot 为 v5、classic envelope 为 v2。classic End 后采用重新开始客户端 revision 生命周期；整个子树验证成功才原地重置计数，保留编辑器身份、dirty 值、焦点与选择。旧、未来、重复完成分别遵守丢弃、拒绝和单次终态约束。

- `tests/collapse_motion.rs`：4 项，包含延迟 Owner、旧/未来/重复完成、受控静默、KeepMounted、无完成宿主及四主题整树退休回归。
- Glory render 5 项共享与 conformance、2 项有限配置单测；HTML 2 项 full-shape oracle；SSR 同步 1 项、stream 1 项；Blitz 2 项显式拒绝和 prefix-query conformance。
- `examples/web/collapse-motion-acceptance.js`：四主题 CSR/采用、真实中间高度、反向、动态内容、焦点和 Chrome AX、Clear/0/Reduce、受控拒绝及 Owner 退休；另含失败采用不修改、classic End 接管和能力不足回退。

Chromium 148.0.7778.96 的独立同源 `command-next` 快照最终通过 12/12，`errors: []`，报告为 `D:/BuildCache/sapid/collapse-motion-browser-command-next.json`。真实进入中间高度约 29px、目标 128px，退出中间约 98px；反向关键帧使用当前 `calc-size` 系数，内容增长后达到 212px。classic End 的 revision 1 原地采用为客户端 0，dirty 输入、选择和焦点保留，关闭终态只观察一次。四张采用截图已保存并检查绿色暗色 390px 移动视口。

原始 Web19 清理计数器 Disposed 失败保留在 `D:/BuildCache/sapid/collapse-motion-browser-failure.json`；清理回调已改为可失败写入，并增加整树 Owner 退休回归。独立快照的通过记录不冒充 Web19 通过。Rust producer 校验完整有限配置；classic JS 解释有限 HTML edits，WASM 采用端额外验证规范 counter，未引入第二套业务语义解释器。
