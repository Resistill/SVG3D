#!/usr/bin/env node
// 参考输出包的编译导出：把编译结果里新项目要对拍的部分写成 JSON。
//   node out/reference/tools/compile-dump.mjs <文件.s3d | 工程目录> <输出.json> [--slim] [--semantics]
// 输入是目录时按工程处理（读 project.json 的 unit，像 tools/project.mjs 的 compile 那样传 factEps / metre，
// 编译 scene.s3d，import 的路径相对工程目录）；输入是文件时仿 bin/s3d.mjs。
// 输出（完整）：{ meta, nodes, facts, errors, warnings, concepts }，nodes 是编译器原样的节点（含 world{R,t}、box、tight）。
// --slim：nodes 只留 path / kind / depth / tight / center（tight 的中心）/ worldT（world.t），facts 原样保留。
import { readFileSync, writeSync, openSync, closeSync, existsSync, statSync } from 'node:fs';
import { resolve, dirname } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const REPO = resolve(dirname(fileURLToPath(import.meta.url)), '../../..');
const { compile, FACT_EPS } = await import(pathToFileURL(resolve(REPO, 'src/s3d.js')).href);
const args = process.argv.slice(2);
const [input, output] = args.filter((a) => !a.startsWith('--'));
if (!input || !output) { console.error('用法: compile-dump.mjs <文件|工程目录> <输出.json> [--slim] [--semantics]'); process.exit(2); }
const slim = args.includes('--slim');

let r, meta;
if (statSync(input).isDirectory()) {
  const dir = resolve(input);
  const cfg = JSON.parse(readFileSync(resolve(dir, 'project.json'), 'utf8'));
  const unit = cfg.unit ?? 1;
  let heightAt = null;
  const { openProject } = await import(pathToFileURL(resolve(REPO, 'tools/project.mjs')).href);
  const P = openProject(dir);
  if (P.usesSurfaceAnchors()) { await P.sourceParts(); }
  r = P.compile();
  meta = { kind: 'project', dir: input, unit, scene: 'scene.s3d', usesSurfaceAnchors: P.usesSurfaceAnchors() };
} else {
  const lib = args.includes('--semantics') ? '\n' + readFileSync(resolve(REPO, 'lib/semantics.s3d'), 'utf8') : '';
  r = compile(readFileSync(input, 'utf8') + lib, { file: input, resolve: (p) => readFileSync(p, 'utf8') });
  meta = { kind: 'file', file: input, semantics: args.includes('--semantics') };
}
meta.slim = slim; meta.nodeCount = r.nodes.length; meta.factCount = r.facts.length;
meta.errorCount = r.errors.length; meta.warningCount = r.warnings.length;
const ctr = (b) => b ? b.min.map((v, i) => (v + b.max[i]) / 2) : null;
const nodeOut = (n) => slim
  ? { path: n.path, kind: n.kind, depth: n.depth, tight: n.tight ?? null, center: ctr(n.tight), worldT: n.world?.t ?? null }
  : n;

// 分段写，避免一次 stringify 超过字符串长度上限
const fd = openSync(output, 'w');
const w = (s) => writeSync(fd, s);
w('{"meta":' + JSON.stringify(meta) + ',"nodes":[');
r.nodes.forEach((n, i) => w((i ? ',\n' : '\n') + JSON.stringify(nodeOut(n))));
w('],"facts":[');
r.facts.forEach((f, i) => w((i ? ',\n' : '\n') + JSON.stringify(f)));
w('],"errors":' + JSON.stringify(r.errors) + ',"warnings":' + JSON.stringify(r.warnings) + ',"concepts":' + JSON.stringify(r.concepts) + '}\n');
closeSync(fd);
process.exitCode = r.errors.length ? 1 : 0;
