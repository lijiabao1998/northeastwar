extends Node3D
class_name CityProps
## Trees, sodium lamps, shop signs and chimney smoke. Positions come from the
## import; the meshes are original.

var _lights: Array[OmniLight3D] = []
var _flash_lights: Array[OmniLight3D] = []
var _font: Font

func build(data: Dictionary) -> void:
	_font = _load_font()
	_trees(data.get("trees", []))
	_lamps(data.get("lamps", []))
	_signs(data.get("signs", []))
	_smoke(data.get("chimneys", []))
	_flashes(data.get("flashes", []))
	GameState.quality_changed.connect(_apply_quality)
	_apply_quality(GameState.quality)

func set_lamp_strength(amount: float) -> void:
	var stride := _stride()
	var i := 0
	for light in _lights:
		var on := (i % stride) == 0
		var kind_energy := 1.15 if str(light.name) == "promenade" else 1.45
		light.light_energy = kind_energy * amount if on else 0.0
		light.visible = on
		i += 1

func _apply_quality(level: String) -> void:
	var stride := 1 if level == "high" else (2 if level == "medium" else 4)
	var i := 0
	for light in _lights:
		var on := (i % stride) == 0
		light.visible = on
		if on and light.light_energy <= 0.01:
			light.light_energy = 0.2
		i += 1

func _stride() -> int:
	match GameState.quality:
		"high":
			return 1
		"low":
			return 4
		_:
			return 2

func _trees(rows: Array) -> void:
	if rows.is_empty():
		return
	var mesh := _tree_mesh()
	var multi := MultiMesh.new()
	multi.transform_format = MultiMesh.TRANSFORM_3D
	multi.mesh = mesh
	multi.instance_count = rows.size()
	for i in rows.size():
		var row: Dictionary = rows[i]
		var xf := Transform3D(Basis.IDENTITY.scaled(Vector3.ONE * float(row.get("scale", 1.0))), Vector3(float(row["x"]), 0.0, float(row["z"])))
		multi.set_instance_transform(i, xf)
	var inst := MultiMeshInstance3D.new()
	inst.multimesh = multi
	inst.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(inst)

func _tree_mesh() -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	_cylinder(st, Vector3.ZERO, 0.09, 0.13, 3.4, 6, Color(0.22, 0.18, 0.15))
	# Pollarded poplar: a trunk and a fist of short cut branches, no leaves.
	var crown := Vector3(0.0, 3.35, 0.0)
	for i in 7:
		var ang := float(i) / 7.0 * TAU
		var dir := Vector3(sin(ang) * 0.55, 0.85, cos(ang) * 0.55).normalized()
		_cylinder_along(st, crown, crown + dir * 0.55, 0.028, Color(0.24, 0.2, 0.16))
	st.generate_normals()
	return st.commit()

func _lamps(rows: Array) -> void:
	var pole := _lamp_mesh()
	var multi := MultiMesh.new()
	multi.transform_format = MultiMesh.TRANSFORM_3D
	multi.mesh = pole
	multi.instance_count = rows.size()
	for i in rows.size():
		var row: Dictionary = rows[i]
		var yaw := float(row["yaw"])
		var xf := Transform3D(Basis(Vector3.UP, yaw), Vector3(float(row["x"]), 0.0, float(row["z"])))
		multi.set_instance_transform(i, xf)
		var light := OmniLight3D.new()
		light.name = str(row.get("kind", "sodium"))
		# The stored yaw points back toward the carriageway (importer convention).
		var arm := Vector3(-sin(yaw), 0.0, -cos(yaw))
		light.position = Vector3(float(row["x"]), 6.7, float(row["z"])) + arm * 2.15
		if light.name == "promenade":
			light.light_color = Color(1.0, 0.82, 0.62)
			light.light_energy = 1.05
			light.omni_range = 16.0
		else:
			light.light_color = Color(1.0, 0.62, 0.28)
			light.light_energy = 1.35
			light.omni_range = 14.0
		light.shadow_enabled = false
		light.light_indirect_energy = 0.4
		add_child(light)
		_lights.append(light)
	var inst := MultiMeshInstance3D.new()
	inst.multimesh = multi
	inst.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	add_child(inst)

func _lamp_mesh() -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var metal := Color(0.18, 0.18, 0.2)
	_cylinder(st, Vector3.ZERO, 0.07, 0.05, 6.4, 6, metal)
	# Curved arm over the road, along local -Z (the yaw forward).
	var pts := [
		Vector3(0, 6.35, 0),
		Vector3(0, 6.7, -0.7),
		Vector3(0, 6.85, -1.45),
		Vector3(0, 6.7, -2.15),
	]
	for i in pts.size() - 1:
		_cylinder_along(st, pts[i], pts[i + 1], 0.035, metal)
	_sphere(st, pts[pts.size() - 1], 0.16, Color(1.0, 0.78, 0.45))
	st.generate_normals()
	var mesh := st.commit()
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.roughness = 0.55
	mat.emission_enabled = true
	mat.emission = Color(1.0, 0.7, 0.35)
	mat.emission_energy_multiplier = 0.35
	mesh.surface_set_material(0, mat)
	return mesh

