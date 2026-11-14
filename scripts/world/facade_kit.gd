extends Node3D
class_name FacadeKit
## The 中央街 block is built from repeated pieces: a tiled wall bay, a
## recessed window with a curtain, an enclosed balcony with a metal grille,
## an AC box, and a ground-floor shopfront with an optional roller shutter.
## Footprints and storey counts come from the imported OSM buildings.
## The old merged facade is hidden inside those footprints so the two do not
## fight.

const FOCUS := Vector2(-268.8, -325.2)
const BAY := 3.15
const FLOOR_H := 3.0
const MAX_BUILDINGS := 4
const MAX_EDGES := 3

var _parts: Dictionary = {}

func _ready() -> void:
	add_to_group("facade_kit")

func build(city: CityData) -> void:
	var plaster := load("res://assets/textures/polyhaven/yellow_plaster_diff.jpg")
	var plaster_n := load("res://assets/textures/polyhaven/yellow_plaster_nor.jpg")
	var white := load("res://assets/textures/polyhaven/white_plaster_diff.jpg")
	var white_n := load("res://assets/textures/polyhaven/white_plaster_nor.jpg")
	var tiles := load("res://assets/textures/polyhaven/facade_tiles_diff.jpg")
	var tiles_n := load("res://assets/textures/polyhaven/facade_tiles_nor.jpg")
	var snow := load("res://assets/textures/polyhaven/snow_02_diff.jpg")
	var snow_n := load("res://assets/textures/polyhaven/snow_02_nor.jpg")
	var concrete := load("res://assets/textures/polyhaven/concrete_diff.jpg")
	var concrete_n := load("res://assets/textures/polyhaven/concrete_nor.jpg")
	var wall_mats: Array[StandardMaterial3D] = [
		_surf(plaster, plaster_n, 0.72, 0.0, Vector3(4, 4, 4), Color(1.45, 1.32, 1.05)),
		_surf(white, white_n, 0.68, 0.0, Vector3(4, 4, 4), Color(1.25, 1.22, 1.15)),
		_surf(tiles, tiles_n, 0.6, 0.0, Vector3(6, 6, 6), Color(1.35, 1.22, 1.05)),
	]
	var snow_mat := _surf(snow, snow_n, 0.88, 0.0, Vector3(2, 2, 2), Color(0.68, 0.74, 0.82))
	var frame_mat := StandardMaterial3D.new()
	frame_mat.albedo_color = Color(0.9, 0.91, 0.9)
	frame_mat.roughness = 0.4
	var glass_mat := StandardMaterial3D.new()
	glass_mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	glass_mat.albedo_color = Color(0.62, 0.74, 0.82, 0.38)
	glass_mat.roughness = 0.06
	glass_mat.emission_enabled = true
	glass_mat.emission = Color(1.0, 0.58, 0.25)
	glass_mat.emission_energy_multiplier = 0.55
	var metal := StandardMaterial3D.new()
	metal.albedo_color = Color(0.16, 0.17, 0.18)
	metal.metallic = 0.65
	metal.roughness = 0.38
	var ac_mat := StandardMaterial3D.new()
	ac_mat.albedo_color = Color(0.78, 0.8, 0.82)
	ac_mat.metallic = 0.35
	ac_mat.roughness = 0.48
	var shutter_mat := StandardMaterial3D.new()
	shutter_mat.albedo_color = Color(0.55, 0.57, 0.58)
	shutter_mat.metallic = 0.45
	shutter_mat.roughness = 0.4
	var rubble_mat := _surf(concrete, concrete_n, 0.92, 0.0, Vector3(1.2, 1.2, 1.2))
	var wall_mesh := _box(Vector3(BAY * 0.98, FLOOR_H * 0.98, 0.22))
	var frame_mesh := _box(Vector3(1.55, 1.7, 0.08))
	var mullion_mesh := _box(Vector3(0.06, 1.55, 0.09))
	var glass_mesh := _box(Vector3(1.28, 1.42, 0.02))
	var curtain_mesh := _box(Vector3(0.7, 1.35, 0.02))
	var sill_mesh := _box(Vector3(1.7, 0.08, 0.28))
	var balcony_mesh := _box(Vector3(2.5, 0.1, 0.95))
	var rail_mesh := _box(Vector3(0.04, 1.05, 0.04))
	var ac_mesh := _box(Vector3(0.78, 0.36, 0.32))
	var shop_glass := _box(Vector3(2.4, 2.15, 0.04))
	var slat_mesh := _box(Vector3(2.35, 0.08, 0.06))
	var curb_mesh := _box(Vector3(1.0, 0.16, 0.22))
	var bank_mesh := _box(Vector3(4.5, 0.85, 1.15))
	var hides: Array[AABB] = []
	var ranked: Array = []
	for building in city.data.get("buildings", []):
		if str(building.get("side", "")) != "cn":
			continue
		var c: Array = building.get("centroid", [0, 0])
		var dist := Vector2(float(c[0]), float(c[1])).distance_squared_to(FOCUS)
		if dist > 90.0 * 90.0:
			continue
		ranked.append({"d": dist, "b": building})
	ranked.sort_custom(func(a, b): return float(a["d"]) < float(b["d"]))
	var kept := 0
	for item in ranked:
		if kept >= MAX_BUILDINGS:
			break
		var building: Dictionary = item["b"]
		var foot: Array = building.get("footprint", [])
		if foot.size() < 3:
			continue
		var id := int(building.get("id", 0))
		var height := float(building.get("height", 12.0))
		var floors := clampi(int(height / FLOOR_H), 2, 8)
		var edges := _longest_edges(foot, MAX_EDGES)
		if edges.is_empty():
			continue
		hides.append(_foot_aabb(foot, height))
		var root := Node3D.new()
		root.name = "B%d" % id
		add_child(root)
		var parts: Array = []
		var brick_diff := load("res://assets/textures/polyhaven/red_brick_diff.jpg")
		var brick_nor := load("res://assets/textures/polyhaven/red_brick_nor.jpg")
		var brick_mat := _surf(brick_diff, brick_nor, 0.78, 0.0, Vector3(2.2, 3.0, 2.2), Color(0.95, 0.78, 0.7))
		var pastel := StandardMaterial3D.new()
		pastel.albedo_color = Color(0.9, 0.74, 0.68)
		pastel.roughness = 0.72
		var blue_glass := StandardMaterial3D.new()
		blue_glass.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		blue_glass.albedo_color = Color(0.45, 0.62, 0.82, 0.55)
		blue_glass.roughness = 0.08
		blue_glass.emission_enabled = true
		blue_glass.emission = Color(0.55, 0.72, 0.95)
		blue_glass.emission_energy_multiplier = 0.35
		var quilt_a := StandardMaterial3D.new()
		quilt_a.albedo_color = Color(0.62, 0.1, 0.1)
		var quilt_b := StandardMaterial3D.new()
		quilt_b.albedo_color = Color(0.12, 0.28, 0.42)
		var awning_mat := StandardMaterial3D.new()
		awning_mat.albedo_color = Color(0.75, 0.12, 0.1)
		var styles: Array[String] = ["russian", "walkup", "shoprow", "tile"]
		var storeys: Array[int] = [3, 6, 2, 5]
		var edge_i := 0
		for edge in edges:
			var a: Vector2 = edge["a"]
			var b: Vector2 = edge["b"]
			var tangent: Vector2 = edge["tangent"]
			var normal: Vector2 = edge["normal"]
			var length: float = edge["length"]
			var bays := maxi(int(length / BAY), 1)
			var out := Vector3(normal.x, 0, normal.y)
			var along := Vector3(tangent.x, 0, tangent.y)
			var basis := Basis(along, Vector3.UP, out).orthonormalized()
			var mixed := edge_i == 0
			for f in floors:
				var floor_node := Node3D.new()
				floor_node.name = "E%dF%d" % [parts.size(), f]
				floor_node.set_meta("y", float(f) * FLOOR_H)
				floor_node.set_meta("out", out)
				root.add_child(floor_node)
				for k in bays:
					var chunk := int(k / 5)
					var style := styles[chunk % styles.size()] if mixed else "walkup"
					var top := storeys[chunk % storeys.size()] if mixed else mini(floors, 6)
					if f >= top:
						continue
					var wall: Material = wall_mats[chunk % wall_mats.size()]
					if style == "walkup":
						wall = brick_mat
					elif style == "russian":
						wall = pastel
					var t := (float(k) + 0.5) / float(bays)
					var p := a.lerp(b, t)
					var origin := Vector3(p.x, float(f) * FLOOR_H + FLOOR_H * 0.5, p.y)
					var xf := Transform3D(basis, origin + out * 0.16)
					_put(floor_node, wall_mesh, wall, xf)
					if f == top - 1:
						_roof(floor_node, basis, origin, out, style, snow_mat, frame_mat, metal, k)
					if f == 0:
						_storefront(floor_node, basis, origin, out, shop_glass, glass_mat if style != "tile" else blue_glass, frame_mesh, frame_mat, slat_mesh, shutter_mat, k)
						_door(floor_node, basis, origin, out, k % 2 == 0, quilt_a if k % 2 == 0 else quilt_b)
						if style == "shoprow" or style == "russian":
							_put(floor_node, _box(Vector3(2.3, 0.08, 1.15)), awning_mat, Transform3D(basis, origin + out * 1.05 + Vector3(0, 0.55, 0)))
					elif style == "russian":
						_arch(floor_node, basis, origin, out, frame_mat, glass_mat)
					elif style == "tile":
						_put(floor_node, glass_mesh, blue_glass, Transform3D(basis, origin + out * 0.3))
						_balcony(floor_node, basis, origin, out, balcony_mesh, frame_mat, rail_mesh, metal, glass_mesh, blue_glass)
					elif (k + f) % 3 == 0:
						_balcony(floor_node, basis, origin, out, balcony_mesh, frame_mat, rail_mesh, metal, glass_mesh, glass_mat)
						_bars(floor_node, basis, origin, out, metal)
					else:
						_window(floor_node, basis, origin, out, frame_mesh, frame_mat, mullion_mesh, glass_mesh, glass_mat, curtain_mesh, k + f)
						_put(floor_node, ac_mesh, ac_mat, Transform3D(basis, origin + out * 0.42 + Vector3(0, 0.85, 0) + along * 0.55))
						if f == 1:
							_bars(floor_node, basis, origin, out, metal)
				parts.append(floor_node)
			_curb_and_bank(root, a, b, tangent, normal, curb_mesh, rubble_mat, bank_mesh, snow_mat)
			if mixed:
				_street_life(root, a, b, tangent, normal)
			edge_i += 1
		_parts[id] = parts
		kept += 1
	_push_hides(hides)
	_skyline(city)

