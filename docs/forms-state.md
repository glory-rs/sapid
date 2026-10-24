# FormState 与提交快照

`src/form_state.rs` 提供不依赖浏览器、HTTP 或 serde 的表单状态模型。业务值保留在应用自己的
`Cage<T>`；注册、滚动、校验和提交均不隐式复制整份表单。`T` 不要求 `Clone` 或 `PartialEq`。
`FormState` 不读取虚拟列表的数据源，只枚举当前注册且尚未退休的字段。

```rust
use sapid::{ControlValue, FieldOptions, FormState};

let form = FormState::new(scope);
let email = scope.cage(String::new());
let email_field = form.register(
    scope,
    "profile.email",
    email,
    FieldOptions::default()
        .validate(|value: &String| {
            if value.contains('@') { vec![] } else { vec!["请输入有效邮箱".into()] }
        })
        .on_reset(move || email.revise(String::clear))
        .serialize("email", |value| ControlValue::Text(value.clone())),
)?;
```

字段 key 是不超过 128 字节的稳定 ASCII 命名空间，可用字母、数字、`-_.:`；相同 FormState
中拒绝重复 key。字段的提交 name 可以重复，与身份 key 分开；name 最长 256 字节且不能含 NUL。
注册 Scope、业务 Cage、FormState 必须属于同一个 Runtime。

## 输入与可访问状态

`FormField::value()` 返回原业务 Cage；`state()` 返回只读 `Bond<FormFieldState>`。
`FormState::state()` 返回只读 `Bond<FormStatus>`。调用方可将 `state().map(...)` 或
`scope.bond(...)` 计算结果传给已有 Field 的 `.error(...)`、`.invalid(...)`。

接收用户输入提议后，由应用先决定是否写入业务 Cage，再调用 `field.changed()`。
该方法只置 dirty、清除旧错误并取消旧校验，不写业务值、不伪造已接受输入。
`field.touch()` 显式标记离开/接触；`field.validate()` 执行校验并将字段标为 touched。
校验策略（输入时、离开时或仅提交时）由应用明确选择。

外部 Cage 写入不设置 touched 或 dirty，不触发变更/提交回调；框架在正常 flush 边界取消旧校验并
清除过时错误。dirty 表示「reset 后曾接受用户修改」，不做隐藏的初始值深拷贝或全值相等比较。
用户改回旧值仍是 dirty。显式 reset 后清除 dirty/touched/errors。

`FieldOptions::disabled(...)` 与控件的 disabled 应接入同一来源。禁用字段跳过校验和序列化；
readonly 字段仍参加校验及成功控件快照。readonly 的真实输入阻止由现有 Field/控件语义负责。

## 可取消的异步校验

`.validate_async(|value| -> ValidationFuture { ... })` 接收当前一个字段的借用。
工厂自行提取未来真正需要的数据，再返回 `Box::pin(async move { ... })`；框架不要求 Clone。
同步校验先运行，存在同步错误时不启动异步任务。结果为 `Vec<String>`，空集合表示通过。
应用必须选择提供 `Host::spawn` 的真实宿主；没有 Host 返回明确 `AccessError::NoHost`，不会 block_on。

新校验、用户接受修改、外部源变化、reset、取消及字段 Owner 退休都会取消任务并撤销发布权限。
非回绕 ticket 与 `Cage::try_version` 一起校验，源在 flush 前发生 A→B→A 也不能接受旧结果。
异步工厂、同步校验器和 reset/编码器均在字段注册 Owner 下 untracked 执行，不在 registry 借用中
执行。校验器应保持纯计算；异步 I/O 和错误文案由应用提供。任务不保留 UI Scope 的拥有引用。

`FormLimits` 默认允许 1024 个 live 字段，每字段最多 16 条错误、错误字符串共 4096 字节。
超过输出预算时清空错误字符串并设置 `validation_overflow = true`，提交仍失败；UI 应同时检查
`!errors.is_empty() || validation_overflow`，可为 overflow 显示应用自己的固定短文案。
这些是注册/保留结果预算，不声称限制应用回调内部的临时内存。

## 提交屏障与完成权限

`form.submit(|token| { ... })` 对当前 live 注册表做完整校验，等待所有异步校验成功后才执行一次回调。
禁用字段不阻塞提交；错误或过时结果不执行回调。`FormStatus.submit` 依次显示
`Validating`、`Submitting`，然后由应用调用 `token.complete(Ok(()))` 或 `token.complete(Err(message))`
置为 `Succeeded`/`Failed`。失败文案按 `FormLimits.error_bytes` 在 UTF-8 边界截短。

回调在 FormState 注册 Owner 下 untracked 执行。token 可由应用自己的异步任务持有，
`is_current()` 可检查是否仍可用。应用决定如何发送数据，不存在自动网络请求。

新提交、取消、reset、注册成员变化和 Owner 退休撤销旧 token。完成时还会再次核对所有已校验
字段的当前源版本，flush 前的外部变化也不能把过时提交标为成功。重复 complete 返回 false。
调用方持有 token 不会延长 FormState/Owner 的生命周期。
重入校验器撤销当前提交时，原调用返回 `FormError::Superseded`，不会覆盖新的状态。

`form.cancel()` 取消表单当前校验并置 Cancelled。
`form.reset()` 在 `Scope::batch` 中执行所有字段的显式 on_reset 并清除元状态，观察者不能看到
逐字段中间值；即便 reset 回调尝试 flush 也只能观察完整批次。未提供 on_reset 时仅重置元状态，
业务值保持原样。框架不保存隐藏的初始值副本。用户回调 panic 按 Rust 正常展开；不提供自动重试。

## 成功控件快照

`form.serialize(SubmissionLimits { entries, bytes })` 返回 `Vec<FormEntry { name, value }>`，
是显式、宿主无关的应用快照。顺序按当前注册 key 的字典序；每个多值字段保持编码器返回次序。
**这不是 DOM 原生 FormData 的树顺序或自动原生提交协议。** 重复 name 保留为独立条目。

每个字段需显式 `.serialize(name, encoder)`：

- 无编码器、空 name、disabled、`ControlValue::Missing`：不产生条目。
- `Text(String)`：产生一个条目，空字符串仍是成功值。
- `Checked { checked, value }`：仅 checked 为 true 产生条目；mixed 是否成功须由应用明确映射。
- `Values(Vec<String>)`：产生同名条目，空集合不产生条目。
- `UnsupportedFile`：返回 `FormError::UnsupportedFiles`；不会读取 ambient 文件路径。

默认快照预算为 1024 条与 1 MiB 的 name+value UTF-8 字节总和；名称重复的字节每次计入。
超预算返回 Capacity，调用方不会拿到部分结果。编码器可以自行做更严格的数据预算；编码器
返回值的临时分配由应用负责。readonly 不影响成功控件规则。

文件句柄、multipart/urlencoded、submitter 的 name/value、真实 HTTP 提交留给宿主/后续文件协议；
该快照接口不声称拥有浏览器原生文件或完整 HTML FormData 等价能力。

## 验证证据

`tests/form_state.rs` 的 22 项测试覆盖无需 Clone 的字段、外部静默变化、同步错误、异步整表屏障、flush 前 ABA、
任务取消、缺失宿主、Owner 退休后释放回调及 key 重用、取消析构中的重新校验及重入撤销、reset 中间 flush 原子性、错误预算、
提交 token 失效、重复字段/缺失/disabled/readonly/checked/多值与文件拒绝。
这些测试验证状态协议，不替代实际浏览器原生提交、OS IME 或读屏验收。
