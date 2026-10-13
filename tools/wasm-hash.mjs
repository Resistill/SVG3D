#!/usr/bin/env node
// 三处哈希对拍的 wasm 一侧：用 node 跑 s3d-wasm，对每个语料文件打印 `sha256  名字`（与 `s3d hash` 同格式）。
// 用法：node tools/wasm-hash.mjs target/wasm32-unknown-unknown/release/s3d_wasm.wasm <文件.s3d>…
import { readFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { basename, dirname, join } from 'node:path';

const [wasmPath, ...files] = process.argv.slice(2);
if (!wasmPath || !files.length) { console.error('用法: node tools/wasm-hash.mjs <s3d_wasm.wasm> <文件.s3d>…'); process.exit(2); }
const { instance } = await WebAssembly.instantiate(readFileSync(wasmPath), {});
const { memory, s3d_alloc, s3d_free, s3d_compile } = instance.exports;

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

function run(json) {
  const bytes = new TextEncoder().encode(json);
  const inPtr = s3d_alloc(bytes.length);
  new Uint8Array(memory.buffer, inPtr, bytes.length).set(bytes);
  const outPtr = s3d_compile(inPtr, bytes.length);
  const len = new DataView(memory.buffer).getUint32(outPtr, true);
  const out = new TextDecoder().decode(new Uint8Array(memory.buffer, outPtr + 4, len));
  s3d_free(inPtr, bytes.length);
  s3d_free(outPtr, 4 + len);
  return out;
}

for (const f of files) {
  const out = run(bundle(f));
  console.log(`${createHash('sha256').update(out).digest('hex')}  ${basename(f)}`);
}
