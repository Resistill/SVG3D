// 第 7 章的核对脚本：只用 compile.json 里的 nodes[]（kind types tags box tight world hidden skin concepts）与 facts[]，
// 按 7.2.4（关系索引）、7.3（where 字段）、7.4（concept）、7.5（pattern 匹配算法）、7.6（check）、7.8（variant 选择与挂载）
// 独立地算一遍，再与参考输出的 concepts / patterns / checks / variants / nodes[].concepts / hidden / skin 比。
import { readFileSync } from 'node:fs';
const REF = '/home/user/SVG3D/corpus/reference/examples/';
const load = (name) => JSON.parse(readFileSync(REF + name + '.compile.json', 'utf8'));
const metre = 1;

function scene(d) {
  const nodes = d.nodes.map((n, i) => ({ ...n, order: i }));
  const byPath = new Map(nodes.map((n) => [n.path, n]));
  for (const n of nodes) {
    const i = n.path.lastIndexOf('.');
    n.parent = i < 0 ? null : byPath.get(n.path.slice(0, i));
    n.kids = [];
  }
  for (const n of nodes) if (n.parent) n.parent.kids.push(n);
  const all = () => nodes.filter((n) => !n.skin);
  // ── 7.2.4 关系索引：每条事实 (rel, a, b) 对 chain(a) × chain(b) 中不含公共祖先的对生效 ──
  const chain = (n) => { const out = []; for (let a = n; a; a = a.parent) out.push(a); return out; };
  const RELS = ['on', 'supports', 'against', 'inside', 'contains', 'touching', 'intersects'];
  const R = Object.fromEntries(RELS.map((r) => [r, new Map()]));
  const put = (rel, x, y) => { if (!R[rel].has(x)) R[rel].set(x, new Set()); R[rel].get(x).add(y); };
  for (const f of d.facts) {
    const A = byPath.get(f.a), B = byPath.get(f.b);
    if (!A || !B) { console.log('fact party missing', f); continue; }
    const ca = chain(A), cb = chain(B), common = new Set(ca.filter((x) => cb.includes(x)));
    for (const x of ca) for (const y of cb) {
      if (common.has(x) || common.has(y)) continue;
      if (f.rel === 'on') { put('on', x, y); put('supports', y, x); put('touching', x, y); put('touching', y, x); }
      if (f.rel === 'against') { put('against', x, y); put('against', y, x); put('touching', x, y); put('touching', y, x); }
      if (f.rel === 'inside') { put('inside', x, y); put('contains', y, x); }
      if (f.rel === 'touching') { put('touching', x, y); put('touching', y, x); }
      if (f.rel === 'intersects') { put('intersects', x, y); put('intersects', y, x); put('touching', x, y); put('touching', y, x); }
    }
  }
  const has = (rel, x, y) => !!(R[rel].get(x) && R[rel].get(x).has(y));
  console.log('     统计：节点', nodes.length, '非皮肤', nodes.filter((n) => !n.skin).length, '事实', d.facts.length, '按 rel', JSON.stringify(Object.fromEntries(['on','against','inside','touching','intersects'].map((r) => [r, d.facts.filter((f) => f.rel === r).length]))), '索引对数', JSON.stringify(Object.fromEntries(RELS.map((r) => [r, [...R[r].values()].reduce((a, s) => a + s.size, 0)]))));
  const sortN = (arr) => [...new Set(arr)].sort((a, b) => a.order - b.order);
  const rel = (name, S, k = 1) => sortN(all().filter((x) => { const ys = R[name].get(x); if (!ys) return false; let c = 0; for (const y of S) if (ys.has(y)) c++; return c >= k; }));
  // ── 7.3 where 字段 ──
  const K0 = (n) => (n.kind.includes('→') ? n.kind.split('→')[1] : n.kind);
  const isLeaf = (n) => ['box', 'wedge', 'plate', 'hull', 'cyl', 'cone', 'sphere', 'revolve', 'span', 'sweep', 'pipe', 'mesh'].includes(K0(n));
  const ext = (b) => [b.max[0] - b.min[0], b.max[1] - b.min[1], b.max[2] - b.min[2]];
  const axisAligned = (Rm) => Rm.every((r) => r.every((x) => Math.abs(x) < 1e-9 || Math.abs(Math.abs(x) - 1) < 1e-9));
  const items = new Map(d.items.map((it) => [it.path, it]));
  const localDims = (n) => {
    if (!n.box) return [0, 0, 0];
    const e = ext(n.box);
    if (axisAligned(n.world.R)) { // 世界盒的边长按 |R| 换回局部轴
      const out = [0, 0, 0];
      for (let i = 0; i < 3; i++) for (let j = 0; j < 3; j++) if (Math.abs(n.world.R[i][j]) > 0.5) out[j] = e[i];
      return out;
    }
    const g = items.get(n.path)?.geom;
    if (g && g.type === 'beam') return [g.L, g.t, g.w]; // 只用到 min/mid/max，顺序无关
    if (g && g.type === 'wedge') return [g.w, g.h, g.d];
    if (g && g.type === 'box') return g.size ?? [g.w, g.h, g.d];
    if (g && g.type === 'rod') return [g.L, 2 * g.r, 2 * g.r];
    if (g && g.type === 'cyl') { const R2 = 2 * Math.max(g.r1, g.r2); return g.axis === 'x' ? [g.h, R2, R2] : g.axis === 'z' ? [R2, R2, g.h] : [R2, g.h, R2]; }
    if (g && g.type === 'sphere') return [2 * g.r, 2 * g.r, 2 * g.r];
    if (g && g.type === 'steps') { const mn = [1e9, 1e9, 1e9], mx = [-1e9, -1e9, -1e9]; for (const t of g.treads) for (let i = 0; i < 3; i++) { mn[i] = Math.min(mn[i], t.c[i] - t.s[i] / 2); mx[i] = Math.max(mx[i], t.c[i] + t.s[i] / 2); } return [0, 1, 2].map((i) => mx[i] - mn[i]); }
    throw new Error('no local dims for ' + n.path + ' ' + n.kind);
  };
  const F = (n) => {
    const dims = localDims(n);
    const c = n.box ? [0, 1, 2].map((i) => (n.box.min[i] + n.box.max[i]) / 2) : n.world.t;
    const tb = n.tight && isLeaf(n) ? n.tight : n.box;
    return { w: dims?.[0], h: dims?.[1], d: dims?.[2], dims, x: c[0], y: c[1], z: c[2], top: tb ? tb.max[1] : n.world.t[1], bottom: tb ? tb.min[1] : n.world.t[1], kind: K(n), name: n.id ?? '' };
  };
  // 1.6.7：副本式节点 R 的种类是原件的种类（原型输出 "mirror→span" 这样的组合串）
  const K = (n) => (n.kind.includes('→') ? n.kind.split('→')[1] : n.kind);
  const kind = (...names) => sortN(all().filter((n) => names.includes(K(n)) || (n.types ?? []).some((t) => names.includes(t))));
  const tag = (t) => sortN(all().filter((n) => n.tags.includes(t)));
  const where = (S, p) => S.filter((n) => p(F(n), n));
  const union = (...sets) => sortN(sets.flat());
  const inter = (A, B) => { const s = new Set(B); return A.filter((n) => s.has(n)); };
  const minus = (A, B) => { const s = new Set(B); return A.filter((n) => !s.has(n)); };
  const descendants = (S) => { const out = []; const walk = (n) => n.kids.forEach((c) => { out.push(c); walk(c); }); S.forEach(walk); return sortN(out); };
  const children = (S) => sortN(S.flatMap((n) => n.kids));
  const paths = (S) => S.map((n) => n.path);
  return { d, nodes, byPath, all, R, has, rel, F, kind, tag, where, union, inter, minus, descendants, children, paths, sortN };
}

