//! 表面集合（原型 `tools/surfaces.mjs`）：若干物体按材质颜色分组，每组与全部各建一棵 BVH；
//! 查询给出距离与最近三角形所属的物体序号。

use s3d_geom::bvh::Bvh;
use s3d_geom::math::Vec3;
use s3d_geom::tri::Tri;
use std::collections::BTreeMap;

pub fn color_key(c: Option<&str>) -> String {
    c.unwrap_or("").to_lowercase()
}

/// 一棵树：三角形 + 每个三角形属于哪个物体
pub struct Group {
    pub bvh: Bvh,
    owner: Vec<u32>,
}

impl Group {
    fn new(objs: &[(&[Tri], u32)]) -> Option<Group> {
        let n: usize = objs.iter().map(|o| o.0.len()).sum();
        if n == 0 {
            return None;
        }
        let mut tris = Vec::with_capacity(n);
        let mut owner = Vec::with_capacity(n);
        for (t, id) in objs {
            tris.extend_from_slice(t);
            owner.extend(std::iter::repeat_n(*id, t.len()));
        }
        Some(Group {
            bvh: Bvh::new(&tris),
            owner,
        })
    }
    /// 封顶 max 之内的最近距离与物体序号
    pub fn closest(&self, p: Vec3, max: f64) -> Option<(f64, u32)> {
        self.bvh.closest(p, max).map(|(i, d, _)| (d, self.owner[i]))
    }
    pub fn crossings(&self, o: Vec3, d: Vec3) -> i32 {
        self.bvh.crossings(o, d)
    }
}

/// { byColor: 颜色 → 树, all: 树 }
pub struct Surfaces {
    pub by_color: BTreeMap<String, Group>,
    pub all: Option<Group>,
}

impl Surfaces {
    /// objs：(三角形, 颜色键, 物体序号)
    pub fn new(objs: &[(&[Tri], &str, u32)]) -> Surfaces {
        let mut groups: BTreeMap<String, Vec<(&[Tri], u32)>> = BTreeMap::new();
        for (t, c, id) in objs {
            groups.entry(c.to_string()).or_default().push((*t, *id));
        }
        let mut by_color = BTreeMap::new();
        for (c, list) in groups {
            if let Some(g) = Group::new(&list) {
                by_color.insert(c, g);
            }
        }
        let all_list: Vec<(&[Tri], u32)> = objs.iter().map(|(t, _, id)| (*t, *id)).collect();
        Surfaces {
            by_color,
            all: Group::new(&all_list),
        }
    }

    /// 同材质优先：有同色的树用它（不多计）；否则用全部表面并多计 penalty（delta）。颜色为空时不多计
    pub fn pick(&self, color: Option<&str>, delta: f64) -> (Option<&Group>, f64) {
        match color {
            Some(c) => match self.by_color.get(&color_key(Some(c))) {
                Some(g) => (Some(g), 0.0),
                None => (self.all.as_ref(), delta),
            },
            None => (self.all.as_ref(), 0.0),
        }
    }

    /// 到一组表面的距离 → (d, 物体序号或 None)：没有目标、或 dMax 以内没有表面时 (dMax, None)
    pub fn nearest(&self, color: Option<&str>, p: Vec3, delta: f64, d_max: f64) -> (f64, Option<u32>) {
        let (g, pen) = self.pick(color, delta);
        match g.and_then(|g| g.closest(p, d_max)) {
            Some((d, o)) => ((d + pen).min(d_max), Some(o)),
            None => (d_max, None),
        }
    }
}
