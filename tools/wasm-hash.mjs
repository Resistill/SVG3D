#!/usr/bin/env node
// 三处哈希对拍的 wasm 一侧：用 node 跑 s3d-wasm，对每个语料文件打印 `sha256  名字`（与 `s3d hash` 同格式）。
// 用法：node tools/wasm-hash.mjs target/wasm32-unknown-unknown/release/s3d_wasm.wasm <文件.s3d>…
//       node tools/wasm-hash.mjs --score <s3d_wasm.wasm> <文件.s3d>…   评分的哈希：源模型取同目录的 <名>.glb
//       （<名>.abs.s3d / <名>.rev.s3d 也用 <名>.glb），与 `s3d score-hash` 同格式
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { basename, dirname, join } from 'node:path';

const argv = process.argv.slice(2);
const scoreMode = argv[0] === '--score';
const [wasmPath, ...files] = scoreMode ? argv.slice(1) : argv;
if (!wasmPath || !files.length) { console.error('用法: node tools/wasm-hash.mjs [--score] <s3d_wasm.wasm> <文件.s3d>…'); process.exit(2); }
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const { memory, s3d_alloc, s3d_free, s3d_compile, s3d_score } = instance.exports;
const glbOf = (f) => join(dirname(f), basename(f).replace(/\.s3d$/, '').replace(/\.(abs|rev)$/, '') + '.glb');

// 收集 import 链上的文件（按文件名索引，与 wasm 侧的查找方式一致）
function bundle(file) {
  const files = {};
  const visit = (p) => {
    const name = basename(p);
    if (files[name] !== undefined) return;
    const text = readFileSync(p, 'utf8');
    files[name] = text;
    for (const m of text.matchAll(/^\s*import\s+"([^"]+)"/gm)) visit(join(dirname(p), m[1]));
  };
  visit(file);
  return JSON.stringify({ main: basename(file), files });
}

function run(json, glb = null) {
  const bytes = new TextEncoder().encode(json);
  const inPtr = s3d_alloc(bytes.length);
  new Uint8Array(memory.buffer, inPtr, bytes.length).set(bytes);
  let outPtr, glbPtr = 0;
  if (glb) {
    glbPtr = s3d_alloc(glb.length);
    new Uint8Array(memory.buffer, glbPtr, glb.length).set(glb);
    outPtr = s3d_score(inPtr, bytes.length, glbPtr, glb.length);
  } else outPtr = s3d_compile(inPtr, bytes.length);
  const len = new DataView(memory.buffer).getUint32(outPtr, true);
  const out = new TextDecoder().decode(new Uint8Array(memory.buffer, outPtr + 4, len));
  s3d_free(inPtr, bytes.length);
  if (glb) s3d_free(glbPtr, glb.length);
  s3d_free(outPtr, 4 + len);
  return out;
}

for (const f of files) {
  const out = scoreMode ? run(bundle(f), new Uint8Array(readFileSync(glbOf(f)))) : run(bundle(f));
  console.log(`${createHash('sha256').update(out).digest('hex')}  ${basename(f)}`);
}
