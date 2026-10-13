import re,glob,os,sys
spec='/home/user/SVG3D/spec/'
heads={}
for f in glob.glob(spec+'0*.md')+glob.glob(spec+'10-*.md'):
    for line in open(f,encoding='utf-8'):
        m=re.match(r'#+\s+(\d+(?:\.\d+)+)\b',line)
        if m: heads[m.group(1)]=os.path.basename(f)
# also chapter-level like 4.7 (## 4.7 ...)
cw=open(spec+'crosswalk.md',encoding='utf-8').read()
refs=set(re.findall(r'(?<![\d.])(\d{1,2}\.\d+(?:\.\d+)*)(?![\d.])',cw))
missing=sorted((r for r in refs if r not in heads and r.split('.')[0] in '1 2 3 4 5 6 7 8 9 10'.split() and r.count('.')>=1), key=lambda s:[int(x) for x in s.split('.')])
print('missing:',missing)
