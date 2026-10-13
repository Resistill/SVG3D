//! 零依赖的 PNG 编码（8 位 RGB、无滤波；原型 `tools/raster.mjs` 的 encodePNG 的定义）。
//! zlib 流自写：deflate 用固定 Huffman 码（BTYPE = 01）+ 贪心 LZ77（哈希链，窗口 32 KiB），
//! CRC32 / Adler32 自写。输出只取决于输入字节，没有任何随平台变化的状态。

/// CRC32（IEEE 802.3，多项式 0xEDB88320）
pub fn crc32(data: &[u8]) -> u32 {
    crc32_update(0xffff_ffff, data) ^ 0xffff_ffff
}

fn crc_table() -> [u32; 256] {
    let mut t = [0u32; 256];
    for (n, e) in t.iter_mut().enumerate() {
        let mut c = n as u32;
        for _ in 0..8 {
            c = if c & 1 != 0 {
                0xedb8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *e = c;
    }
    t
}

fn crc32_update(mut c: u32, data: &[u8]) -> u32 {
    let t = crc_table();
    for &b in data {
        c = t[((c ^ b as u32) & 255) as usize] ^ (c >> 8);
    }
    c
}

/// Adler-32（zlib 尾部的校验和）
pub fn adler32(data: &[u8]) -> u32 {
    let (mut a, mut b) = (1u32, 0u32);
    // 每 5552 字节取一次模，不会溢出
    for chunk in data.chunks(5552) {
        for &x in chunk {
            a += x as u32;
            b += a;
        }
        a %= 65521;
        b %= 65521;
    }
    (b << 16) | a
}

/// 按位写（deflate 的位序：先低位）
struct Bits {
    out: Vec<u8>,
    acc: u64,
    n: u32,
}

impl Bits {
    fn put(&mut self, v: u32, len: u32) {
        self.acc |= (v as u64) << self.n;
        self.n += len;
        while self.n >= 8 {
            self.out.push(self.acc as u8);
            self.acc >>= 8;
            self.n -= 8;
        }
    }
    /// Huffman 码按高位在前写：先把 len 位反转
    fn put_rev(&mut self, code: u32, len: u32) {
        let mut r = 0u32;
        for i in 0..len {
            r |= ((code >> i) & 1) << (len - 1 - i);
        }
        self.put(r, len);
    }
    fn flush(mut self) -> Vec<u8> {
        if self.n > 0 {
            self.out.push(self.acc as u8);
        }
        self.out
    }
}

/// 固定 Huffman 的字面量 / 长度符号
fn put_litlen(b: &mut Bits, sym: u32) {
    match sym {
        0..=143 => b.put_rev(0x30 + sym, 8),
        144..=255 => b.put_rev(0x190 + sym - 144, 9),
        256..=279 => b.put_rev(sym - 256, 7),
        _ => b.put_rev(0xc0 + sym - 280, 8),
    }
}

const LEN_BASE: [u32; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115, 131, 163, 195,
    227, 258,
];
const LEN_EXTRA: [u32; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
];
const DIST_BASE: [u32; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537, 2049, 3073,
    4097, 6145, 8193, 12289, 16385, 24577,
];
const DIST_EXTRA: [u32; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12, 13, 13,
];

fn put_match(b: &mut Bits, len: u32, dist: u32) {
    let li = LEN_BASE.iter().rposition(|&x| x <= len).unwrap();
    put_litlen(b, 257 + li as u32);
    b.put(len - LEN_BASE[li], LEN_EXTRA[li]);
    let di = DIST_BASE.iter().rposition(|&x| x <= dist).unwrap();
    b.put_rev(di as u32, 5);
    b.put(dist - DIST_BASE[di], DIST_EXTRA[di]);
}

const WINDOW: usize = 32768;
const HASH_BITS: u32 = 15;
const MAX_CHAIN: usize = 48;
const MIN_MATCH: usize = 3;
const MAX_MATCH: usize = 258;

fn hash3(d: &[u8], i: usize) -> usize {
    let v = (d[i] as u32) | ((d[i + 1] as u32) << 8) | ((d[i + 2] as u32) << 16);
    (v.wrapping_mul(0x9e37_79b1) >> (32 - HASH_BITS)) as usize
}

/// deflate（RFC 1951）：单个固定 Huffman 块
pub fn deflate(data: &[u8]) -> Vec<u8> {
    let mut b = Bits {
        out: Vec::with_capacity(data.len() / 4 + 16),
        acc: 0,
        n: 0,
    };
    b.put(1, 1); // BFINAL
    b.put(1, 2); // BTYPE = 01 固定 Huffman
    let n = data.len();
    // head：哈希 → 最近位置 + 1（0 表示空）；prev：位置 → 同哈希的上一个位置 + 1
    let mut head = vec![0usize; 1 << HASH_BITS];
    let mut prev = vec![0usize; WINDOW];
    let insert = |head: &mut Vec<usize>, prev: &mut Vec<usize>, i: usize| {
        if i + MIN_MATCH <= n {
            let h = hash3(data, i);
            prev[i % WINDOW] = head[h];
            head[h] = i + 1;
        }
    };
    let mut i = 0;
    while i < n {
        let mut best_len = 0;
        let mut best_dist = 0;
        if i + MIN_MATCH <= n {
            let max_len = MAX_MATCH.min(n - i);
            let mut cand = head[hash3(data, i)];
            let mut chain = 0;
            while cand > 0 && chain < MAX_CHAIN {
                let j = cand - 1;
                if i - j > WINDOW - 1 {
                    break;
                }
                let mut l = 0;
                while l < max_len && data[j + l] == data[i + l] {
                    l += 1;
                }
                if l > best_len {
                    best_len = l;
                    best_dist = i - j;
                    if l == max_len {
                        break;
                    }
                }
                let p = prev[j % WINDOW];
                // 链只往更早的位置走（被覆盖的槽位会指向更晚的位置，到此为止）
                if p == 0 || p > j {
                    break;
                }
                cand = p;
                chain += 1;
            }
        }
        if best_len >= MIN_MATCH {
            put_match(&mut b, best_len as u32, best_dist as u32);
            for k in i..i + best_len {
                insert(&mut head, &mut prev, k);
            }
            i += best_len;
        } else {
            put_litlen(&mut b, data[i] as u32);
            insert(&mut head, &mut prev, i);
            i += 1;
        }
    }
    put_litlen(&mut b, 256);
    b.flush()
}

/// zlib（RFC 1950）：头 78 01 + deflate + Adler-32（大端）
pub fn zlib(data: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    out.extend(deflate(data));
    out.extend(adler32(data).to_be_bytes());
    out
}

fn chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend((data.len() as u32).to_be_bytes());
    let start = out.len();
    out.extend(kind);
    out.extend(data);
    let c = crc32(&out[start..]);
    out.extend(c.to_be_bytes());
}

/// PNG：8 位 RGB（颜色类型 2），每行滤波 0；`rgb` 长度 = w × h × 3
pub fn encode(rgb: &[u8], w: usize, h: usize) -> Vec<u8> {
    assert_eq!(rgb.len(), w * h * 3, "rgb 长度应为 w × h × 3");
    let mut raw = Vec::with_capacity((w * 3 + 1) * h);
    for y in 0..h {
        raw.push(0);
        raw.extend(&rgb[y * w * 3..(y + 1) * w * 3]);
    }
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend((w as u32).to_be_bytes());
    ihdr.extend((h as u32).to_be_bytes());
    ihdr.extend([8, 2, 0, 0, 0]);
    let mut out = vec![137, 80, 78, 71, 13, 10, 26, 10];
    chunk(&mut out, b"IHDR", &ihdr);
    chunk(&mut out, b"IDAT", &zlib(&raw));
    chunk(&mut out, b"IEND", &[]);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksums() {
        assert_eq!(crc32(b"123456789"), 0xcbf4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11e6_0398);
    }

    /// 用 python3 的 zlib 解压，与原文逐字节相同（环境里没有 python3 时跳过）
    fn python_inflate(z: &[u8], tag: &str) -> Option<Vec<u8>> {
        let dir = std::env::temp_dir().join(format!("s3d-png-test-{}-{}", std::process::id(), tag));
        std::fs::create_dir_all(&dir).ok()?;
        let src = dir.join("in.z");
        let dst = dir.join("out.bin");
        std::fs::write(&src, z).ok()?;
        let st = std::process::Command::new("python3")
            .arg("-c")
            .arg("import sys,zlib;open(sys.argv[2],'wb').write(zlib.decompress(open(sys.argv[1],'rb').read()))")
            .arg(&src)
            .arg(&dst)
            .status()
            .ok()?;
        assert!(st.success(), "python3 zlib 解压失败");
        let r = std::fs::read(&dst).ok();
        let _ = std::fs::remove_dir_all(&dir);
        r
    }

    #[test]
    fn zlib_roundtrip_python() {
        // 混合：重复段、伪随机段、长零段、短输入
        let mut data = Vec::new();
        let mut s = 12345u32;
        for i in 0..70000u32 {
            s = s.wrapping_mul(1_103_515_245).wrapping_add(12345);
            data.push(if i % 1000 < 300 {
                (s >> 16) as u8
            } else {
                (i % 7) as u8
            });
        }
        data.extend(vec![0u8; 40000]);
        for (k, d) in [&data[..], b"", b"a", b"abcabcabcabcabc"].iter().enumerate() {
            if let Some(back) = python_inflate(&zlib(d), &format!("z{}", k)) {
                assert_eq!(&back[..], *d);
            }
        }
    }

    #[test]
    fn png_python_decodes() {
        let (w, h) = (37, 23);
        let mut rgb = vec![0u8; w * h * 3];
        for (i, v) in rgb.iter_mut().enumerate() {
            *v = ((i * 7) % 256) as u8;
        }
        let png = encode(&rgb, w, h);
        assert_eq!(&png[..8], &[137, 80, 78, 71, 13, 10, 26, 10]);
        // 取出 IDAT，交给 python 解压，比对去掉滤波字节后的像素
        let len = u32::from_be_bytes(png[33..37].try_into().unwrap()) as usize;
        assert_eq!(&png[37..41], b"IDAT");
        let z = &png[41..41 + len];
        if let Some(raw) = python_inflate(z, "png") {
            let mut back: Vec<u8> = Vec::new();
            for y in 0..h {
                assert_eq!(raw[y * (w * 3 + 1)], 0);
                back.extend(&raw[y * (w * 3 + 1) + 1..(y + 1) * (w * 3 + 1)]);
            }
            assert_eq!(back, rgb);
        }
    }
}
