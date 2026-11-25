//! Articulated winter humanoid figures and stride-locked procedural motion.
//!
//! Modelled after the hierarchical "白面人" rig in `defendtaipei/packages/character-rig`:
//! local +Y is up, local +Z is forward (face, buttons, weapon barrel), local +X is left.

use crate::arsenal::{Faction, VehicleKind};
use crate::look::Rgb;
use crate::soup::Soup;
use std::f32::consts::{FRAC_PI_2, TAU};

pub const THIGH_LEN: f32 = 0.45;
pub const SHIN_LEN: f32 = 0.43;
pub const HIP_HEIGHT: f32 = THIGH_LEN + SHIN_LEN + 0.075;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Role {
    Civilian,
    Militia,
    RussianSoldier,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AnimState {
    Idle,
    Walk,
    Run,
    Aim,
    Shoot,
    Loot,
    HandsUp,
    Crouch,
    Prone,
    Vault,
    Dead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum HeldItem {
    None,
    Rifle,
    Molotov,
    Radio,
}

#[derive(Clone, Copy, Debug)]
pub struct FigureOpts {
    pub role: Role,
    pub coat: Rgb,
    pub pants: Rgb,
    pub item: HeldItem,
    pub female: bool,
    pub coat_upgrade: bool,
    pub optics: bool,
}

impl Default for FigureOpts {
    fn default() -> Self {
        Self {
            role: Role::Civilian,
            coat: Rgb { r: 0.10, g: 0.11, b: 0.13 },
            pants: Rgb { r: 0.14, g: 0.16, b: 0.20 },
            item: HeldItem::None,
            female: false,
            coat_upgrade: false,
            optics: false,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Pose {
    pub hip_y: f32,
    pub hip_yaw: f32,
    pub body_x: f32,
    pub body_z: f32,
    pub sp_x: f32,
    pub sp_y: f32,
    pub ch_x: f32,
    pub ch_y: f32,
    pub hd_x: f32,
    pub hd_y: f32,
    pub th_l: f32,
    pub th_r: f32,
    pub th_lz: f32,
    pub th_rz: f32,
    pub sh_l: f32,
    pub sh_r: f32,
    pub ft_l: f32,
    pub ft_r: f32,
    pub ua_l: f32,
    pub ua_r: f32,
    pub ua_lz: f32,
    pub ua_rz: f32,
    pub ua_ly: f32,
    pub ua_ry: f32,
    pub la_l: f32,
    pub la_r: f32,
}

fn sm(x: f32) -> f32 {
    let t = x.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Stride-locked procedural pose ported from `character-rig/src/motion.ts`.
pub fn pose(state: AnimState, speed: f32, phase: f32, time: f32) -> Pose {
    let mut p = Pose { hip_y: HIP_HEIGHT, ..Default::default() };
    if state == AnimState::Dead {
        p.body_x = -FRAC_PI_2;
        p.hip_y = 0.22;
        p.ua_lz = 0.28;
        p.ua_rz = -0.22;
        p.th_l = -0.08;
        p.sh_l = 0.18;
        return p;
    }
    if state == AnimState::Prone {
        p.body_x = FRAC_PI_2 * 0.92;
        p.hip_y = 0.20;
        p.sp_x = -0.18;
        p.hd_x = -0.30;
        p.ua_l = -0.55;
        p.ua_r = -0.55;
        p.ua_lz = 0.22;
        p.ua_rz = -0.22;
        p.la_l = -0.60;
        p.la_r = -0.60;
        p.th_l = 0.12;
        p.th_r = -0.08;
        p.sh_l = 0.22;
        p.sh_r = 0.18;
        return p;
    }
    if state == AnimState::Crouch {
        p.hip_y = HIP_HEIGHT * 0.62;
        p.sp_x = 0.24;
        p.ch_x = 0.06;
        p.hd_x = -0.16;
        p.th_l = -0.85;
        p.sh_l = 1.45;
        p.ft_l = -0.55;
        p.th_r = -0.65;
        p.sh_r = 1.15;
        p.ft_r = -0.45;
        p.th_lz = 0.10;
        p.th_rz = -0.10;
        p.sp_y = -0.16;
        p.ua_r = -1.00;
        p.ua_ry = 0.20;
        p.ua_rz = -0.05;
        p.la_r = -0.55;
        p.ua_l = -0.90;
        p.ua_ly = -0.40;
        p.ua_lz = 0.08;
        p.la_l = -0.72;
        return p;
    }
    if state == AnimState::Vault {
        p.hip_y = HIP_HEIGHT + 0.36;
        p.sp_x = 0.32;
        p.ch_x = 0.08;
        p.hd_x = -0.12;
        p.th_l = -1.05;
        p.sh_l = 1.35;
        p.ft_l = -0.25;
        p.th_r = 0.45;
        p.sh_r = 0.65;
        p.ft_r = -0.35;
        p.ua_l = -0.75;
        p.ua_lz = 0.24;
        p.la_l = -0.55;
        p.ua_r = -0.95;
        p.ua_rz = -0.14;
        p.la_r = -0.62;
        return p;
    }

    let s = speed.max(0.0);
    let walk_w = sm(s / 1.2);
    let run_w = sm((s - 3.6) / 2.2);
    let aim_w = if matches!(state, AnimState::Aim | AnimState::Shoot) { 1.0 } else { 0.0 };
    let stride = 0.95 + 0.42 * s;
    let duty = 0.55 - 0.20 * run_w;
    let raw = (stride * duty) / (2.0 * (THIGH_LEN + SHIN_LEN));
    let amp = walk_w * raw.clamp(0.20, 0.78).asin();

    let sl = phase.sin();
    let cl = phase.cos();
    let leg = |sinv: f32, cosv: f32| -> (f32, f32) {
        let th = -sinv * amp;
        let swing = cosv.max(0.0);
        let sh = swing * amp * (1.3 + run_w * 1.6) + 0.05 + sinv.max(0.0) * amp * 0.25 * run_w;
        (th, sh)
    };
    (p.th_l, p.sh_l) = leg(sl, cl);
    (p.th_r, p.sh_r) = leg(-sl, -cl);
    p.ft_l = -(p.th_l + p.sh_l * 0.55) * 0.5;
    p.ft_r = -(p.th_r + p.sh_r * 0.55) * 0.5;

    let breath = (time * 1.7).sin();
    let idle_w = 1.0 - walk_w;
    p.hip_y = HIP_HEIGHT - walk_w * (amp * 0.09) * (1.0 - sl.abs());
    p.hip_yaw = -sl * amp * 0.25 + idle_w * (time * 0.37).sin() * 0.05;
    p.sp_y = sl * amp * 0.32;
    p.ch_y = -sl * amp * 0.10;
    p.th_lz = idle_w * 0.04 + 0.05 * walk_w + 0.04 * run_w;
    p.th_rz = -idle_w * 0.03 - 0.05 * walk_w - 0.04 * run_w;
    p.ch_x = breath * 0.018 * idle_w - run_w * 0.05;
    p.hd_x = -breath * 0.012;
    p.hd_y = idle_w * (time * 0.29).sin() * 0.30 * (1.0 - aim_w);
    p.sp_x = 0.04 + run_w * 0.22 + walk_w * 0.04;

    p.ua_l = p.th_r * 0.85;
    p.ua_r = p.th_l * 0.85;
    p.ua_lz = 0.12 + run_w * 0.08 + breath * 0.01;
    p.ua_rz = -0.12 - run_w * 0.08 - breath * 0.01;
    p.la_l = -0.18 - amp * 0.35 - run_w * 0.95 + (-p.th_r).max(0.0) * 0.25;
    p.la_r = -0.18 - amp * 0.35 - run_w * 0.95 + (-p.th_l).max(0.0) * 0.25;

    match state {
        AnimState::Aim | AnimState::Shoot => {
            let kick = if state == AnimState::Shoot { -0.14 } else { 0.0 };
            p.sp_y -= 0.18;
            p.ua_r = -1.05 + kick;
            p.ua_ry = 0.22;
            p.ua_rz = -0.05;
            p.la_r = -0.58 + kick * 0.5;
            p.ua_l = -0.95;
            p.ua_ly = -0.42;
            p.ua_lz = 0.08;
            p.la_l = -0.75;
        }
        AnimState::Loot => {
            p.sp_x = 0.48;
            p.ua_l = -0.85;
            p.ua_r = -0.85;
            p.la_l = -0.35;
            p.la_r = -0.35;
        }
        AnimState::HandsUp => {
            p.ua_l = -1.45;
            p.ua_r = -1.45;
            p.ua_lz = 0.35;
            p.ua_rz = -0.35;
            p.la_l = -0.85;
            p.la_r = -0.85;
        }
        _ => {}
    }
    p
}

#[derive(Clone, Copy)]
struct Frame {
    pos: [f32; 3],
    rot: [[f32; 3]; 3],
}

impl Frame {
    fn root() -> Self {
        Self { pos: [0.0; 3], rot: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] }
    }

    fn child(self, offset: [f32; 3], rx: f32, ry: f32, rz: f32) -> Self {
        let p = self.point(offset);
        // Apply Rz then Rx then Ry in local child axes, matching standard character joints.
        let m = mul_m(mul_m(mat_y(ry), mat_x(rx)), mat_z(rz));
        Self { pos: p, rot: mul_m(self.rot, m) }
    }

    fn point(self, v: [f32; 3]) -> [f32; 3] {
        let r = mul_v(self.rot, v);
        [self.pos[0] + r[0], self.pos[1] + r[1], self.pos[2] + r[2]]
    }

    fn dir(self, n: [f32; 3]) -> [f32; 3] {
        mul_v(self.rot, n)
    }
}

fn mat_x(a: f32) -> [[f32; 3]; 3] {
    let (s, c) = a.sin_cos();
    [[1.0, 0.0, 0.0], [0.0, c, -s], [0.0, s, c]]
}

fn mat_y(a: f32) -> [[f32; 3]; 3] {
    let (s, c) = a.sin_cos();
    [[c, 0.0, s], [0.0, 1.0, 0.0], [-s, 0.0, c]]
}

fn mat_z(a: f32) -> [[f32; 3]; 3] {
    let (s, c) = a.sin_cos();
    [[c, -s, 0.0], [s, c, 0.0], [0.0, 0.0, 1.0]]
}

fn mul_v(m: [[f32; 3]; 3], v: [f32; 3]) -> [f32; 3] {
    [
        m[0][0] * v[0] + m[0][1] * v[1] + m[0][2] * v[2],
        m[1][0] * v[0] + m[1][1] * v[1] + m[1][2] * v[2],
        m[2][0] * v[0] + m[2][1] * v[1] + m[2][2] * v[2],
    ]
}

fn mul_m(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    let mut out = [[0.0; 3]; 3];
    for r in 0..3 {
        for c in 0..3 {
            out[r][c] = a[r][0] * b[0][c] + a[r][1] * b[1][c] + a[r][2] * b[2][c];
        }
    }
    out
}

fn norm3(v: [f32; 3]) -> [f32; 3] {
    let l = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt().max(1.0e-6);
    [v[0] / l, v[1] / l, v[2] / l]
}

/// Outward-wound axis-aligned box transformed by `frame`.
fn add_box(s: &mut Soup, f: Frame, centre: [f32; 3], size: [f32; 3], col: Rgb) {
    let (hx, hy, hz) = (size[0] * 0.5, size[1] * 0.5, size[2] * 0.5);
    let pt = |sx: f32, sy: f32, sz: f32| f.point([centre[0] + sx * hx, centre[1] + sy * hy, centre[2] + sz * hz]);
    // 6 faces, each wound CCW as seen from outside:
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        ([0.0, 0.0, 1.0], [pt(-1.0, -1.0, 1.0), pt(1.0, -1.0, 1.0), pt(1.0, 1.0, 1.0), pt(-1.0, 1.0, 1.0)]),
        ([0.0, 0.0, -1.0], [pt(1.0, -1.0, -1.0), pt(-1.0, -1.0, -1.0), pt(-1.0, 1.0, -1.0), pt(1.0, 1.0, -1.0)]),
        ([1.0, 0.0, 0.0], [pt(1.0, -1.0, 1.0), pt(1.0, -1.0, -1.0), pt(1.0, 1.0, -1.0), pt(1.0, 1.0, 1.0)]),
        ([-1.0, 0.0, 0.0], [pt(-1.0, -1.0, -1.0), pt(-1.0, -1.0, 1.0), pt(-1.0, 1.0, 1.0), pt(-1.0, 1.0, -1.0)]),
        ([0.0, 1.0, 0.0], [pt(-1.0, 1.0, 1.0), pt(1.0, 1.0, 1.0), pt(1.0, 1.0, -1.0), pt(-1.0, 1.0, -1.0)]),
        ([0.0, -1.0, 0.0], [pt(-1.0, -1.0, -1.0), pt(1.0, -1.0, -1.0), pt(1.0, -1.0, 1.0), pt(-1.0, -1.0, 1.0)]),
    ];
    for (n, corners) in faces {
        s.quad(corners, f.dir(n), col);
    }
}

/// Outward-wound multi-ring tapered prism/lathe in local Y (for limbs, coat torso, boots, egg-head).
fn add_lathe(s: &mut Soup, f: Frame, rings: &[(f32, f32)], scale_x: f32, scale_z: f32, sides: usize, col: Rgb) {
    if rings.len() < 2 {
        return;
    }
    for w in rings.windows(2) {
        let ((r0, y0), (r1, y1)) = (w[0], w[1]);
        if (r0 <= 1.0e-4 && r1 <= 1.0e-4) || (y1 - y0).abs() < 1.0e-5 {
            continue;
        }
        let dy = y1 - y0;
        let dr = r0 - r1;
        for i in 0..sides {
            let a0 = i as f32 * TAU / sides as f32;
            let a1 = (i + 1) as f32 * TAU / sides as f32;
            let mid = (a0 + a1) * 0.5;
            // In (+X, +Y, +Z), decreasing azimuth from +Z toward +X keeps CCW winding from outside:
            // Actually angle theta from +X to -Z -> vertex (cos t, sin t) is CCW around +Y!
            let p0 = [a0.cos(), -a0.sin()];
            let p1 = [a1.cos(), -a1.sin()];
            let pm = [mid.cos(), -mid.sin()];
            let n = f.dir(norm3([pm[0] * dy / scale_x, dr, pm[1] * dy / scale_z]));
            if r0 <= 1.0e-4 {
                let bot = f.point([0.0, y0, 0.0]);
                let q1 = f.point([p1[0] * r1 * scale_x, y1, p1[1] * r1 * scale_z]);
                let q0 = f.point([p0[0] * r1 * scale_x, y1, p0[1] * r1 * scale_z]);
                s.tri([bot, q1, q0], n, col);
            } else if r1 <= 1.0e-4 {
                let b0 = f.point([p0[0] * r0 * scale_x, y0, p0[1] * r0 * scale_z]);
                let b1 = f.point([p1[0] * r0 * scale_x, y0, p1[1] * r0 * scale_z]);
                let top = f.point([0.0, y1, 0.0]);
                s.tri([b0, b1, top], n, col);
            } else {
                let b0 = f.point([p0[0] * r0 * scale_x, y0, p0[1] * r0 * scale_z]);
                let b1 = f.point([p1[0] * r0 * scale_x, y0, p1[1] * r0 * scale_z]);
                let t1 = f.point([p1[0] * r1 * scale_x, y1, p1[1] * r1 * scale_z]);
                let t0 = f.point([p0[0] * r1 * scale_x, y1, p0[1] * r1 * scale_z]);
                s.quad([b0, b1, t1, t0], n, col);
            }
        }
    }
    // Cap bottom and top if open.
    if rings[0].0 > 1.0e-4 {
        let r = rings[0].0;
        let y = rings[0].1;
        let c = f.point([0.0, y, 0.0]);
        let n = f.dir([0.0, -1.0, 0.0]);
        for i in 0..sides {
            let a0 = i as f32 * TAU / sides as f32;
            let a1 = (i + 1) as f32 * TAU / sides as f32;
            let q0 = f.point([a0.cos() * r * scale_x, y, -a0.sin() * r * scale_z]);
            let q1 = f.point([a1.cos() * r * scale_x, y, -a1.sin() * r * scale_z]);
            s.tri([c, q1, q0], n, col);
        }
    }
    if rings[rings.len() - 1].0 > 1.0e-4 {
        let r = rings[rings.len() - 1].0;
        let y = rings[rings.len() - 1].1;
        let c = f.point([0.0, y, 0.0]);
        let n = f.dir([0.0, 1.0, 0.0]);
        for i in 0..sides {
            let a0 = i as f32 * TAU / sides as f32;
            let a1 = (i + 1) as f32 * TAU / sides as f32;
            let q0 = f.point([a0.cos() * r * scale_x, y, -a0.sin() * r * scale_z]);
            let q1 = f.point([a1.cos() * r * scale_x, y, -a1.sin() * r * scale_z]);
            s.tri([c, q0, q1], n, col);
        }
    }
}

/// Build an articulated winter figure standing at local origin `(0, 0, 0)`, facing `+Z`.
pub fn build_figure(opts: &FigureOpts, p: &Pose) -> Soup {
    let mut s = Soup::default();
    let k = if opts.female { 0.95 } else { 1.0 };
    let skin = Rgb { r: 0.93, g: 0.90, b: 0.87 };
    let soldier = opts.role == Role::RussianSoldier;
    let coat = if soldier {
        Rgb { r: 0.85, g: 0.87, b: 0.83 }
    } else if opts.coat_upgrade {
        opts.coat.mix(Rgb { r: 0.25, g: 0.22, b: 0.18 }, 0.35)
    } else {
        opts.coat
    };
    let pants = if soldier { Rgb { r: 0.62, g: 0.66, b: 0.62 } } else { opts.pants };
    let boot = Rgb { r: 0.09, g: 0.08, b: 0.08 };
    let trim_dark = Rgb { r: 0.14, g: 0.13, b: 0.12 };
    let fur = Rgb { r: 0.38, g: 0.28, b: 0.19 };
    let red_cloth = Rgb { r: 0.68, g: 0.12, b: 0.12 };
    let blue_iff = Rgb { r: 0.16, g: 0.56, b: 0.96 };

    let body = Frame::root().child([0.0, p.hip_y, 0.0], p.body_x, 0.0, p.body_z);
    let hips = body.child([0.0, 0.0, 0.0], 0.0, p.hip_yaw, 0.0);

    // Pelvis & belted winter waist.
    add_lathe(&mut s, hips, &[(0.12, -0.09), (0.16 * k, -0.02), (0.155 * k, 0.07), (0.135 * k, 0.10)], 1.0, 0.70, 8, pants);
    add_box(&mut s, hips, [0.0, 0.09, 0.0], [0.31 * k, 0.04, 0.22], trim_dark);

    // Winter coat skirt flaring over upper thighs.
    add_lathe(&mut s, hips, &[(0.24 * k, -0.24), (0.21 * k, -0.06), (0.175 * k, 0.08)], 1.0, 0.72, 8, coat.mix(trim_dark, 0.08));

    // Spine & Chest (padded parka silhouette).
    let spine = hips.child([0.0, 0.09, 0.0], p.sp_x, p.sp_y, 0.0);
    let torso_rings: [(f32, f32); 5] = if opts.coat_upgrade || soldier {
        [(0.155 * k, 0.0), (0.175 * k, 0.12), (0.205 * k, 0.28), (0.19 * k, 0.40), (0.09, 0.46)]
    } else {
        [(0.145 * k, 0.0), (0.16 * k, 0.14), (0.19 * k, 0.30), (0.17 * k, 0.41), (0.085, 0.45)]
    };
    add_lathe(&mut s, spine, &torso_rings, 1.0, 0.68, 8, coat);

    let chest = spine.child([0.0, 0.36, 0.0], p.ch_x, p.ch_y, 0.0);
    add_box(&mut s, chest, [0.0, 0.03, 0.0], [0.40 * k, 0.11, 0.22], coat);

    if soldier {
        let vest = Rgb { r: 0.25, g: 0.29, b: 0.20 };
        let pouch = Rgb { r: 0.31, g: 0.35, b: 0.23 };
        add_box(&mut s, chest, [0.0, -0.11, 0.0], [0.34, 0.30, 0.25], vest);
        for px in [-0.10, 0.0, 0.10] {
            add_box(&mut s, chest, [px, -0.16, 0.13], [0.075, 0.11, 0.04], pouch);
        }
        // Pack + radio antenna on back.
        add_box(&mut s, chest, [0.0, -0.11, -0.16], [0.25, 0.26, 0.10], vest);
        add_box(&mut s, chest, [-0.11, 0.14, -0.15], [0.014, 0.34, 0.014], trim_dark);
    } else {
        // Warm scarf or fur collar.
        let collar = if opts.coat_upgrade { fur } else { red_cloth };
        add_box(&mut s, chest, [0.0, 0.085, 0.04], [0.21, 0.08, 0.20], collar);
        if opts.role == Role::Militia {
            // Militia chest strap / pouch.
            add_box(&mut s, chest, [0.0, -0.10, 0.125], [0.24, 0.07, 0.035], Rgb { r: 0.30, g: 0.26, b: 0.20 });
        }
    }

    // Neck & 白面人 egg-head.
    let neck = chest.child([0.0, 0.09, 0.0], 0.0, 0.0, 0.0);
    add_lathe(&mut s, neck, &[(0.048, 0.0), (0.042, 0.10)], 1.0, 1.0, 8, skin);
    let head = neck.child([0.0, 0.09, 0.0], p.hd_x, p.hd_y, 0.0);
    add_lathe(
        &mut s,
        head,
        &[(0.0, -0.02), (0.068, 0.02), (0.090, 0.10), (0.082, 0.17), (0.046, 0.215), (0.0, 0.225)],
        0.92,
        1.04,
        8,
        skin,
    );

    if soldier {
        let helmet = Rgb { r: 0.36, g: 0.41, b: 0.30 };
        add_lathe(&mut s, head, &[(0.112, 0.105), (0.110, 0.16), (0.072, 0.218), (0.0, 0.236)], 0.98, 1.06, 8, helmet);
        // Balaclava / lower face gaiter + optics goggles if equipped.
        add_lathe(&mut s, head, &[(0.088, 0.01), (0.096, 0.085)], 0.96, 1.08, 8, Rgb { r: 0.24, g: 0.25, b: 0.24 });
        if opts.optics {
            add_box(&mut s, head, [0.0, 0.135, 0.098], [0.13, 0.038, 0.045], Rgb { r: 0.12, g: 0.14, b: 0.12 });
        }
    } else if opts.role == Role::Militia {
        // Militia olive/charcoal tactical helmet with bright blue IFF band.
        let m_helmet = Rgb { r: 0.20, g: 0.24, b: 0.22 };
        add_lathe(&mut s, head, &[(0.110, 0.108), (0.108, 0.165), (0.072, 0.220), (0.0, 0.238)], 0.98, 1.06, 8, m_helmet);
        add_lathe(&mut s, head, &[(0.114, 0.120), (0.114, 0.155)], 0.98, 1.06, 8, blue_iff);
    } else {
        // Northeast fur ushanka (雷鋒帽) with earflaps.
        let hat = fur;
        add_lathe(&mut s, head, &[(0.106, 0.12), (0.105, 0.19), (0.068, 0.238), (0.0, 0.248)], 0.98, 1.06, 8, hat);
        add_box(&mut s, head, [0.0, 0.165, 0.095], [0.16, 0.075, 0.04], hat.mix(Rgb { r: 0.55, g: 0.48, b: 0.40 }, 0.25));
        add_box(&mut s, head, [0.108, 0.11, 0.0], [0.045, 0.14, 0.08], hat);
        add_box(&mut s, head, [-0.108, 0.11, 0.0], [0.045, 0.14, 0.08], hat);
    }

    // Legs (left +X, right -X).
    for (side, th, th_z, sh, ft) in [
        (1.0, p.th_l, p.th_lz, p.sh_l, p.ft_l),
        (-1.0, p.th_r, p.th_rz, p.sh_r, p.ft_r),
    ] {
        let hip_j = hips.child([side * 0.105 * k, 0.0, 0.0], th, 0.0, th_z);
        add_lathe(&mut s, hip_j, &[(0.068 * k, -THIGH_LEN), (0.085 * k, -0.02)], 1.0, 1.0, 8, pants);
        if opts.role == Role::Militia && side < 0.0 {
            add_lathe(&mut s, hip_j, &[(0.072 * k, -THIGH_LEN + 0.06), (0.075 * k, -THIGH_LEN + 0.14)], 1.0, 1.0, 8, blue_iff);
        }
        let knee_j = hip_j.child([0.0, -THIGH_LEN, 0.0], sh, 0.0, 0.0);
        add_lathe(&mut s, knee_j, &[(0.056 * k, -SHIN_LEN), (0.066 * k, 0.0)], 1.0, 1.0, 8, pants);
        let foot_j = knee_j.child([0.0, -SHIN_LEN, 0.0], ft, 0.0, 0.0);
        add_box(&mut s, foot_j, [0.0, -0.035, 0.04], [0.115, 0.08, 0.23], boot);
    }

    // Arms (left +X, right -X).
    let arm_col = coat.mix(trim_dark, 0.06);
    let ua_l = chest.child([0.21 * k, 0.01, 0.0], p.ua_l, p.ua_ly, p.ua_lz);
    add_lathe(&mut s, ua_l, &[(0.050 * k, -0.27), (0.062 * k, 0.0)], 1.0, 1.0, 8, arm_col);
    if opts.role == Role::Militia {
        // Tactical blue IFF armband on left upper arm.
        add_lathe(&mut s, ua_l, &[(0.064 * k, -0.17), (0.066 * k, -0.09)], 1.0, 1.0, 8, blue_iff);
    }
    let la_l = ua_l.child([0.0, -0.27, 0.0], p.la_l, 0.0, 0.0);
    add_lathe(&mut s, la_l, &[(0.044 * k, -0.24), (0.050 * k, 0.0)], 1.0, 1.0, 8, arm_col);
    add_box(&mut s, la_l, [0.0, -0.26, 0.01], [0.065, 0.075, 0.085], if opts.coat_upgrade || soldier { trim_dark } else { skin });

    let ua_r = chest.child([-0.21 * k, 0.01, 0.0], p.ua_r, p.ua_ry, p.ua_rz);
    add_lathe(&mut s, ua_r, &[(0.050 * k, -0.27), (0.062 * k, 0.0)], 1.0, 1.0, 8, arm_col);
    let la_r = ua_r.child([0.0, -0.27, 0.0], p.la_r, 0.0, 0.0);
    add_lathe(&mut s, la_r, &[(0.044 * k, -0.24), (0.050 * k, 0.0)], 1.0, 1.0, 8, arm_col);
    let hand_col = if opts.coat_upgrade || soldier { trim_dark } else { skin };
    add_box(&mut s, la_r, [0.0, -0.26, 0.01], [0.065, 0.075, 0.085], hand_col);

    // Held item in right hand frame.
    let hand_r = la_r.child([0.0, -0.27, 0.03], FRAC_PI_2, 0.0, 0.0);
    match opts.item {
        HeldItem::None => {}
        HeldItem::Rifle => {
            let gun = Rgb { r: 0.11, g: 0.12, b: 0.13 };
            let wood = Rgb { r: 0.42, g: 0.26, b: 0.15 };
            add_box(&mut s, hand_r, [0.0, 0.02, 0.02], [0.045, 0.065, 0.36], gun);
            add_box(&mut s, hand_r, [0.0, 0.01, -0.24], [0.04, 0.08, 0.22], if soldier { gun } else { wood });
            add_box(&mut s, hand_r, [0.0, 0.03, 0.36], [0.022, 0.024, 0.36], gun);
            add_box(&mut s, hand_r, [0.0, -0.07, 0.10], [0.028, 0.13, 0.045], gun);
        }
        HeldItem::Molotov => {
            add_box(&mut s, hand_r, [0.0, 0.06, 0.0], [0.07, 0.22, 0.07], Rgb { r: 0.18, g: 0.42, b: 0.18 });
            add_box(&mut s, hand_r, [0.0, 0.20, 0.0], [0.04, 0.09, 0.04], red_cloth);
        }
        HeldItem::Radio => {
            add_box(&mut s, hand_r, [0.0, 0.05, 0.0], [0.06, 0.14, 0.04], trim_dark);
            add_box(&mut s, hand_r, [0.02, 0.18, 0.0], [0.012, 0.18, 0.012], trim_dark);
        }
    }
    s
}

/// Articulated PLA & Russian combat vehicle hull (local +Z = forward, +Y = up).
pub fn build_vehicle_hull(kind: VehicleKind) -> Soup {
    let mut s = Soup::default();
    let root = Frame::root();
    let (hull_col, hull_hi, emblem_col, iff_col) = match kind.faction() {
        Faction::Russia => (
            Rgb { r: 0.21, g: 0.26, b: 0.17 },
            Rgb { r: 0.27, g: 0.32, b: 0.21 },
            Rgb { r: 0.88, g: 0.88, b: 0.84 }, // Russian tactical white 'V'/'Z' stripe
            Rgb { r: 0.88, g: 0.88, b: 0.84 },
        ),
        Faction::Pla => (
            Rgb { r: 0.17, g: 0.27, b: 0.20 }, // PLA alpine dark olive-forest
            Rgb { r: 0.24, g: 0.35, b: 0.26 },
            Rgb { r: 0.84, g: 0.16, b: 0.14 }, // PLA red tactical emblem
            Rgb { r: 0.16, g: 0.60, b: 0.96 }, // Tactical blue IFF stripe along hull
        ),
    };
    let steel = Rgb { r: 0.14, g: 0.15, b: 0.15 };
    let rubber = Rgb { r: 0.08, g: 0.09, b: 0.09 };
    let headlight = Rgb { r: 0.94, g: 0.92, b: 0.72 };

    match kind {
        VehicleKind::T72B3
        | VehicleKind::T80Bvm
        | VehicleKind::T90M
        | VehicleKind::T62M
        | VehicleKind::Tos1A
        | VehicleKind::Bmpt
        | VehicleKind::Ztz99A
        | VehicleKind::Ztz96B
        | VehicleKind::Ztq15 => {
            let (hull_len, wheels, wide_skirt) = match kind {
                VehicleKind::Ztz99A | VehicleKind::T90M => (6.80, 7usize, true),
                VehicleKind::T62M | VehicleKind::Ztq15 => (6.10, 5usize, false),
                _ => (6.50, 6usize, true),
            };
            let track_len = hull_len + 0.15;
            let body_w = if matches!(kind, VehicleKind::Ztq15) { 2.02 } else { 2.26 };
            let track_x = body_w * 0.5 + 0.30;

            for sx in [-track_x, track_x] {
                add_box(&mut s, root, [sx, 0.38, 0.0], [0.60, 0.74, track_len], rubber);
                add_box(&mut s, root, [sx, 0.78, 0.0], [0.66, 0.22, track_len + 0.12], hull_hi);
                for wi in 0..wheels {
                    let frac = wi as f32 / (wheels - 1).max(1) as f32 - 0.5;
                    let wz = frac * (track_len - 1.45);
                    add_box(&mut s, root, [sx * 1.03, 0.34, wz], [0.64, 0.54, 0.54], steel);
                }
                if wide_skirt && !matches!(kind, VehicleKind::T62M) {
                    let blocks = if matches!(kind, VehicleKind::Ztz99A | VehicleKind::T90M) { 7 } else { 6 };
                    for bi in 0..blocks {
                        let bz = -1.75 + bi as f32 * 0.74;
                        add_box(
                            &mut s,
                            root,
                            [sx + sx.signum() * 0.35, 0.72, bz],
                            [0.09, 0.44, 0.66],
                            hull_col,
                        );
                    }
                }
            }
            add_box(&mut s, root, [0.0, 0.72, -0.15], [body_w, 0.72, hull_len - 0.30], hull_col);
            let glacis_z = hull_len * 0.44;
            let glacis = root.child([0.0, 0.88, glacis_z], -0.45, 0.0, 0.0);
            add_box(&mut s, glacis, [0.0, 0.0, 0.0], [body_w - 0.05, 0.28, 1.42], hull_hi);
            if !matches!(kind, VehicleKind::T62M | VehicleKind::Tos1A) {
                for ex in [-0.72f32, -0.24, 0.24, 0.72] {
                    add_box(&mut s, glacis, [ex, 0.16, 0.15], [0.42, 0.11, 0.64], hull_col);
                }
            } else if matches!(kind, VehicleKind::T62M) {
                add_box(&mut s, glacis, [0.0, 0.17, 0.12], [1.95, 0.18, 0.75], steel);
            }
            if matches!(kind, VehicleKind::Tos1A) {
                // Heavy front stabilizer dozer blade for rocket firing stability.
                add_box(&mut s, root, [0.0, 0.38, glacis_z + 0.55], [2.85, 0.56, 0.26], steel);
            }
            if matches!(kind, VehicleKind::T80Bvm | VehicleKind::Ztz99A) {
                add_box(&mut s, root, [0.0, 1.13, -hull_len * 0.32], [1.98, 0.16, 1.95], steel);
            } else {
                add_box(&mut s, root, [0.0, 1.12, -hull_len * 0.32], [1.90, 0.14, 1.80], steel);
            }
            add_box(&mut s, root, [-0.55, 0.95, -hull_len * 0.50], [0.50, 0.50, 0.92], hull_hi);
            add_box(&mut s, root, [0.55, 0.95, -hull_len * 0.50], [0.50, 0.50, 0.92], hull_hi);
            add_box(&mut s, root, [0.0, 0.97, glacis_z + 0.42], [0.55, 0.20, 0.09], emblem_col);
            if kind.faction() == Faction::Pla {
                for sx in [-track_x, track_x] {
                    add_box(&mut s, root, [sx + sx.signum() * 0.38, 0.72, 0.0], [0.05, 0.12, 3.8], iff_col);
                }
            }
            add_box(&mut s, root, [-0.90, 1.05, glacis_z + 0.22], [0.18, 0.16, 0.16], headlight);
            add_box(&mut s, root, [0.90, 1.05, glacis_z + 0.22], [0.18, 0.16, 0.16], headlight);
        }
        VehicleKind::Bmp3 | VehicleKind::Bmd4M | VehicleKind::Zbd04A | VehicleKind::Pgz09 => {
            let hull_len = if matches!(kind, VehicleKind::Bmd4M) { 5.8 } else { 6.6 };
            let hull_w = if matches!(kind, VehicleKind::Pgz09) { 2.48 } else { 2.28 };
            let tx = hull_w * 0.5 + 0.24;
            for sx in [-tx, tx] {
                add_box(&mut s, root, [sx, 0.34, 0.0], [0.46, 0.64, hull_len], rubber);
                add_box(&mut s, root, [sx, 0.72, 0.0], [0.54, 0.18, hull_len + 0.1], hull_hi);
                for wi in 0..6 {
                    let wz = -hull_len * 0.38 + wi as f32 * (hull_len * 0.76 / 5.0);
                    add_box(&mut s, root, [sx * 1.02, 0.30, wz], [0.50, 0.44, 0.44], steel);
                }
            }
            add_box(&mut s, root, [0.0, 0.78, -0.05], [hull_w, 0.70, hull_len], hull_col);
            let glacis = root.child([0.0, 0.85, hull_len * 0.46], -0.48, 0.0, 0.0);
            add_box(&mut s, glacis, [0.0, 0.0, 0.0], [hull_w - 0.08, 0.28, 1.35], hull_hi);
            add_box(&mut s, root, [0.0, 0.66, hull_len * 0.52], [hull_w, 0.26, 0.12], steel); // Water trim vane
            add_box(&mut s, root, [0.0, 1.15, hull_len * 0.22], [0.48, 0.12, 0.48], emblem_col);
            add_box(&mut s, root, [-0.88, 1.02, hull_len * 0.46], [0.16, 0.14, 0.14], headlight);
            add_box(&mut s, root, [0.88, 1.02, hull_len * 0.46], [0.16, 0.14, 0.14], headlight);
        }
        VehicleKind::Btr82A | VehicleKind::Zbl08 => {
            let w = if matches!(kind, VehicleKind::Zbl08) { 2.55 } else { 2.42 };
            let len = if matches!(kind, VehicleKind::Zbl08) { 7.40 } else { 7.10 };
            add_box(&mut s, root, [0.0, 0.98, 0.0], [w, 0.94, len], hull_col);
            let nose = root.child([0.0, 0.96, len * 0.48], -0.42, 0.0, 0.0);
            add_box(&mut s, nose, [0.0, 0.0, 0.0], [w - 0.16, 0.42, 1.18], hull_hi);
            for sx in [-(w * 0.5 + 0.12), w * 0.5 + 0.12] {
                for wz in [-2.45f32, -0.82, 0.82, 2.45] {
                    add_box(&mut s, root, [sx, 0.48, wz], [0.42, 0.92, 0.92], rubber);
                    add_box(&mut s, root, [sx * 1.03, 0.48, wz], [0.46, 0.48, 0.48], steel);
                }
                add_box(&mut s, root, [sx * 0.92, 1.16, 0.0], [0.07, 0.14, 4.8], iff_col);
            }
            add_box(&mut s, root, [-0.88, 1.16, len * 0.49], [0.16, 0.14, 0.14], headlight);
            add_box(&mut s, root, [0.88, 1.16, len * 0.49], [0.16, 0.14, 0.14], headlight);
        }
        VehicleKind::TigrM | VehicleKind::Csk181 => {
            // 4x4 high-mobility armored tactical vehicle (~2.3m wide x 5.6m long).
            add_box(&mut s, root, [0.0, 1.02, -0.45], [2.16, 1.28, 3.60], hull_col);
            add_box(&mut s, root, [0.0, 0.88, 1.95], [2.04, 0.78, 1.65], hull_hi);
            add_box(&mut s, root, [0.0, 1.38, 1.18], [1.96, 0.44, 0.12], steel); // Armored windshield frame
            add_box(&mut s, root, [0.0, 1.25, 2.82], [2.10, 0.28, 0.22], steel); // Heavy bumper
            for sx in [-1.14f32, 1.14] {
                for wz in [-1.65f32, 1.75] {
                    add_box(&mut s, root, [sx, 0.48, wz], [0.40, 0.92, 0.92], rubber);
                    add_box(&mut s, root, [sx * 1.03, 0.48, wz], [0.44, 0.46, 0.46], steel);
                }
                add_box(&mut s, root, [sx * 0.96, 1.18, -0.3], [0.08, 0.20, 0.75], iff_col);
            }
            add_box(&mut s, root, [-0.78, 0.96, 2.80], [0.18, 0.14, 0.14], headlight);
            add_box(&mut s, root, [0.78, 0.96, 2.80], [0.18, 0.14, 0.14], headlight);
        }
    }
    s
}

/// Articulated PLA & Russian armoured turret with main gun aligned along local +Z.
pub fn build_vehicle_turret(kind: VehicleKind) -> Soup {
    let mut s = Soup::default();
    let root = Frame::root();
    let (col, col_hi) = match kind.faction() {
        Faction::Russia => (Rgb { r: 0.22, g: 0.27, b: 0.18 }, Rgb { r: 0.28, g: 0.34, b: 0.22 }),
        Faction::Pla => (Rgb { r: 0.18, g: 0.28, b: 0.21 }, Rgb { r: 0.25, g: 0.36, b: 0.27 }),
    };
    let steel = Rgb { r: 0.14, g: 0.15, b: 0.15 };
    let crate_col = Rgb { r: 0.34, g: 0.24, b: 0.14 };
    let launcher_orange = Rgb { r: 0.42, g: 0.24, b: 0.14 };
    let cage_steel = Rgb { r: 0.26, g: 0.28, b: 0.26 };

    match kind {
        VehicleKind::T72B3 | VehicleKind::T80Bvm | VehicleKind::T90M | VehicleKind::T62M => {
            add_box(&mut s, root, [0.0, 0.30, -0.20], [2.15, 0.60, 2.55], col);
            add_box(&mut s, root, [0.0, 0.38, 0.90], [1.75, 0.48, 0.85], col_hi);
            let cheek_l = root.child([-0.68, 0.30, 1.18], 0.0, -0.38, 0.0);
            add_box(&mut s, cheek_l, [0.0, 0.0, 0.0], [0.76, 0.44, 0.32], col);
            let cheek_r = root.child([0.68, 0.30, 1.18], 0.0, 0.38, 0.0);
            add_box(&mut s, cheek_r, [0.0, 0.0, 0.0], [0.76, 0.44, 0.32], col);
            add_box(&mut s, root, [0.48, 0.68, -0.25], [0.64, 0.22, 0.64], steel);
            add_box(&mut s, root, [0.48, 0.92, 0.15], [0.10, 0.12, 1.15], steel);
            add_box(&mut s, root, [0.0, 0.22, -1.66], [1.85, 0.28, 0.30], crate_col);
            let gun_len = if matches!(kind, VehicleKind::T62M) { 4.2 } else { 4.85 };
            add_box(&mut s, root, [0.0, 0.30, 0.85 + gun_len * 0.5], [0.17, 0.17, gun_len], col_hi);
            add_box(&mut s, root, [0.0, 0.30, 3.10], [0.28, 0.28, 0.72], col);

            if matches!(kind, VehicleKind::T90M | VehicleKind::T62M) {
                // Russo-Ukrainian war field overhead anti-drone cope-cage slat roof!
                for sx in [-1.05f32, 1.05] {
                    for sz in [-1.10f32, 0.95] {
                        add_box(&mut s, root, [sx, 1.05, sz], [0.07, 0.82, 0.07], cage_steel);
                    }
                }
                for si in 0..5 {
                    let sz = -1.10 + si as f32 * 0.52;
                    add_box(&mut s, root, [0.0, 1.46, sz], [2.20, 0.05, 0.08], cage_steel);
                }
                add_box(&mut s, root, [-1.05, 1.46, -0.08], [0.08, 0.06, 2.20], cage_steel);
                add_box(&mut s, root, [1.05, 1.46, -0.08], [0.08, 0.06, 2.20], cage_steel);
            }
        }
        VehicleKind::Ztz99A | VehicleKind::Ztz96B | VehicleKind::Ztq15 => {
            let sc = if matches!(kind, VehicleKind::Ztq15) { 0.88 } else { 1.0 };
            add_box(&mut s, root, [0.0, 0.32, -0.35], [2.25 * sc, 0.62, 2.75 * sc], col);
            // Distinct PLA angled wedge cheek composite ERA armor!
            for side in [-1.0f32, 1.0] {
                let cheek = root.child([side * 0.66 * sc, 0.32, 1.25 * sc], 0.0, side * 0.52, 0.0);
                add_box(&mut s, cheek, [0.0, 0.0, 0.0], [1.10 * sc, 0.54, 0.42], col_hi);
            }
            add_box(&mut s, root, [0.0, 0.30, -2.05 * sc], [2.05 * sc, 0.44, 0.72], crate_col); // Rear storage basket
            add_box(&mut s, root, [-0.55 * sc, 0.72, 0.25], [0.36, 0.30, 0.36], steel); // Commander hunter-killer sight
            add_box(&mut s, root, [0.55 * sc, 0.70, -0.15], [0.60, 0.20, 0.60], steel);
            add_box(&mut s, root, [0.55 * sc, 0.94, 0.25], [0.10, 0.12, 1.15], steel);
            let gun_len = if matches!(kind, VehicleKind::Ztq15) { 4.2 } else { 5.05 };
            add_box(&mut s, root, [0.0, 0.30, 0.95 + gun_len * 0.5], [0.17, 0.17, gun_len], col_hi);
            add_box(&mut s, root, [0.0, 0.30, 3.25], [0.27, 0.27, 0.75], col);
        }
        VehicleKind::Tos1A => {
            // Massive boxy 24-tube 220mm thermobaric MLRS launcher pod elevated slightly upward.
            add_box(&mut s, root, [0.0, 0.22, -0.10], [1.95, 0.42, 2.10], col);
            let pod = root.child([0.0, 0.92, -0.10], -0.18, 0.0, 0.0);
            add_box(&mut s, pod, [0.0, 0.0, 0.0], [2.42, 1.18, 3.15], col_hi);
            // 3x8 rocket tube muzzle grid on front of pod!
            for row in 0..3 {
                for col_i in 0..4 {
                    let rx = -0.78 + col_i as f32 * 0.52;
                    let ry = -0.34 + row as f32 * 0.34;
                    add_box(&mut s, pod, [rx, ry, 1.56], [0.22, 0.22, 0.20], launcher_orange);
                }
            }
        }
        VehicleKind::Bmpt => {
            // Terminator: unmanned low turret with twin 30mm barrels & 4 Ataka ATGM tubes!
            add_box(&mut s, root, [0.0, 0.28, -0.15], [1.75, 0.48, 2.05], col);
            add_box(&mut s, root, [-0.18, 0.38, 1.75], [0.10, 0.10, 2.55], steel);
            add_box(&mut s, root, [0.18, 0.38, 1.75], [0.10, 0.10, 2.55], steel);
            add_box(&mut s, root, [-1.18, 0.36, -0.05], [0.44, 0.52, 1.65], launcher_orange);
            add_box(&mut s, root, [1.18, 0.36, -0.05], [0.44, 0.52, 1.65], launcher_orange);
            add_box(&mut s, root, [0.0, 0.66, 0.15], [0.46, 0.34, 0.46], steel);
        }
        VehicleKind::Pgz09 => {
            // PGZ-09 Self-Propelled Anti-Aircraft Gun: wide turret, twin 35mm barrels + search/track radars!
            add_box(&mut s, root, [0.0, 0.44, -0.15], [2.30, 0.84, 2.75], col);
            for sx in [-0.72f32, 0.72] {
                let gun = root.child([sx, 0.42, 1.15], -0.22, 0.0, 0.0);
                add_box(&mut s, gun, [0.0, 0.0, 0.0], [0.24, 0.26, 0.75], col_hi);
                add_box(&mut s, gun, [0.0, 0.0, 1.45], [0.11, 0.11, 2.85], steel);
            }
            add_box(&mut s, root, [0.0, 0.98, 0.65], [0.55, 0.42, 0.42], crate_col); // Tracking radar nose
            add_box(&mut s, root, [0.0, 1.45, -1.05], [1.95, 0.62, 0.22], crate_col); // Search radar bar
        }
        VehicleKind::Bmp3 | VehicleKind::Bmd4M | VehicleKind::Zbd04A => {
            add_box(&mut s, root, [0.0, 0.32, 0.0], [1.65, 0.62, 1.85], col);
            add_box(&mut s, root, [0.0, 0.32, 2.05], [0.14, 0.14, 2.85], col_hi); // 100mm gun
            add_box(&mut s, root, [0.24, 0.32, 1.95], [0.07, 0.07, 2.65], steel); // 30mm coax barrel
            add_box(&mut s, root, [-0.45, 0.70, 0.10], [0.32, 0.24, 0.32], steel);
            add_box(&mut s, root, [0.45, 0.70, -0.15], [0.38, 0.20, 0.38], steel);
        }
        VehicleKind::Btr82A | VehicleKind::Zbl08 => {
            add_box(&mut s, root, [0.0, 0.26, 0.0], [1.18, 0.52, 1.40], col_hi);
            add_box(&mut s, root, [0.0, 0.32, 1.55], [0.09, 0.09, 2.25], steel);
            add_box(&mut s, root, [-0.18, 0.30, 1.05], [0.06, 0.06, 1.10], steel);
            add_box(&mut s, root, [0.22, 0.45, 0.15], [0.18, 0.22, 0.35], steel);
            add_box(&mut s, root, [-0.64, 0.34, 0.20], [0.16, 0.16, 0.38], col);
            if matches!(kind, VehicleKind::Zbl08) {
                add_box(&mut s, root, [0.72, 0.42, 0.15], [0.24, 0.24, 1.25], launcher_orange); // HJ-73 rail
            } else {
                add_box(&mut s, root, [0.64, 0.34, 0.20], [0.16, 0.16, 0.38], col);
            }
        }
        VehicleKind::TigrM | VehicleKind::Csk181 => {
            // Roof armored gun shield ring + 12.7mm heavy MG / RWS.
            add_box(&mut s, root, [0.0, 0.18, 0.0], [0.95, 0.28, 0.95], col_hi);
            add_box(&mut s, root, [0.0, 0.38, 0.32], [0.85, 0.44, 0.10], col);
            add_box(&mut s, root, [-0.42, 0.38, 0.0], [0.10, 0.44, 0.68], col);
            add_box(&mut s, root, [0.42, 0.38, 0.0], [0.10, 0.44, 0.68], col);
            add_box(&mut s, root, [0.0, 0.46, 0.92], [0.08, 0.08, 1.55], steel);
        }
    }
    s
}

/// Legacy compatibility wrapper delegating to `build_vehicle_hull`.
pub fn build_tank_hull(is_mbt: bool) -> Soup {
    build_vehicle_hull(if is_mbt { VehicleKind::T72B3 } else { VehicleKind::Btr82A })
}

/// Legacy compatibility wrapper delegating to `build_vehicle_turret`.
pub fn build_tank_turret(is_mbt: bool) -> Soup {
    build_vehicle_turret(if is_mbt { VehicleKind::T72B3 } else { VehicleKind::Btr82A })
}

/// Russian `Lancet-3` cruciform X-wing loitering munition mesh (local +Z = forward).
pub fn build_lancet_soup() -> Soup {
    let mut s = Soup::default();
    let root = Frame::root();
    let gray = Rgb { r: 0.44, g: 0.46, b: 0.48 };
    let nose = Rgb { r: 0.18, g: 0.19, b: 0.21 };
    let wing_dark = Rgb { r: 0.30, g: 0.33, b: 0.36 };
    add_box(&mut s, root, [0.0, 0.0, 0.0], [0.16, 0.16, 1.55], gray);
    add_box(&mut s, root, [0.0, 0.0, 0.85], [0.12, 0.12, 0.24], nose);
    for r in [0.785f32, -0.785] {
        let front = root.child([0.0, 0.0, 0.18], 0.0, 0.0, r);
        add_box(&mut s, front, [0.0, 0.0, 0.0], [1.15, 0.04, 0.28], wing_dark);
        let rear = root.child([0.0, 0.0, -0.58], 0.0, 0.0, r);
        add_box(&mut s, rear, [0.0, 0.0, 0.0], [0.92, 0.04, 0.22], wing_dark);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_outward(soup: &Soup) {
        assert!(soup.triangles() > 40, "expected detailed articulated mesh, got {}", soup.triangles());
        for t in soup.idx.chunks_exact(3) {
            let (a, b, c) = (soup.pos[t[0] as usize], soup.pos[t[1] as usize], soup.pos[t[2] as usize]);
            let u = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
            let v = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];
            let cross = [u[1] * v[2] - u[2] * v[1], u[2] * v[0] - u[0] * v[2], u[0] * v[1] - u[1] * v[0]];
            let stored = soup.nor[t[0] as usize];
            let dot = cross[0] * stored[0] + cross[1] * stored[1] + cross[2] * stored[2];
            assert!(dot > 0.0, "inward triangle detected: dot={dot}");
        }
    }

    #[test]
    fn a_figure_has_outward_normals_and_stands_on_the_ground() {
        for role in [Role::Civilian, Role::Militia, Role::RussianSoldier] {
            let opts = FigureOpts {
                role,
                item: if role == Role::Civilian { HeldItem::None } else { HeldItem::Rifle },
                optics: role == Role::RussianSoldier,
                ..Default::default()
            };
            let standing = build_figure(&opts, &pose(AnimState::Idle, 0.0, 0.0, 0.0));
            assert_outward(&standing);
            let min_y = standing.pos.iter().map(|v| v[1]).fold(f32::MAX, f32::min);
            let max_y = standing.pos.iter().map(|v| v[1]).fold(f32::MIN, f32::max);
            assert!(min_y > -0.05 && min_y < 0.05, "{role:?} min_y={min_y}");
            assert!(max_y > 1.65 && max_y < 1.95, "{role:?} max_y={max_y}");

            let fallen = build_figure(&opts, &pose(AnimState::Dead, 0.0, 0.0, 0.0));
            assert_outward(&fallen);
            let dead_max_y = fallen.pos.iter().map(|v| v[1]).fold(f32::MIN, f32::max);
            assert!(dead_max_y < 0.65, "{role:?} dead figure should lie low, got max_y={dead_max_y}");
        }
    }

    #[test]
    fn walking_and_running_swing_the_limbs_and_aiming_raises_the_arms() {
        let idle = pose(AnimState::Idle, 0.0, 0.0, 0.0);
        let walk = pose(AnimState::Walk, 3.4, 1.4, 0.5);
        let run = pose(AnimState::Run, 6.0, 1.4, 0.5);
        let aim = pose(AnimState::Aim, 0.0, 0.0, 0.5);
        let up = pose(AnimState::HandsUp, 0.0, 0.0, 0.5);
        assert!(idle.th_l.abs() < 1.0e-5);
        assert!(walk.th_l.abs() > 0.15);
        assert!(run.sp_x > walk.sp_x);
        assert!(aim.ua_r < -0.9 && aim.ua_l < -0.8);
        assert!(up.ua_l < -1.3 && up.ua_r < -1.3);
    }

    #[test]
    fn all_pla_russian_vehicles_and_lancet_have_outward_normals() {
        for kind in VehicleKind::ALL {
            let hull = build_vehicle_hull(kind);
            let turret = build_vehicle_turret(kind);
            assert!(hull.triangles() > 50, "{kind:?} hull tris={}", hull.triangles());
            assert!(turret.triangles() > 40, "{kind:?} turret tris={}", turret.triangles());
            assert_outward(&hull);
            assert_outward(&turret);
        }
        let lancet = build_lancet_soup();
        assert_outward(&lancet);
    }
}