func drop_between(id: int, y0: float, y1: float) -> void:
	if not _parts.has(id):
		return
	var parts: Array = _parts[id]
	for node in parts:
		if not is_instance_valid(node):
			continue
		var y: float = node.get_meta("y", -1.0)
		if y < y0 - 0.2 or y >= y1 - 0.05:
			continue
		var out: Vector3 = node.get_meta("out", Vector3(0, 0, 1))
		var fall := out * (3.2 + y * 0.15) + Vector3(0, -y + 0.35, 0)
		if GameState.shot_mode:
			node.position = fall
			node.rotation_degrees = Vector3(76, 8, 18)
		else:
			var tween := create_tween()
			tween.tween_property(node, "position", fall, 0.8)
			tween.parallel().tween_property(node, "rotation_degrees", Vector3(76, 8, 18), 0.8)
		_dust(node.global_position + out * 2.0)
		_pile(node.global_position + out * 2.4)

func _roof(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, style: String, snow_mat: Material, white: Material, metal: Material, index: int) -> void:
	var top := origin + Vector3(0, FLOOR_H * 0.55, 0) - out * 1.4
	_put(parent, _box(Vector3(BAY * 0.98, 0.28, 3.2)), snow_mat, Transform3D(basis, top))
	_put(parent, _box(Vector3(BAY, 0.22, 0.28)), white, Transform3D(basis, origin + out * 0.32 + Vector3(0, FLOOR_H * 0.5, 0)))
	if style == "walkup" and index % 5 == 0:
		_put(parent, _box(Vector3(1.3, 1.1, 1.3)), metal, Transform3D(basis, top + Vector3(0, 0.7, 0)))
		_put(parent, _box(Vector3(0.35, 1.4, 0.35)), metal, Transform3D(basis, top + Vector3(0.8, 0.8, 0)))
	for i in 4:
		var x := -1.0 + float(i) * 0.6
		_put(parent, _box(Vector3(0.06, 0.45, 0.06)), snow_mat, Transform3D(basis, origin + out * 0.4 + basis.x * x + Vector3(0, FLOOR_H * 0.28, 0)))

