# Glory + Sapid 展示站

展示站对照 [Mantine 首页](https://mantine.dev/)与
[Button 文档](https://mantine.dev/core/button/)的页面结构，采用 Sapid 自己的品牌：
顶部导航与搜索、大标题首页和真实控件卡片、分类侧栏、组件文档、预览与参数面板、
连接预览的代码卡片、页内目录，以及手机导航和明暗主题。

这是独立的 Glory + Sapid Rust 应用。页面、导航、搜索结果、主题入口、标签页、
代码展开和控件示例均由 Rust Widget 构造。`showcase.html` 只提供样式链接、
挂载容器和 WASM 启动；没有 React、iframe 或 JavaScript UI 模板。
链接导航会加载对应页面；主题通过 URL 保留当前组件并重新挂载。

## 启动与页面

从 Sapid 目录执行：

```powershell
./scripts/check-showcase.ps1 -Serve -Browser
```

脚本设置 D 盘 Cargo、target 和临时目录，并通过 `cargo metadata` 核对实际 target。
产物为 `D:/BuildCache/sapid/showcase`，服务器为 `http://127.0.0.1:8795/`。
首页和 `showcase.html` 使用同一个入口。例如：

- [首页](http://127.0.0.1:8795/)
- [Button 参数面板](http://127.0.0.1:8795/showcase.html?component=P0-01)
- [TextField](http://127.0.0.1:8795/showcase.html?component=P0-02)
- [Button 深色主题](http://127.0.0.1:8795/showcase.html?component=P0-01&scheme=dark)

149 个指南对应路线图的 149 个工作项，并不表示 149 个互不重复的底层控件。
10 个核心示例采用面向文档的独立演示，其余页面直接调用已有的 104 个 Rust 示例族，
包括可选组件和分组变体。源码页显示实际演示函数或实际 Rust 调用，Copy 复制完整源码，
预览中的折叠代码区只影响显示高度。

Button 页的 Variant、Size、Disabled、Loading 控件驱动实际同一个 Button 节点；
按钮激活计数和禁用行为由 Cage 及 Sapid 控件处理。搜索按名称、类别和工作项 ID 过滤，
结果为实际语义链接。手机导航通过 Rust 状态打开和关闭。

## 实现与证据

- 页面与状态：[showcase.rs](../examples/web/src/showcase.rs)。
- 首页：[showcase_home.rs](../examples/web/src/showcase_home.rs)。
- 实际控件示例：[showcase_examples.rs](../examples/web/src/showcase_examples.rs)。
- 149 项目录：[生成器](../scripts/showcase-catalog.py)读取 canonical inventory 和能力账本，
  `--check` 检测目录漂移；[生成结果](../examples/web/src/showcase_catalog.rs)保留真实支持范围。
- 页面样式：[site.css](../examples/web/site.css)，只由此展示站加载。
- Web 挂载与 SSR：[showcase_host.rs](../examples/web/src/showcase_host.rs)、
  [showcase_ssr.rs](../examples/web/src/bin/showcase_ssr.rs)。
- Rust 验收：[showcase_site.rs](../examples/web/tests/showcase_site.rs)与示例模块内测试。
- 浏览器验收：[showcase-acceptance.js](../examples/web/showcase-acceptance.js)、
  [运行器](../scripts/showcase-browser.cjs)。

构建门禁包括 6 项 focused Rust 测试、host 全 target 严格 Clippy、wasm32 严格 Clippy、
WASM 构建，以及 5 个代表页面乘 4 个主题组合的 20 个 SSR/adoption 页面。
149 个指南均通过 CSR 路由提供；20 个代表 SSR 页面不代表已经逐一验收 149 个 SSR 页面。
站点的完整浏览器验收逐一点击 149 个侧栏链接，并检查实际演示、搜索、Button 参数与节点身份、
代码展开/收起、完整复制、Source 页、主题、TextField、手机菜单、SSR 及采用前的编辑/选择区。

最终运行结果、产物哈希和截图见下方记录。原有组件的全量能力验收继续保存在
[widgets-verification.md](widgets-verification.md)；旧的静态能力审计 Gallery 和 8794 夹具没有被替换。
展示站本身采用 Web/SSR 宿主，不扩大各组件的原生、OS IME 或辅助技术支持承诺。

## 2026-10-25 最终运行记录

| 检查 | 结果 | 原始证据 |
|---|---|---|
| 目录同步、Rust 格式与 focused 测试 | 149 项同步，6 项测试通过 | [完整门禁](D:/BuildCache/sapid/showcase-gate.log) |
| host 全 target / wasm32 严格 Clippy、WASM、20 个 SSR 文件 | 通过 | 同上 |
| Chromium 148.0.7778.96 | 178 条报告，0 errors；包含全部 149 路由和 20 个 SSR 页面 | [浏览器 JSON](D:/BuildCache/sapid/showcase-browser.json) |
| README 默认 / 全 feature doctest | 各 5 项通过 | [doctest 日志](D:/BuildCache/sapid/showcase-readme-doctests.log) |
| 原有 8794 夹具的最终哈希复核 | 742 个文件无变化 | [原夹具审计](D:/BuildCache/sapid/showcase-original-fixtures-audit.json) |

本轮产物清单为 [asset-hashes.json](D:/BuildCache/sapid/showcase/asset-hashes.json)，
SHA256 为 `b37cfe082935f7b83bb6246963bb9b83d51ec29d12eb31ed8fcfcdee5a9a9fd3`。
浏览器 JSON 记录相同哈希。源码和逐文件哈希另存于
`D:/BuildCache/sapid/showcase-final-source`，最终核对见
[冻结审计](D:/BuildCache/sapid/showcase-freeze-audit.json)。

实际页面截图：[首页](D:/BuildCache/sapid/showcase-home-light.png)、
[Button 文档](D:/BuildCache/sapid/showcase-button-light.png)、
[深色 TextField](D:/BuildCache/sapid/showcase-textfield-dark.png)、
[360px 手机文档](D:/BuildCache/sapid/showcase-mobile-docs.png)和
[手机导航](D:/BuildCache/sapid/showcase-mobile-navigation.png)。

早期失败继续保留：首轮在 P5-14 启动就绪等待处超时，独立打开正常，
后续完整重跑均通过，未据此推断已定位生产缺陷；见
[首轮失败](D:/BuildCache/sapid/showcase-browser-first-route-failure.json)。
第二轮 Source 验收误用已存在的代码容器作为异步切换就绪条件，现改为等待目标页的
“Rust source”标题后，再断言示例已卸载；见
[原始 Source 失败](D:/BuildCache/sapid/showcase-browser-second-source-wait.json)。
另外修复了手机说明卡片的 inline 背景碎片，并补齐复制源码的 `FlexWrap` 导入；
最终整轮重新构建 WASM/SSR 并重新执行全部 178 条浏览器验收，未复用旧报告前缀。
