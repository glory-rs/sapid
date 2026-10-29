# Glory main 同步（2026-10-29）

Sapid 同步到 Glory `3c0c6576a38ddbd48211012f2a60ecaa7140816e`。
该提交先在 `v2` 提交并推送，再随 GitHub 原生分支重命名成为新的默认 `main`。
旧 `main` 的 `0234a9da52c1543ffdf0737e2df0050d11ac529a` 保留在 `v1`。

## API 和依赖

- 公共 builder、内部受控状态、示例、测试和文档统一使用 `Vary<T>`。
  `Cage<T>`、`Bond<T>` 继续作为属性来源；没有旧名称兼容别名。
- Sapid 继续实现 `glory::ui::Widget`，保留输入版本、焦点、组合态、SSR/接管、
  Layer 和原生语义。Glory `display` 的 HTML 模板 API 属于独立构造路径，
  本次没有把语义控件改为 HTML 包装，也没有宣称 Sapid 已可直接嵌入该 DSL。
- 原生示例的直接 Apace 依赖与 Glory 统一为
  `ee125981e0e5a217e758be5c470df23cdd9e2fc3`，wingy 统一为
  `3b6f5a09697acb936567af64e53c915b0b1a2af9`；更新锁文件以避免同时编译两套引擎类型。
- `deploy/dependencies.json` 固定上述 Glory 提交，CI 与本地 sibling checkout
  使用同一实现。默认依赖图仍只有 Glory facade/core/render/style/ui。
- 重新生成 widget 路线图、能力表/gallery、Mantine API 索引和展示站源码面板。
  冻结的 268 个来源、149 项工作保持不变；宿主能力按实际证据更新。

## 原生能力同步

旧测试还在预期原生拒绝 PointerWithin、Mixed/Radio、editable Choice、树弹层和
中文精确选区。当前 Glory 已实现这些协议。本次将拒绝断言改成消费端验证：
真实悬停取消/恢复时钟；三态引擎属性；单选与只读；ModelSelector 的受控拒绝、
业务值和焦点；TreeSelect/Cascader 的路径值、关闭恢复焦点与物理身份；中文
UTF-16 选区读回。Rating 的 direct-apply 与查询水位也按成功提交协议检查。

两个原生颜色测试曾对全部主题硬编码浅色棋盘背景；现在按当前主题的实际
checker token 检查透明度合成后的精确 RGBA，保留原有容差、像素及身份断言。
生产控件没有为适配旧测试而改变颜色。

`check.ps1 -Native` 现在检查原生示例的全部 features，覆盖可选 files/clipboard。
能力账本把本次实际验证、已有共享宿主协议但缺少本批专门消费者证据、仍然拒绝的
facet 分开记录。没有把共享投影推断为 OS 输入、AT 客户端或读屏验收。

Web 夹具拷贝改为枚举全部静态 HTML，避免手工清单漏掉 `text-motion.html`。
浏览器脚本支持 `--jobs=1..4` 的独立浏览器，默认仍为单 worker；涉及剪贴板的组串行，
整轮保存测试源码和夹具资源 SHA256，结束时检查未变化。生成的色板 CSS 固定使用 LF，
避免 Windows checkout 的 CRLF 触发精确内容比较失败。

## 验证

| 检查 | 结果 | 本地证据 |
|---|---|---|
| Sapid 默认/全功能、共享渲染协议、Web/wasm32 严格检查与 classic stream | 2,701 次测试执行通过，0 失败 | `D:/BuildCache/sapid/check-web-v2-sync.log` |
| Glory Apace shell/accesskit/raster 全目标 | 274 通过，2 项原有忽略 | `D:/BuildCache/sapid/check-v2-sync-initial-native-carousel-failure.log` 中独立的 Glory 阶段 |
| Sapid 原生全部 features | 153 通过，0 失败 | `D:/BuildCache/sapid/native-final-v2-sync.log` |
| Sapid 原生全部 features / 全目标严格 Clippy | 通过 | `D:/BuildCache/sapid/native-final-clippy-v2-sync.log` |
| 展示站 release、149 路由、36 个代表性 SSR 页面和浏览器专项 | 通过；6 个专项脚本共 350 条报告/检查，errors 均为空 | `D:/BuildCache/sapid/showcase-final-v2-sync.log` |
| 控件浏览器矩阵 | Chromium 156.0.8078.4，120 组 / 1,271 条报告，errors 为空；整轮源码/资源哈希一致 | `D:/BuildCache/sapid/browser-results.json`、`browser-final-v2-sync.log` |
| 清单、能力表、源码面板、Mantine 索引 | 生成与离线检查通过 | 对应 `scripts/* --check` |
| 资源打包/保留单元测试、真实缓存换代、主题导出 Rust 编译 | 通过 | `site-assets-v2-sync.log`、`cache-browser-final-v2-sync.log`、`theme-export-v2-sync.log` |

不同配置会重复执行相同测试，执行数不表示互异用例数。原生失败审计保留在
`native-audit-v2-sync.log`；修复后完整门禁以 `native-final-v2-sync.log` 为准。
构建缓存、临时文件和浏览器结果位于 D 盘；所有构建 manifest 的 target 均由 Cargo
metadata 核对。默认依赖树保存在 `default-dependencies-v2-sync.txt`。

浏览器初次在 TextMotion 入口处遇到 404，修复夹具拷贝清单后复验通过；随后
纠正 Display 的旧标题预期和 gallery 的旧原生边界预期。失败日志/成功前缀保留在
`browser-initial-entry-failure.log`、`browser-before-entry-fix.json`；完整诊断轮保留在
`browser-before-final-fixture-fix.json`。该轮图片 404 来自把展示站的指纹媒体构建用于
普通夹具；最终矩阵恢复标准独立 fixture 构建，没有修改图片加载断言。

复现时先用 `./scripts/check.ps1 -Web -Native -Serve -Bindgen <0.2.129 executable>`
构建并在 8794 提供夹具，再在另一终端运行 `node scripts/browser.cjs --jobs=4`。
展示站单独运行 `./scripts/check-showcase.ps1 -Release -Browser -Bindgen <executable>`。
Playwright 模块通过 `SAPID_PLAYWRIGHT` 指定；默认单 worker 也受支持。
