# Sapid 全量验收记录

2026-10-29 的 Glory 新 `main` 同步、`Vary` API、引擎固定版本及重新执行的验证，
见 [Glory main 迁移记录](glory-main-migration-2026-10-29.md)。下文保留原目录验收批次。

本文的 `D:/BuildCache/...` 链接是原始执行机器上的历史产物位置，不是仓库文件或公开下载地址。
跨机器复核时，按 [验收证据归档说明](../RELEASING.md#acceptance-evidence) 获取对应 CI run 或
GitHub Release 的报告及来源哈希；复跑结果应另记日期、产品/测试脚本 revision 与工具版本。
Chrome AX、AccessKit 与自动输入协议证据不能替代 OS IME、读屏语音或实际设备验收。

日期：2026-10-25。冻结目录的 268 个来源条目已归并为 149 项工作，
当前 149 项均已实现并记录逐项证据，覆盖 14 个阶段。
[路线图](widgets-roadmap.md)、[canonical inventory](widget-inventory.json)、
[逐项能力账本](widget-capabilities.md)和 [gallery](../examples/web/gallery.html)保持同步。
完成范围是各项明确的有限契约；可选模块、宿主拒绝与未验收平台分别记录。

后续新增的 Mantine 风格 Glory + Sapid 展示站、149 个文档路由及独立构建/验收证据，
见 [showcase-site.md](showcase-site.md)。下文的 8794 组件矩阵与冻结记录继续保留。

## 最终检查

| 检查 | 结果 | 原始证据 |
|---|---|---|
| Sapid 默认 / 全 feature 测试与严格 Clippy | 通过 | [Web 综合门禁](D:/BuildCache/sapid/final-check-web.log) |
| Glory style/core/render/ui/html/ssr 全 feature 测试与严格 Clippy | 通过 | 同上 |
| Web native / wasm32 严格检查、WASM、四主题 SSR、classic stream 夹具 | 通过，当前 HTML FORMAT13 | 同上 |
| 两个 genuine engine forks、六个引擎/布局包的固定来源与 fork-owned 回归 | 通过，439 次测试执行，0 失败 | [fork 门禁](D:/BuildCache/sapid/final-engine-forks.log) |
| Apace shell/accesskit/raster 全目标回归与严格 Clippy | 通过，216 项通过，2 项原有忽略 | [原生门禁](D:/BuildCache/sapid/final-native-acceptance.log) |
| Sapid 全部原生示例、格式与严格 Clippy | 84 个测试文件、146 项通过，0 失败，严格检查通过 | 同上 |
| Chromium 148.0.7778.96 组件矩阵 | 120 组、1270 条报告，errors 为空 | [最终 JSON](D:/BuildCache/sapid/final-browser-matrix.json) |
| 最终原生证据文案同步后的 gallery | 额外 12 条报告，errors 为空 | [gallery JSON](D:/BuildCache/sapid/final-gallery-browser.json) |
| 最终 README 的默认 / 全 feature doctest | 各 5 项通过 | [doctest 日志](D:/BuildCache/sapid/final-readme-doctests.log) |
| 清单、能力表及文件证据 | 149 项同步，1605 条文件证据路径均存在 | [路径审计](D:/BuildCache/sapid/evidence-audit-current.json) |

不同 feature 配置与 focused 回归会重复执行部分测试；执行数不作为互异用例数。
默认 normal 依赖图仍只有 Glory facade/core/render/style/ui，未引入 serde、libm 或 Apace；
见 [依赖图](D:/BuildCache/sapid/final-default-dependency-tree.log)。
工具版本为 Rust/Cargo 1.98.0、wasm-bindgen 0.2.129。
编译产物、Cargo 缓存、浏览器 profile、临时文件与测试日志均位于 D 盘。
所有检查使用 Cargo metadata 核对实际 target 目录。

## 失败保留与修复

主浏览器矩阵在第 119 组资源日程的测试就绪条件处中断，成功前缀为 118 组、1249 条报告。
40 次复现中有 3 次在真实按钮仍禁用时调用程序点击，事件没有发出；补足启用和最终计数等待后
40/40 次通过。FIFO、两次提案、受控拒绝与已接受值断言均保留，生产实现和公开构建文件未变。
恢复脚本先核对 118 组源码哈希、742 个静态文件哈希和 Chromium 版本，再完成最后两组。
最终 JSON 明确记录 checkpoint、retry 和源码哈希；原始失败保留在
[失败日志](D:/BuildCache/sapid/final-browser-before-resource-fix.log)与
[成功前缀](D:/BuildCache/sapid/final-browser-before-resource-fix.json)。
完整诊断见 [资源日程](resource-schedule.md)。

原生回归修复了两个实际缺陷：ModalsManager FIFO 关闭后丢失最初触发器焦点，
以及 OverflowList 移动条目后隐藏旧容器残留布局引用。
前者只传递本次关闭 managed 层的有效 fallback；后者在真正的
[glory-rs/apace 提交 c127fd96](https://github.com/glory-rs/apace/commit/c127fd9699a89ff8ecba0c8d8c9eccb3330627af)
中修复，依赖、锁文件、来源记录一起固定完整 SHA，没有 vendor 或全局 patch。
原生真实焦点、选择区、preedit、控件身份、布局、命中、像素和 Owner 清理分别验收；
原始失败及边界见 [ModalsManager](modals-manager.md)、[OverflowList](overflow-list.md)
与 [引擎维护](../../glory/docs/engine-forks.md)。

## 构建与快照

主浏览器矩阵的 742 文件哈希清单为
[full-browser-artifact-hashes.json](D:/BuildCache/sapid/full-browser-artifact-hashes.json)，
清单 SHA256 为 `12a4d1312e92e25656b46fcbf91db1e1ef39ff33c1edf281411667cf9ca2d3bf`。
随后仅更新 gallery 的原生证据文案并补跑其 12 条验收；WASM、SSR、脚本和样式未变，
差异与最终清单见 [gallery 构建审计](D:/BuildCache/sapid/final-gallery-artifact-audit.json)。
最终第一方源码及逐文件哈希保存在 `D:/BuildCache/sapid/final-source`；
先前不可变批次、截图和失败日志继续保留。

从 Sapid 目录复现综合检查：

```powershell
./scripts/check.ps1 -Native -Web -Bindgen D:/BuildCache/codmin-glory/tools/bin/wasm-bindgen.exe
python scripts/widget-plan.py --check
python scripts/capability-gallery.py --check
```

另在设置好 AGENTS.md 所要求的 D 盘 Cargo/target/临时目录后，于 Glory 目录执行
`python scripts/check-engine-forks.py --test`。启动静态服务器服务
`D:/BuildCache/sapid/web` 到 `127.0.0.1:8794` 后运行浏览器矩阵：

```powershell
$env:SAPID_PLAYWRIGHT = 'D:/Works/glory-rs/glory/tests/playwright/node_modules/playwright'
node scripts/browser.cjs
```

两项忽略测试需要实际桌面或 Windows UIA 客户端，本次没有把它们算作通过。
原生布局/像素与 Chrome AX、AccessKit 投影均不替代 OS IME、读屏语音或实际设备验收。
RichTextEditor、ModelSelector、文件选择及其他受限原生 facet 的明确拒绝继续保留，
具体支持范围以逐项能力账本为准。