func _door(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, quilt: bool, cloth: Material) -> void:
	var at := origin + out * 0.4 + Vector3(0, -0.7, 0)
	_put(parent, _box(Vector3(0.95, 1.7, 0.1)), cloth, Transform3D(basis, at))
	if not quilt:
		var strip := StandardMaterial3D.new()
		strip.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		strip.albedo_color = Color(0.85, 0.9, 0.86, 0.55)
		for i in 5:
			_put(parent, _box(Vector3(0.08, 1.5, 0.02)), strip, Transform3D(basis, at + out * 0.08 + basis.x * (-0.3 + float(i) * 0.15)))

func _arch(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, white: Material, glass_mat: Material) -> void:
	var at := origin + out * 0.3
	_put(parent, _box(Vector3(1.15, 1.9, 0.08)), white, Transform3D(basis, at))
	_put(parent, _box(Vector3(0.85, 1.5, 0.03)), glass_mat, Transform3D(basis, at - out * 0.02))
	var crown := MeshInstance3D.new()
	var dome := SphereMesh.new()
	dome.radius = 0.48
	dome.height = 0.7
	crown.mesh = dome
	crown.material_override = white
	crown.transform = Transform3D(basis, at + Vector3(0, 0.85, 0))
	crown.scale = Vector3(1.15, 0.55, 0.35)
	parent.add_child(crown)

