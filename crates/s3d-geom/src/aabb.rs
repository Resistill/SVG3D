//! 轴对齐包围盒（spec/02 2.1.2）。零盒 `Z = [0,0]^3` 用 `Aabb::ZERO`。

use crate::math::{Mat3, Vec3};
use crate::transform::Transform;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    /// 零盒：空容器与退化节点用（2.1.2）。
    pub const ZERO: Aabb = Aabb {
        min: Vec3::ZERO,
        max: Vec3::ZERO,
    };

    pub fn new(min: Vec3, max: Vec3) -> Aabb {
        Aabb { min, max }
    }

    /// 有限点集的包围盒；空集返回 None。各轴取最小 / 最大，与点的顺序无关。
    pub fn of_points<I: IntoIterator<Item = Vec3>>(pts: I) -> Option<Aabb> {
        let mut it = pts.into_iter();
        let first = it.next()?;
        let mut b = Aabb {
            min: first,
            max: first,
        };
        for p in it {
            b.min = b.min.min(p);
            b.max = b.max.max(p);
        }
        Some(b)
    }

    /// 盒心 c。
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }
    /// 半尺寸 h_B。
    pub fn half(&self) -> Vec3 {
        (self.max - self.min) * 0.5
    }
    /// 底面中心 b = (c_x, y_0, c_z)。
    pub fn bottom_center(&self) -> Vec3 {
        let c = self.center();
        Vec3::new(c.x, self.min.y, c.z)
    }
    /// 边长 (w, h, d)。
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }
    /// 体积 w·h·d。
    pub fn volume(&self) -> f64 {
        let s = self.size();
        s.x * s.y * s.z
    }
    /// 方向词锚点：盒心 + 半尺寸 ∘ a，a 的分量取 -1 / 0 / 1（spec/03 3.2.1）。
    pub fn anchor(&self, a: Vec3) -> Vec3 {
        self.center() + self.half() * a
    }
    /// 八个角点，顺序固定（x 外层、y 中层、z 内层，各先 min 后 max）。
    pub fn corners(&self) -> [Vec3; 8] {
        let (a, b) = (self.min, self.max);
        [
            Vec3::new(a.x, a.y, a.z),
            Vec3::new(a.x, a.y, b.z),
            Vec3::new(a.x, b.y, a.z),
            Vec3::new(a.x, b.y, b.z),
            Vec3::new(b.x, a.y, a.z),
            Vec3::new(b.x, a.y, b.z),
            Vec3::new(b.x, b.y, a.z),
            Vec3::new(b.x, b.y, b.z),
        ]
    }
    /// 两盒之并。
    pub fn union(&self, o: &Aabb) -> Aabb {
        Aabb {
            min: self.min.min(o.min),
            max: self.max.max(o.max),
        }
    }
    /// 两盒之交；不相交时返回 None（恰好相碰算相交，得到零厚度的盒）。
    pub fn intersection(&self, o: &Aabb) -> Option<Aabb> {
        let min = self.min.max(o.min);
        let max = self.max.min(o.max);
        if min.x <= max.x && min.y <= max.y && min.z <= max.z {
            Some(Aabb { min, max })
        } else {
            None
        }
    }
    /// 盒经变换后的轴对齐盒：8 个角经变换后各轴取最小 / 最大（spec/03 3.1.6 的 `box`）。
    pub fn transformed(&self, t: &Transform) -> Aabb {
        Aabb::of_points(self.corners().iter().map(|p| t.apply(*p))).unwrap()
    }
    /// 盒经旋转矩阵后的轴对齐盒（平移为零）。
    pub fn rotated(&self, r: &Mat3) -> Aabb {
        Aabb::of_points(self.corners().iter().map(|p| *r * *p)).unwrap()
    }
    /// 对角线长度（ε_hit 用，2.1.4）。
    pub fn diagonal(&self) -> f64 {
        crate::math::length(self.size())
    }
    /// 沿轴 k 的重叠量 min(max) − max(min)；负值是间隙（spec/06 6.2.3）。
    pub fn overlap(&self, o: &Aabb, k: usize) -> f64 {
        self.max[k].min(o.max[k]) - self.min[k].max(o.min[k])
    }
}