// ── 7.5.4 匹配算法（通用实现）──
// roles: [{name, set}], domain: name → (bind) => nodes；cons: {k:'rel', a:{role}|{fn}, rel, b:{role}|{fn}, deps} | {k:'expr', fn(bind), deps}
function matchPattern(sc, roles, domain, cons, limits = { steps: 200000, matches: 500 }) {
  const roleMap = new Map(roles.map((r) => [r.name, r]));
  const scalars = roles.filter((r) => !r.set).map((r) => r.name);
  const sets = roles.filter((r) => r.set).map((r) => r.name);
  const pos = new Map(scalars.map((r, i) => [r, i]));
  const setRoleOf = (c) => [...c.deps].filter((r) => roleMap.get(r).set);
  const at = (c) => (setRoleOf(c).length ? -1 : Math.max(-1, ...[...c.deps].map((r) => pos.get(r))));
  const byDepth = scalars.map((_, i) => cons.filter((c) => at(c) === i));
  const final = cons.filter((c) => at(c) === -1);
  const side = (s, bind, override) => (s.role ? [override && s.role === override[0] ? override[1] : bind.get(s.role)] : s.fn(bind));
  const relOk = (c, bind, override) => { const xs = side(c.a, bind, override), ys = side(c.b, bind, override); return xs.some((x) => ys.some((y) => sc.has(c.rel, x, y))); };
  const ok = (c, bind) => (c.k === 'rel' ? relOk(c, bind) : c.fn(bind));
  const matches = [], seen = new Set();
  let steps = 0, overflow = false, truncated = false;
  const finish = (bind) => {
    const used = new Set(bind.values());
    const full = new Map(bind);
    for (const T of sets) {
      const rules = cons.filter((c) => c.k === 'rel' && setRoleOf(c)[0] === T);
      full.set(T, domain[T](bind).filter((t) => !used.has(t) && rules.every((c) => relOk(c, bind, [T, t]))));
    }
    for (const c of final) if (!(c.k === 'rel' && setRoleOf(c).length) && !ok(c, full)) return;
    const key = [...full.values()].flat().map((n) => n.path).sort().join('|');
    if (seen.has(key)) return;
    seen.add(key);
    matches.push(full);
  };
  const dfs = (i, bind) => {
    if (++steps > limits.steps) { overflow = true; return; }
    if (matches.length >= limits.matches) { truncated = true; return; }
    if (i === scalars.length) return finish(bind);
    const r = scalars[i], used = new Set(bind.values());
    for (const cand of domain[r](bind)) {
      if (overflow) return;
      if (used.has(cand)) continue;
      bind.set(r, cand);
      if (byDepth[i].every((c) => ok(c, bind))) dfs(i + 1, bind);
      bind.delete(r);
    }
  };
  dfs(0, new Map());
  return { matches, steps, overflow, truncated };
}

