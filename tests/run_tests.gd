extends SceneTree
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
	print("gdscript tests ok")
	quit(0)

func _section(height: float, y: float, sections: int) -> int:
	var t := clampf(y / height, 0.0, 0.999)
	return mini(sections - 1, int(t * float(sections)))
