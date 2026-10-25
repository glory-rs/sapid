# Glory + Sapid 展示站

展示站对照 [Mantine 首页](https://mantine.dev/)与
[Button 文档](https://mantine.dev/core/button/)的页面结构，采用 Sapid 自己的品牌：
顶部导航与搜索、大标题首页和真实控件卡片、分类侧栏、widget文档、预览与参数面板、
连接预览的代码卡片、页内目录，以及手机导航和明暗主题。

这是独立的 Glory + Sapid Rust 应用。页面、导航、搜索结果、主题入口、标签页、
代码展开和控件示例均由 Rust Widget 构造。`showcase.html` 只提供样式链接、
挂载容器和 WASM 启动；没有 React、iframe 或 JavaScript UI 模板。
链接导航会加载对应页面；主题通过 URL 保留当前widget并重新挂载。

## 站点标识

顶栏与浏览器 favicon 使用同一海蓝色魟鱼标识，颜色固定为 `#0077B6`，不随页面配色切换。
矢量文件为 `examples/web/assets/sapid-logo.svg`，与设计目录的 `sapid-stingray.svg` 保持一致。
SVG 宽高均为 256；顶栏桌面显示 36 × 36，手机显示 32 × 32，保持正方形比例。
图形由共享的浅心形宽翼、镜像复用与向左回弯的尾巴组成；内部完整留白，无中间竖线。
顶栏使用语义 Image，旁边的 Sapid 首页链接提供文字名称，图形本身作为装饰图片。

## 主题定义、基础色与模式覆盖

内置 `BLUE` 和 `GREEN` 是两个 `ThemeDefinition`，各自支持 Light/Dark，展示站仍提供
四种组合。完整的基础 `Palette` 包含所有颜色；可选 `PaletteOverrides` 只覆盖指定字段。
颜色常量全部位于 `src/theme/presets.rs`，`Theme::new` 不生成或替换任何模式颜色。

- `BLUE.default()` 返回 `(ColorScheme, Palette)`，默认模式为 Light。
- `BLUE.light()` / `BLUE.dark()` 返回 `Option<Palette>`；不支持的模式为 None。
- 默认模式不需要声明覆盖；非默认模式用 Some 声明支持，可以是空覆盖。
- `Theme::new(&BLUE, ColorScheme::Dark)?` 创建 widget 使用的已解析主题；不支持的模式返回错误。

详见 [主题定义](theme-definitions.md)。默认 CSS 使用同一解析结果，通过
`cargo run --example palette_css` 生成，widget 不自行注入 CSS。网站的背景、文字、边框、
强调色、焦点与 hover 也读取解析后的数据，不另设暗色颜色常量。

展示站的 Blue/Green 选择属于应用配置（`examples/web/src/themes.rs`），配色与明暗切换
互相保留当前选择和指南。此前单独四个 Palette 常量的设计已经替换；下方旧验收记录
中的哈希只描述对应历史产物，通用报告/截图路径会由最新运行更新。

最新主题定义验收：完整库门禁、原生编译、展示站 host/wasm32 构建与严格 Clippy、
182 条浏览器报告全部通过，零错误。包含四种组合的实际背景/文字颜色检查和 SSR 接管。
日志与当前资产哈希见 [主题定义验收记录](theme-definitions.md#2026-10-25-theme-studio-verification)。

## Theme Studio 调色与本地预览

顶栏 Theme studio 打开 Rust 调色页面，也可直接访问
[调色页面](http://127.0.0.1:8795/showcase.html?widget=theme-studio&theme=green&scheme=dark)。
提供种子色生成、逐项 RGBA/颜色选择器编辑、Base / Light / Dark 独立覆盖、继承重置、
模式支持与默认模式设置、实际 Sapid widget 预览和对比度参考。

保存最多 8 个本地预览后，会加入右上角配色选项。选中时顶部显示复制主题代码、
继续编辑的操作；编辑页可删除预览。数据只保存在当前浏览器，不会上传。
复制内容为完整的可编译 Rust ThemeDefinition，包含所有基础值和稀疏模式覆盖。

这轮审计补齐状态色、高亮、阴影、图表序列、棋盘格及颜色标记的 Palette 字段，
并修复展示示例布局容器未传入当前主题的问题。整页宿主背景与组件共用解析后的颜色。
构建发布使用带内容哈希的 CSS/JS/WASM 目录，防止旧缓存与新颜色变量混用。
详细模型、保存限制与审计范围见 [主题定义](theme-definitions.md#theme-studio-and-color-audit)。

## 顶栏配色入口

次级导航整行已移除。蓝/绿选择改为实际配色的圆形色块，放在顶栏明暗切换左侧；
保留 `Use blue palette` / `Use green palette` 无障碍名称和 `aria-current` 选中状态。
选中项有描边，键盘焦点单独可见。手机首页菜单从 60px 顶栏下方展开。

`header-swatches-final-gate.log` 记录 host/wasm32 构建、严格 Clippy、SSR 和 182 条
浏览器检查通过，零错误；报告中仍包含完整主题切换与指南导航检查。
额外布局断言覆盖 320/360/390px 控制项可见、不重叠和页面无横向溢出。
首页菜单的独立实际检查为 `D:/BuildCache/sapid/header-swatches-home-probe.json`，
菜单与顶栏边缘均为 y=60。截图为 `header-swatches-desktop.png` 和
`header-swatches-mobile.png`，位于 `D:/BuildCache/sapid`。
本次资产清单 SHA256 为 `df7493460c28ca06a1c2d17af623364f749fcacd52beef81035d3179c22c9cbf`。

## 启动与页面

从 Sapid 目录执行：

```powershell
./scripts/check-showcase.ps1 -Serve -Browser
```

脚本设置 D 盘 Cargo、target 和临时目录，并通过 `cargo metadata` 核对实际 target。
产物为 `D:/BuildCache/sapid/showcase`，服务器为 `http://127.0.0.1:8795/`。
首页和 `showcase.html` 使用同一个入口。例如：

- [首页](http://127.0.0.1:8795/)
- [Button 参数面板](http://127.0.0.1:8795/showcase.html?widget=P0-01)
- [TextField](http://127.0.0.1:8795/showcase.html?widget=P0-02)
- [Button 深色主题](http://127.0.0.1:8795/showcase.html?widget=P0-01&scheme=dark)

149 个指南对应路线图的 149 个工作项，并不表示 149 个互不重复的底层控件。
10 个核心示例在 `src/showcase_demos/core_*.rs` 中直接编写，其余 104 个示例族由
`showcase-catalog.py` 从实际夹具生成独立完整的模块；共 114 个实现模块供 149 个指南复用。
目录的 `build` 指向这些模块的 `demo(theme)`，`code` 通过 `include_str!` 读取同一个文件。
因此 Source、Copy 与预览使用完全相同的已编译源码，包含状态、导入和所需辅助模块，
没有依赖展示站私有 `crate::…` 的片段。生成器 `--check` 验证同步；Rust 测试逐一挂载
149 个指南、检查高亮预算和卸载后的节点/任务清理。

复制的文件可作为 `src/lib.rs` 或应用内模块，调用 `demo(theme)` 后交给 Glory 宿主挂载。
当前 Sapid 尚未发布，依赖使用本地 checkout；与此站相同的依赖配置见
[Cargo.toml](../examples/web/Cargo.toml)。Web 的定时示例还使用该 manifest 中的
`wasm-bindgen` 和 `web-sys`；生成模块包含计时器实现。

Web 页面最终由 Glory 的 HTML/DOM 适配器输出，WASM 由 wasm-bindgen 生成的 JS 加载。
`showcase.html` 的手写 JS 仅初始化模块并设置测试就绪标记；路由参数、主题和 SSR
元数据在 Rust 的 `start_showcase` 中读取。页面本体、交互和示例没有 JS UI 构建器。
旧的 `showcase.js`、`gallery.js` 和各手写 HTML 属于 8794 验收夹具，新展示站不加载它们。

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

构建门禁包括 8 项 focused Rust 测试、host 全 target 严格 Clippy、wasm32 严格 Clippy、
WASM 构建，以及 5 个代表页面乘 4 个主题组合的 20 个 SSR/adoption 页面。
149 个指南均通过 CSR 路由提供；20 个代表 SSR 页面不代表已经逐一验收 149 个 SSR 页面。
站点的完整浏览器验收逐一点击 149 个侧栏链接，并检查实际演示、搜索、Button 参数与节点身份、
代码展开/收起、完整复制、Source 页、主题、TextField、手机菜单、SSR 及采用前的编辑/选择区。

最终运行结果、产物哈希和截图见下方记录。原有widget的全量能力验收继续保存在
[widgets-verification.md](widgets-verification.md)；旧的静态能力审计 Gallery 和 8794 夹具没有被替换。
展示站本身采用 Web/SSR 宿主，不扩大各widget的原生、OS IME 或辅助技术支持承诺。

## 2026-10-25 Widget 术语与源码重构验收

- `scripts/check.ps1` 串行完整通过：Sapid 默认/全部 feature 测试和 doctest、两套严格
  Clippy，以及 Glory style/core/render/ui/html/ssr 的全部 feature 测试与严格 Clippy。
  日志：`D:/BuildCache/sapid/widget-reorganization-final-gate.log`。
- `scripts/check-showcase.ps1 -Serve -Browser` 完整通过：8 项 focused Rust 测试、host/wasm32
  严格 Clippy、WASM、20 个 SSR 文档和 Chromium 178 条报告、0 errors；包含全部 149 个指南。
  日志：`D:/BuildCache/sapid/widget-showcase-gate.log`；报告：`D:/BuildCache/sapid/showcase-browser.json`。
- `examples/native/Cargo.toml --all-targets` 编译通过，日志：
  `D:/BuildCache/sapid/widget-native-check.log`。这是编译验证，不增加 OS/AT 验收范围。
- inventory、能力账本、展示目录三个生成器的 `--check` 均通过；源码与文档引用随模块和文件改名同步。
- 新资产清单 SHA256：`03b1d43ef803451322385dfab6d85232cb1f6d0e75e52483e14e05c75e078725`。
  所有构建、临时文件和浏览器产物均在经过 metadata 核对的 D 盘目录。

第一次共享 target 的并行门禁在全部 feature doctest 阶段出现 E0463，行为测试已通过；
原始失败保存在 `D:/BuildCache/sapid/widget-reorganization-first-run.log`。停止并行构建后，
上述完整串行门禁通过。此前的初版快照与哈希记录保留在下一节；通用截图/报告路径现在对应本次产物。

## 2026-10-25 初版运行记录（本次重构前）

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

## Widget 文件组织

有私有子模块的 widget 使用常规目录，例如 `src/collapsible/{mod.rs,motion.rs}`、
`src/accordion/{mod.rs,dynamic.rs}` 和 `src/schedule/resources/{mod.rs,drag.rs,timeline.rs}`。
`src` 内不再用 `#[path]` 改写模块位置；共享的跨宿主验收夹具可显式引用共享文件。
Web 的 72 个原根目录模块已归入 `examples/web/src`，对应的 Rust/native 测试引用同步迁移。
Glory 闭包入口为 `widget(...) -> WidgetFn<_>`，不保留 `component` 兼容别名。


## Theme Studio 最新验收

当前轮次通过 182 项原有浏览器检查、14 项调色流程检查，以及独立的 149 个绿色深色页面审计，均无错误。
快速主题传递门禁对 149 个已编译示例分别使用绿色深色与任意命名的自定义亮色主题，共 298 次挂载，没有残留其他主题类。
浏览器实际复制出的 Rust ThemeDefinition 已在 D 盘独立项目编译，并断言生成色阶与模式覆盖的值。
详细日志、当前资产哈希与截图见 [本轮验收](theme-definitions.md#2026-10-25-theme-studio-verification)。