func _bars(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, metal: Material) -> void:
	var at := origin + out * 0.4
	for i in 4:
		_put(parent, _box(Vector3(0.04, 1.35, 0.04)), metal, Transform3D(basis, at + basis.x * (-0.45 + float(i) * 0.3)))
	_put(parent, _box(Vector3(1.3, 0.04, 0.04)), metal, Transform3D(basis, at + Vector3(0, 0.35, 0)))

func _street_life(parent: Node3D, a: Vector2, b: Vector2, tangent: Vector2, normal: Vector2) -> void:
	var out := Vector3(normal.x, 0, normal.y)
	var along := Vector3(tangent.x, 0, tangent.y)
	var basis := Basis(along, Vector3.UP, out).orthonormalized()
	var mid := a.lerp(b, 0.35)
	var stall := Vector3(mid.x, 0, mid.y) + out * 5.5
	_put(parent, _box(Vector3(1.8, 0.9, 0.8)), _flat(Color(0.45, 0.28, 0.14)), Transform3D(basis, stall + Vector3(0, 0.5, 0)))
	_put(parent, _box(Vector3(2.2, 0.06, 1.6)), _flat(Color(0.15, 0.35, 0.55)), Transform3D(basis, stall + Vector3(0, 1.7, 0)))
	var warm := OmniLight3D.new()
	warm.position = stall + Vector3(0, 1.5, 0)
	warm.light_color = Color(1.0, 0.55, 0.18)
	warm.light_energy = 2.4
	warm.omni_range = 8.0
	warm.shadow_enabled = false
	parent.add_child(warm)
	_label(parent, "烤串", stall + Vector3(0, 2.05, 0), 0.4)
	var fruit_at := Vector3(a.lerp(b, 0.62).x, 0, a.lerp(b, 0.62).y) + out * 5.2
	_put(parent, _box(Vector3(2.4, 0.08, 1.8)), _flat(Color(0.15, 0.42, 0.22)), Transform3D(basis, fruit_at + Vector3(0, 2.0, 0)))
	_put(parent, _box(Vector3(0.08, 2.0, 0.08)), _flat(Color(0.3, 0.28, 0.24)), Transform3D(basis, fruit_at + along * 0.9))
	var coal := Vector3(a.lerp(b, 0.2).x, 0.3, a.lerp(b, 0.2).y) + out * 1.6
	for i in 4:
		_put(parent, _box(Vector3(0.7, 0.28, 0.45)), _flat(Color(0.1, 0.09, 0.08)), Transform3D(basis, coal + Vector3(0, float(i) * 0.22, 0)))
	_put(parent, _box(Vector3(8.0, 0.12, 0.14)), _flat(Color(0.35, 0.36, 0.38)), Transform3D(basis, coal + along * 3.0 + Vector3(0, 0.5, 0)))
	var gate := Vector3(a.lerp(b, 0.82).x, 0, a.lerp(b, 0.82).y) + out * 2.2
	_put(parent, _box(Vector3(0.35, 2.6, 0.35)), _flat(Color(0.75, 0.73, 0.68)), Transform3D(basis, gate + along * -1.4 + Vector3(0, 1.3, 0)))
	_put(parent, _box(Vector3(0.35, 2.6, 0.35)), _flat(Color(0.75, 0.73, 0.68)), Transform3D(basis, gate + along * 1.4 + Vector3(0, 1.3, 0)))
	_put(parent, _box(Vector3(3.2, 0.28, 0.28)), _flat(Color(0.82, 0.16, 0.12)), Transform3D(basis, gate + Vector3(0, 2.5, 0)))
	_label(parent, "小区", gate + Vector3(0, 2.5, 0), 0.35)
	var board := Vector3(a.lerp(b, 0.48).x, 16.5, a.lerp(b, 0.48).y) + out * 0.4
	_put(parent, _box(Vector3(7.5, 3.2, 0.2)), _flat(Color(0.75, 0.08, 0.08)), Transform3D(basis, board))
	_label(parent, "中央街", board + out * 0.2, 0.7)
	for i in 4:
		var t := 0.15 + float(i) * 0.2
		var p := a.lerp(b, t)
		_vehicle(parent, Vector3(p.x, 0, p.y) + out * 6.5 + along * (1.5 if i % 2 == 0 else -1.2), basis, i)

