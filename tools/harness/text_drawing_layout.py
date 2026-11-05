"""Regression for drawing edits moving the outer page or changing the block height.

Usage: python3 tools/harness/text_drawing_layout.py [/tmp/fdl-drawing-layout]
Requires Pillow, like compare.py. Uses a disposable library and fake clipboard.
"""
from pathlib import Path
from PIL import Image
import os
import shutil
import subprocess
import sys
import tempfile
import xml.etree.ElementTree as ET

here = Path(__file__).resolve().parent
work = Path(sys.argv[1] if len(sys.argv) > 1 else '/tmp/fdl-drawing-layout').resolve()
work.mkdir(parents=True, exist_ok=True)
doc_id = 'aaaaaaaa-0000-0000-0000-000000000001'
long_source = 'flowchart TD\n A[Start] --> B[Done]\n' + '\n'.join(f'%% comment {i}' for i in range(80))
env = dict(os.environ, FDL_NOBLINK='1')


def markers(path):
    im = Image.open(path).convert('RGB')
    # Red is reserved for the two document anchors; chart errors are in the right pane.
    rows = []
    for y in range(100, im.height):
        row = im.crop((int(im.width * .30), y, int(im.width * .46), y + 1))
        if sum(r > 140 and r > g * 1.5 and r > b * 1.5 for r, g, b in row.getdata()) > 10:
            rows.append(y)
    assert rows, f'No document anchor visible in {path}'
    return rows


with tempfile.TemporaryDirectory(prefix='quiet-md-drawing-layout-') as temp:
    temp = Path(temp)
    fixture = temp / 'library'
    shutil.copytree(here / 'lib_clean', fixture)
    good, bad, empty = [temp / name for name in ['long.mmd', 'bad.mmd', 'empty.mmd']]
    good.write_text(long_source)
    bad.write_text('flowchart TD\n A[unterminated')
    empty.write_text('')
    for name, paragraphs, click_y in [('partial', 10, 670), ('geometry', 0, 330)]:
        before = '<title>Drawing layout regression</title>\n' + ''.join(f'<p>Paragraph {i}</p>\n' for i in range(paragraphs))
        before += '<p><span text-color="red">ABOVE DRAWING ANCHOR</span></p>\n'
        before += '<text-drawing view="codeChart" theme="default">flowchart LR\n A[Start] --&gt; B[Done]</text-drawing>\n'
        before += '<p><span text-color="red">BELOW DRAWING ANCHOR</span></p>\n'
        (fixture / 'docs' / f'{doc_id}.xml').write_text(before)
        script = temp / f'{name}.txt'
        preview_steps = 'click 1000 350\nwait 500\nkey up\nkey up\nkey r\nwait 400\nkey escape\nwait 400\nshot preview-closed\nclick 550 330\n' if name == 'geometry' else ""
        view_steps = 'click 1118 226\nwait 200\nclick 1050 326\nwait 400\nshot view\nkey cmd-z\nwait 400\nshot view-undo\nkey cmd-shift-z\nwait 400\nshot view-redo\nclick 1118 226\nwait 200\nclick 1050 262\nwait 400\n' if name == 'geometry' else ''
        script.write_text(f'''resize 1280 800
move 300 600
wait 1000
shot initial
{view_steps}# Click below the short source, inside the blank editor area.
click 550 {click_y}
key cmd-a
clip text {good}
key cmd-v
wait 20
shot pending
wait 800
shot rendered
key cmd-z
wait 600
shot undo
key cmd-shift-z
wait 600
shot redo
{preview_steps}key cmd-a
clip text {bad}
key cmd-v
wait 600
shot error
key cmd-a
clip text {empty}
key cmd-v
wait 600
shot empty
''')
        target = work / name
        subprocess.run([sys.executable, str(here / 'run.py'), str(script), str(fixture), str(target)], env=env, check=True)
        expected = markers(target / 'initial.png')
        for shot in ['pending', 'rendered', 'undo', 'redo', 'error', 'empty'] + (['preview-closed'] if preview_steps else []) + (['view', 'view-undo', 'view-redo'] if view_steps else []):
            assert markers(target / f'{shot}.png') == expected, f'{name}: document moved during {shot}'
        saved = ET.fromstring('<document>' + (target / 'lib' / 'docs' / f'{doc_id}.xml').read_text() + '</document>')
        assert not (saved.find('text-drawing').text or ''), f'{name}: blank-pane click failed to focus drawing source'
        assert len(saved.findall('p')) == paragraphs + 2, f'{name}: drawing edit changed outer paragraphs'
        assert 'ABOVE DRAWING ANCHOR' in ''.join(saved.itertext())
        log = (target / 'log').read_text()
        assert 'panicked' not in log and 'unknown step' not in log, log
print(f'Drawing loading, source edits, undo/redo, errors and empty state preserve document anchors: {work}')