func _signs(rows: Array) -> void:
	for row in rows:
		var kind := str(row.get("kind", "shop"))
		var zh := str(row.get("text", ""))
		var ru := str(row.get("text_ru", ""))
		if zh == "" and ru == "":
			continue
		var holder := Node3D.new()
		holder.position = Vector3(float(row["x"]), float(row["y"]), float(row["z"]))
		holder.rotation.y = float(row["yaw"])
		var width := float(row.get("width", 3.0))
		var height := 2.15 if kind == "rooftop" else (0.95 if ru != "" and zh != "" else 0.7)
		if kind == "street":
			height = 0.48
			width = max(width, 1.8)
		var board := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(width, height, 0.07 if kind != "rooftop" else 0.12)
		board.mesh = box
		board.material_override = _sign_material(int(row.get("palette", 0)), kind)
		holder.add_child(board)
		# The holder's -Z points at the street. Label3D faces its own +Z,
		# so turn it around and sit it on the street side of the board.
		var light_board := int(row.get("palette", 0)) == 1 and kind == "shop"
		var main := zh if zh != "" else ru
		var label := _label(main, 0.02 if kind == "rooftop" else 0.012, 64 if kind == "rooftop" else 48)
		label.position = Vector3(0, 0.16 if (zh != "" and ru != "" and kind == "shop") else 0.0, -0.07)
		label.rotation.y = PI
		label.modulate = Color(0.12, 0.08, 0.08) if light_board else Color(1, 0.97, 0.92)
		if kind == "rooftop":
			label.modulate = Color(0.9, 0.08, 0.05)
			label.outline_modulate = Color(0.2, 0.02, 0.02)
			label.outline_size = 16
		holder.add_child(label)
		if zh != "" and ru != "" and kind == "shop":
			var sub := _label(ru, 0.008, 32)
			sub.position = Vector3(0, -0.24, -0.07)
			sub.rotation.y = PI
			sub.modulate = Color(1, 0.93, 0.8) if not light_board else Color(0.15, 0.1, 0.1)
			holder.add_child(sub)
		add_child(holder)

func _label(text: String, pixel: float, size: int) -> Label3D:
	var label := Label3D.new()
	label.text = text
	label.font = _font
	label.font_size = size
	label.pixel_size = pixel
	label.billboard = BaseMaterial3D.BILLBOARD_DISABLED
	label.double_sided = true
	label.texture_filter = BaseMaterial3D.TEXTURE_FILTER_LINEAR
	label.outline_size = 6
	label.outline_modulate = Color(0, 0, 0, 0.85)
	label.modulate = Color(1, 0.97, 0.92)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	return label

func _sign_material(palette: int, kind: String) -> StandardMaterial3D:
	var colors := [
		Color(0.72, 0.06, 0.07),
		Color(0.92, 0.92, 0.9),
		Color(0.08, 0.28, 0.72),
		Color(0.05, 0.42, 0.28),
		Color(0.07, 0.22, 0.55),
	]
	var mat := StandardMaterial3D.new()
	mat.albedo_color = colors[palette % colors.size()]
	if kind == "rooftop":
		mat.albedo_color = Color(0.12, 0.1, 0.1)
		mat.emission_enabled = true
		mat.emission = Color(0.35, 0.02, 0.02)
		mat.emission_energy_multiplier = 0.4
	elif kind == "street":
		mat.albedo_color = colors[4]
		mat.emission_enabled = true
		mat.emission = Color(0.15, 0.35, 0.8)
		mat.emission_energy_multiplier = 0.5
	else:
		mat.emission_enabled = true
		mat.emission = mat.albedo_color
		mat.emission_energy_multiplier = 0.55
	mat.roughness = 0.45
	return mat

func _smoke(rows: Array) -> void:
	for row in rows:
		var x := float(row["x"])
		var z := float(row["z"])
		if abs(x) > 5000.0 or abs(z) > 5000.0:
			continue
		var p := GPUParticles3D.new()
		p.position = Vector3(x, float(row.get("height", 40.0)), z)
		p.amount = 28
		p.lifetime = 9.0
		p.visibility_aabb = AABB(Vector3(-30, -10, -30), Vector3(60, 220, 60))
		var proc := ParticleProcessMaterial.new()
		proc.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_SPHERE
		proc.emission_sphere_radius = 1.2
		proc.direction = Vector3(0.15, 1, 0.05)
		proc.spread = 12.0
		proc.initial_velocity_min = 4.0
		proc.initial_velocity_max = 7.0
		proc.gravity = Vector3(0.4, 0.2, 0)
		proc.scale_min = 4.0
		proc.scale_max = 9.0
		proc.color = Color(0.25, 0.24, 0.22, 0.35)
		p.process_material = proc
		p.draw_pass_1 = _soft_quad(1.6)
		add_child(p)