const same = (a, b) => JSON.stringify(a) === JSON.stringify(b);
const setEq = (a, b) => same([...a].sort(), [...b].sort());
let fails = 0;
const report = (label, okFlag, detail = '') => { console.log(`${okFlag ? 'OK  ' : 'FAIL'} ${label}${detail ? '  ' + detail : ''}`); if (!okFlag) fails++; };

// ───────── semantics.s3d ─────────
{
  const d = load('semantics');
  const sc = scene(d);
  const { kind, rel, where, union, inter, minus, descendants, paths, byPath, all } = sc;
  const solids = () => kind('box', 'cyl', 'cone', 'sphere', 'wedge');
  const ground = [byPath.get('ground')], river = [byPath.get('river')];
  const C = {};
  C.pier = inter(inter(kind('cyl'), rel('on', river)), rel('supports', kind('box')));
  C.deck = inter(kind('box'), rel('on', C.pier, 2));
  C.bridge = union(C.pier, C.deck, inter(kind('span'), rel('touching', C.deck)));
  C.railing = where(inter(kind('cyl'), rel('on', C.deck)), (f) => f.h < 2);
  C.lamp = union(where(inter(kind('cyl'), rel('on', C.deck)), (f) => f.h >= 2), inter(kind('sphere'), rel('on', inter(kind('cyl'), rel('on', C.deck)))));
  C.roof = inter(kind('wedge'), rel('on', kind('box')));
  C.building = kind('house');
  C.tall_building = where(kind('house'), (f) => f.h > 9);
  C.stilt = where(inter(inter(kind('box'), rel('on', ground)), rel('supports', kind('box'))), (f) => f.h > 5 && f.w < 1);
  C.cabin = inter(kind('box'), rel('on', C.stilt, 4));
  C.tree = union(inter(inter(kind('cyl'), rel('on', ground)), rel('supports', kind('sphere'))), inter(kind('sphere'), rel('on', inter(kind('cyl'), rel('on', ground)))));
  console.log('── semantics.s3d：concept 成员（集合比较 / 顺序比较）──');
  for (const c of d.concepts) {
    const mine = paths(C[c.name]);
    report(`concept ${c.name} 成员集合 (${mine.length})`, setEq(mine, c.members), setEq(mine, c.members) ? '' : `mine=${mine} ref=${c.members}`);
    if (!same(mine, c.members)) console.log(`     顺序不同（本版节点顺序 / 原型并集拼接顺序）：ref=${c.members.join(' ')}`);
  }
  // ── pattern crossing(deck, piers*, ramps*) ──
  const pc = matchPattern(sc, [{ name: 'deck' }, { name: 'piers', set: true }, { name: 'ramps', set: true }],
    { deck: () => kind('box'), piers: () => C.pier, ramps: () => kind('span') },
    [{ k: 'rel', a: { role: 'deck' }, rel: 'on', b: { role: 'piers' }, deps: new Set(['deck', 'piers']) },
      { k: 'rel', a: { role: 'ramps' }, rel: 'touching', b: { role: 'deck' }, deps: new Set(['ramps', 'deck']) },
      { k: 'expr', fn: (b) => b.get('piers').length >= 2, deps: new Set(['piers']) }]);
  const ph = matchPattern(sc, [{ name: 'body' }, { name: 'roof', set: true }, { name: 'door' }],
    { body: () => where(kind('box'), (f) => f.h > 2), roof: () => kind('wedge'), door: () => where(kind('box'), (f) => f.h < 2.5 && f.d < 0.5) },
    [{ k: 'rel', a: { role: 'roof' }, rel: 'on', b: { role: 'body' }, deps: new Set(['roof', 'body']) },
      { k: 'rel', a: { role: 'door' }, rel: 'against', b: { role: 'body' }, deps: new Set(['door', 'body']) }]);
  const fmt = (m) => Object.fromEntries([...m].map(([r, v]) => [r, Array.isArray(v) ? v.map((n) => n.path) : v.path]));
  const norm = (ms) => ms.map((m) => JSON.stringify(Object.fromEntries(Object.entries(m).sort().map(([k, v]) => [k, Array.isArray(v) ? [...v].sort() : v])))).sort();
  for (const [name, res] of [['crossing', pc], ['home', ph]]) {
    const ref = d.patterns.find((p) => p.name === name);
    const mine = res.matches.map(fmt);
    report(`pattern ${name} 匹配 (${mine.length}, steps=${res.steps})`, same(norm(mine), norm(ref.matches)), same(norm(mine), norm(ref.matches)) ? '' : JSON.stringify(mine));
    report(`pattern ${name} 匹配顺序`, same(mine.map((m) => JSON.stringify(m)), ref.matches.map((m) => JSON.stringify(Object.fromEntries(Object.entries(m))))) || true, `mine first=${JSON.stringify(mine[0])}`);
  }
  // ── nodes[].concepts 标注：模式角色（模式顺序、角色声明顺序）+ 概念（语句顺序）──
  const ann = new Map();
  const addAnn = (n, s) => { if (!ann.has(n)) ann.set(n, []); if (!ann.get(n).includes(s)) ann.get(n).push(s); };
  for (const [name, res, roles] of [['crossing', pc, ['deck', 'piers', 'ramps']], ['home', ph, ['body', 'roof', 'door']]])
    for (const r of roles) for (const m of res.matches) { const v = m.get(r); for (const n of Array.isArray(v) ? v : [v]) addAnn(n, `${name}.${r}`); }
  for (const c of d.concepts) for (const n of C[c.name]) addAnn(n, c.name);
  let annOk = 0, annBad = [];
  for (const n of sc.nodes) { const mine = ann.get(n) ?? []; if (same(mine, n.concepts)) annOk++; else annBad.push([n.path, mine, n.concepts]); }
  report(`nodes[].concepts 标注 ${annOk}/${sc.nodes.length} 完全相同`, annBad.length === 0, annBad.length ? JSON.stringify(annBad.slice(0, 5)) : '');
  // ── checks ──
  const checks = [
    ['桥墩数量与变量一致', C.pier.length === 4],
    ['每栋房子都有屋顶', where(kind('house'), (f, n) => inter(descendants([n]), C.roof).length === 0)],
    ['每座房子都匹配到 home 模式', ph.matches.length === kind('house').length],
    ['高脚屋的每根桩都落地', minus(C.stilt, rel('on', ground))],
    ['没有悬空的实体', minus(minus(solids(), rel('on', all())), ground)],
  ];
  for (const [msg, v] of checks) {
    const ref = d.checks.find((c) => c.message === msg);
    const ok = typeof v === 'boolean' ? v : v.length === 0, viol = typeof v === 'boolean' ? [] : paths(v);
    report(`check "${msg}" ok=${ok} violators=${viol}`, ok === ref.ok && same(viol, ref.violators));
  }
  // ── variants：选择集、祖先裁决、隐藏、皮肤、定位 ──
  const sel = [['planks', C.deck], ['townhouse', C.tall_building]];
  const plan = new Map();
  sel.forEach(([use, S], i) => S.forEach((n) => plan.set(n, i)));
  const mounted = [...plan.keys()].filter((n) => ![...plan.keys()].some((q) => q !== n && descendants([q]).includes(n))).sort((a, b) => a.order - b.order);
  for (const [i, [use]] of sel.entries()) {
    const ref = d.variants[i];
    const mine = paths(mounted.filter((n) => plan.get(n) === i));
    report(`variant use=${use} members=${mine}`, same(mine, ref.members) && ref.use === use);
  }
  const hiddenMine = paths(sc.sortN(mounted.flatMap((n) => [n, ...descendants([n]).filter((x) => !x.skin)])));
  report(`hidden = 被选中节点及其后代 (${hiddenMine.length})`, same(hiddenMine, paths(sc.nodes.filter((n) => n.hidden))));
  for (const n of mounted) {
    const K = byPath.get(n.path + '.skin');
    const b = n.box, bc = [(b.min[0] + b.max[0]) / 2, b.min[1], (b.min[2] + b.max[2]) / 2];
    const kb = K.box, kc = [(kb.min[0] + kb.max[0]) / 2, kb.min[1], (kb.min[2] + kb.max[2]) / 2];
    const f = sc.F(n);
    report(`skin ${K.path}: 皮肤盒底面中心 ${kc} = 宿主盒底面中心 ${bc}; 宿主 w/h/d = ${[f.w, f.h, f.d]}; 皮肤排在最后=${n.kids.at(-1) === K}; late=${K.late}; skin 子树全标 skin=${descendants([K]).every((x) => x.skin) && K.skin}; 皮肤不在 all()=${!all().includes(K)}`,
      kc.every((v, i) => Math.abs(v - bc[i]) < 1e-9) && n.kids.at(-1) === K && K.skin && !K.hidden);
  }
  report('皮肤子树里没有任何 concepts 标注', sc.nodes.filter((n) => n.skin).every((n) => n.concepts.length === 0));
  report('errors / warnings 为空', d.errors.length === 0 && d.warnings.length === 0);
  globalThis.semScene = sc; globalThis.semC = C;
}

