//! Flat geometry. Everything here is in map metres: x east, y north.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct P2 {
    pub x: f32,
    pub y: f32,
}

impl P2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
    pub fn sub(self, o: P2) -> P2 {
        P2::new(self.x - o.x, self.y - o.y)
    }
    pub fn add(self, o: P2) -> P2 {
        P2::new(self.x + o.x, self.y + o.y)
    }
    pub fn mul(self, k: f32) -> P2 {
        P2::new(self.x * k, self.y * k)
    }
    pub fn dot(self, o: P2) -> f32 {
        self.x * o.x + self.y * o.y
    }
    pub fn cross(self, o: P2) -> f32 {
        self.x * o.y - self.y * o.x
    }
    pub fn len(self) -> f32 {
        self.dot(self).sqrt()
    }
    pub fn dist(self, o: P2) -> f32 {
        self.sub(o).len()
    }
    pub fn norm(self) -> P2 {
        let l = self.len();
        if l < 1.0e-6 {
            P2::new(0.0, 0.0)
        } else {
            self.mul(1.0 / l)
        }
    }
    /// Turned a quarter turn clockwise: the outward side of a counter-clockwise edge.
    pub fn right(self) -> P2 {
        P2::new(self.y, -self.x)
    }
    pub fn lerp(self, o: P2, t: f32) -> P2 {
        P2::new(self.x + (o.x - self.x) * t, self.y + (o.y - self.y) * t)
    }
}

/// Positive when the ring runs counter-clockwise (x east, y north).
pub fn signed_area(ring: &[P2]) -> f32 {
    let mut a = 0.0;
    for i in 0..ring.len() {
        let p = ring[i];
        let q = ring[(i + 1) % ring.len()];
        a += p.cross(q);
    }
    a * 0.5
}

/// The ring without a repeated closing point, running counter-clockwise.
pub fn clean_ring(mut ring: Vec<P2>) -> Vec<P2> {
    if ring.len() > 1 && ring[0].dist(ring[ring.len() - 1]) < 0.01 {
        ring.pop();
    }
    ring.dedup_by(|a, b| a.dist(*b) < 0.01);
    if signed_area(&ring) < 0.0 {
        ring.reverse();
    }
    ring
}

pub fn point_in_ring(p: P2, ring: &[P2]) -> bool {
    let mut inside = false;
    let n = ring.len();
    let mut j = n - 1;
    for i in 0..n {
        let a = ring[i];
        let b = ring[j];
        if (a.y > p.y) != (b.y > p.y) && p.x < (b.x - a.x) * (p.y - a.y) / (b.y - a.y) + a.x {
            inside = !inside;
        }
        j = i;
    }
    inside
}

/// The point of segment a-b nearest to p.
pub fn nearest_on_segment(p: P2, a: P2, b: P2) -> P2 {
    let ab = b.sub(a);
    let l2 = ab.dot(ab);
    if l2 < 1.0e-9 {
        return a;
    }
    let t = (p.sub(a).dot(ab) / l2).clamp(0.0, 1.0);
    a.add(ab.mul(t))
}

pub fn dist_to_polyline(p: P2, line: &[P2]) -> f32 {
    let mut best = f32::MAX;
    for w in line.windows(2) {
        best = best.min(p.dist(nearest_on_segment(p, w[0], w[1])));
    }
    best
}

fn tri_contains(a: P2, b: P2, c: P2, p: P2) -> bool {
    let d1 = b.sub(a).cross(p.sub(a));
    let d2 = c.sub(b).cross(p.sub(b));
    let d3 = a.sub(c).cross(p.sub(c));
    d1 >= 0.0 && d2 >= 0.0 && d3 >= 0.0
}

/// Ear clipping. `ring` must be counter-clockwise and simple. Returns index
/// triples, counter-clockwise. A ring it cannot finish falls back to a fan,
/// so a bad footprint still gets a roof.
pub fn triangulate(ring: &[P2]) -> Vec<[u32; 3]> {
    let n = ring.len();
    let mut out = Vec::new();
    if n < 3 {
        return out;
    }
    let mut idx: Vec<usize> = (0..n).collect();
    let mut guard = 0;
    while idx.len() > 3 && guard < n * n {
        guard += 1;
        let m = idx.len();
        let mut cut = false;
        for i in 0..m {
            let ia = idx[(i + m - 1) % m];
            let ib = idx[i];
            let ic = idx[(i + 1) % m];
            let (a, b, c) = (ring[ia], ring[ib], ring[ic]);
            if b.sub(a).cross(c.sub(b)) <= 1.0e-7 {
                continue;
            }
            let mut clear = true;
            for &k in &idx {
                if k == ia || k == ib || k == ic {
                    continue;
                }
                if tri_contains(a, b, c, ring[k]) {
                    clear = false;
                    break;
                }
            }
            if clear {
                out.push([ia as u32, ib as u32, ic as u32]);
                idx.remove(i);
                cut = true;
                break;
            }
        }
        if !cut {
            break;
        }
    }
    if idx.len() == 3 {
        out.push([idx[0] as u32, idx[1] as u32, idx[2] as u32]);
    } else if idx.len() > 3 {
        for i in 1..idx.len() - 1 {
            out.push([idx[0] as u32, idx[i] as u32, idx[i + 1] as u32]);
        }
    }
    out
}

/// A stable 0..1 number from an id, for choices that must not change between runs.
pub fn hash01(id: i64, salt: u32) -> f32 {
    let mut h = (id as u64) ^ ((salt as u64) << 32) ^ 0x9E37_79B9_7F4A_7C15;
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    h = h.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
    h ^= h >> 33;
    (h >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn square_is_two_triangles_of_the_same_area() {
        let ring = clean_ring(vec![P2::new(0.0, 0.0), P2::new(0.0, 4.0), P2::new(4.0, 4.0), P2::new(4.0, 0.0)]);
        assert!(signed_area(&ring) > 0.0);
        let tris = triangulate(&ring);
        assert_eq!(tris.len(), 2);
        let mut area = 0.0;
        for t in tris {
            area += signed_area(&[ring[t[0] as usize], ring[t[1] as usize], ring[t[2] as usize]]);
        }
        assert!((area - 16.0).abs() < 1.0e-3);
    }

    #[test]
    fn an_l_shape_keeps_its_area() {
        let ring = clean_ring(vec![
            P2::new(0.0, 0.0),
            P2::new(6.0, 0.0),
            P2::new(6.0, 2.0),
            P2::new(2.0, 2.0),
            P2::new(2.0, 6.0),
            P2::new(0.0, 6.0),
        ]);
        let tris = triangulate(&ring);
        assert_eq!(tris.len(), 4);
        let mut area = 0.0;
        for t in tris {
            area += signed_area(&[ring[t[0] as usize], ring[t[1] as usize], ring[t[2] as usize]]);
        }
        assert!((area - 20.0).abs() < 1.0e-3);
        assert!(point_in_ring(P2::new(1.0, 5.0), &ring));
        assert!(!point_in_ring(P2::new(4.0, 4.0), &ring));
    }

    #[test]
    fn hash_is_stable_and_spread() {
        assert_eq!(hash01(42, 1), hash01(42, 1));
        let mut low = 0;
        for id in 0..1000 {
            let h = hash01(id, 7);
            assert!((0.0..1.0).contains(&h));
            if h < 0.5 {
                low += 1;
            }
        }
        assert!(low > 400 && low < 600);
    }
}
