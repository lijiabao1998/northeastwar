//! Triangle soup: the shape of something, ready for any renderer.
//!
//! World space is right-handed with y up. The map's east is +x and the map's
//! north is MINUS z. That minus is the difference between Heihe and its
//! mirror image, so it lives in exactly one function: `world`.

use crate::geom::P2;
use crate::look::Rgb;

#[derive(Clone, Debug, Default)]
pub struct Soup {
    pub pos: Vec<[f32; 3]>,
    pub nor: Vec<[f32; 3]>,
    /// Linear RGBA.
    pub col: Vec<[f32; 4]>,
    pub uv: Vec<[f32; 2]>,
    pub idx: Vec<u32>,
}

/// Map metres (x east, y north) and a height, as a point in the world.
pub fn world(p: P2, up: f32) -> [f32; 3] {
    [p.x, up, -p.y]
}

/// A flat map direction as a world direction.
pub fn world_dir(d: P2) -> [f32; 3] {
    [d.x, 0.0, -d.y]
}

fn to_linear(c: f32) -> f32 {
    if c <= 0.04045 {
        c / 12.92
    } else {
        ((c + 0.055) / 1.055).powf(2.4)
    }
}

pub fn linear(c: Rgb) -> [f32; 4] {
    [to_linear(c.r), to_linear(c.g), to_linear(c.b), 1.0]
}

impl Soup {
    pub fn triangles(&self) -> usize {
        self.idx.len() / 3
    }

    /// Four corners, counter-clockwise seen from the side the normal points to.
    pub fn quad(&mut self, corners: [[f32; 3]; 4], normal: [f32; 3], colour: Rgb) {
        self.quad_uv(corners, normal, colour, [[0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0]]);
    }

