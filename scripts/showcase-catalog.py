"""Generate the Rust documentation catalogue and direct-widget family dispatch."""
import argparse
import json
from pathlib import Path
import re
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CATALOG = ROOT / "examples/web/src/showcase_catalog.rs"
EXAMPLES = ROOT / "examples/web/src/showcase_examples.rs"
WEB = ROOT / "examples/web"
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--check", action="store_true")
args = parser.parse_args()
inventory = json.loads((ROOT / "docs/widget-inventory.json").read_text(encoding="utf-8"))
ledger = json.loads((ROOT / "docs/capability-matrix.json").read_text(encoding="utf-8"))
tasks = inventory["tasks"]
records = {entry["id"]: entry for entry in ledger["tasks"]}
if len(tasks) != 149 or len(records) != 149 or {t["id"] for t in tasks} != set(records):
    raise SystemExit("catalogue requires all 149 canonical tasks and explicit ledger entries")

EXTRA_PAGES = {"P0-08": "drawer", "P2-09": "actions", "P10-07": "json-viewer"}
CORE = {f"P0-{i:02}": name for i, name in enumerate([
    "button", "text_field", "checkbox", "collapsible", "accordion", "tabs", "dialog", "drawer", "popover"
], 1)} | {"P0-14": "catalogue"}
SPECIAL = {
    "index": ("form", "form_themed", "scope, theme.clone()", True),
    "composites": ("composite_form", "form_themed", "scope, theme.clone()", True),
    "charts": ("charts_form", "charts_form", "theme, false", False),
    "cartesian": ("charts_form", "charts_form", "theme, true", False),
    "json-viewer": ("json_tree_example", "json_tree_example", "theme", False),
    "tree-select": ("tree_choices_form", "tree_choices_form", "theme, false", False),
    "cascader": ("tree_choices_form", "tree_choices_form", "theme, true", False),
    "trace": ("trace_form", "form", "scope, theme.clone(), false", True),
    "stack-trace": ("trace_form", "form", "scope, theme.clone(), true", True),
    "public-explanations": ("chat_evidence_form", "chat_evidence_form", "theme, false", False),
    "citations": ("chat_evidence_form", "chat_evidence_form", "theme, true", False),
}
for index, page in enumerate(["polar-charts", "point-charts", "shape-charts", "matrix-charts", "range-charts", "hierarchy-charts"]):
    SPECIAL[page] = ("plots_form", "plots_form", f"theme, {index}", False)