func _flashes(rows: Array) -> void:
	for row in rows:
		var light := OmniLight3D.new()
		light.position = Vector3(float(row[0]), 8.0, float(row[1]))
		light.light_color = Color(1.0, 0.55, 0.25)
		light.omni_range = 180.0
		light.light_energy = 0.0
		light.shadow_enabled = false
		add_child(light)
		_flash_lights.append(light)

func _process(delta: float) -> void:
	if _flash_lights.is_empty() or GameState.alert_stage < 1:
		return
	var rate := 0.35 if GameState.alert_stage == 1 else 1.1
	for light in _flash_lights:
		if randf() < delta * rate:
			light.light_energy = randf_range(6.0, 22.0)
		else:
			light.light_energy = move_toward(light.light_energy, 0.0, delta * 28.0)

func _load_font() -> Font:
	var cjk: FontFile = load("res://assets/fonts/NotoSansSC-VF.ttf")
	var latin: FontFile = load("res://assets/fonts/NotoSans-Regular.ttf")
	if cjk and latin:
		var fb: Array[Font] = []
		fb.append(latin)
		cjk.fallbacks = fb
	return cjk

func _soft_quad(size: float) -> QuadMesh:
	var mesh := QuadMesh.new()
	mesh.size = Vector2(size, size)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
	mat.albedo_color = Color(1, 1, 1, 0.45)
	mat.cull_mode = BaseMaterial3D.CULL_DISABLED
	mesh.material = mat
	return mesh

func _cylinder(st: SurfaceTool, origin: Vector3, r0: float, r1: float, height: float, sides: int, color: Color) -> void:
	for i in sides:
		var a0 := float(i) / float(sides) * TAU
		var a1 := float(i + 1) / float(sides) * TAU
		var p0 := origin + Vector3(cos(a0) * r0, 0, sin(a0) * r0)
		var p1 := origin + Vector3(cos(a1) * r0, 0, sin(a1) * r0)
		var p2 := origin + Vector3(cos(a1) * r1, height, sin(a1) * r1)
		var p3 := origin + Vector3(cos(a0) * r1, height, sin(a0) * r1)
		_quad(st, p0, p1, p2, p3, color)

func _cylinder_along(st: SurfaceTool, a: Vector3, b: Vector3, radius: float, color: Color) -> void:
	var dir := (b - a)
	var length := dir.length()
	if length < 0.001:
		return
	dir /= length
	var up := Vector3.UP
	if abs(dir.dot(up)) > 0.9:
		up = Vector3.RIGHT
	var side := dir.cross(up).normalized()
	var bin := side.cross(dir).normalized()
	var sides := 5
	for i in sides:
		var t0 := float(i) / float(sides) * TAU
		var t1 := float(i + 1) / float(sides) * TAU
		var o0 := side * cos(t0) * radius + bin * sin(t0) * radius
		var o1 := side * cos(t1) * radius + bin * sin(t1) * radius
		_quad(st, a + o0, a + o1, b + o1, b + o0, color)

func _sphere(st: SurfaceTool, center: Vector3, radius: float, color: Color) -> void:
	var stacks := 3
	var slices := 5
	for i in stacks:
		for j in slices:
			var v0 := float(i) / float(stacks) * PI
			var v1 := float(i + 1) / float(stacks) * PI
			var u0 := float(j) / float(slices) * TAU
			var u1 := float(j + 1) / float(slices) * TAU
			var p00 := center + _sph(u0, v0) * radius
			var p01 := center + _sph(u1, v0) * radius
			var p11 := center + _sph(u1, v1) * radius
			var p10 := center + _sph(u0, v1) * radius
			_quad(st, p00, p01, p11, p10, color)

func _sph(u: float, v: float) -> Vector3:
	return Vector3(sin(v) * cos(u), cos(v), sin(v) * sin(u))

func _quad(st: SurfaceTool, a: Vector3, b: Vector3, c: Vector3, d: Vector3, color: Color) -> void:
	st.set_color(color)
	st.add_vertex(a)
	st.set_color(color)
	st.add_vertex(b)
	st.set_color(color)
	st.add_vertex(c)
	st.set_color(color)
	st.add_vertex(a)
	st.set_color(color)
	st.add_vertex(c)
	st.set_color(color)
	st.add_vertex(d)
