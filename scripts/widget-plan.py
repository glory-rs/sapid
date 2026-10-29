"""Render or check the frozen widget comparison; no network/build is required."""
import argparse
from collections import Counter
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
INVENTORY = ROOT / "docs/widget-inventory.json"
REPORT = ROOT / "docs/widgets-roadmap.md"
EXPECTED = {
    "mantine-core": 121, "shadcn-components": 66, "mantine-dates": 15,
    "mantine-charts": 22, "mantine-schedule": 13, "mantine-code": 2,
    "mantine-extensions": 8, "shadcn-ai": 21,
}
STATUS = {"baseline": "已有基线，需补强", "partial": "部分基础", "planned": "未实现", "done": "已完成并记录证据"}

INTRO = """# Sapid widgets 对照与逐项实现计划

核对日期：2026-10-24（目录抓取于 10 月 23–24 日）。Mantine 官方文档显示版本 9.7.3；shadcn 使用本次官方组件目录快照。
这是一个冻结的对照快照，后续上游新增项需要显式核对并更新清单，不自动改变目标范围。
本页由 `docs/widget-inventory.json` 生成；修改清单后运行 `python scripts/widget-plan.py`，
运行 `python scripts/widget-plan.py --check` 检查覆盖完整性和文档同步。
最终全量验收、复现命令与宿主边界见 [验证记录](widgets-verification.md)。
Glory + Sapid 构建的首页、组件文档和实际示例见 [展示站](showcase-site.md)，
运行 `./scripts/check-showcase.ps1 -Serve -Browser` 后打开 `http://127.0.0.1:8795/`。

## 当前判断

原有九个主要组件为 Button、TextField、Checkbox、Collapsible、Accordion、Tabs、
Dialog、Drawer、Popover；第一批已增加 Field、Label/Description/Error、Fieldset/Legend，
并接入 TextField/Checkbox 的标签及 required/readonly/invalid。已有测试证明相应功能，
不表示与某个上游组件全部功能等价。蓝/绿各有明暗主题；有限主题 token/provider、方向、locale、
响应式和运动契约已有独立验收，支持范围与明确拒绝见 P0 和逐组件能力记录。
第二批已增加 Textarea、PasswordInput、InputGroup/Addon/Section，完成 Field、受控状态、
Web/SSR 与原生引擎验收。Textarea 自动高度依赖浏览器 field-sizing:content，Apace 明确拒绝；
固定行数可用于原生。后续批次已增加 Form/FormState、成功控件快照、MixedCheckbox/组、
Switch、RadioGroup/SegmentedControl、动作按钮、展示表面、NumberFormatter 和文档文本/列表组件，
对应证据逐项记录在下表。Toggle、Chip、布局、Typography、Grid/比例、Timeline、Spoiler 与导航
已增加实际宿主证据；Menu、Stepper、辅助文本、图像、提示层、原生选择和应用框架亦已验收，
新批次的完成状态及宿主边界以清单和逐组件证据为准。
DataTable、Tree/FileTree、TreeSelect 和 Cascader 已按有限契约完成实现和宿主验收；
图表、文件、编辑器、日程和聊天模块的当前完成状态以清单及对应证据为准。

名称需要按语义对照：当前 Dialog 对应 Mantine Modal / shadcn Dialog；Mantine Dialog
是另外的非模态提示需求，单列 FloatingDialog。Sheet 已在 Drawer 上增加有限手势与 snap
points；验收和宿主边界见对应证据。Collapse 与 Collapsible 复用展开模型并接入有限运动协议。Mantine 布局 Group
不是 Glory 的 boxless Group。菜单、Tooltip、Combobox 也不能仅改 Popover 的外观就算完成。

Dialog/Popover 已验收 Web/SSR/hydration、Document 跨 App 栈、完整有限锚点定位与滚动锁，
并接通原生 WindowApp 的真实 top layer、焦点/AX、布局和像素。Drawer/Sheet、Menu/Tooltip/Listbox
等其余原生 Layer facet 仍按证据明确拒绝。独立 Portal 与 FocusScope 已完成真实宿主、跨 App 和生命周期验收；
完整主题和其他组件的当前状态均以逐项证据为准。

## 目标边界

1. 覆盖两套官方目录中可见的核心 widgets，并纳入 Mantine 日期/图表/日程/编辑器等扩展。
   shadcn 新增的消息、问卷、Trace 和 AI Elements 也保留条目，作为可选展示模块。
2. 同义名称和紧密变体共享实现，但清单保留每个来源 URL。149 项工作不是 149 个独立 widgets；
   其中包含已有组件补强、公共基础设施和多个变体的共同实现。每个变体都要有自己的验收证据。
3. 不照搬 React/JSX API、hooks、JS 库依赖和代码分发 CLI。继续使用手写 Rust builder、
   Cage/Bond/Vary、Glory Widget 与现有所有权/宿主边界；第三方实现不复制进仓库。
4. 社区 Registry、页面 Blocks、完整业务应用和任意第三方插件不是有限可穷尽的 widgets 清单。
   React 表单库绑定不移植，但对应表单能力列入 P1；主题/provider/方向/locale 是基础设施。
5. 默认 Sapid 保持轻依赖。日期、图表、文件、富文本、日程和聊天组件使用可选模块或独立 crate。
   当前是 Sapid 的 `dates/charts/files/editor/chat/code/json` 可选 features，日程随 `dates`；
   尚未拆为同名独立 crate，也不把可选宿主依赖带入默认组件图。
   图表需要可访问数据表示和跨宿主绘图能力；富文本需要独立文档/编辑协议，不能空壳交付。
6. 实现默认样式、主题 token、尺寸/状态和 unstyled 组合能力；不把黑白原生控件包装当完整设计系统。
   组合部件可表达 Root/Trigger/Content/Label/Description 等角色，但不引入 UI DSL 或任意 HTML API。

## 执行方式与依赖

按下表 P1 → P13 推进，阶段内按任务编号逐个交付。P0 是贯穿所有阶段的补强队列；
依赖 P0 表示先完成当前组件必需的基础，不要求先完成全部原生浮层/动画等工程才开始 Field。
每批实现 1–3 个有共同依赖的组件；复杂项目先把该项拆为可验收子任务，再编码。
未完成当前批的状态/键盘/宿主测试前，不用新增空 builder 的数量代表进展。

首批明确顺序：

1. 已完成 P1-01 Field/InputWrapper，P1-02 Label/Description/Error，P1-03 Fieldset/Legend；
   Glory 的 required/readonly/invalid 和真实标签/分组关联已接通 TextField/Checkbox，证据见对应任务。
2. 已完成 P1-04 Textarea、P1-05 PasswordInput、P1-06 InputGroup；保留组合输入、选区和受控恢复。
3. 已完成 P1-07 FormState/Validation、P1-08 有限提交/重置/序列化契约；不默认接入网络。
4. 已完成 MixedCheckbox/组、Switch、RadioGroup、SegmentedControl、Toggle/Chip；后续批次按有限契约逐项验收。

每完成一项：更新 JSON 状态和实现/测试证据，再重新生成本页。`baseline` 只表示已有本轮之前的
功能基线；`partial` 表示只有部分部件或 Glory 基础；`planned` 表示没有实现。改为 `done` 必须在
该项 `evidence.implementation` 和 `evidence.tests` 中记录实现及验证证据。本轮继续按组件记录实际宿主证据，未完成项保持 partial/planned，不用新增类型数量替代验收。

## Glory 需要随组件补齐的能力

| 使用方 | 应补充的有限语义/宿主能力 |
|---|---|
| Field / Form | 标签激活关联、description/error、required/readonly/invalid、分组、表单值和 submit/reset |
| Switch / Radio / Toggle | 独立内在交互角色、mixed/pressed/checked、组值和实际焦点行为 |
| Select / Menu / Combobox / Tree | menu/listbox/option/treeitem 等有限类型、活动后代、选择/展开/位置、方向导航和 typeahead |
| Tooltip / AlertDialog / Toast | tooltip/alertdialog、live/busy/progress、可撤销宿主计时；延迟回调不得越过退休 Owner |
| Slider / SpinButton / Splitter | 数值范围、步进、值文本、方向、指针捕获、键盘等价操作和拒绝恢复 |
| Layout / Motion / Icon | typed grid/ratio/换行/响应式布局、方向、有限运动/图标/绘图；逐属性更新支持 profile |
| Table / Grid / Schedule | 表头与单元格关联、网格游标、选择与编辑；只遍历挂载窗口，不能扫描大数据源 |
| File / Clipboard / Editor | 受限文件句柄和预算、剪贴板动作及错误、富文档/选择/IME/undo 协议，宿主负责系统访问 |

这些行描述组件所需的能力类别，实际 enum/API 及宿主边界见实现与协议文档。优先扩展共享有限契约、验证、mapper 和实际
适配器；不在 Sapid 内注入 DOM/ARIA/脚本，也不把交互 View 随意改 role 冒充原生控件。
新增语义必须明确 Web、SSR/stream、Apace、AccessKit 的支持或拒绝；不能默默降级成错误语义。

## 统一完成标准

- Rust API 与文档：受控/非受控状态、外部更新静默、稳定 key/关联、组合插槽、主题/unstyled。
- 交互：相关键盘/指针/触摸行为，disabled/readonly、焦点、IME、受控拒绝、空态与边界值。
- 生命周期：Owner 退休、任务/计时器取消、异步晚到结果、有限队列；不保留访问历史或整份数据副本。
- 呈现：SSR escaping、稳定关联、hydration 身份/草稿/焦点保持及失败无副作用；stream 适用项验证。
- Glory 回归：Memory 事务回滚、target/event 校验、preflight/direct apply、成功前缀和查询水位；
  HTML full-shape/字段 Patch 差分 oracle，元数据变更不重写输入值或选择。
- 实际宿主：浏览器键盘/指针和 AX 树、截图；原生支持项运行真实引擎/AccessKit 测试。
  不能以 cargo check 或 headless 协议证明 OS IME、读屏语音或未运行的设备能力。
- 门禁：`scripts/check.ps1`、相关 Web/Native 检查、fmt/Clippy，以及本次真正需要的领域模型测试。
  所有构建、工具缓存与测试产物位于 D 盘，并核对 cargo metadata 的 target_directory。

## 逐项队列

同一行列出的近义组件或变体共享一个工作项，仍逐个验证。来源缩写：M= Mantine core；
S= shadcn 主目录；D=日期；C=图表；T=日程；H=代码；X=其他 Mantine 扩展；A=AI Elements。
完整原始条目、来源 URL、状态和关联任务在 `widget-inventory.json`，没有遗漏或未归属的条目。
"""


