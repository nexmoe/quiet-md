"""Native sidebar drag regression. Uses only generated, throwaway libraries.

Build with --features devtools, then: python3 tools/harness/sidebar_drag.py
Screenshots and each resulting library are kept under the printed temporary path.
"""
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
WORK = Path(tempfile.mkdtemp(prefix="quiet-sidebar-drag-"))
VERSION = re.search(r'^version = "([^"]+)"', (HERE.parent.parent / "Cargo.toml").read_text(), re.M)[1]
# Every case must exercise the same build, even if another build runs meanwhile.
shutil.copy2(os.environ.get("FDL_BIN") or HERE.parent.parent / "target" / "release" / "quiet_md", WORK / "quiet_md")
ENV = {**os.environ, "FDL_BIN": str(WORK / "quiet_md")}


def fixture(path, extra=0, english=True, appearance="light", pinned=False):
    (path / "docs").mkdir(parents=True)
    rows = [("a", "Projects", 900, None), ("b", "Research", 800, None),
            ("c", "Writing", 700, None), ("d", "Weekly plan", 600, None),
            ("e", "Ideas", 500, None), ("a1", "Design notes", 400, "a"),
            ("a2", "Roadmap", 300, "a"), ("a11", "Draft", 200, "a1"),
            ("b1", "Reading list", 100, "b")]
    rows += [(f"extra{i:02}", f"Reference {i + 1:02}", 400 - i, None) for i in range(extra)]
    index = {}
    for i, (key, title, modified, parent) in enumerate(rows):
        (path / "docs" / f"{key}.xml").write_text(f"<title>{title}</title>\n<p>Notes for {title}.</p>\n")
        index[key] = dict(created=100 + i, modified=modified, opened=modified, order=i + 1)
        if parent:
            index[key]["parent"] = parent
        if pinned and key == "c":
            index[key]["pinned"] = True
    (path / "index.json").write_text(json.dumps(index))
    (path / "format.json").write_text('{"format":1}')
    (path / "settings.json").write_text('{"sidebar_width":300}')
    (path / "preferences.json").write_text(json.dumps(dict(english=english, appearance=appearance, theme="feishu", feishu=True, seen_version=VERSION)))
    return index


def run(name, steps, parents=None, roots=None, children=None, unchanged=False, moved=(), **options):
    source = WORK / f"source-{name}"
    original = fixture(source, **options)
    script = WORK / f"{name}.txt"
    script.write_text("resize 1280 800\nmove 900 700\nwait 700\n" + steps + "\nwait 350\nshot result\n")
    output = WORK / name
    subprocess.run(["python3", str(HERE / "run.py"), str(script), str(source), str(output)], env=ENV, check=True)
    log = (output / "log").read_text(errors="replace")
    assert "panicked" not in log, log
    actual = json.loads((output / "lib" / "index.json").read_text())
    for key, parent in (parents or {}).items():
        assert actual[key].get("parent") == parent, (name, key, actual[key])
    for key in original:
        assert (source / "docs" / f"{key}.xml").read_bytes() == (output / "lib" / "docs" / f"{key}.xml").read_bytes(), (name, key)
        if key not in (parents or {}) and key not in moved:
            assert actual[key].get("parent") == original[key].get("parent"), (name, key)
    if roots:
        assert sorted(roots, key=lambda key: actual[key]["root_order"]) == roots, (name, actual)
    if children:
        assert sorted(children, key=lambda key: actual[key]["order"]) == children, (name, actual)
    if unchanged:
        for key, meta in original.items():
            for field in ("parent", "order", "root_order", "modified"):
                assert actual[key].get(field) == meta.get(field), (name, key, field)
    print(f"PASS {name}", flush=True)
    return actual, output


print(WORK, flush=True)
run("nest", "down 120 202\ndragto 160 169\nwait 250\nshot parent-target\nup 160 169",
    parents={"c": "b"}, children=["b1", "c"])
run("return-to-root", "click 30 139\ndown 120 170\ndragto 160 109\nwait 200\nshot root-target\nup 160 109",
    parents={"a1": None}, roots=["a", "b", "c", "d", "e", "a1"])
run("reorder-roots", "down 120 263\ndragto 160 126\nwait 250\nshot insertion-line\nup 160 126",
    roots=["e", "a", "b", "c", "d"])
run("reorder-children", "click 30 139\ndown 120 201\ndragto 160 157\nwait 200\nup 160 157",
    children=["a2", "a1"])
run("move-subtree", "down 120 139\ndragto 160 170\nwait 200\nup 160 170", parents={"a": "b"})
run("reject-descendant", "click 30 139\nclick 48 170\ndown 120 139\ndragto 160 201\nwait 200\nshot invalid-target\nup 160 201", unchanged=True)
run("escape", "down 120 202\ndragto 160 169\nwait 200\nkey escape\nup 160 169", unchanged=True)
run("home-escape", "key cmd-alt-home\ndown 120 202\ndragto 160 169\nwait 200\nkey escape\nup 160 169", unchanged=True)
run("outside", "down 120 202\ndragto 600 350\nwait 200\nup 600 350", unchanged=True)
run("no-op", "down 120 263\ndragto 160 310\nwait 200\nup 160 310", unchanged=True)
run("hover-expand", "down 120 202\ndragto 160 139\nwait 850\nshot expanded\ndragto 160 201\nwait 100\nup 160 201", parents={"c": "a2"})
run("batch", "click 246 109\ndown 120 202\ndragto 160 169\nwait 200\nup 160 169", unchanged=True)
run("dark-chinese", "down 120 202\ndragto 160 169\nwait 250\nshot parent-target\nup 160 169", parents={"c": "b"}, english=False, appearance="dark")
pinned, _ = run("pinned", "click 120 139\ndown 120 277\ndragto 160 244\nwait 200\nup 160 244", parents={"c": "b"}, pinned=True)
assert pinned["c"]["pinned"] and pinned["c"]["opened"] > 700
scrolled, _ = run("scroll", "down 120 139\ndragto 160 740\nwait 1100\nshot scrolled\ndragto 160 705\nwait 100\nup 160 705", extra=40, moved=("a",))
parent = scrolled["a"].get("parent") or ""
assert (parent.startswith("extra") and int(parent[5:]) >= 20) or scrolled["a"].get("root_order", 0) > scrolled["extra20"].get("root_order", 1000), scrolled["a"]
print(f"All sidebar regressions passed. Artifacts: {WORK}", flush=True)