# Reader-facing introductions are separate from the canonical engineering evidence.
LEADS = {
    "P0": [
        "Trigger actions with accessible buttons, themed variants and loading states.",
        "Collect a single line of text with labels, descriptions and controlled values.",
        "Let users choose whether to enable an option or accept a condition.",
        "Reveal supporting content with a compact disclosure control.",
        "Organize related information into expandable sections.",
        "Switch between related views without leaving the current page.",
        "Focus attention on a task in a modal dialog.",
        "Open a side panel for navigation, settings or additional information.",
        "Place interactive content beside the control that opens it.",
        "Build consistent keyboard navigation for collections of interactive items.",
        "Compose focus boundaries and render content into an explicit destination.",
        "Configure colors, typography, spacing, language and reading direction.",
        "Provide accessible text that stays visually hidden.",
        "Explore every widget and compare its documented platform capabilities.",
    ],
    "P1": [
        "Connect a form control with its label, help text and validation state.",
        "Give controls clear labels, useful descriptions and actionable error messages.",
        "Group related controls under a shared legend.",
        "Collect multiline text with fixed rows or content-based sizing.",
        "Enter a password with an optional visibility control.",
        "Arrange an input with leading or trailing content and actions.",
        "Coordinate typed form values, validation and submission feedback.",
        "Submit or reset a form and read the values of its successful controls.",
    ],
    "P2": [
        "Manage multiple checkbox choices, including partially selected states.",
        "Turn a setting on or off with a switch.",
        "Choose one option from a named group of radio controls.",
        "Represent on-or-off actions with toggle buttons.",
        "Choose one value from a compact row of connected options.",
        "Present single or multiple choices as compact selectable chips.",
        "Arrange related buttons into one connected group.",
        "Offer compact icon actions and clearly named close controls.",
        "Toggle navigation with an animated menu button.",
    ],
    "P3": [
        "Build horizontal and vertical layouts with flexible spacing and alignment.",
        "Center content, constrain its width and add intentional space.",
        "Arrange content in responsive columns with spans and consistent gaps.",
        "Keep media or other content at a chosen width-to-height ratio.",
        "Create readable text, headings and consistent typography.",
        "Link to another page or location with an accessible text link.",
        "Present related items as ordered or unordered lists.",
        "Set quotations apart from surrounding text.",
        "Display inline code and draw attention to important words.",
        "Show compact labels, statuses and removable pills.",
        "Group content on themed surfaces with padding, borders and shadows.",
        "Separate content with horizontal or vertical dividers.",
        "Represent people with avatars, initials and grouped images.",
        "Display images with accessible alternatives or use an image as a backdrop.",
        "Add a small indicator or place an icon on a themed surface.",
        "Display keyboard keys and shortcuts in running text.",
        "Explain an empty view and offer a useful next action.",
        "Lay out labeled details and structured content rows.",
        "Show events or milestones in chronological order.",
        "Format numbers, currencies and units for readable display.",
        "Collapse long content behind a show-more control.",
    ],
    "P4": [
        "Organize actions in a keyboard-accessible menu with nested submenus.",
        "Open a menu for the item under the pointer or keyboard focus.",
        "Keep application menus together in a navigable menu bar.",
        "Choose values using the browser's native select control.",
        "Choose one value from a styled list of options.",
        "Combine text entry with a searchable list of known options.",
        "Suggest matching values while allowing users to enter their own text.",
        "Search for and select several values from a list of options.",
        "Enter, paste and remove free-form tags in a single input.",
        "Find commands with search and keyboard navigation in a command palette.",
    ],
    "P5": [
        "Ask users to confirm an important action before continuing.",
        "Reveal a drawer with interactive snap points and gesture dismissal.",
        "Show a short description when a control is hovered or focused.",
        "Preview richer supporting content beside a hovered or focused element.",
        "Call attention to important information, warnings or errors.",
        "Deliver temporary notifications with application-controlled actions.",
        "Indicate that an operation is still in progress.",
        "Show progress as a bar, ring or semicircular indicator.",
        "Show page or application activity with a compact navigation progress bar.",
        "Reserve space for content while it is loading.",
        "Place a visual or loading overlay over a region of the interface.",
        "Keep a nonmodal message visible while users continue working.",
        "Coordinate queued dialogs and confirmations from one place.",
        "Create a movable and resizable floating workspace.",
        "Guide users through a sequence of highlighted interface targets.",
    ],
    "P6": [
        "Show the path to the current page and link to its ancestors.",
        "Move between pages of results with compact pagination controls.",
        "Build nested navigation with links and expandable sections.",
        "Organize application destinations in a sidebar.",
        "Compose a responsive application frame with navigation and main content.",
        "Break a workflow into named steps and show the current stage.",
        "Keep overflowing content in an accessible scrolling region.",
        "Let users resize adjacent panels with a draggable splitter.",
        "Keep an action or supporting element at a chosen viewport position.",
        "Group related actions in a keyboard-navigable toolbar.",
        "Navigate a document through its registered headings.",
        "Move items that no longer fit into an anchored overflow panel.",
        "Animate an indicator between the currently selected elements.",
        "Display moving text and smoothly changing numeric values.",
    ],
    "P7": [
        "Enter numeric values with increment and decrement controls.",
        "Choose a value by moving a thumb along a track.",
        "Select the lower and upper ends of a numeric range.",
        "Choose an angle with a circular slider.",
        "Collect a rating or display a read-only score.",
        "Preview a color in a compact swatch.",
        "Adjust a color's hue or transparency along a slider.",
        "Choose and edit colors with a picker or text input.",
        "Enter a short verification code using connected input slots.",
        "Guide text entry with a structured input mask.",
        "Edit JSON text with explicit parsing and validation feedback.",
    ],
    "P8": [
        "Present structured information in rows, columns and named headers.",
        "Explore tabular data with sorting, selection and windowed rows.",
        "Browse hierarchical items with expandable branches.",
        "Choose a value from a searchable hierarchy.",
        "Choose a nested value by moving through successive columns.",
    ],
    "P9": [
        "Configure civil dates, calendar language and week conventions.",
        "Browse days in a calendar or a compact month view.",
        "Pick a date or date range from a calendar panel.",
        "Type a date with parsing feedback and an optional calendar.",
        "Choose a month with a compact year-based picker.",
        "Choose a year from a navigable grid.",
        "Enter a time and display it in the configured format.",
        "Choose a time from interactive clock fields and a time grid.",
        "Choose a date and time together in a panel or inline view.",
    ],
    "P10": [
        "Select local files through a named file input or action button.",
        "Drop files into a selection area and display their attachments.",
        "Copy explicit text to the clipboard with clear feedback.",
        "Browse slides with navigation controls and swipe gestures.",
        "Open images in an interactive viewer with zoom and navigation.",
        "Display source code with syntax highlighting and copy actions.",
        "Inspect JSON values in an expandable structured view.",
        "Edit a structured rich document with formatting controls.",
    ],
    "P11": [
        "Compose charts with coordinated colors, legends and accessible data.",
        "Compare trends and categories with area, line, bar and composite charts.",
        "Show proportions with pie, donut and radial bar charts.",
        "Explore relationships and ranges with scatter, bubble and candlestick charts.",
        "Compare dimensions, stages and compact trends with specialized charts.",
        "Show patterns and proportions in heatmap, matrix and waffle charts.",
        "Compare values and targets with ranked bars, bullets and gauges.",
        "Explore hierarchies and flows with treemap, sunburst and Sankey charts.",
    ],
    "P12": [
        "Describe scheduled events and their recurrence using civil dates and times.",
        "Arrange events along a daily or weekly timeline.",
        "Browse scheduled events in month, year and compact mobile views.",
        "Read upcoming events as a chronological agenda.",
        "Compare availability and events across multiple resources.",
    ],
    "P13": [
        "Present conversation messages with bubbles, markers and supporting metadata.",
        "Browse long conversations while preserving the reader's position.",
        "Compose a message with suggestions, attachments and explicit send actions.",
        "Guide users through typed questions with step-by-step validation.",
        "Choose from an application-provided catalogue of models and providers.",
        "Offer contextual suggestions that users can explicitly choose.",
        "Request confirmation at a checkpoint before an application action continues.",
        "Present plans, tasks and queues with explicit application-owned status.",
        "Show application-published reasoning summaries and completed steps.",
        "Attach source links, context and inline citations to displayed content.",
        "Inspect public execution traces, tool results and stack information.",
        "Read windowed terminal output and structured test results.",
    ],
}
DESCRIPTIONS = {
    f"{phase}-{number:02}": description
    for phase, descriptions in LEADS.items()
    for number, description in enumerate(descriptions, 1)
}
if set(DESCRIPTIONS) != set(records):
    raise SystemExit("reader introductions must describe every canonical work item")

