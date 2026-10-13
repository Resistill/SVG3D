//! 角色（原型 `tools/roles.mjs`）：每个源部件是结构、装饰、地形、贴花、碰撞还是辅助。
//! 判定顺序：工程规则（先匹配先生效）→ 名字里的词 → 几何。整个场景一起判时再找白模体积（辅助）。

use regex::{Regex, RegexBuilder};
use s3d_geom::math::Vec3;
use s3d_geom::mesh::Part;
use std::collections::BTreeMap;

pub const ROLES: [&str; 6] = ["structure", "decor", "terrain", "decal", "collision", "helper"];

const DECAL: &[&str] = &["decal", "decals", "sticker"];
const COLLISION: &[&str] = &[
    "blocker",
    "blockers",
    "collision",
    "collider",
    "colliders",
    "playerclip",
    "clip",
    "invisiblewall",
    "airwall",
    "blockvolume",
];
const HELPER: &[&str] = &[
    "fog", "blockout", "trigger", "occluder", "navmesh", "killzone", "spawn",
];
const TERRAIN: &[&str] = &[
    "terrain",
    "landscape",
    "ground",
    "road",
    "street",
    "pavement",
    "sidewalk",
    "asphalt",
    "sand",
    "soil",
    "dirt",
];
const STRUCTURE: &[&str] = &[
    "building",
    "wall",
    "exterwall",
    "interwall",
    "stair",
    "stairs",
    "staircase",
    "step",
    "steps",
    "terrace",
    "pillar",
    "column",
    "beam",
    "roof",
    "floor",
    "ceiling",
    "slab",
    "fence",
    "railing",
    "guardrail",
    "handrail",
    "rail",
    "walkway",
    "bridge",
    "support",
    "hsupport",
    "frame",
    "door",
    "interdoor",
    "gate",
    "window",
    "platform",
    "ramp",
    "tower",
    "facade",
    "corridor",
    "passage",
    "arch",
    "canopy",
    "shed",
    "hangar",
    "balcony",
    "parapet",
    "escalator",
    "lift",
    "elevator",
];
const DECOR: &[&str] = &[
    "furniture",
    "lighting",
    "light",
    "lamp",
    "walllamp",
    "chandelier",
    "spotlight",
    "chair",
    "table",
    "sofa",
    "bed",
    "cabinet",
    "cupboard",
    "shelf",
    "desk",
    "computer",
    "monitor",
    "monitors",
    "screen",
    "keyboard",
    "machine",
    "machinery",
    "server",
    "container",
    "crate",
    "box",
    "boxlid",
    "basket",
    "bin",
    "dustbin",
    "trash",
    "bag",
    "barrel",
    "tool",
    "tools",
    "vehicle",
    "car",
    "van",
    "vans",
    "truck",
    "bike",
    "plant",
    "tree",
    "flower",
    "flowerpot",
    "flowerbed",
    "pot",
    "bush",
    "shrub",
    "grass",
    "ivy",
    "vine",
    "leaf",
    "moss",
    "rock",
    "stone",
    "sign",
    "signage",
    "branding",
    "poster",
    "board",
    "doorplate",
    "trademark",
    "logo",
    "trophy",
    "cup",
    "cups",
    "bottle",
    "book",
    "folder",
    "paper",
    "newspaper",
    "kitchen",
    "microwave",
    "coffee",
    "dispenser",
    "extinguisher",
    "firebox",
    "prop",
    "props",
    "emissive",
    "ornament",
    "decoration",
    "statue",
    "cart",
    "carts",
    "bench",
    "umbrella",
    "awning",
    "antenna",
    "conditioner",
    "conditioners",
    "airconditioning",
    "pipe",
    "pipeline",
    "cable",
    "safe",
    "curtain",
    "blind",
    "rug",
    "carpet",
    "cushion",
    "pillow",
    "lid",
    "parts",
    "clock",
    "fan",
    "speaker",
    "camera",
    "mailbox",
    "leuchtstabe",
    "exit",
    "packingbox",
    "supplybox",
    "plasticbox",
    "bucket",
    "mop",
    "sunshade",
    "stand",
    "blinds",
    "instrument",
    "cloth",
    "footcloth",
    "suit",
    "spacesuit",
    "dryer",
    "litter",
    "cover",
    "equipment",
    "engine",
    "centrifuge",
    "charger",
    "armor",
];

fn role_of_word(w: &str) -> Option<&'static str> {
    // 与原型 ROLE_OF 相同：后写的覆盖先写的（structure → decor → terrain）
    let mut r = None;
    if STRUCTURE.contains(&w) {
        r = Some("structure");
    }
    if DECOR.contains(&w) {
        r = Some("decor");
    }
    if TERRAIN.contains(&w) {
        r = Some("terrain");
    }
    r
}