func _vehicle(parent: Node3D, at: Vector3, basis: Basis, index: int) -> void:
	var paint := _flat(Color(0.85, 0.72, 0.1) if index % 2 == 0 else Color(0.15, 0.16, 0.18))
	_put(parent, _box(Vector3(1.7, 1.35, 4.1)), paint, Transform3D(basis, at + Vector3(0, 0.75, 0)))
	var snow := _surf(load("res://assets/textures/polyhaven/snow_02_diff.jpg"), null, 0.9, 0.0, Vector3(1, 1, 1), Color(0.7, 0.74, 0.8))
	_put(parent, _box(Vector3(1.55, 0.18, 3.9)), snow, Transform3D(basis, at + Vector3(0, 1.5, 0)))
	var plate := _flat(Color(0.08, 0.08, 0.1))
	_put(parent, _box(Vector3(0.42, 0.16, 0.03)), plate, Transform3D(basis, at + basis.z * 2.05 + Vector3(0, 0.55, 0)))
	_label(parent, "黑A", at + basis.z * 2.08 + Vector3(0, 0.55, 0), 0.12)

func _label(parent: Node3D, text: String, at: Vector3, height: float) -> void:
	var label := Label3D.new()
	label.text = text
	label.font = load("res://assets/fonts/NotoSansSC-VF.ttf")
	label.font_size = 64
	label.pixel_size = height / 64.0
	label.modulate = Color(1.0, 0.9, 0.35)
	label.outline_size = 8
	label.position = at
	label.billboard = BaseMaterial3D.BILLBOARD_ENABLED
	parent.add_child(label)

func _flat(color: Color) -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.75
	return mat

func _storefront(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, glass: BoxMesh, glass_mat: Material, frame: BoxMesh, frame_mat: Material, slat: BoxMesh, shutter: Material, index: int) -> void:
	var at := origin + out * 0.34 + Vector3(0, -0.35, 0)
	_put(parent, glass, glass_mat, Transform3D(basis, at))
	_put(parent, frame, frame_mat, Transform3D(basis, at + out * 0.04))
	if index % 3 == 0:
		for s in 6:
			var y := 0.55 - float(s) * 0.16
			_put(parent, slat, shutter, Transform3D(basis, at + out * 0.08 + Vector3(0, y, 0)))

