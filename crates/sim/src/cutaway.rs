//! Which buildings stand between the camera and you.
//!
//! The camera is fixed: orthographic, up and to the south-east, looking
//! north-west. A building hides you when, seen down that line, its outline
//! covers the spot where you stand and it is nearer the camera than you are.
//! Those buildings are cut away, the way Project Zomboid does it.

use crate::geom::P2;
use crate::soup::world;

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = dot(a, a).sqrt();
    [a[0] / l, a[1] / l, a[2] / l]
}

/// Where the game's camera stands relative to what it looks at: south-east and
/// up, far enough back that, zoomed all the way out, the ground nearest the
/// camera is not behind it. The game and the tests both use this.
pub const EYE_OFFSET: [f32; 3] = [300.0, 310.0, 300.0];

/// How far the drawn walls stand above a building's roof line (facade.rs).
pub const PARAPET: f32 = 0.7;

/// The camera's axes in the world.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub forward: [f32; 3],
    pub right: [f32; 3],
    pub up: [f32; 3],
}

impl View {
    /// `eye_offset` is where the camera stands relative to what it looks at.
    pub fn from_offset(eye_offset: [f32; 3]) -> View {
        let forward = unit([-eye_offset[0], -eye_offset[1], -eye_offset[2]]);
        let right = unit(cross(forward, [0.0, 1.0, 0.0]));
        let up = cross(right, forward);
        View { forward, right, up }
    }

    /// (across the screen, up the screen, away from the camera) for a world point.
    pub fn screen(&self, p: [f32; 3]) -> [f32; 3] {
        [dot(p, self.right), dot(p, self.up), dot(p, self.forward)]
    }
}

/// A building as the camera sees it: its extent on screen and how near it comes.
#[derive(Clone, Copy, Debug)]
pub struct Silhouette {
    pub u: (f32, f32),
    pub v: (f32, f32),
    /// The nearest the building comes to the camera.
    pub near: f32,
}

impl Silhouette {
    pub fn of(footprint: &[P2], height: f32, view: &View) -> Silhouette {
        Silhouette::of_points(footprint.iter().flat_map(|p| [world(*p, 0.0), world(*p, height + 1.0)]), view)
    }

    /// The silhouette of a set of points in the world.
    pub fn of_points(points: impl IntoIterator<Item = [f32; 3]>, view: &View) -> Silhouette {
        let mut s = Silhouette { u: (f32::MAX, f32::MIN), v: (f32::MAX, f32::MIN), near: f32::MAX };
        for p in points {
            let q = view.screen(p);
            s.u = (s.u.0.min(q[0]), s.u.1.max(q[0]));
            s.v = (s.v.0.min(q[1]), s.v.1.max(q[1]));
            s.near = s.near.min(q[2]);
        }
        s
    }

    /// True when this building's box on screen covers a person standing at `at`
    /// and it comes nearer the camera than they do. Cheap and generous: a first
    /// pass before `Prism::hides`.
    pub fn may_hide(&self, at: P2, view: &View, margin: f32) -> bool {
        let feet = view.screen(world(at, 0.0));
        let head = view.screen(world(at, 1.8));
        let (v0, v1) = (feet[1].min(head[1]) - margin, feet[1].max(head[1]) + margin);
        let overlaps = self.u.0 < feet[0] + margin && self.u.1 > feet[0] - margin && self.v.0 < v1 && self.v.1 > v0;
        overlaps && self.near < feet[2]
    }
}

/// A building as a solid: its footprint, standing `height` metres tall.
#[derive(Clone, Debug)]
pub struct Prism {
    pub footprint: Vec<P2>,
    pub height: f32,
    pub sil: Silhouette,
}

impl Prism {
    pub fn new(footprint: &[P2], height: f32, view: &View) -> Prism {
        Prism { footprint: footprint.to_vec(), height, sil: Silhouette::of(footprint, height, view) }
    }

    /// True when the building is really in the way: a line from the camera to
    /// the person's feet, chest, head or either shoulder (widened by `margin`)
    /// passes through the solid before it reaches them.
    pub fn hides(&self, at: P2, view: &View, margin: f32) -> bool {
        if !self.sil.may_hide(at, view, margin) {
            return false;
        }
        for (start, end) in sight_lines(at, view, margin, self.height) {
            // The same points on the map (north is minus z).
            let a = P2::new(start[0], -start[2]);
            let b = P2::new(end[0], -end[2]);
            for (t0, _) in crate::geom::spans_inside(a, b, &self.footprint) {
                // A point already inside the wall is not hidden by it: the building has
                // to be in front, so the line has to come into it from outside.
                if t0 <= 1.0e-4 {
                    continue;
                }
                // Where the line enters the footprint it is at its lowest over it.
                let h = start[1] + (end[1] - start[1]) * t0;
                if h < self.height {
                    return true;
                }
            }
        }
        false
    }
}

