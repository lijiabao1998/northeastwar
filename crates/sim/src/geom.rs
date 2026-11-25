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

/// A closed ring with every point dropped that lies within `tolerance` metres of
/// the straight line its neighbours would make (Douglas-Peucker). CMAB traces
/// roofs as long chains of centimetre steps; walls are drawn per edge, so those
/// steps must go before a facade is built.
pub fn simplify(ring: &[P2], tolerance: f32) -> Vec<P2> {
    let n = ring.len();
    if n < 4 {
        return ring.to_vec();
    }
    // Split the ring at its two points furthest apart, then simplify each half as a line.
    let mut a = 0;
    let mut b = 0;
    let mut far = -1.0;
    for i in 0..n {
        let d = ring[i].dist(ring[0]);
        if d > far {
            far = d;
            b = i;
        }
    }
    far = -1.0;
    for i in 0..n {
        let d = ring[i].dist(ring[b]);
        if d > far {
            far = d;
            a = i;
        }
    }
    let (a, b) = (a.min(b), a.max(b));
    if a == b {
        return ring.to_vec();
    }
    let first: Vec<P2> = ring[a..=b].to_vec();
    let mut second: Vec<P2> = ring[b..].to_vec();
    second.extend_from_slice(&ring[..=a]);
    let mut out = dp_line(&first, tolerance);
    out.pop();
    let mut rest = dp_line(&second, tolerance);
    rest.pop();
    out.extend(rest);
    if out.len() < 3 {
        return ring.to_vec();
    }
    out
}

fn dp_line(line: &[P2], tolerance: f32) -> Vec<P2> {
    if line.len() <= 2 {
        return line.to_vec();
    }
    let (a, b) = (line[0], line[line.len() - 1]);
    let mut worst = 0.0;
    let mut at = 0;
    for (i, p) in line.iter().enumerate().take(line.len() - 1).skip(1) {
        let d = p.dist(nearest_on_segment(*p, a, b));
        if d > worst {
            worst = d;
            at = i;
        }
    }
    if worst <= tolerance {
        return vec![a, b];
    }
    let mut left = dp_line(&line[..=at], tolerance);
    let right = dp_line(&line[at..], tolerance);
    left.pop();
    left.extend(right);
    left
}

/// The stretches of segment a-b that lie inside `ring`, as fractions along it.
pub fn spans_inside(a: P2, b: P2, ring: &[P2]) -> Vec<(f32, f32)> {
    let n = ring.len();
    let d = b.sub(a);
    // Where the segment crosses the ring's edges.
    let mut cuts = vec![0.0f32, 1.0];
    for i in 0..n {
        let (c, e) = (ring[i], ring[(i + 1) % n]);
        let f = e.sub(c);
        let den = d.cross(f);
        if den.abs() < 1.0e-9 {
            continue;
        }
        let t = c.sub(a).cross(f) / den;
        let u = c.sub(a).cross(d) / den;
        if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) {
            cuts.push(t);
        }
    }
    cuts.sort_by(|x, y| x.total_cmp(y));
    let mut out = Vec::new();
    for k in 0..cuts.len() - 1 {
        let (t0, t1) = (cuts[k], cuts[k + 1]);
        if t1 - t0 > 1.0e-6 && point_in_ring(a.lerp(b, (t0 + t1) * 0.5), ring) {
            out.push((t0, t1));
        }
    }
    out
}

/// True when two edges of the ring cross (edges that share a corner do not count).
pub fn self_intersects(ring: &[P2]) -> bool {
    let n = ring.len();
    let cross = |a: P2, b: P2, c: P2, d: P2| -> bool {
        let d1 = b.sub(a).cross(c.sub(a));
        let d2 = b.sub(a).cross(d.sub(a));
        let d3 = d.sub(c).cross(a.sub(c));
        let d4 = d.sub(c).cross(b.sub(c));
        (d1 > 0.0) != (d2 > 0.0) && (d3 > 0.0) != (d4 > 0.0) && d1 != 0.0 && d2 != 0.0 && d3 != 0.0 && d4 != 0.0
    };
    for i in 0..n {
        for j in i + 1..n {
            if j == i + 1 || (i == 0 && j == n - 1) {
                continue;
            }
            if cross(ring[i], ring[(i + 1) % n], ring[j], ring[(j + 1) % n]) {
                return true;
            }
        }
    }
    false
}

/// `simplify`, but never at the cost of a ring that crosses itself: the tolerance
/// is halved until it does not, the last try at exactly 1 cm. None if even that does not help; the
/// unsimplified ring is no answer, since its centimetre steps are what leaves
/// see-through slits in the walls. A speck that simplifies to fewer than three
/// points comes back as it is, for the caller's size check to drop.
pub fn simplify_safely(ring: &[P2], tolerance: f32) -> Option<Vec<P2>> {
    let mut t = tolerance;
    loop {
        let s = clean_ring(simplify(ring, t));
        if s.len() < 3 || !self_intersects(&s) {
            return Some(s);
        }
        if t <= 0.01 {
            return None;
        }
        t = (t * 0.5).max(0.01);
    }
}

