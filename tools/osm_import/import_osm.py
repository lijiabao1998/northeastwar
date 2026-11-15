#!/usr/bin/env python3
# Copyright (c) 2026 李家宝. All rights reserved.
# This tool derives a database from OpenStreetMap. The output database
# stays © OpenStreetMap contributors, licensed under ODbL 1.0.
"""Reproducible OSM → Godot mesh import.

The Heihe slice is the reference region. The same entry point can take
another Overpass extract (Harbin, Changchun, Shenyang) once a region
file describes its origin and query.

    python3 tools/osm_import/import_osm.py fetch
    python3 tools/osm_import/import_osm.py build

No third-party Python packages: the standard library only.
"""

from __future__ import annotations

import argparse
import json
import math
import struct
import urllib.parse
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
REGION_DIR = Path(__file__).resolve().parent / "regions"

# Mother sculpture, Heilongjiang Park. North is +Z, east is +X, Y up.
LAT0 = 50.2510885
LON0 = 127.4960436
M_LAT = 111_320.0
M_LON = 111_320.0 * math.cos(math.radians(LAT0))

CN_VISUAL = (127.4848, 50.2470, 127.5032, 50.2565)  # west, south, east, north (lat cap is the river)
WALKABLE = (127.4856, 50.2474, 127.5024, 50.2565)
RU_PAD_M = 1100.0  # how far into Blagoveshchensk the skyline is meshed
ICE_CROSS = 0.16  # fraction of the river the player may step onto

CN_PALETTE = [
    (0.80, 0.76, 0.68),
    (0.86, 0.82, 0.74),
    (0.88, 0.74, 0.40),
    (0.64, 0.20, 0.18),
    (0.74, 0.30, 0.32),
    (0.78, 0.76, 0.72),
    (0.58, 0.22, 0.22),
    (0.72, 0.55, 0.38),
]
RU_PALETTE = [
    (0.70, 0.68, 0.64),
    (0.58, 0.60, 0.62),
    (0.76, 0.68, 0.46),
    (0.52, 0.30, 0.26),
    (0.46, 0.48, 0.50),
    (0.64, 0.50, 0.38),
]

ROAD_WIDTH = {
    "primary": 14.0,
    "secondary": 12.0,
    "tertiary": 11.0,
    "tertiary_link": 8.0,
    "residential": 9.0,
    "living_street": 6.0,
    "pedestrian": 10.0,
    "service": 4.5,
    "footway": 2.6,
    "path": 2.0,
    "unclassified": 8.0,
}


def project(lat: float, lon: float) -> tuple[float, float]:
    return ((lon - LON0) * M_LON, (lat - LAT0) * M_LAT)


def unproject(x: float, z: float) -> tuple[float, float]:
    return (LAT0 + z / M_LAT, LON0 + x / M_LON)


def hypot(x: float, z: float) -> float:
    return math.hypot(x, z)


def vsub(a, b):
    return (a[0] - b[0], a[1] - b[1])


def vadd(a, b):
    return (a[0] + b[0], a[1] + b[1])


def vmul(a, s):
    return (a[0] * s, a[1] * s)


def vnorm(a):
    length = hypot(a[0], a[1])
    if length < 1e-8:
        return (0.0, 0.0)
    return (a[0] / length, a[1] / length)


def dist2(a, b) -> float:
    return hypot(a[0] - b[0], a[1] - b[1])


def h01(i: int, salt: float = 0.0) -> float:
    x = math.sin((i + 1) * 127.1 + salt * 311.7) * 43758.5453
    return x - math.floor(x)


def ring_area(poly) -> float:
    area = 0.0
    for i, p in enumerate(poly):
        q = poly[(i + 1) % len(poly)]
        area += p[0] * q[1] - q[0] * p[1]
    return area * 0.5


def ensure_ccw(poly):
    if ring_area(poly) < 0:
        return list(reversed(poly))
    return list(poly)


def clean_ring(poly, eps=0.08):
    if not poly:
        return []
    pts = [poly[0]]
    for p in poly[1:]:
        if dist2(p, pts[-1]) >= eps:
            pts.append(p)
    if len(pts) >= 2 and dist2(pts[0], pts[-1]) < eps:
        pts.pop()
    if len(pts) < 3:
        return pts
    changed = True
    guard = 0
    while changed and guard < 8 and len(pts) >= 3:
        guard += 1
        changed = False
        out = []
        n = len(pts)
        for i in range(n):
            a = pts[(i - 1) % n]
            b = pts[i]
            c = pts[(i + 1) % n]
            cross = (b[0] - a[0]) * (c[1] - b[1]) - (b[1] - a[1]) * (c[0] - b[0])
            if abs(cross) <= 1e-3:
                changed = True
                continue
            out.append(b)
        pts = out
    return pts


def point_in_poly(p, poly) -> bool:
    x, z = p
    inside = False
    n = len(poly)
    for i in range(n):
        x1, z1 = poly[i]
        x2, z2 = poly[(i + 1) % n]
        if (z1 > z) != (z2 > z):
            xin = x1 + (x2 - x1) * (z - z1) / (z2 - z1 + 1e-15)
            if x < xin:
                inside = not inside
    return inside


def _tri_area(a, b, c) -> float:
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])


def _in_tri(p, a, b, c) -> bool:
    a1 = _tri_area(a, b, p)
    a2 = _tri_area(b, c, p)
    a3 = _tri_area(c, a, p)
    return (a1 >= -1e-6 and a2 >= -1e-6 and a3 >= -1e-6) or (a1 <= 1e-6 and a2 <= 1e-6 and a3 <= 1e-6)


def triangulate(poly):
    """Ear clipping. Returns index triples or None."""
    pts = clean_ring(poly)
    pts = ensure_ccw(pts)
    n0 = len(pts)
    if n0 < 3:
        return None, pts
    if n0 == 3:
        return [(0, 1, 2)], pts
    idx = list(range(n0))
    tris = []
    guard = 0
    while len(idx) > 3 and guard < n0 * n0:
        guard += 1
        n = len(idx)
        ear = False
        for i in range(n):
            i0 = idx[(i - 1) % n]
            i1 = idx[i]
            i2 = idx[(i + 1) % n]
            a, b, c = pts[i0], pts[i1], pts[i2]
            if _tri_area(a, b, c) <= 1e-6:
                continue
            if any(
                _in_tri(pts[idx[j]], a, b, c)
                for j in range(n)
                if idx[j] not in (i0, i1, i2)
            ):
                continue
            tris.append((i0, i1, i2))
            del idx[i]
            ear = True
            break
        if not ear:
            return None, pts
    if len(idx) == 3:
        tris.append((idx[0], idx[1], idx[2]))
    return tris, pts


def clip_polygon(poly, bounds):
    """Sutherland–Hodgman clip to (minx, minz, maxx, maxz)."""
    minx, minz, maxx, maxz = bounds

    def clip(subject, inside, intersect):
        if not subject:
            return []
        out = []
        prev = subject[-1]
        prev_in = inside(prev)
        for cur in subject:
            cur_in = inside(cur)
            if cur_in:
                if not prev_in:
                    out.append(intersect(prev, cur))
                out.append(cur)
            elif prev_in:
                out.append(intersect(prev, cur))
            prev = cur
            prev_in = cur_in
        return out

    def hit_x(a, b, x):
        t = (x - a[0]) / (b[0] - a[0] + 1e-15)
        return (x, a[1] + t * (b[1] - a[1]))

    def hit_z(a, b, z):
        t = (z - a[1]) / (b[1] - a[1] + 1e-15)
        return (a[0] + t * (b[0] - a[0]), z)

    poly = clip(poly, lambda p: p[0] >= minx, lambda a, b: hit_x(a, b, minx))
    poly = clip(poly, lambda p: p[0] <= maxx, lambda a, b: hit_x(a, b, maxx))
    poly = clip(poly, lambda p: p[1] >= minz, lambda a, b: hit_z(a, b, minz))
    poly = clip(poly, lambda p: p[1] <= maxz, lambda a, b: hit_z(a, b, maxz))
    return clean_ring(poly, eps=0.4)


def vertical_spans(poly, x: float):
    hits = []
    for i, a in enumerate(poly):
        b = poly[(i + 1) % len(poly)]
        if (a[0] - x) * (b[0] - x) <= 0 and abs(a[0] - b[0]) > 1e-6:
            t = (x - a[0]) / (b[0] - a[0])
            hits.append(a[1] + t * (b[1] - a[1]))
    hits.sort()
    return hits