/// The six lines the cut-away judges, from a person standing at `at` toward the
/// camera: from the feet, chest, head, either shoulder (widened by `margin`, at
/// most 1.2 m) and just over the head. Each runs until it is 2 m above `top`;
/// going toward the camera is going up, so past that nothing lower can be hit.
pub fn sight_lines(at: P2, view: &View, margin: f32, top: f32) -> [([f32; 3], [f32; 3]); 6] {
    let back = [-view.forward[0], -view.forward[1], -view.forward[2]];
    let reach = (top + 2.0) / back[1].max(0.05);
    // Shoulders a little wider than a person, so a wall right beside you is not "in front".
    let shoulder = margin.min(1.2);
    let side = [view.right[0] * shoulder, 0.0, view.right[2] * shoulder];
    [(0.1, 0.0), (1.0, 0.0), (1.8, 0.0), (1.0, 1.0), (1.0, -1.0), (2.6, 0.0)].map(|(up, s)| {
        let start = world(at, up);
        let start = [start[0] + side[0] * s, start[1], start[2] + side[2] * s];
        let end = [start[0] + back[0] * reach, start[1] + back[1] * reach, start[2] + back[2] * reach];
        (start, end)
    })
}

/// True when the segment from `a` to `b` passes through the triangle, from either
/// side, after leaving `a` (Moller and Trumbore).
pub fn segment_hits(a: [f32; 3], b: [f32; 3], t: &[[f32; 3]; 3]) -> bool {
    let d = sub(b, a);
    let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
    let p = cross(d, e2);
    let det = dot(e1, p);
    if det.abs() < 1.0e-9 {
        return false;
    }
    let inv = 1.0 / det;
    let s = sub(a, t[0]);
    let u = dot(s, p) * inv;
    if !(0.0..=1.0).contains(&u) {
        return false;
    }
    let q = cross(s, e1);
    let v = dot(d, q) * inv;
    if v < 0.0 || u + v > 1.0 {
        return false;
    }
    let along = dot(e2, q) * inv;
    along > 1.0e-4 && along <= 1.0
}

/// One building as the cut-away judges it: a prism up to the top of its parapet,
/// and the drawn triangles of what stands above that (a corner crown). The crown
/// is judged by its own triangles: a column standing in for it would cut the
/// building away when a sight line only passed through the air beside the spire.
#[derive(Clone, Debug)]
pub struct Occluder {
    pub body: Prism,
    pub above: Vec<[[f32; 3]; 3]>,
    above_sil: Option<Silhouette>,
    above_top: f32,
}

impl Occluder {
    /// `height` is the roof line; the parapet is added here.
    pub fn new(footprint: &[P2], height: f32, above: Vec<[[f32; 3]; 3]>, view: &View) -> Occluder {
        let above_sil = if above.is_empty() { None } else { Some(Silhouette::of_points(above.iter().flatten().copied(), view)) };
        let above_top = above.iter().flatten().map(|p| p[1]).fold(0.0, f32::max);
        Occluder { body: Prism::new(footprint, height + PARAPET, view), above, above_sil, above_top }
    }

    /// The occluder of a building in the scene: what the game cuts away by.
    pub fn of_block(block: &crate::scene::Block, view: &View) -> Occluder {
        Occluder::new(&block.footprint, block.height, block.crown.clone(), view)
    }

    pub fn hides(&self, at: P2, view: &View, margin: f32) -> bool {
        self.body.hides(at, view, margin) || self.above_hides(at, view, margin)
    }

    /// True when a sight line passes through what stands above the parapet.
    pub fn above_hides(&self, at: P2, view: &View, margin: f32) -> bool {
        let Some(sil) = &self.above_sil else { return false };
        if !sil.may_hide(at, view, margin) {
            return false;
        }
        sight_lines(at, view, margin, self.above_top).iter().any(|(a, b)| self.above.iter().any(|t| segment_hits(*a, *b, t)))
    }
}

