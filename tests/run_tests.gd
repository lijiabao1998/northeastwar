extends SceneTree

var _data_ok := false
## Headless checks that do not need a window: the slice data and the imported mesh.

func _init() -> void:
	var text := FileAccess.get_file_as_string("res://data/heihe/city.json")
	var data = JSON.parse_string(text)
	if data == null:
		push_error("city.json did not parse")
		quit(1)
		return
	var attribution := str(data["meta"]["attribution"])
	if attribution.find("OpenStreetMap") < 0:
		push_error("missing OSM attribution")
		quit(1)
		return
	if int(data["meta"]["counts"]["buildings_cn"]) < 50:
		push_error("too few Chinese buildings")
		quit(1)
		return
	var names: PackedStringArray = []
	for road in data["roads"]:
		names.append(str(road["name"]))
	if not ("王肃街" in names and "中央街" in names):
		push_error("central streets missing")
		quit(1)
		return
	var packed = load("res://data/heihe/city.gltf")
	if packed == null:
		push_error("gltf failed to import")
		quit(1)
		return
	var city: Node = packed.instantiate()
	var found := {}
	for child in city.get_children():
		found[str(child.name)] = true
		if child is MeshInstance3D:
			var mesh := (child as MeshInstance3D).mesh
			if mesh == null or mesh.get_surface_count() < 1:
				push_error("empty mesh " + child.name)
				quit(1)
				return
			var arrays: Array = mesh.surface_get_arrays(0)
			if arrays[Mesh.ARRAY_VERTEX] == null:
				push_error("no vertices on " + child.name)
				quit(1)
				return
	for required in ["Facade", "Ice", "Road", "European"]:
		if not found.has(required):
			push_error("missing node " + required)
			quit(1)
			return
	# Same rule as Destruction.section_index: height is split into equal storeys.
	if _section(18.0, 0.4, 3) != 0 or _section(18.0, 7.0, 3) != 1 or _section(18.0, 14.0, 3) != 2:
		push_error("section index")
		quit(1)
		return
	_data_ok = true

func _initialize() -> void:
	## The autoloads exist from here on, so the scripts that use them can load.
	if not _data_ok:
		return
	var why := _survival_rules()
	if why != "":
		push_error(why)
		print("FAILED: " + why)
		quit(1)
		return
	print("gdscript tests ok")
	quit(0)

func _survival_rules() -> String:
	## The enemy table and the rules that read it. No window, no scene.
	var RuForces = load("res://scripts/enemy/ru_forces.gd")
	var Squads = load("res://scripts/combat/squads.gd")
	var Scavenge = load("res://scripts/survival/scavenge.gd")
	if RuForces == null or Squads == null or Scavenge == null:
		return "a survival script failed to load"
	var forces: Dictionary = RuForces.table()
	for part in ["infantry_group", "rifle", "grenade", "recon_drone", "mortar_82", "fpv"]:
		if not forces.has(part):
			return "forces.json has no " + part
	var size: Array = forces["infantry_group"]["size"]
	if int(size[0]) < 2 or int(size[1]) > 5:
		return "an infantry group is two to five men"
	# Sight shrinks as the light goes, and optics only help at night.
	if not (RuForces.sight("day", false) > RuForces.sight("dusk", false) and RuForces.sight("dusk", false) > RuForces.sight("night_lit", false) and RuForces.sight("night_lit", false) > RuForces.sight("night_dark", false)):
		return "sight must fall with the light"
	if RuForces.sight("night_dark", true) <= RuForces.sight("night_dark", false):
		return "night optics must see further in the dark"
	if RuForces.sight("day", true) != RuForces.sight("day", false):
		return "night optics must not change daylight"
	# A rifle is worse the further away you are, and never certain.
	var last := 1.0
	for metres in [5.0, 20.0, 40.0, 70.0, 84.0]:
		var chance: float = RuForces.hit_chance(metres)
		if chance > last or chance >= 0.95:
			return "hit chance must fall with range"
		last = chance
	if RuForces.hit_chance(300.0) != 0.0:
		return "nobody hits at 300 m here"
	# Close and sure is quick, far and unsure is slow.
	if RuForces.notice_time(5.0, 60.0) >= RuForces.notice_time(55.0, 60.0):
		return "a near figure is noticed sooner"
	# Standing still in a doorway is the least you can show. Running is the most.
	if not (Squads.exposure("still", true) < Squads.exposure("walk", true) and Squads.exposure("walk", true) < Squads.exposure("walk", false) and Squads.exposure("walk", false) < Squads.exposure("run", false)):
		return "exposure order"
	# A soldier walking north does not see what is behind him.
	var here := Vector3.ZERO
	if not Squads.in_arc(PI, here, Vector3(0, 0, 10), 100.0):
		return "ahead should be inside the arc"
	if Squads.in_arc(PI, here, Vector3(0, 0, -10), 100.0):
		return "behind should be outside the arc"
	# Shop signs decide what a door holds.
	var kinds := {
		"肯德基": "food", "瑞幸咖啡": "drink", "牙科": "pharmacy", "汉庭连锁酒店": "hotel",
		"以纯": "clothes", "中国银行": "office", "小米之家": "shop",
	}
	for sign in kinds:
		if Scavenge.kind_of(sign, "") != kinds[sign]:
			return "shop kind of " + sign + " is " + Scavenge.kind_of(sign, "")
	if Scavenge.kind_of("", "Ресторан пельменей") != "food":
		return "a Russian-only sign should still be read"
	for i in 20:
		var found: Dictionary = Scavenge.roll("pharmacy")
		if int(found.get("bandage", 0)) < 2:
			return "a pharmacy always has bandages"
	return ""

func _section(height: float, y: float, sections: int) -> int:
	var t := clampf(y / height, 0.0, 0.999)
	return mini(sections - 1, int(t * float(sections)))