def bank_sampler(poly, step=8.0):
    if len(poly) < 3:
        return lambda _x: None
    xs = [p[0] for p in poly]
    minx, maxx = min(xs), max(xs)
    samples = []
    x = minx
    while x <= maxx:
        hits = vertical_spans(poly, x)
        if len(hits) >= 2:
            samples.append((x, hits[0], hits[-1]))
        x += step
    if not samples:
        return lambda _x: None

    def at(xq):
        if xq <= samples[0][0]:
            return samples[0][1], samples[0][2]
        if xq >= samples[-1][0]:
            return samples[-1][1], samples[-1][2]
        for i in range(len(samples) - 1):
            x0, s0, n0 = samples[i]
            x1, s1, n1 = samples[i + 1]
            if x0 <= xq <= x1:
                t = (xq - x0) / (x1 - x0 + 1e-9)
                return (s0 + (s1 - s0) * t, n0 + (n1 - n0) * t)
        return samples[-1][1], samples[-1][2]

    at.samples = samples  # type: ignore[attr-defined]
    return at


def parse_levels(tags, osm_id: int, area: float, side: str):
    if "building:levels" in tags:
        try:
            levels = float(str(tags["building:levels"]).split(";")[0])
            if levels > 0:
                return max(1, min(40, int(round(levels)))), "osm_levels"
        except ValueError:
            pass
    if "height" in tags:
        raw = str(tags["height"]).lower().replace("m", "").strip()
        try:
            meters = float(raw.split(";")[0])
            if meters > 0:
                return max(1, min(40, int(round(meters / 3.0)))), "osm_height"
        except ValueError:
            pass
    kind = tags.get("building", "yes")
    if kind in ("house", "detached", "garage", "garages", "shed", "cabin", "kiosk"):
        return (1 if area < 90 else 2), "estimated_from_footprint"
    if kind in ("industrial", "warehouse", "service"):
        return 2, "estimated_from_footprint"
    if side == "ru":
        if area < 120:
            return 1 + int(h01(osm_id, 1) * 2), "estimated_from_footprint"
        if area < 450:
            return 5, "estimated_from_footprint"
        if area < 1100:
            return 9, "estimated_from_footprint"
        return 12, "estimated_from_footprint"
    # Chinese core: mostly 6–8 storey tiled slabs, a few mid-rises. OSM has
    # almost no building:levels here (checked November 2026), so this is estimated.
    if area < 70:
        return 1, "estimated_from_footprint"
    if area < 180:
        return 2 + int(h01(osm_id, 2) * 2), "estimated_from_footprint"
    if area < 420:
        return 5 + int(h01(osm_id, 3) * 3), "estimated_from_footprint"
    levels = 6 + int(h01(osm_id, 4) * 3)  # 6..8
    if h01(osm_id, 5) > 0.86 and area > 500:
        levels = 11 + int(h01(osm_id, 6) * 5)  # occasional tower
    return levels, "estimated_from_footprint"


def palette_for(osm_id: int, side: str, levels: int):
    if side == "ru":
        return RU_PALETTE[int(h01(osm_id, 8) * len(RU_PALETTE)) % len(RU_PALETTE)]
    # Taller slabs cycle the red / yellow / beige ceramic colours seen on
    # Heihe streets; low houses stay cream and off-white.
    if levels <= 2:
        choices = [CN_PALETTE[0], CN_PALETTE[1], CN_PALETTE[5], CN_PALETTE[7]]
    else:
        choices = CN_PALETTE
    return choices[int(h01(osm_id, 9) * len(choices)) % len(choices)]


def is_cyrillic(text: str) -> bool:
    return any("\u0400" <= ch <= "\u04ff" for ch in text)


def names_of(tags: dict) -> tuple[str, str]:
    zh = tags.get("name:zh") or ""
    ru = tags.get("name:ru") or ""
    name = tags.get("name") or ""
    if not zh and name and not is_cyrillic(name):
        zh = name
    if not ru and name and is_cyrillic(name):
        ru = name
    if not zh and not ru:
        zh = name
    return zh.strip(), ru.strip()


def short_zh(text: str) -> str:
    out = []
    depth = 0
    for ch in text:
        if ch in "（(":
            depth += 1
            continue
        if ch in "）)":
            depth = max(0, depth - 1)
            continue
        if depth == 0:
            out.append(ch)
    return "".join(out).strip()


class Mesh:
    def __init__(self):
        self.data: list[float] = []

    def tri(self, p0, p1, p2, uv0, uv1, uv2, uvb0, uvb1, uvb2, c0, c1, c2):
        ax, ay, az = p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]
        bx, by, bz = p2[0] - p0[0], p2[1] - p0[1], p2[2] - p0[2]
        nx, ny, nz = ay * bz - az * by, az * bx - ax * bz, ax * by - ay * bx
        length = math.sqrt(nx * nx + ny * ny + nz * nz)
        if length < 1e-8:
            return
        nx, ny, nz = nx / length, ny / length, nz / length
        for p, uv, uvb, c in (
            (p0, uv0, uvb0, c0),
            (p1, uv1, uvb1, c1),
            (p2, uv2, uvb2, c2),
        ):
            self.data.extend(
                (
                    p[0], p[1], p[2],
                    nx, ny, nz,
                    uv[0], uv[1],
                    uvb[0], uvb[1],
                    c[0], c[1], c[2], c[3],
                )
            )

    def quad(self, pts, uvs, uvbs, cols):
        self.tri(pts[0], pts[1], pts[2], uvs[0], uvs[1], uvs[2], uvbs[0], uvbs[1], uvbs[2], cols[0], cols[1], cols[2])
        self.tri(pts[0], pts[2], pts[3], uvs[0], uvs[2], uvs[3], uvbs[0], uvbs[2], uvbs[3], cols[0], cols[2], cols[3])

    @property
    def vertices(self) -> int:
        return len(self.data) // 14


def col(rgb, a=1.0):
    return (rgb[0], rgb[1], rgb[2], a)


def add_wall(mesh: Mesh, a, b, y0, y1, rgb, seed, shop, uv_solid=False):
    dx, dz = b[0] - a[0], b[1] - a[1]
    length = hypot(dx, dz)
    if length < 0.15 or y1 <= y0 + 0.05:
        return
    p0 = (a[0], y0, a[1])
    p1 = (b[0], y0, b[1])
    p2 = (b[0], y1, b[1])
    p3 = (a[0], y1, a[1])
    if uv_solid:
        u0, u1, v0, v1 = 0.0, length, -1.0, -1.0
    else:
        u0, u1, v0, v1 = 0.0, length, y0, y1
    uvb = (seed, 1.0 if shop else 0.0)
    c = col(rgb)
    # Outward for a CCW footprint is (dz, -dx). Winding below faces that way.
    mesh.quad(
        [p0, p3, p2, p1],
        [(u0, v0), (u0, v1), (u1, v1), (u1, v0)],
        [uvb, uvb, uvb, uvb],
        [c, c, c, c],
    )


def add_roof(mesh: Mesh, poly, y, rgb, snow=False):
    tris, pts = triangulate(poly)
    if not tris:
        return False
    c = col(rgb)
    for i0, i1, i2 in tris:
        p = []
        uvs = []
        for i in (i0, i1, i2):
            x, z = pts[i]
            yy = y
            if snow:
                n = h01(int(x * 10) ^ int(z * 10), 2.2)
                yy = y + 0.04 + n * 0.16
            p.append((x, yy, z))
            uvs.append((x * 0.15, z * 0.15))
        uvb = (0.2, 0.0)
        mesh.tri(p[0], p[1], p[2], uvs[0], uvs[1], uvs[2], uvb, uvb, uvb, c, c, c)
    return True