def validate(data):
    tasks = {item["id"]: item for item in data["tasks"]}
    phases = {item["id"]: item for item in data["phases"]}
    assert len(tasks) == len(data["tasks"]), "duplicate task id"
    assert len(phases) == len(data["phases"]), "duplicate phase id"
    assert Counter(item["source"] for item in data["catalogue"]) == EXPECTED, "snapshot counts changed; explicitly re-audit EXPECTED"
    entries = {item["id"]: item for item in data["catalogue"]}
    assert len(entries) == len(data["catalogue"]), "duplicate catalogue id"
    assert len({item["url"] for item in entries.values()}) == len(entries), "duplicate upstream page"
    source_ids = {source["id"] for source in data["sources"]}
    for item in entries.values():
        assert item["source"] in source_ids
        assert item["task"] in tasks, item["id"]
        assert item["id"] in tasks[item["task"]]["sources"], item["id"]
        assert item["url"].startswith(("https://mantine.dev/", "https://ui.shadcn.com/"))
    for task in tasks.values():
        assert task["phase"] in phases
        assert task["status"] in STATUS, task["id"]
        assert task["name"] and task["work"]
        if task["status"] == "done":
            assert task.get("evidence", {}).get("implementation"), task["id"] + " missing implementation evidence"
            assert task.get("evidence", {}).get("tests"), task["id"] + " missing test evidence"
        assert len(task["sources"]) == len(set(task["sources"]))
        for key in task["sources"]:
            assert entries[key]["task"] == task["id"], key
    resolved = set()
    for phase in data["phases"]:
        assert set(phase["depends_on"]) <= resolved, "phases must be in dependency order"
        resolved.add(phase["id"])
    # Baselines are the nine actual public builders, not aliases or Glory nodes.
    assert {task["name"] for task in tasks.values() if task.get("baseline_at_snapshot")} == {
        "Button", "TextField", "Checkbox", "Collapsible", "Accordion", "Tabs", "Dialog", "Drawer", "Popover",
    }


