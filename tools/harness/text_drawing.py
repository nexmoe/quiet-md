"""Exercise text drawing against a disposable library, using only the fake clipboard.

Usage: python3 tools/harness/text_drawing.py [/tmp/fdl-text-drawing]
FDL_BIN may select a build. Screenshots and the saved library stay in the output directory.
"""
from pathlib import Path
from PIL import Image
import json
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET

here = Path(__file__).resolve().parent
work = Path(sys.argv[1] if len(sys.argv) > 1 else '/tmp/fdl-text-drawing').resolve()
work.mkdir(parents=True, exist_ok=True)
doc_id = 'aaaaaaaa-0000-0000-0000-000000000001'
valid = 'flowchart TD\n    A[中文测试] --> B[Native Rust]\n    B --> C[已保存]'

with tempfile.TemporaryDirectory(prefix='quiet-md-drawing-fixture-') as temp:
    fixture = Path(temp) / 'library'
    shutil.copytree(here / 'lib_clean', fixture)
    (fixture / 'docs' / f'{doc_id}.xml').write_text('''<title>Text drawing verification</title>
<p>Native Rust Mermaid preview — 中文与 English</p>
<text-drawing view="codeChart" theme="default" record='{"data":"","quiet_md":{"zoom":3.0,"split":0.5,"pan":[-77.0,-99.0]}}'>flowchart LR
    A[开始 Start] --&gt; B{保存 Save?}
    B --&gt;|是 Yes| C[完成 Done]
    B --&gt;|否 No| A</text-drawing>
<p></p>
''')
    good = Path(temp) / 'valid.mmd'
    bad = Path(temp) / 'invalid.mmd'
    good.write_text(valid)
    bad.write_text('flowchart TD\n    A[unterminated')
    script = Path(temp) / 'drawing.txt'
    script.write_text(f'''resize 1280 800
move 300 600
wait 2500
click 1172 226
wait 300
click 500 120
key cmd-a
clip text {good}
key cmd-v
wait 800
shot full-edited
key cmd-z
wait 800
shot full-undo
key cmd-shift-z
wait 800
shot full-redo
key cmd-a
clip text {bad}
key cmd-v
wait 800
shot syntax-error
key cmd-z
wait 800
shot syntax-undo
key escape
wait 400
shot inline
click 1118 226
wait 250
shot views
click 1050 326
wait 400
shot chart-only
click 1118 226
wait 200
click 1050 262
wait 300
drag 780 310 855 310
wait 300
shot split
click 1000 350
wait 700
move 1100 600
shot image-preview
key up
key up
key up
wait 400
shot preview-zoom
key r
wait 400
shot preview-rotate
drag 1000 380 1080 300
wait 300
shot preview-pan
key escape
wait 400
move 300 600
shot preview-closed
''')
    subprocess.run([sys.executable, str(here / 'run.py'), str(script), str(fixture), str(work)], check=True)
    log = (work / 'log').read_text()
    assert 'panicked' not in log and 'unknown step' not in log, log
    reopen = Path(temp) / 'reopen.txt'
    reopen.write_text('resize 1280 800\nmove 300 600\nwait 2500\nshot reopened\n')
    subprocess.run([sys.executable, str(here / 'run.py'), str(reopen), str(work / 'lib'), str(work / 'reopen')], check=True)

saved = (work / 'lib' / 'docs' / f'{doc_id}.xml').read_text()
drawing = ET.fromstring(f'<document>{saved}</document>').find('text-drawing')
assert drawing is not None and drawing.text == valid, 'Source or undo/redo did not survive saving'
assert drawing.get('view') == 'codeChart', 'View selection did not persist'
assert drawing.get('theme') == 'default', 'Editing changed the imported theme'
layout = json.loads(drawing.get('record', '{}')).get('quiet_md', {})
assert layout.get('zoom', 1) == 3, 'Viewing the chart changed its saved zoom'
assert layout.get('split', 0.5) > 0.5, 'Divider position did not persist'
assert layout.get('pan') == [-77, -99], 'Viewing the chart changed its saved pan'
assert (work / 'reopen' / 'lib' / 'docs' / f'{doc_id}.xml').read_text() == saved, 'Reopening changed the saved drawing'
assert max(Image.open(work / 'image-preview.png').getpixel((700, 200))[:3]) < 100, 'Click did not open the image viewer'
assert min(Image.open(work / 'preview-closed.png').getpixel((700, 200))[:3]) > 220, 'Escape did not close the image viewer'
print(f'Text drawing source, history, view and viewport verified; screenshots: {work}')