func _window(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, frame: BoxMesh, frame_mat: Material, mullion: BoxMesh, glass: BoxMesh, glass_mat: Material, curtain: BoxMesh, salt: int) -> void:
	var at := origin + out * 0.28
	_put(parent, frame, frame_mat, Transform3D(basis, at))
	_put(parent, mullion, frame_mat, Transform3D(basis, at + out * 0.02))
	_put(parent, glass, glass_mat, Transform3D(basis, at - out * 0.02))
	var cloth := StandardMaterial3D.new()
	cloth.albedo_color = Color(0.45, 0.16, 0.14) if salt % 2 == 0 else Color(0.2, 0.28, 0.42)
	cloth.roughness = 0.8
	_put(parent, curtain, cloth, Transform3D(basis, at - out * 0.08 + Vector3(0.15, 0, 0)))

func _balcony(parent: Node3D, basis: Basis, origin: Vector3, out: Vector3, slab: BoxMesh, slab_mat: Material, bar: BoxMesh, metal: Material, glass: BoxMesh, glass_mat: Material) -> void:
	var deck := origin + out * 0.7 + Vector3(0, -1.15, 0)
	_put(parent, slab, slab_mat, Transform3D(basis, deck))
	_put(parent, glass, glass_mat, Transform3D(basis, deck + out * 0.42 + Vector3(0, 0.6, 0)))
	for i in 6:
		var x := -1.0 + float(i) * 0.4
		var along := basis.x * x
		_put(parent, bar, metal, Transform3D(basis, deck + out * 0.46 + along + Vector3(0, 0.55, 0)))

func _curb_and_bank(parent: Node3D, a: Vector2, b: Vector2, tangent: Vector2, normal: Vector2, curb: BoxMesh, curb_mat: Material, bank: BoxMesh, snow_mat: Material) -> void:
	var mid := a.lerp(b, 0.5)
	var out := Vector3(normal.x, 0, normal.y)
	var along := Vector3(tangent.x, 0, tangent.y)
	var basis := Basis(along, Vector3.UP, out).orthonormalized()
	var length := a.distance_to(b)
	var curb_inst := MeshInstance3D.new()
	var mesh := curb.duplicate() as BoxMesh
	mesh.size = Vector3(length, 0.16, 0.28)
	curb_inst.mesh = mesh
	curb_inst.material_override = curb_mat
	curb_inst.transform = Transform3D(basis, Vector3(mid.x, 0.08, mid.y) + out * 2.6)
	parent.add_child(curb_inst)
	var count := maxi(int(length / 6.0), 1)
	for i in count:
		var t := (float(i) + 0.5) / float(count)
		var p := a.lerp(b, t)
		var inst := MeshInstance3D.new()
		inst.mesh = bank
		inst.material_override = snow_mat
		inst.transform = Transform3D(basis, Vector3(p.x, 0.4, p.y) + out * 4.0)
		parent.add_child(inst)

func _skyline(city: CityData) -> void:
	var z0 := city.bank_z(0.0) + 58.0
	var heights: Array[float] = [12.0, 20.0, 9.0, 28.0, 15.0, 22.0, 11.0, 18.0, 8.0, 16.0]
	var mat := ShaderMaterial.new()
	mat.shader = load("res://shaders/distant_windows.gdshader")
	for i in heights.size():
		var h: float = heights[i]
		var box := BoxMesh.new()
		box.size = Vector3(8.0 + float(i % 3) * 2.0, h, 7.0)
		var inst := MeshInstance3D.new()
		inst.mesh = box
		inst.material_override = mat
		var x := -46.0 + float(i) * 10.5
		inst.position = Vector3(x, h * 0.5, z0 + float(i % 2) * 4.0)
		add_child(inst)
		var light := OmniLight3D.new()
		light.position = inst.position + Vector3(0, h * 0.25, 6.0)
		light.light_color = Color(1.0, 0.62, 0.28)
		light.light_energy = 2.2
		light.omni_range = 18.0
		light.shadow_enabled = false
		add_child(light)

func _put(parent: Node3D, mesh: Mesh, mat: Material, xf: Transform3D) -> void:
	var inst := MeshInstance3D.new()
	inst.mesh = mesh
	inst.material_override = mat
	inst.transform = xf
	parent.add_child(inst)

func _box(size: Vector3) -> BoxMesh:
	var mesh := BoxMesh.new()
	mesh.size = size
	return mesh

func _surf(diff: Texture2D, nor: Texture2D, rough: float, metal: float, uv: Vector3, tint: Color = Color(1, 1, 1)) -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.albedo_texture = diff
	mat.albedo_color = tint
	if nor:
		mat.normal_enabled = true
		mat.normal_texture = nor
	mat.roughness = rough
	mat.metallic = metal
	mat.uv1_scale = uv
	return mat

