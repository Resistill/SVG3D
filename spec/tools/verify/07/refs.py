import re,glob,os
heads={}
for f in glob.glob('spec/*.md'):
    for line in open(f,encoding='utf-8'):
        m=re.match(r'#+\s+(\d+(?:\.\d+)*)\b',line)
        if m: heads[m.group(1)]=f
txt=open('spec/07-semantics.md',encoding='utf-8').read()
refs=set(re.findall(r'(?<![\d.])(\d{1,2}\.\d+(?:\.\d+)?)(?![\d.])',txt))
missing=sorted((r for r in refs if r not in heads and r.split('.')[0] in '1 2 3 4 5 6 7 8 9 10'.split()), key=lambda s:[int(x) for x in s.split('.')])
print("missing:",missing)
