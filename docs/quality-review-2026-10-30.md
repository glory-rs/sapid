# Sapid 项目质量评审与改进报告

日期：2026 年 10 月 30 日。评审对象为 Sapid 0.6.0，源码提交 `89bf47c411a5efa087f4908da79a7f5e4747fb2d`。

Sapid 的基础工程质量较好：有限输入预算、受控状态、Owner 生命周期、宿主语义分层以及跨宿主测试都有实际实现。当前更值得投入的方向是恢复完整门禁、消除重复派生计算、补齐焦点可见性，以及把发布验证绑定到不可变提交。本轮没有确认可直接利用的高危漏洞，也没有确认新的高优先级受控状态或资源泄漏缺陷；依赖公告审计尚未完成，不能据此推断依赖没有漏洞。

## 评审范围与证据

由三个并行 agent 分别审查安全、性能与生命周期、架构及可访问性，主 agent 交叉复核并执行检查。覆盖源码、集成测试、样式、公开 API、CI、发布、部署和文档。静态规模为 197 个 Rust 源文件、约 9.16 万行库源码、128 个顶层集成测试文件；规模只描述维护负担，不代表质量分数。

本地使用 Rust 1.98.0，Glory sibling checkout 为 `cd750c8c4f159119fbf5fede688166b42ad95073`，工作区起始时干净。部署配置固定的 Glory 是 `e06bda4db278fc3d981331d6811c284c5a4559db`。下述本地运行结果只证明当前 sibling 组合，不等同于重新验证部署固定版本或 crates.io 消费端组合。

证据分为三类：**运行复现**表示本轮实际执行；**源码确认**表示可以从调用关系或配置确定；**改进建议**表示未证明已经产生故障。P2 表示应安排修复或建设，P3 表示较低优先级；实施顺序另见末尾计划，不把全部建议都视为发布阻断项。

## 优先处理的问题

| 编号 | 优先级 | 方向 | 发现 | 证据 |
|---|---|---|---|---|
| F01 | P2 | 构建可靠性 | Native 示例格式检查失败，完整脚本提前退出 | 运行复现 |
| F02 | P2 | 发布安全 | 检查与发布分别解析可移动 tag，未绑定同一 SHA | 源码确认，存在权限前提 |
| F03 | P2 | 可访问性 | 两个浅色主题焦点环对白色背景不足 3:1 | 数值复现与样式确认 |
| F04 | P2 | 性能 | 合法资源周布局重复展开同一批事件 57 次 | 源码确认与 release 探针 |
| F05 | P2 | 性能 | DataTable 翻页、隐藏列重新验证、过滤和排序全部本地行 | 源码确认，延迟未测量 |
| F06 | P3 | 性能 | Plot 关闭数据表仍生成表格派生结果 | 源码确认，分配量未测量 |
| F07 | P3 | 样式隔离 | 全局 trigger 选择器影响宿主应用 | 最小浏览器用例复现 |
| F08 | P3 | API 易用性 | 部分公共错误类型缺少标准错误接口 | 源码确认 |

### F01 Native 格式检查使完整门禁中断

[check.ps1](D:/Works/glory-rs/sapid/scripts/check.ps1:59) 在 Native 阶段对独立示例 workspace 执行 `cargo fmt --manifest-path examples/native/Cargo.toml -- --check`。本轮在此处非零退出，涉及 43 个不同文件；例如 [accordion_dynamic.rs](D:/Works/glory-rs/sapid/examples/native/tests/accordion_dynamic.rs:33) 的函数体起始花括号格式。

项目 [rustfmt.toml](D:/Works/glory-rs/sapid/rustfmt.toml:1) 包含 nightly 专用配置，而 [CI](D:/Works/glory-rs/sapid/.github/workflows/ci.yml:67) 安装稳定版 1.98.0。运行产生配置被忽略的警告及实际格式差异。警告本身不是本次退出原因，格式差异才是。主库 `cargo fmt --all` 不覆盖这个独立 workspace，因此基础检查通过不能推出 Native 格式检查通过。