def group(task):
    phase, number = task["id"].split("-")
    n = int(number)
    if phase == "P0":
        return {1: "Buttons", 2: "Inputs", 3: "Inputs", 4: "Data display", 5: "Data display", 6: "Navigation", 7: "Overlays", 8: "Overlays", 9: "Overlays", 10: "Navigation", 11: "Overlays", 12: "Extensions", 13: "Typography", 14: "Extensions"}[n]
    if phase == "P1": return "Inputs"
    if phase == "P2": return "Buttons" if n in (4, 7, 8, 9) else "Inputs"
    if phase == "P3":
        return "Layout" if n <= 4 else "Typography" if n in (5, 6, 7, 8, 9, 16) else "Data display"
    if phase == "P4": return "Combobox" if 4 <= n <= 9 else "Navigation"
    if phase == "P5": return "Overlays" if n in (1, 2, 3, 4, 12, 13, 14, 15) else "Feedback"
    if phase == "P6": return "Layout" if n in (7, 8, 9) else "Navigation"
    if phase == "P7": return "Inputs"
    if phase == "P8": return "Combobox" if n in (4, 5) else "Data display"
    return {"P9": "Dates", "P10": "Extensions", "P11": "Charts", "P12": "Dates", "P13": "Chat"}[phase]


def target(page):
    if page in SPECIAL:
        module, function, arguments, scoped = SPECIAL[page]
    else:
        module = page.replace("-", "_") + "_form"
        paths = [WEB / "src" / f"{module}.rs", WEB / f"{module}.rs"]
        path = next((p for p in paths if p.is_file()), None)
        if path is None: raise SystemExit(f"no Rust demo module for {page}")
        source = path.read_text(encoding="utf-8")
        scoped = bool(re.search(r"pub fn form\(\s*scope:\s*&Scope", source))
        function, arguments = ("form", "scope, theme.clone()") if scoped else (module, "theme")
    return module, function, arguments, scoped