func _longest_edges(foot: Array, limit: int) -> Array:
	var c := Vector2.ZERO
	for p in foot:
		c += Vector2(float(p[0]), float(p[1]))
	c /= float(foot.size())
	var edges: Array = []
	var n := foot.size()
	for i in n:
		var a := Vector2(float(foot[i][0]), float(foot[i][1]))
		var b := Vector2(float(foot[(i + 1) % n][0]), float(foot[(i + 1) % n][1]))
		var len := a.distance_to(b)
		if len < 8.0:
			continue
		var tangent := (b - a) / len
		var normal := Vector2(tangent.y, -tangent.x)
		var mid := (a + b) * 0.5
		if normal.dot(c - mid) > 0.0:
			normal = -normal
		var score := len - mid.distance_to(FOCUS) * 0.35
		edges.append({"a": a, "b": b, "length": len, "tangent": tangent, "normal": normal, "score": score})
	edges.sort_custom(func(p, q): return float(p["score"]) > float(q["score"]))
	if edges.size() > limit:
		edges.resize(limit)
	return edges

func _foot_aabb(foot: Array, height: float) -> AABB:
	var min_x := 1.0e9
	var min_z := 1.0e9
	var max_x := -1.0e9
	var max_z := -1.0e9
	for p in foot:
		min_x = minf(min_x, float(p[0]))
		max_x = maxf(max_x, float(p[0]))
		min_z = minf(min_z, float(p[1]))
		max_z = maxf(max_z, float(p[1]))
	return AABB(Vector3(min_x - 1.2, -0.2, min_z - 1.2), Vector3(max_x - min_x + 2.4, height + 1.5, max_z - min_z + 2.4))

func _push_hides(boxes: Array[AABB]) -> void:
	var view := get_tree().get_first_node_in_group("city_view")
	if view == null or not ("cut_materials" in view):
		return
	var count := mini(4, boxes.size())
	for mat in view.cut_materials:
		mat.set_shader_parameter("hide_n", float(count))
		for i in 4:
			var mn := Vector3.ZERO
			var mx := Vector3.ZERO
			if i < count:
				mn = boxes[i].position
				mx = boxes[i].position + boxes[i].size
			mat.set_shader_parameter("ha%d" % i, mn)
			mat.set_shader_parameter("hb%d" % i, mx)

func _dust(at: Vector3) -> void:
	var puff := GPUParticles3D.new()
	puff.position = at + Vector3(0, 1.0, 0)
	puff.amount = 24
	puff.lifetime = 2.6
	puff.one_shot = true
	puff.explosiveness = 0.7
	puff.preprocess = 0.35
	var proc := ParticleProcessMaterial.new()
	proc.direction = Vector3(0.1, 1, 0)
	proc.spread = 28.0
	proc.initial_velocity_min = 1.5
	proc.initial_velocity_max = 4.0
	proc.gravity = Vector3(0, 0.2, 0)
	proc.scale_min = 1.2
	proc.scale_max = 2.8
	proc.color = Color(0.45, 0.4, 0.34, 0.55)
	puff.process_material = proc
	puff.draw_pass_1 = _soft(1.8, Color(0.55, 0.5, 0.42, 0.45))
	add_child(puff)
	puff.restart()

func _pile(at: Vector3) -> void:
	var concrete := load("res://assets/textures/polyhaven/concrete_diff.jpg")
	var mat := StandardMaterial3D.new()
	mat.albedo_texture = concrete
	mat.roughness = 0.95
	for i in 5:
		var inst := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(1.1 + float(i % 2) * 0.6, 0.55, 0.8)
		inst.mesh = box
		inst.material_override = mat
		var ang := float(i) * 1.2
		inst.position = at + Vector3(cos(ang) * 1.3, 0.3 + float(i % 3) * 0.35, sin(ang) * 1.3)
		inst.rotation = Vector3(0.3 * float(i % 2), ang, 0.2)
		add_child(inst)

func _soft(size: float, color: Color) -> QuadMesh:
	var mesh := QuadMesh.new()
	mesh.size = Vector2(size, size)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
	mat.albedo_color = color
	mesh.material = mat
	return mesh
