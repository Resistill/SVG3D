// 第 3 章规则的独立核对：用原型只取 (1) 属性值求值（第 1 章）、(2) 局部包围盒 / 曲面锚点 / 曲线采样（第 2 章），
// 本脚本按 spec/03-solving.md 的公式自己算每个节点的局部变换 T_n，再与参考输出 compile.json 的 world 比较：
//   W_mine = W_ref(parent) ∘ T_mine  vs  W_ref(n)
import fs from 'node:fs';
import * as P from './s3d_patched.js';
const { compile, DIRS, dirWord, _curves, FACES, AXES, CURVES, PATH_CURVES, LINKS } = P;
const EX = '/home/user/SVG3D/corpus/examples/';
const REF = '/home/user/SVG3D/corpus/reference/examples/';
const names = process.argv.slice(2).length ? process.argv.slice(2) : ['basics', 'curves', 'rotation', 'semantics', 'spaceport', 'spaceport_blueprint', 'spaceport_edit'];

// —— 向量与矩阵 ——
const add = (a, b) => [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
const sub = (a, b) => [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
const scl = (a, s) => [a[0] * s, a[1] * s, a[2] * s];
const dot = (a, b) => a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
const cross = (a, b) => [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
const len = (a) => Math.hypot(a[0], a[1], a[2]);
const unit = (a) => { const l = len(a); return l ? scl(a, 1 / l) : [0, 0, 0]; };
const rad = (d) => (d * Math.PI) / 180;
const I3 = [[1, 0, 0], [0, 1, 0], [0, 0, 1]];
const mv = (M, v) => M.map((r) => r[0] * v[0] + r[1] * v[1] + r[2] * v[2]);
const mm = (A, B) => A.map((r) => [0, 1, 2].map((j) => r[0] * B[0][j] + r[1] * B[1][j] + r[2] * B[2][j]));
const tr = (A) => [0, 1, 2].map((i) => [A[0][i], A[1][i], A[2][i]]);
const det3 = (A) => dot(A[0], cross(A[1], A[2]));
const cols = (a, b, c) => [[a[0], b[0], c[0]], [a[1], b[1], c[1]], [a[2], b[2], c[2]]];
const Ry = (a) => { const c = Math.cos(rad(a)), s = Math.sin(rad(a)); return [[c, 0, s], [0, 1, 0], [-s, 0, c]]; };
const Rx = (a) => { const c = Math.cos(rad(a)), s = Math.sin(rad(a)); return [[1, 0, 0], [0, c, -s], [0, s, c]]; };
const Rroll = (a) => { const c = Math.cos(rad(a)), s = Math.sin(rad(a)); return [[c, s, 0], [-s, c, 0], [0, 0, 1]]; };
const Raxis = (axis, a) => { const [x, y, z] = unit(axis), c = Math.cos(rad(a)), s = Math.sin(rad(a)), C = 1 - c;
  return [[c + x * x * C, x * y * C - z * s, x * z * C + y * s], [y * x * C + z * s, c + y * y * C, y * z * C - x * s], [z * x * C - y * s, z * y * C + x * s, c + z * z * C]]; };
const T3 = (R, t = [0, 0, 0]) => ({ R, t });
const ID = T3(I3);
const app = (T, p) => add(mv(T.R, p), T.t);
const compose = (A, B) => T3(mm(A.R, B.R), app(A, B.t));
const inv = (A) => { const Rt = tr(A.R); return T3(Rt, scl(mv(Rt, A.t), -1)); };
const reflect = (q, n) => T3([0, 1, 2].map((i) => [0, 1, 2].map((j) => (i === j ? 1 : 0) - 2 * n[i] * n[j])), scl(n, 2 * dot(q, n)));
const rotAbout = (q, axis, a) => { const M = Raxis(axis, a); return T3(M, sub(q, mv(M, q))); };
const perp = (v) => unit(cross(v, Math.abs(v[1]) < 0.9 ? [0, 1, 0] : [1, 0, 0]));
const corners = (b) => { const o = []; for (const x of [b.min[0], b.max[0]]) for (const y of [b.min[1], b.max[1]]) for (const z of [b.min[2], b.max[2]]) o.push([x, y, z]); return o; };
const bounds = (pts) => { const min = [1 / 0, 1 / 0, 1 / 0], max = [-1 / 0, -1 / 0, -1 / 0]; for (const p of pts) for (let k = 0; k < 3; k++) { min[k] = Math.min(min[k], p[k]); max[k] = Math.max(max[k], p[k]); } return pts.length ? { min, max } : { min: [0, 0, 0], max: [0, 0, 0] }; };
const dominant = (a) => { const ax = a.map(Math.abs), m = Math.max(...ax); if (m === 0) return [0, 0, 0]; if (ax[1] === m) return [0, Math.sign(a[1]), 0]; if (ax[0] === m) return [Math.sign(a[0]), 0, 0]; return [0, 0, Math.sign(a[2])]; };
const isRef = (v) => v && typeof v === 'object' && v.ref === true;
const isSet = (v) => v && typeof v === 'object' && Array.isArray(v.nodes) && !v.ref;
const mkRef = (node, anchor = null, offset = [0, 0, 0]) => ({ ref: true, node, anchor, offset });

let S, ref, stats, fails, tol = 1e-6;
const bump = (cat, ok, path, msg) => { stats[cat] ??= { ok: 0, bad: 0 }; if (ok) stats[cat].ok++; else { stats[cat].bad++; fails.push(`${cat} ${path} ${msg}`); } };

// —— 参考世界变换 ——
const Wref = (n) => { if (!n || n.kind === 'root' || n.isSet) return ID; const r = ref.get(n.path); if (!r || !r.world) throw new Error('no ref world for ' + n.path); return r.world; };
const Lref = (n) => compose(inv(Wref(n.parent)), Wref(n));
const relRef = (B, F) => (B.isSet ? inv(Wref(F)) : compose(inv(Wref(F)), Wref(B)));
const setBox = (sn) => bounds(sn.members.flatMap((m) => corners(ref.get(m.path).box)));
const boxOf = (n) => (n.isSet ? setBox(n) : S.box(n));

// —— 3.2 锚点 ——
function anchorLocal(n, a) {
  if (a === 'origin') return { p: [0, 0, 0], nrm: [0, 0, 0] };
  if (a && a.surf) return S.surfaceAnchor(n, a);
  if (a && a.t !== undefined) { const c = _curves.curveAt(S.curve(n), a.t); return { p: c.p, nrm: [0, 0, 0], tan: c.tan }; }
  const b = boxOf(n), c = scl(add(b.min, b.max), 0.5), h = scl(sub(b.max, b.min), 0.5);
  return { p: [c[0] + h[0] * a[0], c[1] + h[1] * a[1], c[2] + h[2] * a[2]], nrm: dominant(a) };
}
function anchorIn(r, F, dflt) {
  const a = anchorLocal(r.node, r.anchor ?? dflt), T = relRef(r.node, F);
  return { q: add(app(T, a.p), r.offset ?? [0, 0, 0]), nrm: mv(T.R, a.nrm), tan: a.tan && mv(T.R, a.tan) };
}
function pointOf(v, F) {
  if (Array.isArray(v) && v.length === 3) return v;
  if (isSet(v)) v = mkRef(S.setNode(v));
  if (isRef(v)) return anchorIn(v, F, DIRS.center).q;
  throw new Error('not a point');
}
const ev = (n, k, d) => S.eval(n, k, d);
const num = (n, k, d) => S.num(n, k, d);
function pinSpec(n) { // 3.4.1
  const ast = n.props.get('pin'); const a = ast && ast.k === 'envd' ? ast.ast : ast;
  if (a && a.k === 'path' && a.segs.length === 1) { if (a.segs[0] === 'origin') return 'origin'; const d = dirWord(a.segs[0]); if (d) return d; }
  return S.vec(n, 'pin');
}
// —— 3.3 关系 ——
function relation(n, Rm) {
  const Pn = n.parent, pin = n.props.has('pin') ? pinSpec(n) : null;
  const pinP = (d) => anchorLocal(n, pin ?? d).p;
  if (n.props.has('on')) { const r = S.ref(n, 'on'), a = anchorIn(r, Pn, DIRS.top); return { ref: r, q: a.q, nrm: mv(relRef(r.node, Pn).R, DIRS.top), pin: pinP(DIRS.bottom), kind: 'on' }; }
  if (n.props.has('under')) { const r = S.ref(n, 'under'), a = anchorIn(r, Pn, DIRS.bottom); return { ref: r, q: a.q, nrm: mv(relRef(r.node, Pn).R, DIRS.bottom), pin: pinP(DIRS.top), kind: 'under' }; }
  if (n.props.has('against')) {
    const r = S.ref(n, 'against'), a = anchorIn(r, Pn);
    let face = pin;
    if (!face) { let best = 1 / 0; for (const f of FACES) { const s = dot(mv(Rm, DIRS[f]), a.nrm); if (s < best) { best = s; face = DIRS[f]; } } }
    return { ref: r, q: a.q, nrm: a.nrm, pin: anchorLocal(n, face).p, kind: 'against' };
  }
  if (n.props.has('inside')) { const r = S.ref(n, 'inside'), a = anchorIn(r, Pn, DIRS.center); return { ref: r, q: a.q, nrm: [0, 0, 0], pin: pinP(DIRS.center), kind: 'inside' }; }
  if (n.props.has('at')) {
    const v = ev(n, 'at');
    if (isRef(v)) { const a = anchorIn(v, Pn, DIRS.center); return { ref: v, q: a.q, nrm: a.nrm, pin: pinP(DIRS.bottom), kind: 'at-ref' }; }
    return { ref: null, q: pointOf(v, Pn), nrm: [0, 0, 0], pin: pinP(DIRS.bottom), kind: 'at-pt' };
  }
  if (pin) return { ref: null, q: [0, 0, 0], nrm: [0, 0, 0], pin: pinP(pin), kind: 'pin-only' };
  return null;
}
// —— 3.5 朝向 ——
function orientation(n) {
  const Pn = n.parent; let base = I3;
  const rk = ['on', 'under', 'against', 'inside'].find((k) => n.props.has(k));
  if (rk) { base = relRef(S.ref(n, rk).node, Pn).R; if (det3(base) < 0) base = mm(base, [[-1, 0, 0], [0, 1, 0], [0, 0, 1]]); }
  if (n.kind === 'repeat') return base;
  let Rm = mm(base, Ry(num(n, 'turn', 0)));
  if (n.props.has('aim')) {
    const tgt = pointOf(ev(n, 'aim'), Pn), r0 = relation(n, Rm);
    const from = r0 ? r0.q : n.props.has('offset') ? S.vec(n, 'offset') : [0, 0, 0];
    const d = unit(sub(tgt, from));
    const a = unit(dirWord(S.sym(n, 'aimwith', 'front')));
    const b = Math.abs(a[1]) < 0.99 ? [0, 1, 0] : [0, 0, 1];
    const b1 = unit(sub(b, scl(a, dot(b, a))));
    let e = mv(Rm, b); e = sub(e, scl(d, dot(e, d))); e = len(e) < 1e-6 ? perp(d) : unit(e);
    Rm = mm(cols(d, e, cross(d, e)), tr(cols(a, b1, cross(a, b1))));
  }
  Rm = mm(Rm, mm(Rx(num(n, 'tilt', 0)), Rroll(num(n, 'roll', 0))));
  if (n.props.has('rot')) { const v = ev(n, 'rot'); Rm = mm(Rm, Raxis(v.slice(0, 3), v[3])); }
  return Rm;
}
// —— 3.9.5 方向规格 ——
function dirSpec(n, key, F, frameOf, dflt) {
  const v = n.props.has(key) ? ev(n, key) : null;
  const inFrame = (d) => (frameOf ? mv(relRef(frameOf, F).R, d) : d);
  if (v == null) return inFrame(dflt);
  if (typeof v === 'string') return unit(inFrame(AXES[v] ?? dirWord(v)));
  if (Array.isArray(v) && v.length === 3) return unit(v);
  if (isRef(v)) { if (v.anchor && v.anchor.t !== undefined) return unit(anchorIn(v, F).tan); if (Array.isArray(v.anchor)) return unit(mv(relRef(v.node, F).R, v.anchor)); }
  throw new Error('bad dir');
}
function mirrorIn(n, F) {
  const v = ev(n, 'across', 'x'); let q, nrm;
  if (typeof v === 'string') { q = [0, 0, 0]; nrm = AXES[v]; }
  else if (Array.isArray(v)) { q = v; nrm = dirSpec(n, 'axis', F, null, [1, 0, 0]); }
  else if (isRef(v)) {
    if (Array.isArray(v.anchor) && !n.props.has('axis') && len(v.anchor) > 0) { const a = anchorIn(v, F); q = a.q; nrm = a.nrm; }
    else { q = anchorIn(v, F, DIRS.center).q; nrm = dirSpec(n, 'axis', F, v.node, [1, 0, 0]); }
  } else throw new Error('bad across');
  return reflect(q, unit(nrm));
}
function rotationIn(n, F, k) {
  const count = n.src ? 1 : Math.round(num(n, 'n', 1)), angle = num(n, 'angle', 360 / (count + 1));
  const v = ev(n, 'about', null), q = v == null ? [0, 0, 0] : pointOf(v, F);
  return rotAbout(q, dirSpec(n, 'axis', F, null, [0, 1, 0]), angle * k);
}
// —— 3.8 实例 ——
function instLocal(n) {
  const R = n.parent, turn = num(R, 'turn', 0), { index, flat, grid } = n.inst;
  if (R.props.has('along')) {
    const c = S.curveRef(R, 'along'), cv = S.curve(c.node), N = R.total;
    const s = cv.closed ? flat / N : N > 1 ? flat / (N - 1) : 0;
    const a = anchorIn({ ...c, anchor: { t: s } }, R);
    const mode = ev(R, 'orient', 'yaw'); let M = I3;
    if (mode === 'yaw' && Math.hypot(a.tan[0], a.tan[2]) > 1e-9) M = Ry((Math.atan2(-a.tan[2], a.tan[0]) * 180) / Math.PI);
    if (mode === 'full') { const sg = cross(a.tan, [0, 1, 0]); const side = len(sg) > 1e-6 ? unit(sg) : [0, 0, 1]; const T = unit(a.tan); M = cols(T, cross(side, T), side); }
    return { T: T3(mm(M, Ry(turn * flat)), a.q), cat: 'instance-along-' + mode };
  }
  const step = S.vec(R, 'step', [0, 0, 0]), cv0 = ev(R, 'center', false), centered = cv0 === true || cv0 === 'true' || cv0 === 1;
  let t;
  if (grid) t = index.map((x, k) => (x - (centered ? ((R.counts[k] ?? 1) - 1) / 2 : 0)) * step[k]);
  else t = scl(step, flat - (centered ? (R.counts[0] - 1) / 2 : 0));
  return { T: T3(Ry(turn * flat), t), cat: grid ? 'instance-grid' : 'instance-linear' };
}
// —— 3.10 span ——
function spanLocal(n) {
  const Pn = n.parent, A = pointOf(ev(n, 'from'), Pn); let B;
  if (n.props.has('drop')) {
    const r = S.ref(n, 'drop'), dir = dirSpec(n, 'dir', Pn, null, [0, -1, 0]), face = anchorIn(r, Pn, DIRS.top);
    const nrm = len(face.nrm) > 0 ? face.nrm : mv(relRef(r.node, Pn).R, DIRS.top);
    const s = dot(sub(face.q, A), nrm) / dot(dir, nrm); B = add(A, scl(dir, s));
  } else B = pointOf(ev(n, 'to'), Pn);
  const d = sub(B, A), L = len(d), H = Math.hypot(d[0], d[2]), mid = scl(add(A, B), 0.5);
  if (n.props.has('steps')) return { T: T3(Ry(H < 1e-9 ? 0 : (Math.atan2(-d[2], d[0]) * 180) / Math.PI), mid), cat: 'span-steps' + (n.props.has('drop') ? '-drop' : '') };
  const ex = scl(d, 1 / L); let ey = sub([0, 1, 0], scl(ex, ex[1])); ey = len(ey) < 1e-6 ? perp(ex) : unit(ey);
  return { T: T3(cols(ex, ey, cross(ex, ey)), mid), cat: (n.props.has('r') ? 'span-rod' : 'span-beam') + (n.props.has('drop') ? '-drop' : '') };
}
// —— 总入口：按 3.1.2 的表 ——
function local(n) {
  if (!n.parent || n.isSet) return { T: ID, cat: 'root' };
  if (n.ghostOf) return { T: Lref(n.ghostOf.twin ? n.ghostOf.twinOf : n.ghostOf), cat: 'shadow' };
  if (n.src) { const Pn = n.parent; const M = n.kind === 'mirror' ? mirrorIn(n, Pn) : rotationIn(n, Pn, 1); return { T: compose(M, relRef(n.src, Pn)), cat: n.kind + '-of' }; }
  if (n.copyOf) { const { container: C, k } = n.copyOf; return { T: C.kind === 'mirror' ? mirrorIn(C, C) : rotationIn(C, C, k), cat: C.kind + '-group' }; }
  if (n.kind === 'span') return spanLocal(n);
  if (LINKS.has(n.kind)) return { T: ID, cat: n.kind };
  if (PATH_CURVES.has(n.kind)) return { T: ID, cat: 'pathcurve' };
  if (n.inst) return instLocal(n);
  const Rm = orientation(n); let t = [0, 0, 0];
  const r = relation(n, Rm); const parts = [];
  if (r) { t = sub(r.q, mv(Rm, r.pin)); parts.push(r.kind); if (n.props.has('pin')) parts.push('pin'); const g = num(n, 'gap', 0); if (g) { t = add(t, scl(r.nrm, g)); parts.push('gap'); } }
  else parts.push(n.skin ? 'skin' : 'none');
  if (n.props.has('align')) {
    parts.push(n.props.has('alignto') ? 'alignto' : 'align');
    const tgt = n.props.has('alignto') ? S.ref(n, 'alignto') : r.ref, Pn = n.parent;
    for (const f of S.names(n, 'align')) {
      const T0 = T3(Rm, t), ax = { x: 0, y: 1, z: 2 }[f];
      if (ax !== undefined) { const q = anchorIn({ ...tgt, anchor: DIRS.center, offset: [0, 0, 0] }, Pn).q, p = app(T0, anchorLocal(n, DIRS.center).p); t = [...t]; t[ax] += q[ax] - p[ax]; continue; }
      const dv = dirWord(f), a = anchorIn({ ...tgt, anchor: dv, offset: [0, 0, 0] }, Pn);
      let mine = dv, best = -1 / 0; for (const g of FACES) { const s = dot(mv(Rm, DIRS[g]), a.nrm); if (s > best) { best = s; mine = DIRS[g]; } }
      const p = app(T0, anchorLocal(n, mine).p); t = add(t, scl(a.nrm, dot(sub(a.q, p), a.nrm)));
    }
  }
  if (n.props.has('offset')) { t = add(t, S.vec(n, 'offset')); parts.push('offset'); }
  for (const k of ['turn', 'tilt', 'roll', 'rot', 'aim']) if (n.props.has(k)) parts.push(k);
  if (n.skin) parts.unshift('skin');
  return { T: T3(Rm, t), cat: 'general:' + parts.join('+') };
}

const maxDiff = (A, B) => Math.max(...A.R.flat().map((x, i) => Math.abs(x - B.R.flat()[i])), ...A.t.map((x, i) => Math.abs(x - B.t[i])));
for (const name of names) {
  const file = EX + name + '.s3d';
  const lib = fs.readFileSync('/home/user/resistill/svg3d-prototype/lib/semantics.s3d', 'utf8');
  const res = compile(fs.readFileSync(file, 'utf8'), { file, resolve: (p) => fs.readFileSync(p, 'utf8') });
  S = P.LAST_S;
  const J = JSON.parse(fs.readFileSync(REF + name + '.compile.json', 'utf8'));
  ref = new Map(J.nodes.map((n) => [n.path, n]));
  stats = {}; fails = [];
  let boxOk = 0, boxBad = 0, lateMine = [];
  for (const n of S.flat) {
    if (!n.parent || n.isSet) continue;
    const r = ref.get(n.path); if (!r) { fails.push('no ref node ' + n.path); continue; }
    let L;
    try { L = local(n); } catch (e) { bump('ERR', false, n.path, String(e.message)); continue; }
    const W = compose(Wref(n.parent), L.T);
    const d = maxDiff(W, r.world);
    bump(L.cat, d <= tol, n.path, 'Δ=' + d.toExponential(2));
    // 3.1.6：box = AABB(W(B_n))
    const b = bounds(corners(S.box(n)).map((p) => app(r.world, p)));
    const bd = Math.max(...b.min.map((x, i) => Math.abs(x - r.box.min[i])), ...b.max.map((x, i) => Math.abs(x - r.box.max[i])));
    if (bd <= tol) boxOk++; else { boxBad++; fails.push(`box ${n.path} Δ=${bd.toExponential(2)}`); }
    if (r.late) lateMine.push(n.path);
  }
  console.log(`\n== ${name}: ${S.flat.length - 1} nodes, ref errors ${J.errors.length}, my compile errors ${res.errors.length}`);
  for (const [k, v] of Object.entries(stats).sort()) console.log(`  ${k.padEnd(44)} ok ${String(v.ok).padStart(4)}  bad ${v.bad}`);
  console.log(`  box check ok ${boxOk} bad ${boxBad}; late in ref: ${JSON.stringify(lateMine)}`);
  for (const f of fails.slice(0, 20)) console.log('  FAIL ' + f);
}