def render(data):
    parts = [INTRO]
    by_id = {entry["id"]: entry for entry in data["catalogue"]}
    for phase in data["phases"]:
        parts.append(f'\n### {phase["id"]} · {phase["title"]}\n\n{phase["goal"]}\n')
        deps = "、".join(phase["depends_on"]) or "现有 Glory/Sapid 基线"
        parts.append(f'\n依赖：{deps}。\n\n| 编号 | 工作项 | 当前状态 | 实现与验收重点 | 来源 |\n|---|---|---|---|---|\n')
        for task in data["tasks"]:
            if task["phase"] != phase["id"]:
                continue
            refs = "、".join(f'[{ref}]({by_id[ref]["url"]})' for ref in task["sources"]) or "Sapid/Glory 共享能力"
            parts.append(f'| {task["id"]} | {task["name"]} | {STATUS[task["status"]]} | {task["work"]} | {refs} |\n')
    counts = Counter(item["status"] for item in data["tasks"])
    parts.append('\n## 快照覆盖校验\n\n| 目录 | 原始条目数 |\n|---|---:|\n')
    for name, count in EXPECTED.items():
        parts.append(f'| {name} | {count} |\n')
    parts.append(f'\n共 {len(data["catalogue"])} 个来源条目，归并为 {len(data["tasks"])} 项工作：'
                 f'{counts["baseline"]} 项已有基线、{counts["partial"]} 项部分基础、{counts["planned"]} 项未实现、{counts["done"]} 项按后续完成标准关闭。'
                 '日程目录包含事件数据与重复规则两项模型文档，基础设施也单独计入，不能把总数解释为缺失组件数量。\n')
    parts.append('\n## 对照来源\n\n')
    for source in data["sources"]:
        parts.append(f'- [{source["id"]}]({source["url"]})\n')
    parts.append('\n命名差异另核对 [Mantine Dialog](https://mantine.dev/core/dialog/)、'
                 '[Mantine Modal](https://mantine.dev/core/modal/) 和 '
                 '[shadcn Drawer](https://ui.shadcn.com/docs/components/base/drawer)。\n')
    return "".join(parts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="validate without rewriting the report")
    args = parser.parse_args()
    data = json.loads(INVENTORY.read_text(encoding="utf-8"))
    validate(data)
    expected = render(data)
    if args.check:
        assert REPORT.read_text(encoding="utf-8") == expected, "roadmap is stale; run python scripts/widget-plan.py"
    else:
        REPORT.write_text(expected, encoding="utf-8")
    print(f'Validated {len(data["catalogue"])} source entries, {len(data["tasks"])} tasks, {len(data["phases"])} phases.')


if __name__ == "__main__":
    main()