def add_box(mesh: Mesh, center, size, rgb, uv_solid=True):
    cx, cy, cz = center
    hx, hy, hz = size[0] * 0.5, size[1] * 0.5, size[2] * 0.5
    faces = [
        # +x, -x, +y, -y, +z, -z
        [(1, -1, -1), (1, -1, 1), (1, 1, 1), (1, 1, -1)],
        [(-1, -1, 1), (-1, -1, -1), (-1, 1, -1), (-1, 1, 1)],
        [(-1, 1, -1), (1, 1, -1), (1, 1, 1), (-1, 1, 1)],
        [(-1, -1, 1), (1, -1, 1), (1, -1, -1), (-1, -1, -1)],
        [(-1, -1, 1), (-1, 1, 1), (1, 1, 1), (1, -1, 1)],
        [(1, -1, -1), (1, 1, -1), (-1, 1, -1), (-1, -1, -1)],
    ]
    c = col(rgb)
    uvb = (0.5, 0.0)
    for face in faces:
        pts = []
        for sx, sy, sz in face:
            pts.append((cx + sx * hx, cy + sy * hy, cz + sz * hz))
        if uv_solid:
            uvs = [(0, -1), (0, -1), (1, -1), (1, -1)]
        else:
            uvs = [(0, 0), (0, 1), (1, 1), (1, 0)]
        mesh.quad(pts, uvs, [uvb, uvb, uvb, uvb], [c, c, c, c])


def add_balcony(glass: Mesh, snow: Mesh, a, b, y0, y1, depth, seed):
    dx, dz = b[0] - a[0], b[1] - a[1]
    length = hypot(dx, dz)
    if length < 2.0:
        return
    nx, nz = dz / length, -dx / length
    a2 = (a[0] + nx * depth, a[1] + nz * depth)
    b2 = (b[0] + nx * depth, b[1] + nz * depth)
    c = col((0.75, 0.82, 0.88))
    uvb = (seed, 0.0)

    def glass_face(p, q):
        span = hypot(q[0] - p[0], q[1] - p[1])
        p0 = (p[0], y0, p[1])
        p1 = (q[0], y0, q[1])
        p2 = (q[0], y1, q[1])
        p3 = (p[0], y1, p[1])
        glass.quad(
            [p0, p3, p2, p1],
            [(0, 0), (0, y1 - y0), (span, y1 - y0), (span, 0)],
            [uvb, uvb, uvb, uvb],
            [c, c, c, c],
        )

    glass_face(a2, b2)
    glass_face(a, a2)
    glass_face(b2, b)
    # snow cap
    snow_y = y1
    cap = col((0.93, 0.94, 0.96))
    s0 = (a[0], snow_y, a[1])
    s1 = (b[0], snow_y, b[1])
    s2 = (b2[0], snow_y + 0.08, b2[1])
    s3 = (a2[0], snow_y + 0.08, a2[1])
    snow.quad([s0, s1, s2, s3], [(0, 0), (1, 0), (1, 1), (0, 1)], [uvb, uvb, uvb, uvb], [cap, cap, cap, cap])


def add_parapet(mesh: Mesh, poly, y, height, rgb):
    n = len(poly)
    for i in range(n):
        a = poly[i]
        b = poly[(i + 1) % n]
        add_wall(mesh, a, b, y, y + height, rgb, 0.4, False, uv_solid=True)


def add_mound(mesh: Mesh, x, z, rx, ry, rz, y0, rgb):
    stacks, slices = 3, 6
    c = col(rgb)

    def pt(i, j):
        phi = (i / stacks) * math.pi * 0.5
        theta = (j / slices) * math.pi * 2.0
        return (
            x + math.cos(theta) * math.cos(phi) * rx,
            y0 + math.sin(phi) * ry,
            z + math.sin(theta) * math.cos(phi) * rz,
        )

    uvb = (0.3, 0.0)
    for i in range(stacks):
        for j in range(slices):
            p00, p10 = pt(i, j), pt(i + 1, j)
            p11, p01 = pt(i + 1, j + 1), pt(i, j + 1)
            mesh.quad([p00, p01, p11, p10], [(0, 0), (1, 0), (1, 1), (0, 1)], [uvb] * 4, [c] * 4)


def add_car(body: Mesh, glass: Mesh, x, z, yaw, rgb):
    # yaw is the heading of the car's forward axis in the x/z plane.
    fwd = (math.sin(yaw), math.cos(yaw))
    right = (fwd[1], -fwd[0])

    def world(lx, ly, lz):
        return (x + right[0] * lx + fwd[0] * lz, ly, z + right[1] * lx + fwd[1] * lz)

    def box(mesh, l, y, w, h, d, rgb_):
        # axis-aligned in car space, then rotated via corners
        hx, hy, hz = w * 0.5, h * 0.5, d * 0.5
        cy = y
        corners = []
        signs = [(-1, -1, -1), (1, -1, -1), (1, -1, 1), (-1, -1, 1), (-1, 1, -1), (1, 1, -1), (1, 1, 1), (-1, 1, 1)]
        for sx, sy, sz in signs:
            corners.append(world(l + sx * hx, cy + sy * hy, sz * hz))
        faces = [
            (0, 1, 5, 4),
            (3, 2, 6, 7),
            (4, 5, 6, 7),
            (0, 3, 2, 1),
            (1, 2, 6, 5),
            (0, 4, 7, 3),
        ]
        c = col(rgb_)
        uvb = (0.0, 0.0)
        for f in faces:
            pts = [corners[i] for i in f]
            mesh.quad(pts, [(0, 0), (1, 0), (1, 1), (0, 1)], [uvb] * 4, [c] * 4)

    # length along forward (z in car space). Parked cars are ~4.4 m long.
    box(body, 0.0, 0.55, 1.75, 0.7, 4.3, rgb)
    box(glass, 0.0, 1.15, 1.55, 0.62, 2.1, (0.05, 0.07, 0.09))
    box(body, 0.0, 0.28, 1.85, 0.18, 4.4, tuple(max(0.0, c * 0.4) for c in rgb))


def ribbon(points, half_w):
    n = len(points)
    if n < 2:
        return [], []
    dirs = [vnorm(vsub(points[i + 1], points[i])) for i in range(n - 1)]
    left, right = [], []
    for i in range(n):
        if i == 0:
            d = dirs[0]
            s = (-d[1], d[0])
            scale = 1.0
        elif i == n - 1:
            d = dirs[-1]
            s = (-d[1], d[0])
            scale = 1.0
        else:
            d0, d1 = dirs[i - 1], dirs[i]
            s0 = (-d0[1], d0[0])
            s1 = (-d1[1], d1[0])
            s = vnorm(vadd(s0, s1))
            denom = s[0] * s0[0] + s[1] * s0[1]
            if denom > 0.4:
                scale = min(1.8, 1.0 / denom)
            else:
                s = s0
                scale = 1.0
        off = vmul(s, half_w * scale)
        left.append(vadd(points[i], off))
        right.append(vsub(points[i], off))
    return left, right


def add_ribbon(mesh: Mesh, points, width, y, rgb, along_uv=True):
    left, right = ribbon(points, width * 0.5)
    if len(left) < 2:
        return
    c = col(rgb)
    uvb = (0.0, 0.0)
    acc = 0.0
    for i in range(len(points) - 1):
        span = dist2(points[i], points[i + 1])
        u0, u1 = (acc, acc + span) if along_uv else (0.0, span)
        acc += span
        pts = [
            (left[i][0], y, left[i][1]),
            (right[i][0], y, right[i][1]),
            (right[i + 1][0], y, right[i + 1][1]),
            (left[i + 1][0], y, left[i + 1][1]),
        ]
        # UV.x is across the road (0 left .. 1 right). UV.y runs along it.
        uvs = [(0, u0), (1, u0), (1, u1), (0, u1)]
        mesh.quad(pts, uvs, [uvb] * 4, [c] * 4)


def outward_normal(a, b):
    dx, dz = b[0] - a[0], b[1] - a[1]
    length = hypot(dx, dz) or 1.0
    return (dz / length, -dx / length), length


def nearest_edge(footprint, roads):
    best = None
    best_d = 1e18
    n = len(footprint)
    for i in range(n):
        a = footprint[i]
        b = footprint[(i + 1) % n]
        normal, length = outward_normal(a, b)
        if length < 4.0:
            continue
        mid = ((a[0] + b[0]) * 0.5, (a[1] + b[1]) * 0.5)
        d = min(dist_to_polyline(mid, road["points"]) for road in roads) if roads else 0.0
        # Prefer edges that face a mapped street.
        score = d - length * 0.02
        if score < best_d:
            best_d = score
            best = (a, b, normal, length, mid, d)
    return best


