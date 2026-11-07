#!/usr/bin/env python3
"""Generate the README avatar strip from GitHub's contributors API."""
import base64
import html
import json
import os
from pathlib import Path
import subprocess
from urllib.parse import urlsplit, urlunsplit, parse_qsl, urlencode
from urllib.request import Request, urlopen

ROOT = Path(__file__).resolve().parent.parent


def main():
    repository = os.environ.get('GITHUB_REPOSITORY', 's1dashu/quiet-md')
    pages = json.loads(subprocess.check_output([
        'gh', 'api', '--paginate', '--slurp',
        f'repos/{repository}/contributors?per_page=100',
    ], text=True))
    contributors = [person for page in pages for person in page if person['type'] == 'User']
    if not contributors:
        raise RuntimeError('GitHub returned no contributors; keeping the existing image')
    contributors.sort(key=lambda person: (-person['contributions'], person['login'].lower()))
    columns = min(12, len(contributors))
    rows = (len(contributors) + columns - 1) // columns
    width, height = columns * 68 - 4, rows * 68 - 4
    parts = [f'<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{width}" height="{height}" viewBox="0 0 {width} {height}">',
             '<title>Quiet.md contributors</title>']
    for index, person in enumerate(contributors):
        url = urlsplit(person['avatar_url'])
        if url.scheme != 'https' or url.hostname != 'avatars.githubusercontent.com':
            raise ValueError('Unexpected GitHub avatar host')
        query = dict(parse_qsl(url.query))
        query['s'] = '128'
        avatar_url = urlunsplit(url._replace(query=urlencode(query)))
        with urlopen(Request(avatar_url, headers={'User-Agent': 'quiet-md-contributors'}), timeout=30) as response:
            content_type = response.headers.get_content_type()
            if content_type not in ('image/png', 'image/jpeg', 'image/webp'):
                raise ValueError(f'Unexpected avatar content type: {content_type}')
            avatar = base64.b64encode(response.read()).decode('ascii')
        name = html.escape(person['login'])
        x, y = index % columns * 68, index // columns * 68
        parts.extend([
            f'<g transform="translate({x} {y})"><title>{name}</title>',
            f'<defs><clipPath id="avatar-{index}"><circle cx="32" cy="32" r="31.5"/></clipPath></defs>',
            f'<image width="64" height="64" clip-path="url(#avatar-{index})" xlink:href="data:{content_type};base64,{avatar}"/>',
            '<circle cx="32" cy="32" r="31.5" fill="none" stroke="#c0c0c0" stroke-width="1"/>',
            '</g>',
        ])
    parts.append('</svg>')
    output = ROOT / 'docs/images/contributors.svg'
    content = '\n'.join(parts) + '\n'
    # Only replace the file after every API request and avatar download succeeds.
    if not output.exists() or output.read_text() != content:
        output.write_text(content)
    print(f'Generated {len(contributors)} contributors: ' + ', '.join(p['login'] for p in contributors))


if __name__ == '__main__':
    main()