def literal(text):
    # JSON strings are valid Rust UTF-8 string literals for these metadata values.
    return json.dumps(text, ensure_ascii=False)


# Each displayed source is an entire Rust module compiled into the preview.
# Keep adapter bootstraps and test-only drivers outside these widget modules.
def fixture_source(module):
    path = WEB / "src" / f"{module}.rs"
    text = path.read_text(encoding="utf-8")
    boundary = re.search(r'\n#\[cfg\((?:target_arch = "wasm32"|test)\)\]\nmod (?:browser|tests) \{', text)
    if boundary:
        text = text[:boundary.start()]
    # Shared plot data is included explicitly, rather than a file-path override.
    text = text.replace('#[path = "../../fixtures/plot_samples.rs"]\npub mod samples;',
                        'pub mod samples {\n' + (WEB.parent / "fixtures/plot_samples.rs").read_text(encoding="utf-8") + '\n}')
    return text.strip()


def formatted(text):
    result = subprocess.run(["rustfmt", "--edition", "2024", "--emit", "stdout"],
                            input=text, text=True, encoding="utf-8", capture_output=True, check=True)
    return result.stdout.replace("\r\n", "\n")


def timer_browser():
    browser = (WEB / "src/menu_form.rs").read_text(encoding="utf-8")
    browser = browser[browser.index('mod browser {'):browser.index('    thread_local!')]
    browser = browser.replace('    use super::*;\n', '').replace('    use glory::web::WebApp;\n', '')
    return '\n#[cfg(target_arch = "wasm32")]\n' + browser + '}\n'


def bundle(page):
    module, function, arguments, scoped = target(page)
    body = fixture_source(module)
    dependencies = []
    # The only cross-fixture dependencies are a host timer and JSON demo.
    if module == "menu_form":
        body += timer_browser()
    if re.search(r'(?:crate|super)::menu_form::', body):
        timer = fixture_source("menu_form")
        timer = timer[timer.index('#[cfg(not(target_arch = "wasm32"))]'):]
        timer += timer_browser()
        dependencies.append('#[allow(dead_code)]\nmod menu_form {\n' + timer + '\n}')
        body = body.replace('crate::menu_form::', 'super::menu_form::')
    if 'crate::json_tree_example::' in body:
        dependencies.append('mod json_tree_example {\n' + fixture_source('json_tree_example') + '\n}')
        body = body.replace('crate::json_tree_example::', 'super::json_tree_example::')
    expression = f'fixture::{function}({arguments})'
    if scoped:
        expression = f'widget(move |scope| {expression}.0.show_in(scope))'
    return formatted('use glory::prelude::*;\n\npub fn demo(theme: sapid::Theme) -> impl Widget {\n    '
                     + expression + '\n}\n\n#[allow(dead_code, unused_imports)]\nmod fixture {\n'
                     + body + '\n}\n' + '\n'.join(dependencies))