def dist_to_polyline(p, pts) -> float:
    best = 1e18
    for i in range(len(pts) - 1):
        a, b = pts[i], pts[i + 1]
        ab = vsub(b, a)
        lab = ab[0] * ab[0] + ab[1] * ab[1]
        t = 0.0 if lab < 1e-8 else max(0.0, min(1.0, ((p[0] - a[0]) * ab[0] + (p[1] - a[1]) * ab[1]) / lab))
        q = (a[0] + ab[0] * t, a[1] + ab[1] * t)
        best = min(best, dist2(p, q))
    return best


def stitch_rings(ways):
    """Join OSM way fragments that share endpoints into closed rings."""
    pool = []
    for way in ways:
        if len(way) >= 2:
            pool.append(list(way))
    used = [False] * len(pool)
    rings = []
    for i in range(len(pool)):
        if used[i]:
            continue
        ring = pool[i][:]
        used[i] = True
        progressed = True
        while progressed:
            progressed = False
            for j in range(len(pool)):
                if used[j]:
                    continue
                w = pool[j]
                if dist2(ring[-1], w[0]) < 0.4:
                    ring.extend(w[1:])
                elif dist2(ring[-1], w[-1]) < 0.4:
                    ring.extend(reversed(w[:-1]))
                elif dist2(ring[0], w[-1]) < 0.4:
                    ring = w[:-1] + ring
                elif dist2(ring[0], w[0]) < 0.4:
                    ring = list(reversed(w[1:])) + ring
                else:
                    continue
                used[j] = True
                progressed = True
        rings.append(ring)
    return rings


def way_points_latlon(el):
    geom = el.get("geometry") or []
    return [(p["lat"], p["lon"]) for p in geom]


def load_overpass(path: Path):
    with path.open(encoding="utf-8") as fh:
        data = json.load(fh)
    return data


def assemble_water(elements):
    relation = next((e for e in elements if e["type"] == "relation" and e.get("tags", {}).get("natural") == "water"), None)
    outers, inners = [], []
    if relation:
        for member in relation.get("members", []):
            geom = member.get("geometry") or []
            pts = [project(p["lat"], p["lon"]) for p in geom]
            if len(pts) < 2:
                continue
            if member.get("role") == "inner":
                inners.append(pts)
            else:
                outers.append(pts)
    outer_rings = stitch_rings(outers)
    inner_rings = stitch_rings(inners)
    # Clip to a generous window around the slice so the Amur mesh stays local.
    bounds = (
        project(50.24, 127.47)[0],
        project(50.24, 127.47)[1],
        project(50.28, 127.53)[0],
        project(50.28, 127.53)[1],
    )
    # project() of a south-west corner is not axis-aligned in the way I just
    # wrote: x depends only on lon, z only on lat, so compute explicitly.
    bounds = (
        project(0, 127.472)[0],
        project(50.242, 0)[1],
        project(0, 127.528)[0],
        project(50.278, 0)[1],
    )
    clipped = []
    for ring in outer_rings:
        poly = clip_polygon(ring, bounds)
        if len(poly) >= 3 and abs(ring_area(poly)) > 500:
            clipped.append(ensure_ccw(poly))
    islands = []
    for ring in inner_rings:
        poly = clip_polygon(ring, bounds)
        if len(poly) >= 3 and abs(ring_area(poly)) > 400:
            islands.append(ensure_ccw(poly))
    # Standalone ponds.
    ponds = []
    for el in elements:
        tags = el.get("tags", {})
        if el["type"] == "way" and tags.get("natural") == "water":
            pts = [project(p["lat"], p["lon"]) for p in el.get("geometry") or []]
            pts = clean_ring(pts, 0.2)
            if len(pts) >= 3 and abs(ring_area(pts)) > 80:
                ponds.append(ensure_ccw(pts))
    return clipped, islands, ponds


def collect_buildings(elements):
    buildings = []
    for el in elements:
        if el["type"] != "way" or "building" not in el.get("tags", {}):
            continue
        tags = el["tags"]
        if tags.get("building") in ("roof", "no"):
            continue
        pts = [project(p["lat"], p["lon"]) for p in el.get("geometry") or []]
        pts = clean_ring(pts, 0.12)
        if len(pts) < 3:
            continue
        pts = ensure_ccw(pts)
        area = abs(ring_area(pts))
        if area < 16 or area > 40000:
            continue
        cx = sum(p[0] for p in pts) / len(pts)
        cz = sum(p[1] for p in pts) / len(pts)
        buildings.append(
            {
                "id": el["id"],
                "tags": tags,
                "footprint": pts,
                "area": area,
                "centroid": (cx, cz),
            }
        )
    return buildings


def collect_roads(elements):
    roads = []
    for el in elements:
        tags = el.get("tags", {})
        if el["type"] != "way" or "highway" not in tags:
            continue
        kind = tags["highway"]
        if kind not in ROAD_WIDTH:
            continue
        pts = [project(p["lat"], p["lon"]) for p in el.get("geometry") or []]
        cleaned = []
        for p in pts:
            if not cleaned or dist2(cleaned[-1], p) > 0.6:
                cleaned.append(p)
        if len(cleaned) < 2:
            continue
        zh, ru = names_of(tags)
        roads.append(
            {
                "id": el["id"],
                "highway": kind,
                "name": zh or tags.get("name") or "",
                "name_ru": ru,
                "width": ROAD_WIDTH[kind],
                "points": cleaned,
            }
        )
    return roads


def collect_pois(elements):
    pois = []
    for el in elements:
        tags = el.get("tags", {})
        interesting = any(k in tags for k in ("shop", "amenity", "tourism", "historic"))
        if not interesting:
            continue
        if el["type"] == "node" and "lat" in el:
            x, z = project(el["lat"], el["lon"])
        elif el["type"] == "way" and el.get("geometry"):
            pts = [project(p["lat"], p["lon"]) for p in el["geometry"]]
            if not pts:
                continue
            x = sum(p[0] for p in pts) / len(pts)
            z = sum(p[1] for p in pts) / len(pts)
        else:
            continue
        zh, ru = names_of(tags)
        if not zh and not ru:
            continue
        pois.append(
            {
                "id": el["id"],
                "x": x,
                "z": z,
                "name_zh": zh,
                "name_ru": ru,
                "shop": tags.get("shop"),
                "amenity": tags.get("amenity"),
                "tourism": tags.get("tourism"),
                "historic": tags.get("historic"),
            }
        )
    return pois


def split_runs(points, ok):
    runs, cur = [], []
    for p in points:
        if ok(p):
            cur.append(p)
        elif len(cur) >= 2:
            runs.append(cur)
            cur = []
        else:
            cur = []
    if len(cur) >= 2:
        runs.append(cur)
    return runs


def write_gltf(path: Path, meshes: dict[str, Mesh]):
    """Write glTF 2.0 with one buffer view per attribute so Godot imports cleanly."""
    blob = bytearray()
    buffer_views = []
    accessors = []
    materials = []
    nodes = []
    mesh_defs = []

    def push_floats(values):
        while len(blob) % 4:
            blob.append(0)
        start_b = len(blob)
        blob.extend(struct.pack("<%df" % len(values), *values))
        return start_b, len(blob) - start_b

    def deinterleave(data, count, offset, width):
        out = []
        for i in range(count):
            base = i * 14 + offset
            out.extend(data[base : base + width])
        return out

    kept = [(name, mesh) for name, mesh in meshes.items() if mesh.vertices >= 3]
    for name, mesh in kept:
        count = mesh.vertices
        layout = [
            ("POSITION", "VEC3", 0, 3, True),
            ("NORMAL", "VEC3", 3, 3, False),
            ("TEXCOORD_0", "VEC2", 6, 2, False),
            ("TEXCOORD_1", "VEC2", 8, 2, False),
            ("COLOR_0", "VEC4", 10, 4, False),
        ]
        attributes = {}
        for attr, typ, offset, width, with_bounds in layout:
            values = deinterleave(mesh.data, count, offset, width)
            start_b, length = push_floats(values)
            buffer_views.append({"buffer": 0, "byteOffset": start_b, "byteLength": length, "target": 34962})
            acc = {"bufferView": len(buffer_views) - 1, "componentType": 5126, "count": count, "type": typ}
            if with_bounds:
                acc["min"] = [min(values[i::width]) for i in range(width)]
                acc["max"] = [max(values[i::width]) for i in range(width)]
            attributes[attr] = len(accessors)
            accessors.append(acc)
        materials.append({
            "name": name,
            "doubleSided": True,
            "pbrMetallicRoughness": {"baseColorFactor": [1, 1, 1, 1], "metallicFactor": 0.0, "roughnessFactor": 0.85},
            "extras": {"shader": name},
        })
        mesh_defs.append({"name": name, "primitives": [{"attributes": attributes, "material": len(materials) - 1, "mode": 4}]})
        nodes.append({"name": name, "mesh": len(mesh_defs) - 1})

    gltf = {
        "asset": {"version": "2.0", "generator": "northeastwar osm_import"},
        "scene": 0,
        "scenes": [{"name": "Heihe", "nodes": list(range(len(nodes)))}],
        "nodes": nodes,
        "meshes": mesh_defs,
        "materials": materials,
        "accessors": accessors,
        "bufferViews": buffer_views,
        "buffers": [{"uri": path.with_suffix(".bin").name, "byteLength": len(blob)}],
    }
    path.write_text(json.dumps(gltf, ensure_ascii=False), encoding="utf-8")
    path.with_suffix(".bin").write_bytes(blob)
    return len(blob), [name for name, _ in kept]