/// 节点名的最后一段 → 词（小写）：下划线、数字、空格、点分开，驼峰拆开
pub fn name_words(name: &str) -> Vec<String> {
    let leaf = name.rsplit('/').next().unwrap_or("");
    let leaf = match leaf.rfind('@') {
        Some(i) if leaf[i + 1..].chars().all(|c| c.is_ascii_digit()) && i + 1 < leaf.len() => &leaf[..i],
        _ => leaf,
    };
    // ([a-z])([A-Z]) → "$1 $2"
    let cs: Vec<char> = leaf.chars().collect();
    let mut s1 = String::new();
    for i in 0..cs.len() {
        s1.push(cs[i]);
        if i + 1 < cs.len() && cs[i].is_ascii_lowercase() && cs[i + 1].is_ascii_uppercase() {
            s1.push(' ');
        }
    }
    // ([A-Z]+)([A-Z][a-z]) → "$1 $2"
    let cs: Vec<char> = s1.chars().collect();
    let mut s2 = String::new();
    for i in 0..cs.len() {
        if i > 0
            && cs[i].is_ascii_uppercase()
            && cs[i - 1].is_ascii_uppercase()
            && i + 1 < cs.len()
            && cs[i + 1].is_ascii_lowercase()
        {
            s2.push(' ');
        }
        s2.push(cs[i]);
    }
    s2.split(|c: char| !c.is_ascii_alphabetic())
        .filter(|w| !w.is_empty())
        .map(|w| w.to_ascii_lowercase())
        .collect()
}

fn block_re() -> Regex {
    RegexBuilder::new(r"(?:^|[^A-Za-z])block[_ ]?(?:wall|plane|volume)")
        .case_insensitive(true)
        .build()
        .unwrap()
}

fn is_collision(name: &str, words: &[String]) -> bool {
    if words.iter().any(|w| COLLISION.contains(&w.as_str())) {
        return true;
    }
    let leaf = name.rsplit('/').next().unwrap_or("");
    if block_re().is_match(leaf) {
        return true;
    }
    let segs: Vec<&str> = name.split('/').collect();
    segs[..segs.len().saturating_sub(1)]
        .iter()
        .any(|x| COLLISION.contains(&x.to_ascii_lowercase().as_str()))
}

/// 工程的角色规则（roles.json 的一条）
#[derive(Clone, Debug, Default)]
pub struct RoleRule {
    pub role: String,
    pub name: Option<String>,
    pub material: Option<String>,
    pub meta: BTreeMap<String, String>,
}

struct CompiledRule {
    role: String,
    name: Option<Regex>,
    material: Option<Regex>,
    meta: Vec<(String, Regex)>,
}

fn re(s: &str) -> Regex {
    // 写错的规则按"永不匹配"处理
    RegexBuilder::new(s)
        .case_insensitive(true)
        .build()
        .unwrap_or_else(|_| Regex::new(r"[^\s\S]").unwrap())
}

fn compile_rules(extra: &[RoleRule]) -> Vec<CompiledRule> {
    extra
        .iter()
        .map(|r| CompiledRule {
            role: r.role.clone(),
            name: r.name.as_deref().map(re),
            material: r.material.as_deref().map(re),
            meta: r.meta.iter().map(|(k, v)| (k.clone(), re(v))).collect(),
        })
        .collect()
}

fn classify(part: &Part, rules: &[CompiledRule]) -> String {
    let name = part.name.clone().unwrap_or_default();
    let mat = part.material.name.clone().unwrap_or_default();
    for r in rules {
        if r.name.is_none() && r.material.is_none() && r.meta.is_empty() {
            continue;
        }
        if let Some(nr) = &r.name {
            if name.is_empty() || !nr.is_match(&name) {
                continue;
            }
        }
        if let Some(mr) = &r.material {
            if !mr.is_match(&mat) {
                continue;
            }
        }
        if !r
            .meta
            .iter()
            .all(|(k, x)| part.meta.get(k).map(|v| x.is_match(v)).unwrap_or(false))
        {
            continue;
        }
        return r.role.clone();
    }
    let words = name_words(&name);
    if is_collision(&name, &words) {
        return "collision".to_string();
    }
    if words.iter().any(|w| DECAL.contains(&w.as_str())) {
        return "decal".to_string();
    }
    if words.iter().any(|w| HELPER.contains(&w.as_str())) {
        return "helper".to_string();
    }
    for w in words.iter().rev() {
        if let Some(r) = role_of_word(w) {
            return r.to_string();
        }
    }
    geometric_role(part).to_string()
}

fn bbox(part: &Part) -> (Vec3, Vec3) {
    let mut lo = Vec3::splat(f64::INFINITY);
    let mut hi = Vec3::splat(f64::NEG_INFINITY);
    for t in &part.tris {
        for v in t {
            lo = lo.min(*v);
            hi = hi.max(*v);
        }
    }
    (lo, hi)
}

