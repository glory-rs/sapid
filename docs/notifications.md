# Notification 与 Toaster

`notification(key)` 是独立具名消息，沿用 Alert 的 severity 与独立
`Announcement::{Off,Polite,Assertive}`。可加入一个真实 action Button 和独立关闭按钮。
独立 action 只调用应用回调；是否关闭由应用的 visible 源决定。
`toaster(key)`（`notifications(key)`）提供应用局部、Owner 所有的有限队列。
不注册全局单例、不夺取焦点、不安装网络或路由拦截器。

```rust
use sapid::{notification_request, toaster};

let messages = toaster("application").on_ready(|handle| {
    let _token = handle.show(
        notification_request("saved-document", "Saved", "Changes are stored.")
            .action("Undo", || { /* application command */ }),
    ).unwrap();
});
```

默认容量 32、同时展示 4，显式范围为 1..=64 和 1..=min(8,capacity)。
key 最多 64 字节，title/action label 256、message 4096；接受非空 canonical
文字，拒绝 NUL/CR。show 对完整 key 去重，在原 FIFO 位置建立新 lifetime，
旧 token 立即失效并收到 Replaced。满队列仍可替换已有 key，新 key 被原子拒绝。
update 保留同一 row/实际控件身份；dismiss/clear/action 在回调前移除请求。
已完成 token 返回 false，其他 Toaster 的 token 返回 typed error。
回调在注册 Owner 下 untracked 执行，无图或队列借用，可重入 show/clear。
退休仅释放 captures，不调用业务完成回调。

默认永久显示。timeout 必须显式提供 `NotificationClock::new(now,sleep)`，
时长大于零且不超过 24 小时。now 返回单调 Duration，sleep 返回可取消本地 future；
应用负责宿主时钟与唤醒服务。所有可见 deadline 共享一个 OwnerDelay/task，
队列中的消息在进入可见区后才开始计时。暂停保存剩余时间；继续使用剩余时间。
过早完成的 sleep 不使消息提前过期，也不在当前 task 回调中同步构造下一个 task。
任务取消及用户 future 析构均在内部借用之外，旧 wake 不能复活已退休消息。

focus 与显式 `handle.pause(bool)` 可暂停计时。可选 `.pause_on_hover(true)`
需要宿主 PointerWithin；默认 false 的策略保持不变。新版 Glory 原生已有该共享协议，
本组件的专门原生悬停验收仍与默认计时/布局证据区分。只保留当前 live focus/hover
消息的数字身份，row Owner 清理释放对应暂停，即使退休后没有 terminal focus
事件。无已完成消息、已访问来源或每帧历史。Toaster 不负责位置，示例用 Affix
放置在 viewport 角落；也可直接置入普通布局。

模型测试 `tests/notifications.rs` 10 项通过，包括单 task 的 Executor、精确 fake
clock 残余时间、过早完成、满队列替换、动作顺序、Owner 清理和 future 析构重入。
`examples/native/tests/notifications.rs` 两项检查四主题真实布局、动作及更新身份，
原生日志见 `D:/BuildCache/sapid/notifications-native-final.log`。原生未承诺 hover
兴趣或 OS 屏幕阅读器公告。Web fixture 为 `notifications.html`，四主题 CSR/
hydration 与实际时钟验收由 `notifications-acceptance.js` 执行。

固定 Web13 的 Chromium **148.0.7778.96** 实际 **10/10**，errors=[]，报告
`D:/BuildCache/sapid/notifications-browser-web13.json`。八组主题/CSR/hydration
检查实际 polite AX、焦点不被夺取、更新身份、同 key 替换、动作/关闭、手机宽度
与 Owner 退休。另两组检查真实时钟的 hover/focus/manual 暂停、排队后晋升开始
计时和退休后 late wake。精确残余毫秒与单 task 断言仍以 fake clock 模型测试
为证据，不从墙钟测试推断。green-dark hydration 截图已目视检查。
