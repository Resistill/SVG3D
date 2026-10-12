// 把工程的 score.json / compare-*.png / render 返回的 PNG / project.json 收进参考包。
//   node collect.mjs <名字> <工程目录>      （在仓库根目录跑）
import { readFileSync, copyFileSync, existsSync, readdirSync } from 'node:fs';
import { basename, join } from 'node:path';
const [name, proj] = process.argv.slice(2);
const dst = `out/reference/work/${name}`;
copyFileSync(join(proj, 'score.json'), join(dst, 'score.json'));
copyFileSync(join(proj, 'project.json'), join(dst, 'project.json'));
for (const f of readdirSync(join(proj, 'renders'))) if (/^compare-.*\.png$/.test(f)) copyFileSync(join(proj, 'renders', f), join(dst, f));
for (const v of ['iso', 'top', 'iso2']) {
  const j = JSON.parse(readFileSync(join(dst, `render-${v}.json`), 'utf8'));
  copyFileSync(j.file, join(dst, `render-${v}.png`));
}
console.log(name, 'collected', readdirSync(dst).join(' '));