def build(overpass_path: Path, out_dir: Path):
    raw = load_overpass(overpass_path)
    elements = raw["elements"]
    timestamp = raw.get("osm3s", {}).get("timestamp_osm_base", "")
    water_polys, islands, ponds = assemble_water(elements)
    if not water_polys:
        raise SystemExit("No river polygon survived clipping. Refusing to build an empty bank.")
    main_water = max(water_polys, key=lambda p: abs(ring_area(p)))
    banks = bank_sampler(main_water, step=10.0)

    west, south, east, _north = CN_VISUAL
    vis_minx, vis_minz = project(south, west)
    vis_maxx, _ = project(south, east)
    # north edge of the Chinese visual set is the river, not a latitude.

    def span_at(x):
        return banks(x)

    buildings = collect_buildings(elements)
    roads_all = collect_roads(elements)
    pois = collect_pois(elements)

    meshes = {name: Mesh() for name in (
        "Facade", "European", "Glass", "Roof", "Snow", "Road", "Sidewalk",
        "Ice", "Promenade", "Ground", "Car", "CarGlass", "Railing", "Prop",
    )}

    # Ground first, large enough to sit under the slice and the near skyline.
    sample_n = banks(0.0)
    if sample_n is None:
        raise SystemExit("River bank sampler failed at the Mother sculpture meridian.")
    ground_minx, ground_maxx = vis_minx - 40, vis_maxx + 80
    ground_minz, ground_maxz = vis_minz - 40, sample_n[1] + RU_PAD_M
    add_box_flat = meshes["Ground"]
    gcol = col((0.86, 0.88, 0.90))
    add_box_flat.quad(
        [
            (ground_minx, 0.0, ground_minz),
            (ground_maxx, 0.0, ground_minz),
            (ground_maxx, 0.0, ground_maxz),
            (ground_minx, 0.0, ground_maxz),
        ],
        [(0, 0), (1, 0), (1, 1), (0, 1)],
        [(0, 0)] * 4,
        [gcol] * 4,
    )

    for poly in water_polys:
        add_roof(meshes["Ice"], poly, 0.04, (0.78, 0.84, 0.88), snow=False)
    for pond in ponds:
        c = pond[0]
        sp = span_at(c[0])
        # Ponds on land only; the main channel is already meshed.
        if sp and sp[0] - 5 <= c[1] <= sp[1] + 5:
            continue
        add_roof(meshes["Ice"], pond, 0.045, (0.80, 0.86, 0.90), snow=False)
    for island in islands:
        add_roof(meshes["Snow"], island, 0.22, (0.93, 0.94, 0.96), snow=True)

    # Promenade strip and railing along the Chinese bank.
    samples = banks.samples  # type: ignore[attr-defined]
    rail_rgb = (0.12, 0.13, 0.14)
    prom_rgb = (0.45, 0.38, 0.36)
    for i in range(len(samples) - 1):
        x0, s0, _n0 = samples[i]
        x1, s1, _n1 = samples[i + 1]
        if x1 < vis_minx - 20 or x0 > vis_maxx + 20:
            continue
        depth = 22.0
        p0 = (x0, 0.05, s0)
        p1 = (x1, 0.05, s1)
        p2 = (x1, 0.05, s1 - depth)
        p3 = (x0, 0.05, s0 - depth)
        c = col(prom_rgb)
        meshes["Promenade"].quad([p0, p1, p2, p3], [(0, 0), (1, 0), (1, 1), (0, 1)], [(0, 0)] * 4, [c] * 4)
        # Railing posts + two rails on the water edge.
        span = max(1.0, x1 - x0)
        posts = max(1, int(span / 4.0))
        for k in range(posts):
            t = k / posts
            x = x0 + (x1 - x0) * t
            z = s0 + (s1 - s0) * t
            add_box(meshes["Railing"], (x, 0.55, z), (0.08, 1.05, 0.08), rail_rgb)
        for rail_y in (0.45, 0.95):
            a = (x0, s0)
            b = (x1, s1)
            add_wall(meshes["Railing"], a, b, rail_y, rail_y + 0.04, rail_rgb, 0.0, False, uv_solid=True)

    skipped = {"tri": 0, "water": 0, "side": 0}
    height_sources = {}
    records = []
    cn_footprints = []

    for bld in buildings:
        cx, cz = bld["centroid"]
        sp = span_at(cx)
        if sp is None:
            skipped["side"] += 1
            continue
        south_z, north_z = sp
        if south_z + 1.5 < cz < north_z - 1.5:
            skipped["water"] += 1
            continue
        if cz <= south_z + 1.5:
            side = "cn"
            if not (vis_minx <= cx <= vis_maxx and vis_minz - 30 <= cz <= south_z + 2):
                skipped["side"] += 1
                continue
        else:
            side = "ru"
            if cz > north_z + RU_PAD_M or not (vis_minx - 80 <= cx <= vis_maxx + 160):
                skipped["side"] += 1
                continue
        levels, source = parse_levels(bld["tags"], bld["id"], bld["area"], side)
        style = "slab" if levels >= 4 else "low"
        floor_h = 3.0
        # The pink arched memorial on 王肃街. OSM has the footprint and the
        # museum node, but no levels; the licensed street photo shows two floors.
        euro = False
        for poi in pois:
            if poi["name_zh"] == "旅俄华侨纪念馆" and point_in_poly((poi["x"], poi["z"]), bld["footprint"]):
                euro = True
                levels = 2
                floor_h = 4.05
                source = "poi_plus_reference_photo"
                style = "euro_pink"
                break
        height = levels * floor_h
        rgb = (0.86, 0.18, 0.42) if euro else palette_for(bld["id"], side, levels)
        seed = h01(bld["id"], 3.3)
        zh, ru = names_of(bld["tags"])
        shop = bool(bld["tags"].get("shop") or bld["tags"].get("amenity") in {
            "restaurant", "cafe", "bank", "pharmacy", "fast_food", "bar", "clinic", "post_office"
        })
        detail = side == "cn"
        wall_mesh = meshes["European"] if euro else meshes["Facade"]
        ok = add_roof(meshes["Roof"], bld["footprint"], height, (0.25, 0.25, 0.27))
        if not ok:
            skipped["tri"] += 1
            continue
        add_roof(meshes["Snow"], bld["footprint"], height + 0.02, (0.94, 0.95, 0.96), snow=True)
        for i, a in enumerate(bld["footprint"]):
            b = bld["footprint"][(i + 1) % len(bld["footprint"])]
            add_wall(wall_mesh, a, b, 0.0, height, rgb, seed, shop and detail)
        if detail and levels >= 4 and not euro:
            add_parapet(wall_mesh, bld["footprint"], height, 0.7, rgb)
            # Stair bulkhead, a small box on the roof. Very common on these slabs.
            tris, pts = triangulate(bld["footprint"])
            if tris:
                i0, i1, i2 = tris[0]
                bx = (pts[i0][0] + pts[i1][0] + pts[i2][0]) / 3.0
                bz = (pts[i0][1] + pts[i1][1] + pts[i2][1]) / 3.0
                add_box(wall_mesh, (bx, height + 1.3, bz), (3.4, 2.6, 2.8), rgb)
            # Enclosed balconies on the long faces. This is the northeast silhouette:
            # white-framed glass bands, not open railings.
            edges = []
            for i, a in enumerate(bld["footprint"]):
                b = bld["footprint"][(i + 1) % len(bld["footprint"])]
                edges.append((dist2(a, b), a, b))
            edges.sort(reverse=True)
            for length, a, b in edges[:2]:
                if length < 12.0:
                    continue
                for floor in range(1, levels - 1):
                    y0 = floor * floor_h + 0.35
                    y1 = floor * floor_h + 2.7
                    add_balcony(meshes["Glass"], meshes["Snow"], a, b, y0, y1, 0.72, seed)
        elif detail and not euro:
            add_parapet(wall_mesh, bld["footprint"], height, 0.35, rgb)
        height_sources[source] = height_sources.get(source, 0) + 1
        record = {
            "id": bld["id"],
            "side": side,
            "style": style,
            "levels": levels,
            "height": round(height, 2),
            "floor_h": floor_h,
            "height_source": source,
            "name_zh": zh,
            "name_ru": ru,
            "building": bld["tags"].get("building"),
            "shop": shop,
            "centroid": [round(cx, 2), round(cz, 2)],
            "collision": side == "cn",
        }
        if side == "cn":
            record["footprint"] = [[round(p[0], 2), round(p[1], 2)] for p in bld["footprint"]]
            cn_footprints.append(bld["footprint"])
        records.append(record)

    def on_land_road(p):
        sp = span_at(p[0])
        if sp is None:
            return False
        if p[1] > sp[0] - 2.0:
            return False
        return vis_minx - 10 <= p[0] <= vis_maxx + 10 and vis_minz - 20 <= p[1]

    road_runs = []
    for road in roads_all:
        for run in split_runs(road["points"], on_land_road):
            if len(run) < 2:
                continue
            item = dict(road)
            item["points"] = run
            road_runs.append(item)
            asphalt = road["highway"] not in ("footway", "path", "pedestrian", "living_street")
            if asphalt:
                add_ribbon(meshes["Road"], run, road["width"], 0.08, (0.16, 0.16, 0.17))
                # Brick sidewalks on both sides, the red pavers in the winter photos.
                left, right = ribbon(run, road["width"] * 0.5 + 1.5)
                add_ribbon(meshes["Sidewalk"], left, 2.6, 0.055, (0.48, 0.24, 0.20))
                add_ribbon(meshes["Sidewalk"], right, 2.6, 0.055, (0.48, 0.24, 0.20))
            else:
                add_ribbon(meshes["Sidewalk"], run, max(road["width"], 3.5), 0.06, (0.50, 0.26, 0.21))

    # Cars, snow mounds, trees, lamps along mapped roads. Trees and lamps are
    # not in the OSM extract for this neighbourhood (verified: zero natural=tree
    # and zero street_lamp nodes), so they are placed from the road geometry.
    trees, lamps, cars, snow_piles = [], [], [], []
    car_colors = [(0.85, 0.85, 0.88), (0.10, 0.10, 0.11), (0.55, 0.58, 0.62), (0.15, 0.18, 0.28), (0.75, 0.75, 0.78)]
    for road in road_runs:
        pts = road["points"]
        if road["highway"] in ("footway", "path"):
            continue
        # Walk the polyline by distance.
        cursor = 0.0
        next_tree = 6.0
        next_lamp = 4.0
        next_car = 9.0
        next_snow = 3.0
        both = road["name"] in ("中央街", "王肃街") or road["highway"] in ("pedestrian", "tertiary")
        for i in range(len(pts) - 1):
            a, b = pts[i], pts[i + 1]
            seg = dist2(a, b)
            direction = vnorm(vsub(b, a))
            side = (-direction[1], direction[0])
            travelled = 0.0
            while travelled < seg:
                cursor += 0.5
                travelled += 0.5
                if travelled > seg:
                    break
                t = travelled / seg
                pos = (a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t)
                half = road["width"] * 0.5
                if cursor >= next_snow:
                    next_snow = cursor + 7.0 + h01(int(cursor * 10), road["id"]) * 4.0
                    for sgn in (-1, 1):
                        ox = pos[0] + side[0] * sgn * (half + 0.15)
                        oz = pos[1] + side[1] * sgn * (half + 0.15)
                        if any(point_in_poly((ox, oz), fp) for fp in cn_footprints):
                            continue
                        rx = 1.1 + h01(int(ox), 1) * 1.4
                        rz = 0.7 + h01(int(oz), 2) * 0.8
                        ry = 0.35 + h01(int(ox + oz), 3) * 0.45
                        shade = 0.88 + h01(int(ox), 4) * 0.08
                        add_mound(meshes["Snow"], ox, oz, rx, ry, rz, 0.08, (shade, shade + 0.01, shade + 0.02))
                        snow_piles.append([round(ox, 2), round(oz, 2)])
                if cursor >= next_tree:
                    next_tree = cursor + 13.0 + h01(int(cursor), road["id"]) * 5.0
                    for sgn in ((-1, 1) if both else (-1,)):
                        ox = pos[0] + side[0] * sgn * (half + 2.4)
                        oz = pos[1] + side[1] * sgn * (half + 2.4)
                        if any(point_in_poly((ox, oz), fp) for fp in cn_footprints):
                            continue
                        trees.append({
                            "x": round(ox, 2),
                            "z": round(oz, 2),
                            "scale": round(0.85 + h01(int(ox * 3), oz) * 0.5, 3),
                            "source": "placed_along_osm_road",
                        })
                if cursor >= next_lamp:
                    next_lamp = cursor + (22.0 if both else 28.0)
                    sgn = -1 if int(road["id"]) % 2 == 0 else 1
                    ox = pos[0] + side[0] * sgn * (half + 0.7)
                    oz = pos[1] + side[1] * sgn * (half + 0.7)
                    if any(point_in_poly((ox, oz), fp) for fp in cn_footprints):
                        continue
                    # Arm points back toward the carriageway.
                    aim = math.atan2(-side[0] * sgn, -side[1] * sgn)
                    near_bank = False
                    sp = span_at(ox)
                    if sp and abs(oz - sp[0]) < 28:
                        near_bank = True
                    lamps.append({
                        "x": round(ox, 2),
                        "z": round(oz, 2),
                        "yaw": round(aim, 4),
                        "kind": "promenade" if near_bank else "sodium",
                        "source": "placed_along_osm_road",
                    })
                if cursor >= next_car and road["highway"] not in ("pedestrian",):
                    next_car = cursor + 14.0 + h01(int(cursor), 6) * 10.0
                    if h01(int(cursor * 4), road["id"]) > 0.25:
                        sgn = 1 if h01(int(cursor), 7) > 0.5 else -1
                        ox = pos[0] + side[0] * sgn * (half - 1.15)
                        oz = pos[1] + side[1] * sgn * (half - 1.15)
                        yaw = math.atan2(direction[0], direction[1])
                        if sgn > 0:
                            yaw += math.pi
                        rgb = car_colors[int(h01(int(ox * 5), oz) * len(car_colors)) % len(car_colors)]
                        add_car(meshes["Car"], meshes["CarGlass"], ox, oz, yaw, rgb)
                        cars.append({"x": round(ox, 2), "z": round(oz, 2), "yaw": round(yaw, 3), "source": "dressing"})
                elif cursor >= next_car:
                    next_car = cursor + 40.0

    # Signs from real POI names, plus street blades and a few rooftop characters.
    signs = []
    cn_buildings = [b for b in buildings if any(r["id"] == b["id"] and r["side"] == "cn" for r in records)]
    # index records by id
    by_id = {r["id"]: r for r in records}

    def find_building_at(x, z):
        best = None
        best_area = 1e18
        for bld in buildings:
            rec = by_id.get(bld["id"])
            if not rec or rec["side"] != "cn":
                continue
            if point_in_poly((x, z), bld["footprint"]) or dist2((x, z), bld["centroid"]) < 18:
                if point_in_poly((x, z), bld["footprint"]) and bld["area"] < best_area:
                    best = bld
                    best_area = bld["area"]
        if best:
            return best
        best_d = 22.0
        for bld in buildings:
            rec = by_id.get(bld["id"])
            if not rec or rec["side"] != "cn":
                continue
            d = dist2((x, z), bld["centroid"])
            if d < best_d:
                best_d = d
                best = bld
        return best

    used_sign_slots = {}
    for poi in pois:
        if not (vis_minx <= poi["x"] <= vis_maxx and vis_minz - 10 <= poi["z"] <= 80):
            continue
        bld = find_building_at(poi["x"], poi["z"])
        if not bld:
            continue
        edge = nearest_edge(bld["footprint"], road_runs)
        if not edge:
            continue
        _a, _b, normal, length, mid, _d = edge
        slot = used_sign_slots.get(bld["id"], 0)
        used_sign_slots[bld["id"]] = slot + 1
        # Slide additional signs along the facade.
        tangent = (-normal[1], normal[0])
        offset = (slot - 0.5) * min(6.0, length * 0.25)
        px = mid[0] + normal[0] * 0.55 + tangent[0] * offset
        pz = mid[1] + normal[1] * 0.55 + tangent[1] * offset
        yaw = math.atan2(-normal[0], -normal[1])
        text = poi["name_zh"] or poi["name_ru"]
        if not text:
            continue
        width = max(2.4, min(9.0, 0.62 * max(len(poi["name_zh"]), len(poi["name_ru"]) / 2)))
        signs.append({
            "kind": "shop",
            "text": poi["name_zh"],
            "text_ru": poi["name_ru"],
            "x": round(px, 2),
            "y": 3.35,
            "z": round(pz, 2),
            "yaw": round(yaw, 4),
            "width": round(width, 2),
            "palette": int(h01(poi["id"], 1) * 5),
            "source": "osm",
        })

    # Rooftop characters for hotels and a handful of other short real names.
    rooftop_candidates = []
    seen_roofs = set()

    def add_roof_candidate(score, levels, label, bld, source):
        if not label or not (2 <= len(label) <= 8):
            return
        if bld["id"] in seen_roofs:
            return
        rec = by_id.get(bld["id"])
        if not rec or rec["side"] != "cn" or rec["levels"] < 4:
            return
        seen_roofs.add(bld["id"])
        rooftop_candidates.append((score, levels, label, rec, source))

    for rec in records:
        if rec["side"] != "cn":
            continue
        label = short_zh(rec["name_zh"])
        score = 0
        if any(k in rec["name_zh"] for k in ("酒店", "宾馆", "饭店", "旅馆")):
            score += 5
        if "纪念" in rec["name_zh"] or "博物" in rec["name_zh"] or "商场" in rec["name_zh"]:
            score += 4
        if score <= 0:
            continue
        bld = next((b for b in buildings if b["id"] == rec["id"]), None)
        if bld:
            add_roof_candidate(score, rec["levels"], label, bld, "osm_name")
    # Hotel and museum names in this extract live on POI nodes more often than
    # on the building way. Hang the rooftop characters on the building that contains them.
    for poi in pois:
        label = short_zh(poi["name_zh"])
        score = 0
        if poi.get("tourism") == "hotel" or any(k in poi["name_zh"] for k in ("酒店", "宾馆", "饭店", "旅馆")):
            score = 6
        elif poi.get("tourism") == "museum" or "纪念" in poi["name_zh"] or "博物" in poi["name_zh"]:
            score = 4
        if score == 0:
            continue
        bld = find_building_at(poi["x"], poi["z"])
        if bld:
            add_roof_candidate(score, by_id[bld["id"]]["levels"], label, bld, "osm_poi")
    rooftop_candidates.sort(key=lambda item: (-item[0], -item[1]))
    for _score, _levels, label, rec, source in rooftop_candidates[:14]:
        bld = next(b for b in buildings if b["id"] == rec["id"])
        edge = nearest_edge(bld["footprint"], road_runs)
        if not edge:
            continue
        _a, _b, normal, _length, mid, _d = edge
        yaw = math.atan2(-normal[0], -normal[1])
        signs.append({
            "kind": "rooftop",
            "text": label,
            "text_ru": "",
            "x": round(mid[0] + normal[0] * 0.8, 2),
            "y": round(rec["height"] + 2.4, 2),
            "z": round(mid[1] + normal[1] * 0.8, 2),
            "yaw": round(yaw, 4),
            "width": round(max(4.0, len(label) * 1.15), 2),
            "palette": 0,
            "source": source,
        })

    # Street blades, one per named road, at the middle of its longest run.
    by_name = {}
    for road in road_runs:
        if not road["name"]:
            continue
        length = sum(dist2(road["points"][i], road["points"][i + 1]) for i in range(len(road["points"]) - 1))
        prev = by_name.get(road["name"])
        if prev is None or length > prev[0]:
            by_name[road["name"]] = (length, road)
    for name, (_length, road) in by_name.items():
        pts = road["points"]
        mid = pts[len(pts) // 2]
        direction = vnorm(vsub(pts[min(len(pts) - 1, len(pts) // 2 + 1)], pts[max(0, len(pts) // 2 - 1)]))
        side = (-direction[1], direction[0])
        px = mid[0] + side[0] * (road["width"] * 0.5 + 0.4)
        pz = mid[1] + side[1] * (road["width"] * 0.5 + 0.4)
        yaw = math.atan2(-side[0], -side[1])
        signs.append({
            "kind": "street",
            "text": name,
            "text_ru": road.get("name_ru") or "",
            "x": round(px, 2),
            "y": 3.5,
            "z": round(pz, 2),
            "yaw": round(yaw, 4),
            "width": round(max(2.2, len(name) * 0.55), 2),
            "palette": 4,
            "source": "osm",
        })

    # Heat: standing at a shop sign counts as shelter from the wind.
    heat = [{"x": s["x"], "z": s["z"], "radius": 3.2} for s in signs if s["kind"] == "shop"]

    # Mother sculpture and boundary stone. Abstract stand-in, real coordinates.
    landmarks = []
    for el in elements:
        if el["type"] != "node":
            continue
        tags = el.get("tags", {})
        if tags.get("tourism") == "artwork" or tags.get("historic") == "boundary_stone":
            x, z = project(el["lat"], el["lon"])
            kind = "boundary_stone" if tags.get("historic") == "boundary_stone" else "sculpture"
            zh, ru = names_of(tags)
            landmarks.append({"kind": kind, "name_zh": zh or ("母亲" if kind == "sculpture" else tags.get("name", "")), "name_ru": ru, "x": round(x, 2), "z": round(z, 2), "osm": el["id"]})
            if kind == "sculpture":
                add_box(meshes["Prop"], (x, 0.7, z), (2.4, 1.4, 1.6), (0.55, 0.55, 0.56))
                add_box(meshes["Prop"], (x, 2.3, z), (0.55, 1.8, 0.4), (0.45, 0.32, 0.18))
                add_box(meshes["Prop"], (x + 0.28, 1.9, z), (0.28, 0.7, 0.28), (0.45, 0.32, 0.18))
                # Paved disc of the square.
                meshes["Promenade"].quad(
                    [(x - 16, 0.07, z - 12), (x + 16, 0.07, z - 12), (x + 16, 0.07, z + 14), (x - 16, 0.07, z + 14)],
                    [(0, 0), (1, 0), (1, 1), (0, 1)],
                    [(0, 0)] * 4,
                    [col((0.42, 0.40, 0.40))] * 4,
                )
            else:
                add_box(meshes["Prop"], (x, 0.9, z), (0.7, 1.8, 0.45), (0.18, 0.18, 0.19))

    chimneys = []
    for el in elements:
        tags = el.get("tags", {})
        if el["type"] == "node" and tags.get("man_made") == "chimney":
            x, z = project(el["lat"], el["lon"])
            if abs(x) > 6000 or abs(z) > 6000:
                continue
            chimneys.append({"name": tags.get("name") or "烟囱", "x": round(x, 2), "z": round(z, 2), "height": 48, "source": "osm", "osm": el["id"]})
            add_box(meshes["Prop"], (x, 24, z), (3.2, 48, 3.2), (0.45, 0.42, 0.40))
        if el["type"] == "way" and tags.get("power") == "plant":
            pts = [project(p["lat"], p["lon"]) for p in el.get("geometry") or []]
            if not pts:
                continue
            x = sum(p[0] for p in pts) / len(pts)
            z = sum(p[1] for p in pts) / len(pts)
            chimneys.append({
                "name": tags.get("name") or "热电厂",
                "x": round(x, 2),
                "z": round(z, 2),
                "height": 70,
                "source": "osm",
                "osm": el["id"],
                "note": tags.get("plant:source", ""),
                "output": tags.get("plant:output:electricity", ""),
            })
            add_box(meshes["Prop"], (x, 35, z), (4.0, 70, 4.0), (0.42, 0.40, 0.38))

    # Gameplay anchors.
    bank_line = [[round(x, 2), round(s, 2)] for x, s, _n in samples if vis_minx - 30 <= x <= vis_maxx + 30]
    ice_limit = []
    flash_points = []
    for x, s, n in samples:
        if not (vis_minx <= x <= vis_maxx):
            continue
        ice_limit.append([round(x, 2), round(s + (n - s) * ICE_CROSS, 2)])
        if int(x) % 80 < 12:
            flash_points.append([round(x, 2), round(s + (n - s) * 0.62, 2)])

    # Spawn on 中央街, toward the south end of the pedestrian way, facing along it.
    spawn = {"x": -40.0, "z": -180.0, "yaw": math.pi}
    central = [r for r in road_runs if r["name"] == "中央街"]
    if central:
        central.sort(key=lambda r: min(p[1] for p in r["points"]))
        run = central[0]
        # A point about 40 m up the run from its south end.
        a, b = run["points"][0], run["points"][min(1, len(run["points"]) - 1)]
        direction = vnorm(vsub(b, a))
        # Prefer the end with the smaller z (further south) as the start.
        if run["points"][0][1] > run["points"][-1][1]:
            run_pts = list(reversed(run["points"]))
        else:
            run_pts = run["points"]
        spawn_pt = run_pts[0]
        acc = 0.0
        for i in range(len(run_pts) - 1):
            step = dist2(run_pts[i], run_pts[i + 1])
            if acc + step > 35:
                t = (35 - acc) / step
                spawn_pt = (
                    run_pts[i][0] + (run_pts[i + 1][0] - run_pts[i][0]) * t,
                    run_pts[i][1] + (run_pts[i + 1][1] - run_pts[i][1]) * t,
                )
                direction = vnorm(vsub(run_pts[i + 1], run_pts[i]))
                break
            acc += step
            direction = vnorm(vsub(run_pts[i + 1], run_pts[i]))
        yaw = math.atan2(-direction[0], -direction[1])
        spawn = {"x": round(spawn_pt[0], 2), "z": round(spawn_pt[1], 2), "yaw": round(yaw, 4)}

    mother = next((lm for lm in landmarks if lm["kind"] == "sculpture"), {"x": 0, "z": 0})
    cameras = {
        "street": {
            "position": [spawn["x"] - 2.2, 1.65, spawn["z"] - 6.5],
            "look_at": [spawn["x"] + 8.0, 2.2, spawn["z"] + 26.0],
        },
        "river": {
            "position": [mother["x"] - 6.0, 1.7, mother["z"] - 10.0],
            "look_at": [mother["x"] + 10.0, 18.0, mother["z"] + 520.0],
        },
        "lights": {
            "position": [spawn["x"] + 6.0, 1.35, spawn["z"] + 18.0],
            "look_at": [spawn["x"] - 4.0, 2.4, spawn["z"] + 34.0],
        },
    }

    # Paths for pedestrians: sidewalk centre-lines.
    paths = []
    for road in road_runs:
        if road["highway"] in ("footway", "path", "service"):
            continue
        left, right = ribbon(road["points"], road["width"] * 0.5 + 1.6)
        if len(left) >= 2:
            paths.append([[round(p[0], 2), round(p[1], 2)] for p in left])
        if len(right) >= 2 and road["highway"] in ("residential", "tertiary", "pedestrian"):
            paths.append([[round(p[0], 2), round(p[1], 2)] for p in right])

    bin_bytes, mesh_names = write_gltf(out_dir / "city.gltf", meshes)
    city = {
        "meta": {
            "name": "heihe_riverfront",
            "title": "黑河江岸，中央街到母亲广场",
            "generated_by": "tools/osm_import/import_osm.py",
            "osm_timestamp": timestamp,
            "origin": {"lat": LAT0, "lon": LON0, "note": "母亲雕塑 node/7779151765"},
            "attribution": "© OpenStreetMap contributors, ODbL 1.0",
            "query": "tools/osm_import/regions/heihe_riverfront.overpassql",
            "axes": "X east, Y up, Z north, meters",
            "yaw_convention": "Godot rotation.y; local -Z maps to world (sin yaw, cos yaw) with a sign flip, see player.gd",
            "spawn": spawn,
            "cameras": cameras,
            "walkable_x": [round(project(south, WALKABLE[0])[0], 2), round(project(south, WALKABLE[2])[0], 2)],
            "walkable_z": [round(project(WALKABLE[1], west)[1], 2), round(max(p[1] for p in ice_limit) if ice_limit else 40, 2)],
            "height_sources": height_sources,
            "skipped": skipped,
            "mesh_bytes": bin_bytes,
            "meshes": mesh_names,
            "counts": {
                "buildings": len(records),
                "buildings_cn": sum(1 for r in records if r["side"] == "cn"),
                "buildings_ru": sum(1 for r in records if r["side"] == "ru"),
                "roads": len(road_runs),
                "pois": len(pois),
                "signs": len(signs),
                "trees": len(trees),
                "lamps": len(lamps),
                "cars": len(cars),
                "chimneys": len(chimneys),
            },
        },
        "buildings": records,
        "roads": [
            {
                "id": r["id"],
                "name": r["name"],
                "name_ru": r["name_ru"],
                "highway": r["highway"],
                "width": r["width"],
                "points": [[round(p[0], 2), round(p[1], 2)] for p in r["points"]],
            }
            for r in road_runs
        ],
        "bank": bank_line,
        "ice_limit": ice_limit,
        "flashes": flash_points,
        "landmarks": landmarks,
        "signs": signs,
        "trees": trees,
        "lamps": lamps,
        "cars": cars,
        "chimneys": chimneys,
        "heat": heat[:80],
        "paths": paths,
        "mother": {"x": mother["x"], "z": mother["z"]},
    }
    out_path = out_dir / "city.json"
    out_path.write_text(json.dumps(city, ensure_ascii=False, separators=(",", ":")), encoding="utf-8")
    report = {
        "osm_timestamp": timestamp,
        "counts": city["meta"]["counts"],
        "height_sources": height_sources,
        "skipped": skipped,
        "spawn": spawn,
        "mother": city["mother"],
        "chimneys": chimneys,
        "named_roads": sorted({r["name"] for r in road_runs if r["name"]}),
        "mesh_bytes": bin_bytes,
        "gltf_meshes": mesh_names,
    }
    (out_dir / "import_report.json").write_text(json.dumps(report, ensure_ascii=False, indent=2), encoding="utf-8")
    print(json.dumps(report, ensure_ascii=False, indent=2))
    return report


def fetch(region_path: Path, out_path: Path):
    region = json.loads(region_path.read_text(encoding="utf-8"))
    query_path = region_path.parent / region["query_file"]
    query = query_path.read_text(encoding="utf-8")
    data = urllib.parse.urlencode({"data": query}).encode()
    req = urllib.request.Request(
        region["overpass_url"],
        data=data,
        headers={"User-Agent": "northeastwar-osm-import/0.1 (heihe prototype)"},
    )
    with urllib.request.urlopen(req, timeout=240) as resp:
        body = resp.read()
    out_path.parent.mkdir(parents=True, exist_ok=True)
    out_path.write_bytes(body)
    print(f"wrote {out_path} ({len(body)} bytes)")


def main():
    parser = argparse.ArgumentParser(description="Import OSM into a Godot-ready Heihe slice")
    sub = parser.add_subparsers(dest="cmd", required=True)
    p_fetch = sub.add_parser("fetch", help="download Overpass JSON for the Heihe region")
    p_fetch.add_argument("--region", type=Path, default=REGION_DIR / "heihe_riverfront.json")
    p_fetch.add_argument("--output", type=Path, default=ROOT / "data" / "heihe" / "overpass.json")
    p_build = sub.add_parser("build", help="convert an Overpass extract into city.json + city.gltf")
    p_build.add_argument("--input", type=Path, default=ROOT / "data" / "heihe" / "overpass.json")
    p_build.add_argument("--out-dir", type=Path, default=ROOT / "data" / "heihe")
    args = parser.parse_args()
    if args.cmd == "fetch":
        fetch(args.region, args.output)
    else:
        args.out_dir.mkdir(parents=True, exist_ok=True)
        build(args.input, args.out_dir)


if __name__ == "__main__":
    main()