/// 整个场景一起判：先逐个按规则，再找白模体积（默认名、三角形少、整个包住至少 10 个别的部件）
pub fn assign_roles(parts: &[Part], extra: &[RoleRule]) -> Vec<String> {
    let rules = compile_rules(extra);
    let mut roles: Vec<String> = parts.iter().map(|p| classify(p, &rules)).collect();
    let boxes: Vec<(Vec3, Vec3)> = parts.iter().map(bbox).collect();
    let generic = RegexBuilder::new(
        r"^(cube|box|cylinder|sphere|plane|object|mesh|node|default|geometry)(_?\d+)*(@\d+)?$",
    )
    .case_insensitive(true)
    .build()
    .unwrap();
    for i in 0..parts.len() {
        let leaf = parts[i]
            .name
            .as_deref()
            .unwrap_or("")
            .rsplit('/')
            .next()
            .unwrap_or("");
        if roles[i] == "helper" || leaf.is_empty() || !generic.is_match(leaf) || parts[i].tris.len() > 200 {
            continue;
        }
        let b = boxes[i];
        let mut n = 0;
        for (j, bj) in boxes.iter().enumerate() {
            if n >= 10 {
                break;
            }
            if j != i && (0..3).all(|k| bj.0[k] >= b.0[k] && bj.1[k] <= b.1[k]) {
                n += 1;
            }
        }
        if n >= 10 {
            roles[i] = "helper".to_string();
        }
    }
    roles
}

/// 没有名字、或名字里没有线索：按几何。很薄、散成很多块的是贴花；其余当结构
pub fn geometric_role(part: &Part) -> &'static str {
    let (lo, hi) = bbox(part);
    let sz = hi - lo;
    let mn = sz.x.min(sz.y).min(sz.z);
    let mx = sz.x.max(sz.y).max(sz.z);
    if scatter_of(part, 3, 0.1).0 && mn < 0.02 * mx {
        return "decal";
    }
    "structure"
}

fn weld_key(p: Vec3) -> [i64; 3] {
    let r = |x: f64| (x * 1e4 + 0.5).floor() as i64;
    [r(p.x), r(p.y), r(p.z)]
}

/// 分散：互不相连的块的包围盒体积之和远小于整体包围盒。返回 (scattered, 分量数, ratio)
pub fn scatter_of(part: &Part, min_comps: usize, max_ratio: f64) -> (bool, usize, f64) {
    let tris = &part.tris;
    let n = tris.len();
    if n < 2 * min_comps {
        return (false, 0, 1.0);
    }
    let mut parent: Vec<usize> = (0..n).collect();
    fn find(parent: &mut [usize], mut i: usize) -> usize {
        while parent[i] != i {
            parent[i] = parent[parent[i]];
            i = parent[i];
        }
        i
    }
    let mut owner: BTreeMap<[i64; 3], usize> = BTreeMap::new();
    for (i, t) in tris.iter().enumerate() {
        for v in t {
            let k = weld_key(*v);
            match owner.get(&k) {
                None => {
                    owner.insert(k, i);
                }
                Some(&o) => {
                    let a = find(&mut parent, i);
                    let b = find(&mut parent, o);
                    if a != b {
                        parent[a] = b;
                    }
                }
            }
        }
    }
    let mut groups: Vec<Vec<usize>> = Vec::new();
    let mut idx: BTreeMap<usize, usize> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        let g = *idx.entry(r).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[g].push(i);
    }
    if groups.len() < min_comps {
        return (false, groups.len(), 1.0);
    }
    let vol = |ids: &[usize]| {
        let mut lo = Vec3::splat(f64::INFINITY);
        let mut hi = Vec3::splat(f64::NEG_INFINITY);
        for &i in ids {
            for v in &tris[i] {
                lo = lo.min(*v);
                hi = hi.max(*v);
            }
        }
        let s = (hi - lo).max(Vec3::splat(1e-6));
        s.x * s.y * s.z
    };
    let all: Vec<usize> = (0..n).collect();
    let ratio = groups.iter().map(|g| vol(g)).sum::<f64>() / vol(&all);
    (ratio < max_ratio, groups.len(), ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn words() {
        assert_eq!(name_words("WallLamp"), vec!["wall", "lamp"]);
        assert_eq!(name_words("HLight"), vec!["h", "light"]);
        assert_eq!(name_words("Road_FlowerBed@3"), vec!["road", "flower", "bed"]);
        assert_eq!(
            name_words("Dormitory_02_ExterWall_01"),
            vec!["dormitory", "exter", "wall"]
        );
    }

    #[test]
    fn classify_names() {
        let mk = |name: &str| Part {
            name: Some(name.to_string()),
            uid: None,
            from: None,
            tris: vec![],
            material: s3d_geom::mesh::Material {
                name: None,
                color: "#000000".into(),
                metal: None,
                rough: None,
            },
            mesh: None,
            matrix: None,
            role: None,
            meta: BTreeMap::new(),
        };
        assert_eq!(classify(&mk("WallLamp"), &[]), "decor");
        assert_eq!(classify(&mk("ConcreteWall"), &[]), "structure");
        assert_eq!(classify(&mk("SM_EFX_BlockPlane"), &[]), "collision");
        assert_eq!(classify(&mk("Blocker/Thing"), &[]), "collision");
        assert_eq!(classify(&mk("Road_01"), &[]), "terrain");
        assert_eq!(classify(&mk("Fog_Volume"), &[]), "helper");
    }
}
