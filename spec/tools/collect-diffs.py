#!/usr/bin/env python3
"""把各章末尾"与原型的差异"表逐行转引进 09-compat.md 的 9.3.6。

用法：在仓库根目录运行 `python3 spec/tools/collect-diffs.py`。
脚本只重写 9.3.6 小节（从 "### 9.3.6" 到 "## 9.4" 之间），其他内容不动。
"""
import re, pathlib

ROOT = pathlib.Path(__file__).resolve().parents[2]
SPEC = ROOT / 'spec'

# (文件, 小节标题前缀, 列的含义：节 / 情形 / 原型 / 本版 / 备注 的列号，从 0 起；None 表示该表没有这一列)
TABLES = [
    ('01-syntax.md',     '## 1.11',   dict(sec=0, item=1, proto=2, new=3, note=4, note_name='类')),
    ('02-geometry.md',   '### 2.8.3', dict(sec=0, item=1, proto=2, new=3, note=4, note_name='语料')),
    ('03-solving.md',    '### 3.11.5', dict(sec=0, item=1, proto=2, new=3, note=4, note_name='理由')),
    ('04-derivation.md', '### 4.8.5', dict(sec=None, item=0, proto=1, new=2, note=3, note_name='理由')),
    ('05-layers.md',     '### 5.5.4', dict(sec=0, item=1, proto=2, new=3, note=4, note_name='理由')),
    ('06-facts.md',      '### 6.8.5', dict(sec=0, item=1, proto=2, new=3, note=4, note_name='理由')),
    ('07-semantics.md',  '### 7.9.4', dict(sec=0, item=1, proto=2, new=3, note=4, note_name='理由')),
    ('08-metadata.md',   '### 8.6.2', dict(sec=None, item=0, proto=1, new=2, note=3, note_name='语料')),
]

def section_lines(text, heading):
    lines = text.split('\n'); out = []; on = False
    for ln in lines:
        if ln.startswith(heading):
            on = True; continue
        if on and ln.startswith('#'):
            break
        if on:
            out.append(ln)
    return out

def split_row(ln):
    # 表格行：去掉首尾的 |，按未转义的 | 切分
    body = ln.strip()
    assert body.startswith('|')
    body = body[1:]
    if body.endswith('|'):
        body = body[:-1]
    cells = re.split(r'(?<!\\)\|', body)
    return [c.strip() for c in cells]

def collect():
    blocks = []
    for fname, heading, cols in TABLES:
        text = (SPEC / fname).read_text(encoding='utf-8')
        rows = []
        for ln in section_lines(text, heading):
            if not ln.startswith('| '):
                continue
            cells = split_row(ln)
            if set(''.join(cells)) <= set('-: '):
                continue  # 分隔行
            if cells[cols['item']] in ('情形', '项'):
                continue  # 表头
            sec = cells[cols['sec']] if cols['sec'] is not None else ''
            rows.append((sec, cells[cols['item']], cells[cols['proto']], cells[cols['new']],
                         cells[cols['note']] if cols['note'] is not None and cols['note'] < len(cells) else ''))
        blocks.append((fname, heading.lstrip('# '), cols['note_name'], rows))
    return blocks

def render(blocks):
    out = ['### 9.3.6 各章差异表的逐行转引', '',
           '9.3.1–9.3.5 是按类别整理的摘要。**完整清单以本节为准**：下面的表由 `spec/tools/collect-diffs.py` 从各章末尾的差异表逐行转引（不改一字），各章的表是定义处，本节随之再生成；读者在 9.3.1–9.3.5 里找不到的项到这里查。第 10 章没有单独的表，它的差异已逐条写进 9.3.1–9.3.5。', '']
    total = 0
    for fname, heading, note_name, rows in blocks:
        out.append(f'**{fname} 的 {heading}**（{len(rows)} 行）')
        out.append('')
        out.append(f'| 节 | 情形 / 项 | 原型 | 本版 | {note_name} |')
        out.append('|---|---|---|---|---|')
        for sec, item, proto, new, note in rows:
            out.append(f'| {sec} | {item} | {proto} | {new} | {note} |')
        out.append('')
        total += len(rows)
    out.insert(3, f'共 {total} 行。')
    out.insert(4, '')
    return '\n'.join(out)

def main():
    p = SPEC / '09-compat.md'
    s = p.read_text(encoding='utf-8')
    body = render(collect())
    start = s.find('### 9.3.6')
    end = s.find('\n## 9.4')
    assert end > 0
    if start < 0:
        start = end + 1
        s = s[:start] + body + '\n\n' + s[start:]
    else:
        s = s[:start] + body + '\n' + s[end:]
    p.write_text(s, encoding='utf-8')
    print('9.3.6 written')

if __name__ == '__main__':
    main()
