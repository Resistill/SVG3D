#!/usr/bin/env python3
"""检查 spec/*.md 里的小节引用是否都指向存在的标题。

用法：python3 spec/tools/check-refs.py   （只报告，不改文件；退出码 1 表示有悬空引用）
规则：引用形如 "3.4.2"、"3.4.2 节"、"02-geometry.md 的 2.1.8"；章号 1–10 后至少一个 ".数字"。
排除：版本号 / 数值（前后紧跟数字、小数点、字母、`$`）、表格里的坐标。
"""
import re, pathlib, sys
SPEC = pathlib.Path(__file__).resolve().parents[1]
files = {f.name: f.read_text(encoding='utf-8') for f in sorted(SPEC.glob('*.md')) if f.name[0:2].isdigit()}
chapter_of = {int(n[:2]): n for n in files}
headings = {}
for name, text in files.items():
    for m in re.finditer(r'^#{2,4} (\d+(?:\.\d+)+)\b', text, re.M):
        headings.setdefault(int(m.group(1).split('.')[0]), set()).add(m.group(1))
# 允许引用的"小节号"= 标题号；"x.y.z 第 n 条/步"也只看 x.y.z
ref_re = re.compile(r'(?<![\d.\w$])(10|[1-9])\.(\d+)(?:\.(\d+))?(?:\.(\d+))?(?![\d.]|\s*[xX×]|\s*(?:m|cm|mm|秒|倍|度|°|%|个|条|行|位|点|次|字节|M|K|µ|e|\)))')
dangling = []
suspicious = []
for name, text in files.items():
    in_code = False
    for ln_no, ln in enumerate(text.split('\n'), 1):
        if ln.strip().startswith('```'):
            in_code = not in_code; continue
        if in_code or '$' in ln and ln.count('$') >= 2 and re.search(r'\$[^$]*\d+\.\d+[^$]*\$', ln) and not re.search(r'\d+\.\d+\.\d+', ln):
            pass
        # 去掉行内公式与代码，避免数值
        scrub = re.sub(r'\$[^$]*\$', ' ', ln)
        scrub = re.sub(r'`[^`]*`', ' ', scrub)
        for m in ref_re.finditer(scrub):
            ref = m.group(0)
            parts = ref.split('.')
            ch = int(parts[0])
            if ch not in chapter_of: continue
            # 两段的引用（如 "3.4"）只在后面不跟数字时算小节号；允许
            if ref not in headings.get(ch, set()):
                if len(parts) == 2:
                    suspicious.append((name, ln_no, ref, scrub.strip()[:120]))  # 两段的多半是小数（1.94、5.6），只提示
                else:
                    dangling.append((name, ln_no, ref, scrub.strip()[:120]))
for d in dangling:
    print(f'{d[0]}:{d[1]} {d[2]} :: {d[3]}')
print(f'\n{len(dangling)} dangling references (3+ 段)；{len(suspicious)} 个两段数字不是标题（多半是小数，用 --all 查看）')
if '--all' in sys.argv:
    for d in suspicious: print(f'  ? {d[0]}:{d[1]} {d[2]} :: {d[3]}')
sys.exit(1 if dangling else 0)