**影响与处理：** 完整 `-Web -Native` 脚本在执行后续 Native 示例测试和 Web 阶段前退出。统一格式化工具链和配置，按 CI 使用的工具对所有独立 workspace 格式化；也可以明确固定专用于格式化的 nightly。不要仅删除检查。验收应使用完整脚本，而不是只跑主库格式检查。本轮仅记录问题，没有改动这些文件。

### F02 发布验证没有冻结产品提交

[release.yml](D:/Works/glory-rs/sapid/.github/workflows/release.yml:20) 将 tag/ref 传给检查作业，发布作业随后在 [另一处 checkout](D:/Works/glory-rs/sapid/.github/workflows/release.yml:51) 再解析同一个字符串。[release.py](D:/Works/glory-rs/sapid/scripts/release.py:154) 只比较发布时的 HEAD 与发布时的 tag，没有比较通过检查时的 SHA；浏览器 provenance 在上传前也没有与发布 HEAD 做强制比较。

**触发前提：** 有权更新 tag 的人员或凭证在两次 checkout 之间把 tag 从 A 移到 B。此时 B 可以沿用 A 的检查结果和浏览器证据。手动 `workflow_dispatch` 输入 tag 同样存在这一条件。本轮只读 GitHub 查询得到 rulesets 为 `[]`，现有 `v0.6.0` release 为 `immutable:false`；这不等于任意用户能修改 tag，也没有证明发生过利用。

**改进：** 在流程开始时只解析一次 tag 到完整 commit SHA，通过 job output 传给 checks 与 publish。两个作业都 checkout 该 SHA；发布前校验检查 provenance、`browser-provenance.product_commit` 与当前 HEAD 相同。tag 与冻结 SHA 不同应拒绝发布。增加流程测试，模拟“检查 A 后 tag 变成 B”，要求在 registry 上传之前停止。

### F03 浅色焦点环对比度不足

[主题预设](D:/Works/glory-rs/sapid/src/theme/presets.rs:108) 使用蓝色 `#74c0fc`，绿色预设在 [同文件](D:/Works/glory-rs/sapid/src/theme/presets.rs:133) 使用 `#8ce99a`。[按钮和 Checkbox 样式](D:/Works/glory-rs/sapid/styles/sapid.css:22)、Switch、Menu 等把 `--sp-ring` 用作有偏移的焦点 outline。

| 浅色主题 | Ring | 白色 surface 上的对比度 |
|---|---|---|
| Blue | `#74c0fc` | 1.964800:1 |
| Green | `#8ce99a` | 1.475001:1 |