/// How far from the camera a spot on the ground is.
pub fn depth_of(at: P2, view: &View) -> f32 {
    view.screen(world(at, 0.0))[2]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn square(cx: f32, cy: f32, half: f32) -> Vec<P2> {
        vec![P2::new(cx - half, cy - half), P2::new(cx + half, cy - half), P2::new(cx + half, cy + half), P2::new(cx - half, cy + half)]
    }

    #[test]
    fn the_camera_is_south_east_and_looks_north_west() {
        let view = View::from_offset(EYE_OFFSET);
        // Further north-west on the map is further from the camera.
        assert!(depth_of(P2::new(-10.0, 10.0), &view) > depth_of(P2::new(0.0, 0.0), &view));
        assert!(depth_of(P2::new(10.0, -10.0), &view) < depth_of(P2::new(0.0, 0.0), &view));
    }

    #[test]
    fn a_tall_block_south_east_of_you_hides_you_and_one_behind_you_does_not() {
        let view = View::from_offset(EYE_OFFSET);
        let me = P2::new(0.0, 0.0);
        // South-east of you, between you and the camera.
        let front = Prism::new(&square(14.0, -14.0, 6.0), 18.0, &view);
        assert!(front.hides(me, &view, 1.5));
        // North-west of you, behind you: you stand in front of it.
        let behind = Prism::new(&square(-14.0, 14.0, 6.0), 18.0, &view);
        assert!(!behind.hides(me, &view, 1.5));
        // Off to one side: not on the line at all.
        let aside = Prism::new(&square(40.0, 40.0, 6.0), 18.0, &view);
        assert!(!aside.hides(me, &view, 1.5));
        // A low wall south-east of you, far away, is short enough to leave you seen.
        let low = Prism::new(&square(40.0, -40.0, 3.0), 2.0, &view);
        assert!(!low.hides(me, &view, 1.5));
    }

    #[test]
    fn standing_in_front_of_an_l_shaped_block_does_not_cut_it_away() {
        // In screen terms: e runs to the right (north-east on the map), n away up
        // the screen (north-west). The long wing is right behind you; the short wing
        // comes forward off to your right. Its box on screen covers you and it comes
        // nearer the camera than you, but nothing of it is between you and the camera.
        let view = View::from_offset(EYE_OFFSET);
        let e = P2::new(0.70710678, 0.70710678);
        let n = P2::new(-0.70710678, 0.70710678);
        let at = |a: f32, b: f32| e.mul(a).add(n.mul(b));
        let l = vec![at(-30.0, 15.0), at(20.0, 15.0), at(20.0, -20.0), at(30.0, -20.0), at(30.0, 25.0), at(-30.0, 25.0)];
        let prism = Prism::new(&crate::geom::clean_ring(l), 21.0, &view);
        let me = P2::new(0.0, 0.0);
        assert!(prism.sil.may_hide(me, &view, 2.5), "the box test alone would cut it away");
        assert!(!prism.hides(me, &view, 2.5));
        // Behind the long wing, with the wing between you and the camera, it is cut away.
        assert!(prism.hides(at(0.0, 32.0), &view, 2.5));
    }

    // Sight lines the box test alone gets wrong (review, 16 November 2026).

    #[test]
    fn a_big_block_in_front_hides_you_even_when_the_sight_line_ends_inside_it() {
        // The game's camera. Every line toward it ends over the roof, inside this footprint.
        let view = View::from_offset(EYE_OFFSET);
        let me = P2::new(0.0, 0.0);
        let big = Prism::new(&square(40.0, -40.0, 35.0), 18.0, &view);
        assert!(big.hides(me, &view, 2.5));
    }

    #[test]
    fn a_tall_block_further_off_still_hides_you() {
        // 30 m tall, its near corner 28 m to the south-east: the line from your feet
        // enters it at about 21 m up.
        let view = View::from_offset(EYE_OFFSET);
        let tall = Prism::new(&square(25.0, -25.0, 5.0), 30.0, &view);
        assert!(tall.hides(P2::new(0.0, 0.0), &view, 2.5));
    }

    #[test]
    fn a_shed_just_in_front_does_not_hide_you() {
        // 3 m tall, its near corner 5 m to the south-east: the camera looks over it.
        let view = View::from_offset(EYE_OFFSET);
        let me = P2::new(0.0, 0.0);
        let shed = Prism::new(&square(6.5, -6.5, 3.0), 3.0, &view);
        assert!(shed.sil.may_hide(me, &view, 2.5));
        assert!(!shed.hides(me, &view, 2.5));
    }

    #[test]
    fn a_block_covering_only_your_shoulder_still_hides_you() {
        // The camera's right on the map, from the test view: south-west to north-east.
        let view = View::from_offset(EYE_OFFSET);
        let me = P2::new(0.0, 0.0);
        let r = P2::new(view.right[0], -view.right[2]);
        let toward = P2::new(0.70710678, -0.70710678);
        // A tall narrow block in front of you, offset sideways so the centre lines pass it by.
        let c = toward.mul(12.0).add(r.mul(2.0));
        let ring = vec![c.add(P2::new(-1.0, -1.0)), c.add(P2::new(1.0, -1.0)), c.add(P2::new(1.0, 1.0)), c.add(P2::new(-1.0, 1.0))];
        let block = Prism::new(&ring, 20.0, &view);
        assert!(block.hides(me, &view, 2.5));
    }
}
