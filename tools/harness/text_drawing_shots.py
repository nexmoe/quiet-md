"""Capture the text-drawing PR illustrations with disposable notes and a fake clipboard.

Usage: python3 tools/harness/text_drawing_shots.py [/tmp/quiet-md-drawing-shots]
Uses the window server to include the native window background, without activating it.
"""
from pathlib import Path
from xml.sax.saxutils import escape
import json
import os
import shutil
import subprocess
import sys
import tempfile

here = Path(__file__).resolve().parent
work = Path(sys.argv[1] if len(sys.argv) > 1 else '/tmp/quiet-md-drawing-shots').resolve()
work.mkdir(parents=True, exist_ok=True)
doc_id = 'aaaaaaaa-0000-0000-0000-000000000001'
flow = '''flowchart TD
    A[编辑 Mermaid 源码] --> B{语法正确？}
    B -->|是| C[原生 SVG 预览]
    B -->|否| D[保留源码与上次预览]
    C --> E[点击放大 · 缩放 · 旋转]
    D --> A'''
emoji = '''mindmap
    id1["`**Pele** is the
goddess of volcanoes
Unicode works too: 🌋`"]
      id2["`She lives in **the** crater of Kīlauea... a *very old story* that is still told today`"]
      id3[Lava builds new land]'''

with tempfile.TemporaryDirectory(prefix='quiet-md-drawing-shots-') as temp:
    temp = Path(temp)
    for name, title, source, steps in [
        ('editing', 'Mermaid 文本绘图', flow, ['shot inline', 'click 1118 226', 'wait 250', 'shot views', 'key escape', 'click 1172 226', 'wait 350', 'shot fullscreen']),
        ('unicode', 'Pele 官方示例 · Unicode', emoji, ['click 1172 226', 'wait 350', 'shot fullscreen', 'click 1000 400', 'wait 600', 'move 300 600', 'shot image-preview']),
        ('recovery', '输入出错 · 内容依然保留', flow, ['click 1172 226', 'wait 300', 'click 500 120', 'key cmd-a', f'clip text {temp / "invalid.mmd"}', 'key cmd-v', 'wait 700', 'move 300 600', 'shot invalid', 'key cmd-z', 'wait 700', 'shot recovered']),
    ]:
        fixture = temp / name
        shutil.copytree(here / 'lib_clean', fixture)
        (fixture / 'preferences.json').write_text(json.dumps({'english': False, 'feishu': True, 'appearance': 'light', 'auto_update': False, 'remote_images': False, 'quick_hotkey': False, 'seen_version': '0.1.5'}))
        xml = f'<title>{title}</title>\n<p>源码、图表与并排视图 · 点击图表打开图片预览</p>\n<text-drawing view="codeChart" theme="default">{escape(source)}</text-drawing>\n<p></p>\n'
        (fixture / 'docs' / f'{doc_id}.xml').write_text(xml)
        (temp / 'invalid.mmd').write_text('flowchart TD\n    A[缺少闭合括号')
        script = temp / f'{name}.txt'
        script.write_text('\n'.join(['resize 1280 800', 'move 300 600', 'wait 1800', *steps]) + '\n')
        output = work / name
        subprocess.run([sys.executable, str(here / 'run.py'), str(script), str(fixture), str(output)],
                       env=dict(os.environ, FDL_NOBLINK='1', FDL_NOPARK='1', FDL_WINDOW_CAPTURE='1'), check=True)
        log = (output / 'log').read_text()
        assert 'panicked' not in log and 'unknown step' not in log, log
        print(f'{name}: {output}', flush=True)
