"""Repeated previews of diagrams sharing a Metal texture must not invalidate each other.

Usage: python3 tools/harness/text_drawing_atlas.py [/tmp/fdl-drawing-atlas]
Uses a disposable library and never touches the system clipboard.
"""
from pathlib import Path
from PIL import Image, ImageChops
import os
import shutil
import subprocess
import sys
import tempfile

here = Path(__file__).resolve().parent
work = Path(sys.argv[1] if len(sys.argv) > 1 else '/tmp/fdl-drawing-atlas').resolve()
work.mkdir(parents=True, exist_ok=True)
doc_id = 'aaaaaaaa-0000-0000-0000-000000000001'

with tempfile.TemporaryDirectory(prefix='quiet-md-drawing-atlas-') as temp:
    temp = Path(temp)
    fixture = temp / 'library'
    shutil.copytree(here / 'lib_clean', fixture)
    xml = '''<title>Shared diagram texture regression</title>
<p>Repeated previews must preserve both diagrams.</p>
<text-drawing view="codeChart" theme="default">flowchart LR
 A[First] --&gt; B[Preview]</text-drawing>
<text-drawing view="codeChart" theme="default">flowchart LR
 C[Second] --&gt; D[Still visible]</text-drawing>
<p>Document end</p>
'''
    (fixture / 'docs' / f'{doc_id}.xml').write_text(xml)
    script = temp / 'atlas.txt'
    steps = ['resize 1280 1000', 'move 300 600', 'wait 1200', 'shot initial']
    for i in range(6):
        steps += ['click 1000 350', 'wait 600', f'shot preview-{i}', 'key escape', 'wait 500', f'shot closed-{i}']
    steps += ['move 300 600', 'shot complete']
    script.write_text('\n'.join(steps) + '\n')
    subprocess.run([sys.executable, str(here / 'run.py'), str(script), str(fixture), str(work)],
                   env=dict(os.environ, FDL_NOBLINK='1'), check=True)
    log = (work / 'log').read_text()
    assert 'panicked' not in log and 'fatal runtime error' not in log, log
    assert (work / 'complete.png').exists(), 'The app stopped before the last frame'
    initial = Image.open(work / 'initial.png').convert('RGB')
    scale = initial.width / 1280
    area = tuple(round(v * scale) for v in (800, 540, 1180, 750))
    other = initial.crop(area)
    assert other.getextrema()[0][0] < 200, 'The second diagram did not render in the fixture'
    for i in range(6):
        preview = Image.open(work / f'preview-{i}.png').convert('RGB')
        assert max(preview.getpixel((round(350 * scale), round(100 * scale)))) < 100, 'Preview did not open'
        closed = Image.open(work / f'closed-{i}.png').convert('RGB')
        assert closed.size == initial.size, 'Window size changed during the test'
        assert ImageChops.difference(other, closed.crop(area)).getbbox() is None, f'Preview {i} corrupted the other diagram sharing its texture'
    assert (work / 'lib' / 'docs' / f'{doc_id}.xml').read_text() == xml, 'Viewing diagrams changed the document'
print(f'Repeated diagram previews preserve shared textures: {work}')