pages = {}
entries = []
outputs = {}
modules = {}
for task in tasks:
    identity = task["id"]
    page = EXTRA_PAGES.get(identity) or next(iter(records[identity]["pages"]), None)
    if not page: raise SystemExit(f"missing explicit demo for {identity}")
    if identity in CORE:
        key = 'core_' + CORE[identity]
        code = (WEB / f"src/showcase_demos/{key}.rs").read_text(encoding="utf-8")
    else:
        key = page.replace('-', '_')
        pages[page] = key
        code = bundle(page)
    if len(code.encode()) > 65_536: raise SystemExit(f"source exceeds CodeHighlight budget: {identity}")
    outputs[WEB / f"src/showcase_demos/{key}.rs"] = code
    modules[key] = code
    description = DESCRIPTIONS[identity]
    values = [("id", identity), ("name", task["name"]), ("group", group(task)),
              ("description", description), ("example", page),
              ("web", records[identity]["web"]["state"]),
              ("native", records[identity]["native"]["state"]),
              ("native_note", records[identity]["native"]["notes"])]
    entries.append("    Entry { " + ", ".join(f"{key}: {literal(value)}" for key, value in values)
                   + f', code: include_str!("showcase_demos/{key}.rs"), build: crate::showcase_demos::{key}_widget' + " },")

catalog = '''//! Generated by `python scripts/showcase-catalog.py`; edit canonical inventory or demo sources.
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub id: &'static str,
    pub name: &'static str,
    pub group: &'static str,
    pub description: &'static str,
    pub example: &'static str,
    pub code: &'static str,
    pub build: fn(sapid::Theme) -> Box<dyn glory::ui::Widget>,
    pub web: &'static str,
    pub native: &'static str,
    pub native_note: &'static str,
}

#[rustfmt::skip]
pub const ENTRIES: &[Entry] = &[
''' + "\n".join(entries) + '''
];

pub fn find(id: &str) -> Option<&'static Entry> {
    ENTRIES.iter().find(|entry| entry.id == id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_covers_every_work_item_once_with_real_rust_examples() {
        assert_eq!(ENTRIES.len(), 149);
        for (index, entry) in ENTRIES.iter().enumerate() {
            assert!(ENTRIES[..index].iter().all(|other| other.id != entry.id));
            assert_eq!(find(entry.id).unwrap().name, entry.name);
            assert!(
                !entry.name.is_empty()
                    && !entry.group.is_empty()
                    && !entry.description.is_empty()
            );
            assert!(entry.code.contains("fn ") && !entry.code.contains("iframe"));
        }
        assert!(find("unknown").is_none());
    }
}
'''
registry = "//! Generated compiled widget examples; source panels include these exact files.\nuse glory::ui::Widget;\n"
for key in sorted(modules):
    registry += f"mod {key};\n"
    registry += f"pub fn {key}_widget(theme: sapid::Theme) -> Box<dyn Widget> {{ Box::new({key}::demo(theme)) }}\n"
outputs[WEB / "src/showcase_demos/mod.rs"] = formatted(registry)
outputs[CATALOG] = catalog
for path, content in outputs.items():
    if args.check:
        if not path.is_file() or path.read_text(encoding="utf-8") != content:
            raise SystemExit(f"stale generated showcase: {path}")
    else:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content, encoding="utf-8", newline="\n")
print(f"showcase catalogue: {len(entries)} entries, {len(pages)} Rust families, {len(CORE)} clean demos; all source modules compiled")
