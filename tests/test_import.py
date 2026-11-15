#!/usr/bin/env python3
"""Geometry unit tests and checks against the committed Heihe extract."""

import importlib.util
import json
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def load_importer():
    path = ROOT / "tools" / "osm_import" / "import_osm.py"
    spec = importlib.util.spec_from_file_location("import_osm", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class GeometryTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.imp = load_importer()

    def test_project_roundtrip(self):
        lat, lon = 50.2510885, 127.4960436
        x, z = self.imp.project(lat, lon)
        self.assertAlmostEqual(x, 0.0, places=4)
        self.assertAlmostEqual(z, 0.0, places=4)
        back_lat, back_lon = self.imp.unproject(120.0, -80.0)
        x2, z2 = self.imp.project(back_lat, back_lon)
        self.assertAlmostEqual(x2, 120.0, places=3)
        self.assertAlmostEqual(z2, -80.0, places=3)

    def test_square_and_l(self):
        square = [(0, 0), (4, 0), (4, 3), (0, 3)]
        tris, pts = self.imp.triangulate(square)
        self.assertIsNotNone(tris)
        self.assertEqual(len(tris), 2)
        ell = [(0, 0), (6, 0), (6, 2), (2, 2), (2, 5), (0, 5)]
        tris, _pts = self.imp.triangulate(ell)
        self.assertIsNotNone(tris)
        self.assertGreaterEqual(len(tris), 4)

    def test_clip_keeps_inside(self):
        poly = [(-10, -10), (10, -10), (10, 10), (-10, 10)]
        clipped = self.imp.clip_polygon(poly, (-2, -2, 2, 2))
        self.assertGreaterEqual(len(clipped), 4)
        for x, z in clipped:
            self.assertGreaterEqual(x, -2.01)
            self.assertLessEqual(x, 2.01)
            self.assertGreaterEqual(z, -2.01)
            self.assertLessEqual(z, 2.01)


class HeiheExtractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.city = json.loads((ROOT / "data" / "heihe" / "city.json").read_text(encoding="utf-8"))

    def test_attribution_and_streets(self):
        meta = self.city["meta"]
        self.assertIn("OpenStreetMap", meta["attribution"])
        self.assertIn("ODbL", meta["attribution"])
        names = {road["name"] for road in self.city["roads"]}
        self.assertIn("王肃街", names)
        self.assertIn("中央街", names)
        self.assertIn("文化街", names)

    def test_river_and_mother(self):
        self.assertGreater(len(self.city["bank"]), 10)
        self.assertGreater(len(self.city["ice_limit"]), 10)
        self.assertAlmostEqual(self.city["mother"]["x"], 0.0, places=2)
        self.assertAlmostEqual(self.city["mother"]["z"], 0.0, places=2)
        self.assertGreater(self.city["meta"]["counts"]["buildings_cn"], 50)
        self.assertGreater(self.city["meta"]["counts"]["buildings_ru"], 50)

    def test_real_places_present(self):
        chimneys = " ".join(item.get("name", "") for item in self.city["chimneys"])
        self.assertIn("黑河市热电", chimneys)
        texts = " ".join(sign.get("text", "") + sign.get("text_ru", "") for sign in self.city["signs"])
        self.assertIn("黑河国际饭店", texts)
        self.assertTrue(any("\u0400" <= ch <= "\u04ff" for ch in texts))
        styles = {b["style"] for b in self.city["buildings"]}
        self.assertIn("euro_pink", styles)

    def test_meshes_exist(self):
        gltf = json.loads((ROOT / "data" / "heihe" / "city.gltf").read_text(encoding="utf-8"))
        names = [node["name"] for node in gltf["nodes"]]
        for required in ("Facade", "European", "Ice", "Road", "Sidewalk", "Promenade"):
            self.assertIn(required, names)
        blob = (ROOT / "data" / "heihe" / "city.bin").stat().st_size
        self.assertEqual(blob, gltf["buffers"][0]["byteLength"])
        self.assertGreater(blob, 1_000_000)


if __name__ == "__main__":
    unittest.main()