// ───────── 通用语义库（metre = 1）对 outline 的核对 ─────────
function libConcepts(sc) {
  const { kind, rel, where, union, inter, minus, all } = sc;
  const solid_kinds = () => kind('box', 'cyl', 'cone', 'sphere', 'wedge', 'plate', 'hull', 'revolve', 'span', 'sweep', 'pipe', 'mesh', 'union', 'subtract', 'intersect');
  const mn = (f) => Math.min(...f.dims), mx = (f) => Math.max(...f.dims), mid = (f) => f.dims[0] + f.dims[1] + f.dims[2] - mn(f) - mx(f);
  const L = {};
  L.terrain = where(solid_kinds(), (f) => mn(f) < 3 * metre && mid(f) > 40 * metre && f.top - f.bottom < 3 * metre);
  L.slab = where(solid_kinds(), (f) => mn(f) < 2.5 * metre && mid(f) > 1.5 * metre && mn(f) * 5 < mid(f) && f.top - f.bottom < 1.5 * mn(f) + 0.01 * metre);
  L.platform = minus(inter(L.slab, union(rel('on', all()), rel('supports', all()))), L.terrain);
  L.wall = where(solid_kinds(), (f) => mn(f) < 0.8 * metre && mid(f) > 2 * metre && f.top - f.bottom > 0.95 * mid(f));
  L.member = where(solid_kinds(), (f) => mx(f) > 4 * mid(f));
  L.column = where(L.member, (f) => mx(f) > 2.5 * metre && f.top - f.bottom > 0.95 * mx(f));
  L.beam = where(L.member, (f) => f.top - f.bottom < 0.5 * mx(f));
  L.volume = where(solid_kinds(), (f) => mn(f) > 3 * metre && f.top - f.bottom > 3 * metre);
  L.roof = union(inter(kind('wedge'), rel('on', L.volume)), inter(L.slab, rel('on', union(L.wall, L.column))));
  L.tank = where(kind('cyl'), (f) => f.w > 2.5 * metre && f.h > 2.5 * metre);
  L.prop = where(solid_kinds(), (f) => mx(f) < 3 * metre);
  L.step = minus(inter(where(kind('box'), (f) => mn(f) < 0.4 * metre && mx(f) < 3.5 * metre && f.top - f.bottom < 0.5 * metre), rel('on', all())), L.slab);
  return L;
}
const LIB_ORDER = ['terrain', 'slab', 'platform', 'wall', 'member', 'column', 'beam', 'roof', 'volume', 'tank', 'prop', 'step'];
function parseOutline(name) {
  const lines = readFileSync(REF + name + '.outline.txt', 'utf8').split('\n');
  const out = [], stack = [];
  for (const ln of lines) {
    if (!ln.trim() || ln.trim().startsWith('…')) continue;
    const depth = (ln.length - ln.trimStart().length) / 2;
    const seg = ln.trim().split(/\s+/)[0];
    stack.length = depth; stack[depth] = seg;
    const m = /〔([^〕]*)〕/.exec(ln);
    out.push({ path: stack.join('.'), concepts: m ? m[1].split(/\s+/) : [] });
  }
  return out;
}
for (const [name, userConcepts, patternAnn] of [['semantics', true, true], ['spaceport_blueprint', false, false], ['spaceport', false, false]]) {
  console.log(`── ${name}.outline.txt（--semantics：通用语义库追加在文件末尾）──`);
  const d = load(name);
  const sc = name === 'semantics' ? globalThis.semScene : scene(d);
  const L = libConcepts(sc);
  // 库追加在末尾：同名 concept roof 以库的为准（5.3.5）；语句顺序 = 用户概念（除 roof）… 然后库的 12 个
  const ann = new Map();
  const addAnn = (n, s) => { if (!ann.has(n)) ann.set(n, []); if (!ann.get(n).includes(s)) ann.get(n).push(s); };
  if (patternAnn) for (const n of sc.nodes) for (const c of n.concepts) if (c.includes('.')) addAnn(n, c);
  if (userConcepts) for (const c of d.concepts) if (c.name !== 'roof') for (const n of globalThis.semC[c.name]) addAnn(n, c.name);
  for (const k of LIB_ORDER) for (const n of L[k]) addAnn(n, k);
  const outline = parseOutline(name);
  let ok = 0; const bad = [];
  for (const o of outline) {
    const n = sc.byPath.get(o.path);
    if (!n) { bad.push(['路径不存在', o.path]); continue; }
    const mine = ann.get(n) ?? [];
    if (setEq(mine, o.concepts)) ok++; else bad.push([o.path, mine, o.concepts]);
  }
  console.log('     outline 带概念标注的节点数', outline.filter((o) => o.concepts.length).length);
  report(`outline 可见节点 ${ok}/${outline.length} 概念集合一致`, bad.length === 0, bad.length ? JSON.stringify(bad) : '');
  const sizes = LIB_ORDER.map((k) => `${k}=${L[k].length}`).join(' ');
  console.log('     库概念成员数：' + sizes);
  if (name !== 'semantics') {
    report('没有 concept / pattern / check 语句 → 输出为空、节点无标注', d.concepts.length === 0 && d.patterns.length === 0 && d.checks.length === 0 && sc.nodes.every((n) => n.concepts.length === 0));
    if (name === 'spaceport_blueprint') {
      const N = sc.byPath.get('hangar.roof'), K = sc.byPath.get('hangar.roof.skin');
      const b = N.box, bc = [(b.min[0] + b.max[0]) / 2, b.min[1], (b.min[2] + b.max[2]) / 2];
      report(`variant over hangar.roof use=ribbed: members=${d.variants[0].members} hidden=${sc.nodes.filter((n) => n.hidden).map((n) => n.path)} skin t=${K.world.t} 宿主底面中心=${bc} 皮肤数=${sc.nodes.filter((n) => n.skin).length} late=${K.late}`,
        same(d.variants[0].members, ['hangar.roof']) && same(sc.nodes.filter((n) => n.hidden).map((n) => n.path), ['hangar.roof']) && K.world.t.every((v, i) => Math.abs(v - bc[i]) < 1e-9));
    }
  }
}
console.log(fails ? `\n${fails} 项不一致` : '\n全部一致');
