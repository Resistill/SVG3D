// 重新按源码数每个文件的语句行 / 注释行 / 空行，并与 crosswalk.md 里的区间与注释行号对照
import fs from 'node:fs';
const root = '/home/user/SVG3D/corpus/examples/';
const files = ['basics.s3d','curves.s3d','rotation.s3d','semantics.s3d','spaceport.s3d','spaceport_blueprint.s3d','spaceport_edit.s3d','lib/semantics.s3d'];
// crosswalk 里声称的覆盖区间（覆盖率一行）与注释/空行
const claimed = {
 'basics.s3d': { stmt: '3–6、8、11–13、16–18、21–25、28–32、35–36', cmt: '1、10、15、20、27、34', blank: '2、7、9、14、19、26、33' },
 'curves.s3d': { stmt: '5–11、13、16–18、20–22、25–31、34–39、42、45、48–50、52–56、59–61、63–68、71', cmt: '1–3、15、19、24、33、41、44、47、51、58、62、70', blank: '4、12、14、23、32、40、43、46、57、69' },
 'rotation.s3d': { stmt: '3–8、10、13–16、18–24、27–38、41–44、47–56、59–70', cmt: '1、12、17、26、40、46、58', blank: '2、9、11、25、39、45、57' },
 'semantics.s3d': { stmt: '8–11、13–22、24–25、28–36、38–41、44–49、51–59、62–66、68–73、76、78–88、91–98、101–115、118–131、134–138、141–146', cmt: '1–5、7、27、37、43、50、61、67、75、90、100、117、133、140', blank: '6、12、23、26、42、60、74、77、89、99、116、132、139' },
 'spaceport.s3d': { stmt: '4–10、12、15、17–20、22、24–27、30–35、37–39、41、44、47–48、51–54、57–61、63–66、68、71、74–78、81–82、85–89、92–95、98–101、104–112', cmt: '1、2、14、21、23、29、36、43、46、50、56、62、70、73、80、84、91、97、103', blank: '3、11、13、16、28、40、42、45、49、55、67、69、72、79、83、90、96、102' },
 'spaceport_blueprint.s3d': { stmt: '4、6–12、15–20', cmt: '1–3、14', blank: '5、13' },
 'spaceport_edit.s3d': { stmt: '4、6–10、13–15', cmt: '1–3、12', blank: '5、11' },
 'lib/semantics.s3d': { stmt: '11、14、16、17、19、21、23、24、26、28、30、32、34', cmt: '1–8、10、13、15、18、20、22、25、27、29、31、33', blank: '9、12' },
};
const expand = s => { const out = new Set(); for (const part of s.split('、')) { const m = part.match(/^(\d+)(?:[–-](\d+))?$/); if (!m) throw new Error('bad ' + part); const a = +m[1], b = m[2] ? +m[2] : a; for (let i = a; i <= b; i++) out.add(i); } return out; };
let totS = 0, totC = 0, totB = 0, totL = 0;
for (const f of files) {
  const text = fs.readFileSync(root + f, 'utf8');
  const lines = text.split('\n');
  if (lines[lines.length - 1] === '') lines.pop();
  const stmt = new Set(), cmt = new Set(), blank = new Set();
  lines.forEach((l, i) => { const t = l.trim(); const n = i + 1; if (t === '') blank.add(n); else if (t.startsWith('//')) cmt.add(n); else stmt.add(n); });
  const c = claimed[f];
  const cs = expand(c.stmt), cc = expand(c.cmt), cb = expand(c.blank);
  const diff = (a, b) => [...a].filter(x => !b.has(x));
  console.log(`${f}: lines ${lines.length} stmt ${stmt.size} cmt ${cmt.size} blank ${blank.size} | claimed stmt ${cs.size} cmt ${cc.size} blank ${cb.size}`);
  const d1 = diff(stmt, cs), d2 = diff(cs, stmt), d3 = diff(cmt, cc), d4 = diff(cc, cmt), d5 = diff(blank, cb), d6 = diff(cb, blank);
  if (d1.length || d2.length || d3.length || d4.length || d5.length || d6.length) console.log('  MISMATCH', { stmtMissing: d1, stmtExtra: d2, cmtMissing: d3, cmtExtra: d4, blankMissing: d5, blankExtra: d6 });
  totS += stmt.size; totC += cmt.size; totB += blank.size; totL += lines.length;
}
console.log({ totL, totS, totC, totB });
