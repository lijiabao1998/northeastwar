"""Cut CMAB buildings around central Heihe and write them in this game's map metres.

Usage:
  python -I tools/cmab/slice_heihe.py <cmab.shp> <out.json>

CMAB v7 (Zhang, Yecheng; Zhao, Huimin; Long, Ying, 2024), CC BY 4.0,
https://doi.org/10.6084/m9.figshare.27992417.v7

Modified: clipped to the box below (centre inside), reprojected from
EPSG:3857 to the same local metres tools/osm_import/import_osm.py uses
(origin the Mother sculpture, x east, y north, equirectangular around it),
rounded to centimetres. Nothing else is changed here; which CMAB building
is used, and with what height, is decided in crates/sim/src/cmab.rs.

The input is untrusted downloaded data: run with python -I, from outside the
folder it was extracted to.
"""
import json
import math
import os
import sys

import shapefile

R = 6378137.0

# Same origin and scale as tools/osm_import/import_osm.py. Keep them in step.
LAT0 = 50.2510885
LON0 = 127.4960436
M_LAT = 111_320.0
M_LON = 111_320.0 * math.cos(math.radians(LAT0))

# The OSM slice is 127.4848-127.5032 E, 50.2470-50.2565 N. This box is a
# little wider on the land side so the edge of the map is not a cliff.
WEST, SOUTH, EAST, NORTH = 127.4780, 50.2420, 127.5110, 50.2580


def merc(lon, lat):
    return math.radians(lon) * R, math.log(math.tan(math.pi / 4 + math.radians(lat) / 2)) * R


def unmerc(x, y):
    return math.degrees(x / R), math.degrees(2 * math.atan(math.exp(y / R)) - math.pi / 2)


def project(lon, lat):
    return (lon - LON0) * M_LON, (lat - LAT0) * M_LAT


def number(v):
    try:
        f = float(v)
    except (TypeError, ValueError):
        return None
    return None if math.isnan(f) else f


def main():
    shp, out = sys.argv[1], sys.argv[2]
    xmin, ymin = merc(WEST, SOUTH)
    xmax, ymax = merc(EAST, NORTH)
    sf = shapefile.Reader(shp, encoding="utf-8")
    fields = [f[0] for f in sf.fields[1:]]
    buildings = []
    seen_ids = set()
    for sr in sf.iterShapeRecords(bbox=(xmin, ymin, xmax, ymax)):
        shape, rec = sr.shape, dict(zip(fields, sr.record))
        bx = shape.bbox
        cx, cy = (bx[0] + bx[2]) / 2, (bx[1] + bx[3]) / 2
        if not (xmin <= cx <= xmax and ymin <= cy <= ymax):
            continue
        parts = list(shape.parts) + [len(shape.points)]
        rings = []
        for a, b in zip(parts, parts[1:]):
            ring = []
            for px, py in shape.points[a:b]:
                lon, lat = unmerc(px, py)
                x, y = project(lon, lat)
                ring.append([round(x, 2), round(y, 2)])
            rings.append(ring)
        bid = rec.get("merged_id", rec.get("id"))
        bid = int(bid) if number(bid) is not None else len(buildings)
        if bid in seen_ids:
            bid = 10_000_000 + len(buildings)
        seen_ids.add(bid)
        buildings.append({
            "id": bid,
            "rings": rings,
            "height_m": number(rec.get("Height")),
            "function": rec.get("Function"),
            "age": rec.get("Age"),
            "quality": rec.get("Quality"),
            "structure": rec.get("Structure"),
            "style": rec.get("Style"),
        })
    layer = os.path.splitext(os.path.basename(shp))[0]
    meta = {
        "source": f"CMAB v7, https://doi.org/10.6084/m9.figshare.27992417.v7, file heilongjiang.zip -> {layer}",
        "license": "CC BY 4.0, https://creativecommons.org/licenses/by/4.0/",
        "credit": "Zhang, Yecheng; Zhao, Huimin; Long, Ying (2024). CMAB-The World's First National-Scale Multi-Attribute Building Dataset, v7. figshare. Dataset.",
        "modified": "clipped to the box (centre inside); EPSG:3857 -> local metres around the Mother sculpture, x east, y north, as tools/osm_import/import_osm.py",
        "box_west_south_east_north": [WEST, SOUTH, EAST, NORTH],
        "origin_lat_lon": [LAT0, LON0],
        "fields": fields,
        "note": "Outlines are CMAB rooftop polygons. Heights and attributes are model estimates, not surveys.",
    }
    with open(out, "w", encoding="utf-8", newline="\n") as f:
        json.dump({"meta": meta, "buildings": buildings}, f, ensure_ascii=False, separators=(",", ":"))
    print("buildings", len(buildings), "fields", fields)


if __name__ == "__main__":
    main()