这些作者自定义的焦点指示颜色低于 3:1。这里比较的是焦点环与相邻背景，并非要求所有控件边框或不同状态之间都达到 3:1。[W3C 对 SC 1.4.11 的解释](https://www.w3.org/WAI/WCAG22/Understanding/non-text-contrast.html)及[焦点可见性的说明](https://www.w3.org/WAI/WCAG22/Understanding/focus-visible.html)明确包含此类焦点状态。

**改进：** 为浅色模式选择更深的 ring，或采用经验证的双色环；保留暗色模式合适的颜色。扩展[已有主题对比度测试](D:/Works/glory-rs/sapid/tests/fields.rs:218)，覆盖 ring 与 canvas/surface/input，并在实际键盘导航中检查焦点环没有被裁剪。通过 `palette_css` 同步生成样式，避免直接修改生成 CSS 后与 Rust 预设分叉。

### F04 资源周布局重复展开事件

[layout_timeline](D:/Works/glory-rs/sapid/src/schedule_layout/mod.rs:133) 先完整展开一次，然后对每个资源、每个日期调用 `layout_day`；[layout_day](D:/Works/glory-rs/sapid/src/schedule_layout/mod.rs:218) 每次又完整展开、分配并排序同一数据。当前最多 8 个可见资源，7 天视图因此调用 `expand` 共 `1 + 8 × 7 = 57` 次。注册资源上限是 64，不能将其误当作可见资源上限。

[组件验证](D:/Works/glory-rs/sapid/src/schedule/resources/mod.rs:558)以及[渲染准备](D:/Works/glory-rs/sapid/src/schedule/resources/timeline.rs:30)还会分别执行展开和布局，进一步增加一次更新的计算成本。

**本轮测量：** 独立 release 探针构造 512 个事件，其中 112 个年度长事件、400 个错峰短事件；8 个可见资源、7 天、512 occurrences、56 columns、1120 slots、1184 placements、最大 15 lanes，全部通过现有预算。每个操作预热 3 次、测量 30 次，三轮进程结果如下。

| 运行 | 单次 expand 均值 | 单资源单日布局均值 | 资源周布局均值 |
|---|---|---|---|
| 初轮 | 1.865 ms | 1.862 ms | 105.895 ms |
| 复测一 | 2.000 ms | 2.260 ms | 134.531 ms |
| 复测二 | 1.872 ms | 1.841 ms | 140.148 ms |

这是本机纯模型 CPU 时间，不是浏览器帧时间；不是普通日程的平均负载，也没有测量修复后的加速比。初轮与扩展检查并行，复测在该检查退出后执行，数值仍有系统噪声。可以确认的是合法较重输入下重复展开成本显著，与源码中的 57 次调用一致。

**改进：** 一次生成经过验证的 occurrence 快照，内部日布局接受该快照；按资源和日期分桶后生成 placements。对同一数据版本复用快照，避免全局无限缓存。保留公开 `layout_day` 的独立校验契约。验收要比较几何、排序、冲突和 stale occurrence 行为，并在同一 fixture 上记录前后耗时、分配量及组件更新耗时。

### F05 DataTable 变化粒度过粗

[DataTable 的 plan source](D:/Works/glory-rs/sapid/src/data_table/mod.rs:436) 依赖整个 query，其中既有 filter/sort/page，也有纯展示字段 hidden。[local_data_plan](D:/Works/glory-rs/sapid/src/data_table/model.rs:214) 每次完整校验记录，按 cell 创建小写字符串进行过滤，排序全部匹配行，最后才分页。

因此隐藏列或翻页也会重复遍历有效上限内的全部数据，时间成本包括 payload 校验、过滤以及可能的 `O(n log n)` 排序。现有行窗口化能减少挂载节点，不能消除这些模型计算。本轮没有测定此处的实际延迟，不能把它称为已复现卡顿。

**改进：** 将数据版本、过滤/排序、分页、列显示拆成独立派生层；仅 filter/sort/data 变化重建匹配索引，page 变化切片，hidden 变化更新展示。必要时在数据接纳后缓存规范化文本，但应计入总内存预算。公开纯函数继续校验不可信输入；组件内部复用已验证数据。验收使用 4096 行、多列和接近 payload 预算的输入，统计翻页、筛选及隐藏列的耗时和临时分配。

### F06 Plot 不显示数据表时仍构造表格

[Plot 的模型派生](D:/Works/glory-rs/sapid/src/plot_widget.rs:247) 无条件执行 `plot_table::tables`，直到[挂载阶段](D:/Works/glory-rs/sapid/src/plot_widget.rs:479)才判断 `show_table`。`.data_table(false)` 能省去表格 UI，但没有省去表格格式化和字符串存储。

**改进：** 关闭表格时跳过表格派生，或将表格放入独立、按需计算的派生层。保留图形辅助名称、选中详情和必要的可访问替代信息。用 1024 cells 矩阵更新场景测量分配量；不要未经测量承诺收益比例。

### F07 CSS 选择器越过库命名空间

[sapid.css](D:/Works/glory-rs/sapid/styles/sapid.css:24) 中 `[class$='-trigger']` 没有限定 Sapid，也没有排除 `sapid-unstyled`。宿主自己的 `<div class="app-menu-trigger">` 会获得 `cursor:pointer`，部分禁用或 unstyled trigger 也可能呈现错误光标。

本轮在 Chromium 156.0.8078.4 的最小 CSS fixture 中读回宿主元素光标：导入前为 `auto`，导入后为 `pointer`。同一用例还读回了两种浅色按钮的实际 2px 焦点环颜色；它是样式隔离复现，不替代完整 widget 交互验收。

**改进：** 使用明确的 Sapid trigger 类，统一应用 unstyled 和 disabled 条件。增加一页宿主隔离 fixture，比较导入样式前后宿主类及 unstyled 控件的 computed style。这类隔离回归比单纯检查 CSS 字符串更有价值。

### F08 公共错误类型缺少标准接口

[DataTableError](D:/Works/glory-rs/sapid/src/data_table/model.rs:92)、[TreeError](D:/Works/glory-rs/sapid/src/tree_model.rs:57)、[ChartError](D:/Works/glory-rs/sapid/src/chart_model.rs:62) 有 `message()`，没有 `Display` 和 `std::error::Error`。调用者不能直接通过 `?` 传播到标准 boxed error 或常见应用错误链，和库内其他已实现这些 trait 的错误类型不一致。

**改进：** 复用现有 message 实现标准错误接口，不改错误枚举语义。增加一个公共 API 编译示例验证 `?` 传播即可，无需为每个枚举复制大量同构测试。

## 安全与供应链改进

### 构建工具与凭证

[部署 workflow](D:/Works/glory-rs/sapid/.github/workflows/deploy-showcase.yml:74) 下载固定版本的 wasm-bindgen 二进制归档后直接解压执行，缓存命中只检查文件存在，没有审查过的 hash 校验。固定版本不能检测同名发布资产被替换。建议固定 SHA-256，并校验下载文件以及缓存中实际使用的二进制。

[deploy-pages.ps1](D:/Works/glory-rs/sapid/scripts/deploy-pages.ps1:27) 在 Cloudflare token 已存在的环境中通过 `npx --yes wrangler@版本` 动态安装执行工具，传递依赖没有 lockfile。建议在没有部署 token 的准备阶段按 lockfile 安装，在部署阶段只调用已安装工具。第三方部署工具运行时本来就需要 token；目标是减少带凭证时的安装和依赖解析面。这两项是供应链加固，不代表已经发生入侵。

### 持续依赖公告审计

主库、Web、Native 三份 lockfile 分别含 40、243、459 个 package 记录，包含不同宿主及开发依赖，不能把数字都算作默认生产依赖。CI 未接入 RustSec 公告检查。本轮环境没有安装 cargo-audit，因此没有完成当前公告数据库的漏洞匹配。

建议在 CI 与定时检查中覆盖三份 lockfile，记录公告数据库 revision、依赖路径及例外到期时间。静态组件预算、Rust 的内存安全或严格 Clippy 都不能替代依赖公告审计。[RustSec 官方项目](https://github.com/RustSec/rustsec)说明了 cargo-audit 与公告数据库的职责。

### 安全边界已有明确设计

值得保留的设计包括：`unsafe_code = "forbid"`、JSON 解析前的字节/深度限制、文件 FileToken 与当前选择成员校验、结构化富文本、有限编辑历史、Citation 的 HTTP(S)/fragment allowlist，以及将 chat 的授权和执行交给应用。

[Anchor](D:/Works/glory-rs/sapid/src/anchor.rs:70) 原样接收 href，但[文档](D:/Works/glory-rs/sapid/docs/anchor.md:37)已经明确把 URI 策略交给应用。建议提供可选共享 URL policy，降低调用者传入不可信地址的风险；这不是本轮确认的无条件 XSS 漏洞。Showcase 可以补充经过 WASM 和样式验收的 CSP、`frame-ancestors`，属于纵深防御。

## 测试体系与可维护性

### 把已有验收接入自动门禁

[check-showcase.ps1 的 Browser 分支](D:/Works/glory-rs/sapid/scripts/check-showcase.ps1:98) 有 Theme Studio、主题导出、导航、路由、样式和 Mantine API 专项；[部署 workflow](D:/Works/glory-rs/sapid/.github/workflows/deploy-showcase.yml:85)没有传 `-Browser`，之后仅显式执行 showcase、cache、palette-order 三个浏览器脚本。主 CI 也不执行上述新增 showcase 专项。

建议将已有检查整理为一个可从 CI 调用的完整 runner，按 PR 和部署风险分层执行，避免“本地有脚本、提交后没有自动保障”。验收报告应列出实际 suite 清单，而不只给一个总成功状态。不要把未接入的脚本数误当作已发生的功能回归。

### 验证公开 MSRV 承诺

[Cargo.toml](D:/Works/glory-rs/sapid/Cargo.toml:5) 声明 Rust 1.94，当前 CI 只验证 1.98.0。建议至少在声明的最低版本上检查默认和全部库 features，并另以当前稳定版检查宿主组合。这里是兼容性证据缺口，本轮没有证明 1.94 无法构建。`rust-version` 的含义及其支持承诺见 [Cargo 官方文档](https://doc.rust-lang.org/stable/cargo/reference/rust-version.html)。

### 建立运行时性能基线

已有 [WASM size 记录](D:/Works/glory-rs/sapid/docs/wasm-size.md:1)及 sibling Glory 中的独立矩阵，避免重复建设。它们主要测传输体积，不替代更新耗时和内存测量。建议增加少量有代表性的运行时基线：资源周布局、DataTable 筛选/翻页、Tree 展开与筛选、Plot 更新、编辑器 undo/redo，以及反复挂载卸载后的存活 Owner/节点/查询数量。

先保留稳定输入和结果正确性，再记录模型计算、浏览器更新、分配量及 WASM 体积。预算内输入也可能有高计算成本；不要为了跑分快而取消输入校验或缩小已经承诺的能力。

### 文档与证据应随版本归档

README 仍有“publishing / standalone versioned Glory dependency pending”的段落，与当前版本、release 配置及发布说明不一致。部分历史验收记录依赖作者机器上的 `D:/BuildCache/...` 链接，外部维护者不能直接取得这些证据。

建议清理发布状态文案，区分本地 sibling 开发、CI 固定 revision、registry 消费三个组合。将关键结果摘要、源码 SHA、工具版本和 artifact hash 随 CI/release 归档，仓库文档链接到可访问的构建或发布证据。本轮报告保留本机证据路径用于当前工作区复核，不把它当作永久公共证据。

## 本轮验证结果

| 验证 | 结果 | 证据 |
|---|---|---|
| `scripts/check.ps1` | 通过，含默认/全部 features 测试、共享 Glory 测试及严格 Clippy；2702 次通过、0 失败、2 次忽略 | [基础日志](D:/BuildCache/sapid/review/check.log) |
| `scripts/check.ps1 -Web -Native` | 在 Native 示例格式检查退出；此前测试阶段通过不代表整个脚本通过 | [失败日志](D:/BuildCache/sapid/review/check-web-native.log) |
| Glory shell/accesskit/raster 测试和严格 Clippy | 完整脚本中已执行通过；该 Native 测试阶段为 277 次通过、2 项忽略 | 同上 |
| Native 示例全部 features 单独测试 | 155 次通过、0 失败；仅绕开格式阶段继续收集行为证据 | [测试日志](D:/BuildCache/sapid/review/native-tests.log) |
| Native 示例全部 features / all-targets 严格 Clippy | 通过 | [Clippy 日志](D:/BuildCache/sapid/review/native-clippy.log) |
| Web 扩展完整检查，指定 wasm-bindgen 0.2.129 | 通过，含 Web native/wasm32 严格检查、WASM 构建、SSR 与 stream 夹具生成 | [固定工具版本后的日志](D:/BuildCache/sapid/review/check-web-pinned.log) |
| widget inventory / showcase catalog 同步检查 | 通过，268 来源条目、149 工作项 | 本轮执行对应 `--check` |
| 发布计划、发布脚本测试、静态资源测试 | 计划通过，脚本测试 11 项、静态资源测试 5 项通过 | 本轮执行 `release.py`、`test_release.py`、`test-site-assets.py` |
| 日程 release 性能探针 | 合法 fixture 校验通过，完成三轮测量 | [探针源码](D:/BuildCache/sapid/review/perf-probe/src/main.rs)、[初轮](D:/BuildCache/sapid/review/perf-probe.log)、[复测一](D:/BuildCache/sapid/review/perf-repeat-1.log)、[复测二](D:/BuildCache/sapid/review/perf-repeat-2.log) |
| 焦点颜色与宿主 CSS 隔离 | 数值及最小 Chromium fixture 复现 | [对比度](D:/BuildCache/sapid/review/focus-contrast.json)、[浏览器结果](D:/BuildCache/sapid/review/css-probe.json) |

测试数为执行次数，默认/全 features、不同脚本会重复执行相同用例，不能相加当作互异用例数。被忽略的用例没有计为通过。Native 协议、布局和 AccessKit 投影不替代实际 OS IME、读屏语音或真机验收。

本轮第一次 Web 扩展构建使用了本机默认的 wasm-bindgen 0.2.127，在工具版本检查处停止；它属于验证环境问题，不列为项目缺陷。后续改用项目已有的固定 0.2.129 工具重跑并通过。

额外选择的 Field、DataTable、ResourceSchedule、Plot 四组完整浏览器回归未执行：自动审批拒绝了启动/停止本地验收服务器的命令，仅返回 `blocked by policy`，没有提供更具体原因。没有把旧夹具或历史浏览器报告算作本轮通过；上述最小 CSS 浏览器用例已实际执行。RustSec 公告审计、最低 Rust 1.94 构建及部署固定 Glory SHA 的隔离复验也未执行。

## 分阶段实施计划

| 顺序 | 工作包 | 完成标准 |
|---|---|---|
| 1 | 统一格式工具链，恢复完整 Native/Web 门禁 | 干净 checkout 执行完整脚本成功，已有忽略项独立列出 |
| 2 | 冻结发布 SHA，绑定浏览器 provenance | 模拟 tag 移动或证据 SHA 不同会在上传前失败 |
| 3 | 修复浅色 ring 与全局 CSS 选择器 | 主题对比度检查、真实键盘检查、宿主隔离 fixture 通过 |
| 4 | 资源日程一次展开并复用快照 | 原有几何及交互回归通过，相同探针给出前后结果 |
| 5 | DataTable 分层派生、Plot 按需表格 | 翻页/隐藏列不再全量过滤排序，关闭表格不再生成表格内容 |
| 6 | 补齐 CI 专项、MSRV、依赖公告与工具完整性 | suite 清单可见，声明版本和三份 lockfile 均有结果 |
| 7 | 公共错误 trait、文档与证据归档 | 应用错误传播示例可编译，版本与 artifact 来源一致 |

初始评审阶段只新增本报告，未修改生产实现、测试断言、依赖、发布流程或样式。初始性能探针与运行日志保存在 `D:/BuildCache/sapid/review`。

## 修复交付记录

2026 年 10 月 30 日追加：按上述七批实施计划完成修复和回归测试，分为以下独立 PR。初次交付时均已转为可评审状态；后续复核及合并记录见末节。已有的 [#4](https://github.com/sapid-ui/sapid/pull/4) 是本轮起点，其余六个修复 PR 原以 #5 为基线。

| 批次 | 对应问题与修复 | PR |
|---|---|---|
| 1 | F01：稳定格式配置、独立 package 门禁、Native/DSL 的 D 盘 target/TEMP 初始化与隔离 | [#5](https://github.com/sapid-ui/sapid/pull/5) |
| 2 | F02：冻结发布 SHA、完整证据绑定、每次上传重试前检查 tag、追加而不覆盖历史发布证据 | [#9](https://github.com/sapid-ui/sapid/pull/9) |
| 3 | F03/F07：浅色 ring 对比度、真实 Tab 检查、Sapid trigger 样式隔离 | [#6](https://github.com/sapid-ui/sapid/pull/6) |
| 4 | F04：资源及普通日程复用当前 occurrence/几何快照，保留公开输入校验 | [#7](https://github.com/sapid-ui/sapid/pull/7) |
| 5 | F05/F06：DataTable 接纳数据与过滤/排序分层、翻页和列显示复用索引、Plot 按需生成表格 | [#11](https://github.com/sapid-ui/sapid/pull/11) |
| 6 | 工具 hash/lockfile/receipt、完整九组 showcase runner、MSRV、三份 lockfile 审计、Theme Studio 驱动时序回归 | [#10](https://github.com/sapid-ui/sapid/pull/10) |
| 7 | F08：标准错误 trait 与真实 `?` 传播回归、发布状态及证据边界文档 | [#8](https://github.com/sapid-ui/sapid/pull/8) |

修复组合在隔离 checkout 中验证，依赖固定为部署使用的 Glory `e06bda4db278fc3d981331d6811c284c5a4559db`，未改动其他会话正在维护的 sibling。以下是修复后的本地结果，与前文初始评审结果分开记录：

| 验证 | 修复后结果 | 证据 |
|---|---|---|
| 完整 `check.ps1 -Web -Native` | 通过，含严格 Clippy；3,176 次测试执行通过、0 失败、4 次忽略；重复 feature 配置不计为独立用例 | [日志](D:/BuildCache/sapid/fixes/integration/full-check-final.log) |
| 实际 CSR/hydration 和 Native DSL | 通过；新增路径回归包含真实 MSVC 链接 | [日志](D:/BuildCache/sapid/fixes/integration/dsl-check-final.log) |
| 组件浏览器矩阵 | 121/121 组通过，1,276 条报告，错误为空 | [结果](D:/BuildCache/sapid/fixes/integration/browser-results.json) |
| Showcase | 9/9 组通过，含 Theme Studio 的旧驱动失败负对照与修复后正常/8 倍 CPU 限速回归 | [结果](D:/BuildCache/sapid/fixes/integration/showcase-acceptance-final/results.json) |
| 发布回归 | 26/26，通过真实本地 Git tag 漂移及上传重试场景；未实际发布 | [日志](D:/BuildCache/sapid/fixes/integration/release-tests-final.log) |
| Rust 1.94.0 | 默认及全部 features 的 all-target 检查通过 | [组合全部 features 日志](D:/BuildCache/sapid/fixes/integration/msrv-all.log) |
| 工具完整性与 runner | 10 个完整性回归、4 个 runner 故障回归、真实安装/缓存篡改拒绝及审计器回归通过 | [完整性日志](D:/BuildCache/sapid/fixes/infra/parent-tool-tests.log)、[runner 日志](D:/BuildCache/sapid/fixes/infra/parent-runner-tests.log) |

三份 lockfile 当前审计未发现已知漏洞；Native Unix 依赖链仍有 4 条未维护公告（ttf-parser、zbus_names、zvariant、zvariant_derive）。未使用忽略列表，也未按 Windows 平台删掉跨平台结果；迁移需上游宿主引擎配合，已在 #10 的 `docs/dependency-quality.md` 记录。Native 协议测试仍不等同于真机读屏/IME 验收。

性能对照保持相同输入和结果：资源周布局三次均值从 105.895/134.531/140.148 ms 变为 2.269/2.228/2.175 ms；实际 Memory App 的 4096 行、8 列、8 MiB 表格翻页均值范围从 5.400–6.512 ms 变为 0.234–0.328 ms，列显示从 4.750–9.320 ms 变为 0.043–0.079 ms。均为本机观测，不是 CI 时间阈值；另有确定性工作次数回归验证是否重复派生。

初次交付时远端 CI 尚在重新运行；后续已全部通过，具体复核结果见下文。可选 URL policy、Showcase CSP 及更广泛的 Tree/编辑器性能基线仍属后续建设建议，不属于已修复的 F01–F08。

## GitHub PR 复核与合并

同日根据后续授权复核全部 9 个未合并 PR（#4–#12）。三个 agent 独立复核发布/供应链、性能/状态语义及锁文件策略，主 agent 核对最新 head、CI、评论和合并依赖。未发现未解决的 review threads。#5–#11 最新 head 检查全部成功；#4 原有 Native 格式失败由 #5 修复，因此将包含 #4 的 #5 改为以 main 为基线并保留 merge 历史合入，GitHub 同时确认 #4 已合并。其余六个修复 PR 随后依次合入 main，组合源码树与前述本地完整验收的隔离 checkout 相同。

新增 [#12](https://github.com/sapid-ui/sapid/pull/12) 起初未达到合并条件，已在本轮补齐：

- 根库锁文件取消跟踪后，统一为 check、DSL、MSRV、审计和发布入口准备缺失锁；使用 Rust 版本兼容解析，核验 target，并保留已有本地锁及两份应用锁的原字节。
- 审计归档实际扫描的锁文件及摘要；缺失应用锁仍失败。
- 成功发布验收归档根锁并将 SHA-256 纳入 provenance。发布作业在完整来源/证据/tag 门禁通过后、任何 Cargo 操作前恢复同一根锁；已有锁不一致、证据缺失或篡改均拒绝，旧 tag 的跟踪锁保持不变。
- 两处稀疏工具 checkout 增加新 helper，并把根锁作为按 run/attempt 命名的 release 证据附件保留。

#12 的修复 head 为 `bd5a7e5275fca99ff740dbef2cc50ab1878b209f`。5 条真实缺锁回归、34 条发布回归、实际 Sapid Rust 1.94 默认/全部 features 的 all-target 检查、真实审计回归及最终提交的两份真实 sparse checkout 检查均通过。实际 Sapid 打包及源码 provenance 检查通过，未执行 registry 发布。

远端 #12 的全部 5 项 checks 成功：[完整门禁与 121 组组件浏览器验收](https://github.com/sapid-ui/sapid/actions/runs/41013370915)、[showcase 九组验收](https://github.com/sapid-ui/sapid/actions/runs/41013370868)、[MSRV 两组与依赖审计](https://github.com/sapid-ui/sapid/actions/runs/41013371109)。下载成功 artifact 后再次确认：`complete: true`、121 completed suites、1,276 reports、错误为空，报告/runner/根锁摘要一致。CI 记录的 GitHub PR 合并提交为 `ebd23da85129bdc3413c5dacbad4b6df1aa35b8c`，其源码树与待合并 head 完全一致；根锁 SHA-256 为 `59595540bc7dfe4756f6dd9f344ef09442f3c3cada8b86b01969a63dbc5429d0`。

最终 #4–#12 全部合并，当前未合并 PR 数为 0；main 为 `43f5122da8b7043ed44c83d71d08e280b9db9c5e`，其源码树与 #12 通过检查的源码树一致。合并后的 main 自动流程会重新运行，不将其尚未结束的运行计为通过。4 条 Native 依赖维护公告仍按前述文档保留并追踪。

本次证据：[根锁及远端验收核验](D:/BuildCache/sapid/fixes/pr12-review/ci-evidence-validation.json)、[最终提交 sparse checkout](D:/BuildCache/sapid/fixes/pr12-review/final-sparse/bd5a7e5/summary.json)、[真实 Sapid 打包](D:/BuildCache/sapid/fixes/pr12-review/final-package.log)。