/// How much of `line` runs inside `ring`, in metres.
pub fn length_inside(line: &[P2], ring: &[P2]) -> f32 {
    let mut total = 0.0;
    for w in line.windows(2) {
        let len = w[0].dist(w[1]);
        for (t0, t1) in spans_inside(w[0], w[1], ring) {
            total += len * (t1 - t0);
        }
    }
    total
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
    fn a_line_through_a_square_is_measured_inside_it() {
        let ring = clean_ring(vec![P2::new(0.0, 0.0), P2::new(10.0, 0.0), P2::new(10.0, 10.0), P2::new(0.0, 10.0)]);
        // Straight across: 10 m inside.
        assert!((length_inside(&[P2::new(-5.0, 5.0), P2::new(15.0, 5.0)], &ring) - 10.0).abs() < 1.0e-3);
        // Starting inside, bending out: 5 + 0.
        assert!((length_inside(&[P2::new(5.0, 5.0), P2::new(5.0, 20.0), P2::new(30.0, 20.0)], &ring) - 5.0).abs() < 1.0e-3);
        // Passing by: nothing.
        assert_eq!(length_inside(&[P2::new(-5.0, 12.0), P2::new(15.0, 12.0)], &ring), 0.0);
    }

    #[test]
    fn simplify_drops_centimetre_steps_and_keeps_corners() {
        // A 20 x 10 m rectangle traced as 5 cm stairs along its long sides.
        let mut ring = Vec::new();
        for i in 0..400 {
            ring.push(P2::new(i as f32 * 0.05, if i % 2 == 0 { 0.0 } else { 0.04 }));
        }
        ring.push(P2::new(20.0, 0.0));
        ring.push(P2::new(20.0, 10.0));
        for i in (0..400).rev() {
            ring.push(P2::new(i as f32 * 0.05, if i % 2 == 0 { 10.0 } else { 9.96 }));
        }
        let ring = clean_ring(ring);
        let simple = clean_ring(simplify(&ring, 0.3));
        assert!(simple.len() <= 6, "{} points left", simple.len());
        assert!((signed_area(&simple) - 200.0).abs() < 2.0, "{}", signed_area(&simple));
        // Every edge long enough to get a wall.
        for i in 0..simple.len() {
            assert!(simple[i].dist(simple[(i + 1) % simple.len()]) >= 0.3);
        }
    }

    #[test]
    fn simplify_never_makes_a_ring_cross_itself() {
        // A 10 x 10 m block whose south wall dips 0.25 m in the middle, with a notch from
        // the north wall reaching down to 0.1 m: plain Douglas-Peucker at 0.3 m drops the
        // dip and the notch pokes through the wall (found in review, 16 November 2026).
        let ring = clean_ring(vec![
            P2::new(0.0, 0.25),
            P2::new(5.0, 0.0),
            P2::new(10.0, 0.25),
            P2::new(10.0, 10.0),
            P2::new(5.2, 10.0),
            P2::new(5.0, 0.1),
            P2::new(4.8, 10.0),
            P2::new(0.0, 10.0),
        ]);
        assert!(!self_intersects(&ring));
        assert!(self_intersects(&clean_ring(simplify(&ring, 0.3))), "the case no longer shows the problem");
        assert!(!self_intersects(&simplify_safely(&ring, 0.3).unwrap()));
        // A ring that crosses itself whatever the tolerance is refused, not passed on.
        let bowtie = vec![P2::new(0.0, 0.0), P2::new(10.0, 10.0), P2::new(10.0, 0.0), P2::new(0.0, 10.0)];
        assert!(simplify_safely(&bowtie, 0.3).is_none());
        // A ring that only 1 cm saves is saved: the last try is exactly 1 cm, not 1.9 cm.
        let fine = clean_ring(vec![
            P2::new(0.0, 0.015),
            P2::new(5.0, 0.0),
            P2::new(10.0, 0.015),
            P2::new(10.0, 10.0),
            P2::new(5.2, 10.0),
            P2::new(5.0, 0.0075),
            P2::new(4.8, 10.0),
            P2::new(0.0, 10.0),
        ]);
        assert!(self_intersects(&clean_ring(simplify(&fine, 0.01875))), "the case no longer shows the problem");
        let saved = simplify_safely(&fine, 0.3).expect("1 cm saves it");
        assert!(!self_intersects(&saved));
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

    #[test]
    fn a_line_ending_inside_counts_to_its_end() {
        let ring = clean_ring(vec![P2::new(0.0, 0.0), P2::new(10.0, 0.0), P2::new(10.0, 10.0), P2::new(0.0, 10.0)]);
        // Coming in from outside and stopping inside: 5 m.
        assert!((length_inside(&[P2::new(-5.0, 5.0), P2::new(5.0, 5.0)], &ring) - 5.0).abs() < 1.0e-3);
        let s = spans_inside(P2::new(-5.0, 5.0), P2::new(5.0, 5.0), &ring);
        assert_eq!(s.len(), 1);
        assert!((s[0].0 - 0.5).abs() < 1.0e-5 && (s[0].1 - 1.0).abs() < 1.0e-6, "{s:?}");
        // Wholly inside: all of it.
        assert!((length_inside(&[P2::new(2.0, 5.0), P2::new(8.0, 5.0)], &ring) - 6.0).abs() < 1.0e-3);
    }
}
