"""Freeze official Mantine API facts and render a reproducible, non-parity audit.

Refresh upstream facts explicitly with --refresh [--snapshot D:/.../llms-full.txt].
Normal runs only read the checked-in facts and local Rust sources. --check is offline.
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import urllib.request

ROOT = Path(__file__).resolve().parents[1]
FACTS = ROOT / 'docs/mantine-api-audit.json'
DECISIONS = ROOT / 'docs/mantine-api-decisions.json'
REPORT = ROOT / 'docs/mantine-api-comparison.md'
SURFACE = ROOT / 'docs/mantine-api-surface.md'
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--refresh', action='store_true')
parser.add_argument('--snapshot', type=Path)
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
inventory = json.loads((ROOT / 'docs/widget-inventory.json').read_text(encoding='utf-8'))
catalogue = [c for c in inventory['catalogue'] if c['source'].startswith('mantine')]
tasks = {t['id']: t for t in inventory['tasks']}
decisions = json.loads(DECISIONS.read_text(encoding='utf-8'))
ledger = {t['id']: t for t in json.loads((ROOT / 'docs/capability-matrix.json').read_text(encoding='utf-8'))['tasks']}
if set(decisions['tasks']) != {c['task'] for c in catalogue}:
    raise SystemExit('Decisions must cover every Mantine-mapped task, without extra tasks')
if args.check and args.refresh:
    raise SystemExit('--check cannot refresh upstream facts')

if args.refresh:
    raw = args.snapshot.read_bytes() if args.snapshot else urllib.request.urlopen('https://mantine.dev/llms-full.txt', timeout=60).read()
    source = raw.decode('utf-8')
    markers = list(re.finditer(r'^### (.+)\nPackage: ([^\n]+)\n', source, re.M))
    blocks = {m[1]: (m[2], source[m.end(): markers[i+1].start() if i+1 < len(markers) else len(source)]) for i, m in enumerate(markers)}
    expected = {c['name'] for c in catalogue}
    extra = [(m[1], m[2]) for m in markers if m[2] not in {'@mantine/hooks', '@mantine/form', '@mantine/x'} and m[1] not in expected and m[1] not in {'Get started', 'Getting started'}]
    if extra:
        raise SystemExit(f'Unmapped upstream pages need explicit review: {extra}')
    facts = {'schema_version': 1, 'review_date': '2026-10-25', 'mantine_docs_version': inventory['mantine_docs_version'],
             'source': 'https://mantine.dev/llms-full.txt', 'sha256': hashlib.sha256(raw).hexdigest(), 'entries': []}
    for item in catalogue:
        package, block = blocks[item['name']]
        prose = re.sub(r'^```[^\n]*\n.*?^```[ \t]*$', '', block, flags=re.M | re.S)
        props = prose.split('#### Props', 1)[1].split('#### Styles API', 1)[0] if '#### Props' in prose else ''
        types = {}
        for line in props.splitlines():
            fields = re.split(r'(?<!\\)\|', line)[1:-1]
            if len(fields) >= 3:
                key = fields[0].strip()
                if not key or key == 'Prop' or re.fullmatch(r'[-:]+', key):
                    continue
                if not re.fullmatch(r'[a-zA-Z_][\w-]*', key):
                    raise SystemExit(f'Unparsed public prop in {item["name"]}: {key}')
                value = fields[1].strip().replace('\\|', '|')
                types.setdefault(key, [])
                if value not in types[key]:
                    types[key].append(value)
        facts['entries'].append({**item, 'package': package, 'props': sorted(types),
                                'prop_types': types,
                                'sections': re.findall(r'^## (.+)', prose, re.M), 'document_sha256': hashlib.sha256(block.encode()).hexdigest()})
else:
    facts = json.loads(FACTS.read_text(encoding='utf-8'))
if {c['id'] for c in catalogue} != {c['id'] for c in facts['entries']}:
    raise SystemExit('Upstream audit facts do not cover the canonical Mantine catalogue')

def sources(task):
    result = []
    for value in tasks[task]['evidence']['implementation']:
        path = value.split(' (')[0]
        if path.endswith('.rs') and (ROOT / path).is_file() and not path.startswith('examples/'):
            result.append(path)
    return sorted(set(result))

def methods(paths):
    result = {}
    for path in paths:
        text = (ROOT / path).read_text(encoding='utf-8')
        for match in re.finditer(r'pub fn ([a-z_0-9]+)|forward!\((\w+)', text):
            result.setdefault(match[1] or match[2], []).append(f'{path}:{text.count(chr(10), 0, match.start()) + 1}')
        for macro, relative in [('common_methods!', 'src/appearance.rs'), ('text_input_methods!', 'src/text_input.rs')]:
            if macro in text:
                support = (ROOT / relative).read_text(encoding='utf-8')
                start = support.find('macro_rules! ' + macro[:-1])
                for match in re.finditer(r'pub fn ([a-z_0-9]+)', support[start:]):
                    result.setdefault(match[1], []).append(relative)
    return result

def snake(value):
    return re.sub(r'(?<!^)(?=[A-Z])', '_', value).lower()

lines = [
    '# Mantine 全组件 API 与功能差异核实', '',
    f"核实日期：{facts['review_date']}；官方文档版本：{facts['mantine_docs_version']}。",
    f"逐项覆盖 **{len(catalogue)} 个 Mantine 条目**：121 Core、15 Dates、22 Charts、13 Schedule、8 官方扩展、2 Code。",
    '官方来源：[Core 目录](https://mantine.dev/core/package/)、[官方扩展](https://mantine.dev/x/extensions/)、',
    '[完整文档（包含功能示例和 Props 表）](https://mantine.dev/llms-full.txt)。', '',
    '## 结论与核实边界', '',
    '旧 inventory 的 done 表示有限功能批次已交付，**不表示 Mantine API 等价**。本报告逐项比对官方文档、',
    '本地公开 Rust 入口、实现和能力矩阵；没有把文件存在、同名方法、近似拼写或 SSR HTML 当成功能验收。',
    'Props 表包括同页子部件的公开属性；继承的原生 HTML 属性和所有 React 类型重载不一定在表内。',
    '功能表是经实现核实的差异与设计决策，附录是完整文档表面的可追溯索引；未映射属性是核实候选，不能直接解释为缺失。',
    f"快照记录 {sum(len(entry['props']) for entry in facts['entries']):,} 个按文档条目计数的 Props 字段与其类型；不同条目的同名字段重复计数，不是同等数量的缺失功能。",
    '历史宿主证据沿用 capability-matrix，新增功能的当前运行证据单列，未逐个重验所有上游功能组合。', '',
    'Rust 的 Widget、Cage/Bond/Lotus、Owner 和有限宿主协议保持不变；不移植 React hooks、任意 DOM/ARIA 注入、',
    'polymorphic component、任意 Recharts/Tiptap/Embla 插件或全局 provider。通用尺寸、颜色、圆角等可通过 typed Style/Theme 组合，',
    '不因缺少同名便捷 prop 就重复建立状态。原生支持、浏览器协议和实际 OS/读屏验收分别记账。', '',
    '部分 theme/size/variant、声明预算与组合策略属于挂载期配置，并非所有 Mantine prop 都可直接传 Lotus。',
    '响应式值/禁用/只读等按各 builder 明确暴露的绑定工作；Style 组合不等于已提供上游同名动态 API，也不以重建节点冒充原位更新。', '',
    '## 按报告执行的批次', '',
    '| 批次 | 优先级 | 要补齐的行为 | 归属与状态 |', '| --- | --- | --- | --- |']
for batch in decisions['batches']:
    lines.append(f"| {batch['id']} | {batch['priority']} | {batch['work']} | {batch['status']} |")
lines += ['', '## 逐组件核对', '', '| Mantine / 官方文档 | Sapid 现有对应 | 经核实的差异与借鉴决策 | 优先级 |', '| --- | --- | --- | --- |']
for entry in facts['entries']:
    d = decisions['tasks'][entry['task']]
    note = d.get('overrides', {}).get(entry['name'], d['differences'])
    lines.append(f"| [{entry['name']}]({entry['url']}) | {d['current']} | {note} | {d['priority']} |")
surface_start = len(lines)
lines += ['', '## 完整 API 表面索引（候选不是等价性结论）', '',
          '每项记录官方属性、功能章节、Sapid 对应实现路径和公开入口。共享文件可能包含同族其他 builder；',
          '下列入口列表是导航证据，不是某个 builder 支持所有入口的声明。受控回调、Field 组合、Style 声明和宿主限制须结合上表阅读。', '']
for entry in facts['entries']:
    paths = sources(entry['task'])
    api = methods(paths)
    candidates = [p for p in entry['props'] if snake(p) not in api]
    lines += [f"### {entry['name']} — {entry['id']}", '',
              f"来源：[{entry['package']}]({entry['url']})；对应 `{entry['task']}`；原生有限能力状态 `{ledger[entry['task']]['native']['state']}`。", '',
              '**官方 Props / 子部件 Props：** ' + (', '.join(f'`{p}`' for p in entry['props']) or '该页没有独立 Props 表。'), '',
              '**官方功能章节：** ' + '；'.join(entry['sections']), '',
              '**本地实现：** ' + '、'.join(f'[{p}](../{p})' for p in paths), '',
              '**本地公开入口（同族）：** ' + ', '.join(f'`{name}`' for name in sorted(api)), '',
              '**需要语义映射的属性候选：** ' + (', '.join(f'`{p}`' for p in candidates) or '无直接拼写候选；仍不推断功能等价。'), '']
surface_end = len(lines)
lines += ['## 本轮验证与复现', ''] + decisions['verification'] + ['',
          '复现：`python scripts/mantine-api-audit.py`；离线检查：`python scripts/mantine-api-audit.py --check`。',
          '上游事实仅在显式 `--refresh` 时更新；完整文档缓存在 D 盘，仓库只保存属性/章节事实和 SHA256，未复制第三方实现。',
          f"文档快照 SHA256：`{facts['sha256']}`。", '']
surface = '\n'.join(['# Mantine API 表面与本地入口索引', '', '[决策与差异报告](mantine-api-comparison.md)。'] + lines[surface_start:surface_end])
output = '\n'.join(lines[:surface_start] + ['', '完整 Props、功能章节与本地入口见 [API 表面索引](mantine-api-surface.md)；机器事实见 [快照](mantine-api-audit.json)。', ''] + lines[surface_end:])
if args.check:
    if REPORT.read_text(encoding='utf-8') != output:
        raise SystemExit('Stale Mantine comparison report; run scripts/mantine-api-audit.py')
    if SURFACE.read_text(encoding='utf-8') != surface:
        raise SystemExit('Stale Mantine API surface; run scripts/mantine-api-audit.py')
else:
    if args.refresh:
        FACTS.write_text(json.dumps(facts, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    REPORT.write_text(output, encoding='utf-8')
    SURFACE.write_text(surface, encoding='utf-8')
print(f"Mantine audit: {len(facts['entries'])} official entries, {len(decisions['tasks'])} reviewed task families, complete catalogue coverage")