    pub fn quad_uv(&mut self, corners: [[f32; 3]; 4], normal: [f32; 3], colour: Rgb, uv: [[f32; 2]; 4]) {
        let base = self.pos.len() as u32;
        let c = linear(colour);
        for i in 0..4 {
            self.pos.push(corners[i]);
            self.nor.push(normal);
            self.col.push(c);
            self.uv.push(uv[i]);
        }
        self.idx.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn tri(&mut self, corners: [[f32; 3]; 3], normal: [f32; 3], colour: Rgb) {
        let base = self.pos.len() as u32;
        let c = linear(colour);
        for corner in corners {
            self.pos.push(corner);
            self.nor.push(normal);
            self.col.push(c);
            self.uv.push([0.0, 0.0]);
        }
        self.idx.extend_from_slice(&[base, base + 1, base + 2]);
    }

    /// A flat polygon at one height, facing up. `ring` is counter-clockwise on the map.
    pub fn flat(&mut self, ring: &[P2], up: f32, colour: Rgb) {
        let base = self.pos.len() as u32;
        let c = linear(colour);
        for p in ring {
            self.pos.push(world(*p, up));
            self.nor.push([0.0, 1.0, 0.0]);
            self.col.push(c);
            self.uv.push([p.x * 0.1, p.y * 0.1]);
        }
        for t in crate::geom::triangulate(ring) {
            self.idx.extend_from_slice(&[base + t[0], base + t[1], base + t[2]]);
        }
    }

    /// A strip of ground along a line: a road, a pavement, a painted stripe.
    pub fn ribbon(&mut self, line: &[P2], width: f32, up: f32, colour: Rgb) {
        if line.len() < 2 {
            return;
        }
        let half = width * 0.5;
        let c = linear(colour);
        let base = self.pos.len() as u32;
        for i in 0..line.len() {
            let before = if i > 0 { line[i].sub(line[i - 1]).norm() } else { line[1].sub(line[0]).norm() };
            let after = if i + 1 < line.len() { line[i + 1].sub(line[i]).norm() } else { before };
            let along = before.add(after).norm();
            let along = if along.len() < 0.5 { before } else { along };
            let side = along.right();
            // Mitre, capped so a sharp bend does not shoot a spike across the map.
            let scale = (1.0 / along.dot(before).max(0.5)).min(2.0);
            for sign in [-1.0f32, 1.0] {
                let p = line[i].add(side.mul(half * scale * sign));
                self.pos.push(world(p, up));
                self.nor.push([0.0, 1.0, 0.0]);
                self.col.push(c);
                self.uv.push([p.x * 0.1, p.y * 0.1]);
            }
        }
        for i in 0..(line.len() as u32 - 1) {
            let a = base + i * 2;
            // left, right, next-left, next-right, wound to face up
            self.idx.extend_from_slice(&[a, a + 1, a + 2, a + 1, a + 3, a + 2]);
        }
    }

    /// An upright box standing on the map at `at`, turned by `yaw` radians
    /// (counter-clockwise on the map, 0 = long side along east).
    pub fn boxed(&mut self, at: P2, base_up: f32, size: [f32; 3], yaw: f32, colour: Rgb) {
        let (sx, sy, sz) = (size[0] * 0.5, size[1], size[2] * 0.5);
        let ax = P2::new(yaw.cos(), yaw.sin());
        let ay = P2::new(-yaw.sin(), yaw.cos());
        let corner = |u: f32, v: f32| at.add(ax.mul(u * sx)).add(ay.mul(v * sz));
        let ring = [corner(-1.0, -1.0), corner(1.0, -1.0), corner(1.0, 1.0), corner(-1.0, 1.0)];
        for i in 0..4 {
            let a = ring[i];
            let b = ring[(i + 1) % 4];
            let n = b.sub(a).norm().right();
            self.quad(
                [world(a, base_up), world(b, base_up), world(b, base_up + sy), world(a, base_up + sy)],
                world_dir(n),
                colour,
            );
        }
        self.flat(&ring, base_up + sy, colour);
    }

    /// A cone or a pyramid on a ring: a spire, a dome made of facets.
    pub fn cone(&mut self, centre: P2, base_up: f32, radius: f32, height: f32, sides: usize, colour: Rgb) {
        let tip = world(centre, base_up + height);
        for i in 0..sides {
            let a0 = i as f32 / sides as f32 * std::f32::consts::TAU;
            let a1 = (i + 1) as f32 / sides as f32 * std::f32::consts::TAU;
            let p0 = centre.add(P2::new(a0.cos(), a0.sin()).mul(radius));
            let p1 = centre.add(P2::new(a1.cos(), a1.sin()).mul(radius));
            let mid = P2::new(((a0 + a1) * 0.5).cos(), ((a0 + a1) * 0.5).sin());
            let slope = radius / height.max(0.01);
            let n = [mid.x, slope, -mid.y];
            let l = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            self.tri([world(p0, base_up), world(p1, base_up), tip], [n[0] / l, n[1] / l, n[2] / l], colour);
        }
    }

    pub fn append(&mut self, other: &Soup) {
        let base = self.pos.len() as u32;
        self.pos.extend_from_slice(&other.pos);
        self.nor.extend_from_slice(&other.nor);
        self.col.extend_from_slice(&other.col);
        self.uv.extend_from_slice(&other.uv);
        self.idx.extend(other.idx.iter().map(|i| i + base));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn north_is_minus_z_and_the_world_is_not_mirrored() {
        // East, north, up on the map must stay a right-handed set in the world.
        let e = world(P2::new(1.0, 0.0), 0.0);
        let n = world(P2::new(0.0, 1.0), 0.0);
        let up = [0.0, 1.0, 0.0];
        let cross = [e[1] * n[2] - e[2] * n[1], e[2] * n[0] - e[0] * n[2], e[0] * n[1] - e[1] * n[0]];
        assert_eq!(n, [0.0, 0.0, -1.0]);
        assert!((cross[0] - up[0]).abs() < 1.0e-6 && (cross[1] - up[1]).abs() < 1.0e-6 && (cross[2] - up[2]).abs() < 1.0e-6);
    }

    #[test]
    fn a_box_faces_outward() {
        let mut s = Soup::default();
        s.boxed(P2::new(0.0, 0.0), 0.0, [2.0, 3.0, 2.0], 0.0, Rgb { r: 1.0, g: 1.0, b: 1.0 });
        // Every triangle's winding must agree with its stored normal.
        for t in s.idx.chunks(3) {
            let (a, b, c) = (s.pos[t[0] as usize], s.pos[t[1] as usize], s.pos[t[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let n = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let stored = s.nor[t[0] as usize];
            assert!(n[0] * stored[0] + n[1] * stored[1] + n[2] * stored[2] > 0.0);
        }
        assert_eq!(s.triangles(), 10);
    }
}
