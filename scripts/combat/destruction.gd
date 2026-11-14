extends Node
class_name Destruction
## OSM buildings break into a few vertical chunks. The intact city stays one
## merged mesh. A chunk that fails is cut out of that mesh and replaced by a
## falling piece plus pooled rubble. Only a budget of nearby buildings is
## tracked, so a laptop does not fracture the whole of Heilongjiang.

const RUBBLE_CAP := 28

var _buildings: Array = []
var _active: Dictionary = {}
var _cuts: Array[AABB] = []
var _rubble: Array[MeshInstance3D] = []
var _rubble_i: int = 0
var _chunks: Array[MeshInstance3D] = []
var _focus: Vector3 = Vector3.ZERO
var _last_point: Vector3 = Vector3.ZERO

func _ready() -> void:
	add_to_group("destruction")

func budget_for(level: String) -> int:
	match level:
		"low":
			return 8
		"high":
			return 48
		_:
			return 24

func sections_for(level: String) -> int:
	match level:
		"low":
			return 2
		"high":
			return 4
		_:
			return 3

func section_index(height: float, y: float, sections: int) -> int:
	if height <= 0.1 or sections < 1:
		return 0
	var t := clampf(y / height, 0.0, 0.999)
	return mini(sections - 1, int(t * float(sections)))

func bind(city: CityData) -> void:
	_buildings.clear()
	for building in city.data.get("buildings", []):
		if str(building.get("side", "")) != "cn":
			continue
		var foot: Array = building.get("footprint", [])
		if foot.size() < 3:
			continue
		var c: Array = building.get("centroid", [0, 0])
		_buildings.append({
			"id": int(building.get("id", 0)),
			"height": float(building.get("height", 9.0)),
			"x": float(c[0]),
			"z": float(c[1]),
			"foot": foot,
		})
	_rebuild_active(Vector3.ZERO)
	GameState.quality_changed.connect(func(_l): _rebuild_active(_focus))

func _rebuild_active(focus: Vector3) -> void:
	_focus = focus
	var keep := {}
	for id in _active:
		if _active[id].get("hurt", 0.0) > 0.0:
			keep[id] = _active[id]
	var ranked: Array = _buildings.duplicate()
	ranked.sort_custom(func(a, b):
		var da := Vector2(a.x, a.z).distance_squared_to(Vector2(focus.x, focus.z))
		var db := Vector2(b.x, b.z).distance_squared_to(Vector2(focus.x, focus.z))
		return da < db
	)
	var cap := budget_for(GameState.quality)
	var sections := sections_for(GameState.quality)
	_active = keep
	var added := 0
	for building in ranked:
		if _active.has(building.id):
			continue
		if added >= cap:
			break
		var hp: Array[float] = []
		hp.resize(sections)
		hp.fill(100.0)
		_active[building.id] = {
			"building": building,
			"sections": sections,
			"hp": hp,
			"hurt": 0.0,
		}
		added += 1

func damage_at(point: Vector3, radius: float, amount: float) -> void:
	_last_point = point
	_rebuild_active(point)
	for id in _active.keys():
		var state: Dictionary = _active[id]
		var building: Dictionary = state.building
		var dx: float = float(building.x) - point.x
		var dz: float = float(building.z) - point.z
		if dx * dx + dz * dz > radius * radius:
			continue
		var section := section_index(building.height, point.y, int(state.sections))
		_hurt_section(id, section, amount)

func _ensure(id: int) -> bool:
	if _active.has(id):
		return true
	for building in _buildings:
		if building.id != id:
			continue
		var sections := sections_for(GameState.quality)
		var hp: Array[float] = []
		hp.resize(sections)
		hp.fill(100.0)
		_active[id] = {"building": building, "sections": sections, "hp": hp, "hurt": 0.0}
		return true
	return false

func collapse_building(id: int, hint: Vector3 = Vector3(1.0e8, 0, 0)) -> void:
	if hint.x < 1.0e7:
		_last_point = hint
	if not _ensure(id):
		return
	var state: Dictionary = _active[id]
	for i in int(state.sections):
		_hurt_section(id, i, 400.0)

func breach_facade(id: int, hint: Vector3) -> void:
	# Street level only, so the upper floors stay standing and the hole reads.
	_last_point = hint
	if not _ensure(id):
		return
	_hurt_section(id, 0, 400.0)
	_hurt_section(id, 1, 55.0)

func nearest_id(point: Vector3) -> int:
	var best := -1
	var best_d := 1e12
	for building in _buildings:
		var d := Vector2(building.x, building.z).distance_squared_to(Vector2(point.x, point.z))
		if d < best_d:
			best_d = d
			best = building.id
	return best

func _hurt_section(id: int, section: int, amount: float) -> void:
	var state: Dictionary = _active[id]
	var hp: Array = state.hp
	if section < 0 or section >= hp.size() or float(hp[section]) <= 0.0:
		return
	hp[section] = float(hp[section]) - amount
	state.hurt = float(state.hurt) + amount
	var building: Dictionary = state.building
	var sections := int(state.sections)
	var y0: float = float(building.height) * float(section) / float(sections)
	var y1: float = float(building.height) * float(section + 1) / float(sections)
	var at := Vector3(float(building.x), (y0 + y1) * 0.5, float(building.z))
	if float(hp[section]) > 0.0:
		_mark_crack(at, clampf(1.0 - float(hp[section]) / 100.0, 0.15, 1.0))
		return
	hp[section] = 0.0
	_mark_crack(Vector3(at.x, y1 + 1.5, at.z), 0.7)
	_carve(building, y0, y1)
	var hint := _last_point
	if hint == Vector3.ZERO:
		hint = Vector3(float(building.x), 0.0, float(building.z))
	_drop_chunk(building, y0, y1, hint)
	_rubble_at(building, y0, hint)
	var standing := false
	for value in hp:
		if float(value) > 0.0:
			standing = true
	if not standing:
		_drop_collision(id)

func _carve(building: Dictionary, y0: float, y1: float) -> void:
	var bounds := _bounds(building.foot)
	var box := AABB(Vector3(bounds.position.x - 0.4, y0 - 0.15, bounds.position.y - 0.4), Vector3(bounds.size.x + 0.8, y1 - y0 + 0.3, bounds.size.y + 0.8))
	_cuts.append(box)
	while _cuts.size() > 8:
		_cuts.pop_front()
	_push_uniforms()

func _push_uniforms() -> void:
	var view := get_tree().get_first_node_in_group("city_view")
	if view == null or not ("cut_materials" in view):
		return
	var mats: Array = view.cut_materials
	var count := mini(8, _cuts.size())
	for mat in mats:
		mat.set_shader_parameter("cuts", float(count))
		for i in 8:
			var mn := Vector3.ZERO
			var mx := Vector3.ZERO
			if i < _cuts.size():
				mn = _cuts[i].position
				mx = _cuts[i].position + _cuts[i].size
			mat.set_shader_parameter("a%d" % i, mn)
			mat.set_shader_parameter("b%d" % i, mx)
	_hide_signs()

func _mark_crack(at: Vector3, amount: float) -> void:
	var view := get_tree().get_first_node_in_group("city_view")
	if view == null or not ("cut_materials" in view):
		return
	for mat in view.cut_materials:
		mat.set_shader_parameter("crack_at", at)
		mat.set_shader_parameter("crack", amount)

func _hide_signs() -> void:
	for node in get_tree().get_nodes_in_group("street_sign"):
		if not node is Node3D:
			continue
		var at: Vector3 = (node as Node3D).global_position
		var gone := false
		for box in _cuts:
			if box.grow(0.8).has_point(at):
				gone = true
				break
		node.visible = not gone

func _bounds(foot: Array) -> Rect2:
	var min_x := 1e9
	var min_z := 1e9
	var max_x := -1e9
	var max_z := -1e9
	for p in foot:
		min_x = minf(min_x, float(p[0]))
		max_x = maxf(max_x, float(p[0]))
		min_z = minf(min_z, float(p[1]))
		max_z = maxf(max_z, float(p[1]))
	return Rect2(min_x, min_z, max_x - min_x, max_z - min_z)

func _drop_chunk(building: Dictionary, y0: float, y1: float, point: Vector3) -> void:
	var edge := _nearest_edge(building.foot, point)
	var mesh := _slab(edge, y0, y1)
	var body := MeshInstance3D.new()
	body.mesh = mesh
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.62, 0.56, 0.46)
	mat.roughness = 0.92
	body.material_override = mat
	add_child(body)
	var normal: Vector2 = edge.normal
	var tangent: Vector2 = edge.tangent
	var slide := float(_chunks.size()) * 3.2
	var fall := Vector3(normal.x * 6.5 + tangent.x * slide, -y0 + 0.5, normal.y * 6.5 + tangent.y * slide)
	var tilt := Vector3(72, 6, 18)
	if GameState.shot_mode:
		body.position = fall
		body.rotation_degrees = tilt
	else:
		var tween := create_tween()
		tween.tween_property(body, "position", fall, 0.7)
		tween.parallel().tween_property(body, "rotation_degrees", tilt, 0.7)
	_chunks.append(body)
	while _chunks.size() > 12:
		var old: MeshInstance3D = _chunks.pop_front()
		if is_instance_valid(old):
			old.queue_free()
	var mid: Vector2 = edge.mid
	_smoke(Vector3(mid.x + normal.x * 1.5, y0 + 1.2, mid.y + normal.y * 1.5))

func _slab(edge: Dictionary, y0: float, y1: float) -> ArrayMesh:
	var mid: Vector2 = edge.mid
	var tangent: Vector2 = edge.tangent
	var normal: Vector2 = edge.normal
	var half := minf(8.0, float(edge.length) * 0.45)
	var c0 := mid - tangent * half
	var c1 := mid + tangent * half
	var push := normal * 0.45
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	_quad(st, Vector3(c0.x, y0, c0.y), Vector3(c1.x, y0, c1.y), Vector3(c1.x, y1, c1.y), Vector3(c0.x, y1, c0.y))
	_quad(st, Vector3(c0.x + push.x, y1, c0.y + push.y), Vector3(c1.x + push.x, y1, c1.y + push.y), Vector3(c1.x + push.x, y0, c1.y + push.y), Vector3(c0.x + push.x, y0, c0.y + push.y))
	st.generate_normals()
	return st.commit()

func _quad(st: SurfaceTool, p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3) -> void:
	st.add_vertex(p0)
	st.add_vertex(p3)
	st.add_vertex(p2)
	st.add_vertex(p0)
	st.add_vertex(p2)
	st.add_vertex(p1)

func _nearest_edge(foot: Array, point: Vector3) -> Dictionary:
	var best_d := 1.0e12
	var mid := Vector2(point.x, point.z)
	var tangent := Vector2(1, 0)
	var normal := Vector2(0, 1)
	var length := 4.0
	var n := foot.size()
	var at := Vector2(point.x, point.z)
	for i in n:
		var a := Vector2(float(foot[i][0]), float(foot[i][1]))
		var b := Vector2(float(foot[(i + 1) % n][0]), float(foot[(i + 1) % n][1]))
		var ab := b - a
		var len := ab.length()
		if len < 0.2:
			continue
		var t := clampf((at - a).dot(ab) / (len * len), 0.0, 1.0)
		var p := a + ab * t
		var d := p.distance_squared_to(at)
		if d < best_d:
			best_d = d
			mid = p
			tangent = ab / len
			normal = Vector2(tangent.y, -tangent.x)
			length = len
	if normal.dot(at - mid) < 0.0:
		normal = -normal
	return {"mid": mid, "normal": normal, "tangent": tangent, "length": length}

func _rubble_at(building: Dictionary, y0: float, point: Vector3) -> void:
	var edge := _nearest_edge(building.foot, point)
	var mid: Vector2 = edge.mid
	var normal: Vector2 = edge.normal
	var tangent: Vector2 = edge.tangent
	var pieces := 2 if GameState.quality == "low" else 4
	for k in pieces:
		if _rubble.size() >= RUBBLE_CAP and k > 0:
			break
		var mesh := _take_rubble()
		var along := (float(k) - float(pieces) * 0.5) * 2.4
		mesh.position = Vector3(mid.x + normal.x * (2.2 + float(k) * 0.7) + tangent.x * along, 0.45 + float(k % 2) * 0.55, mid.y + normal.y * (2.2 + float(k) * 0.7) + tangent.y * along)
		mesh.rotation = Vector3(0.4 * float(k % 2), float(k) * 0.7, 0.25)
		mesh.visible = true
		if mesh.get_node_or_null("Hit") == null:
			var body := StaticBody3D.new()
			body.name = "Hit"
			body.collision_layer = 1
			var col := CollisionShape3D.new()
			var box := BoxShape3D.new()
			box.size = Vector3(1.8, 0.9, 1.3)
			col.shape = box
			body.add_child(col)
			mesh.add_child(body)
	if y0 < 8.0:
		_fire(Vector3(mid.x + normal.x * 1.4, 0.7, mid.y + normal.y * 1.4))

func _take_rubble() -> MeshInstance3D:
	if _rubble.size() < RUBBLE_CAP:
		var inst := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(1.6, 0.7, 1.1)
		inst.mesh = box
		var mat := StandardMaterial3D.new()
		mat.albedo_color = Color(0.42, 0.4, 0.38)
		mat.roughness = 1.0
		inst.material_override = mat
		add_child(inst)
		_rubble.append(inst)
	var mesh := _rubble[_rubble_i % _rubble.size()]
	_rubble_i += 1
	return mesh

func _fire(at: Vector3) -> void:
	var flame := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(3.2, 2.4, 1.6)
	flame.mesh = box
	flame.position = at
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = Color(1.0, 0.32, 0.05)
	mat.emission_enabled = true
	mat.emission = Color(1.0, 0.28, 0.02)
	mat.emission_energy_multiplier = 3.0
	flame.material_override = mat
	add_child(flame)
	_chunks.append(flame)

func _smoke(at: Vector3) -> void:
	var puff := GPUParticles3D.new()
	puff.position = at
	puff.amount = 16
	puff.lifetime = 2.4
	puff.one_shot = true
	puff.explosiveness = 0.8
	var proc := ParticleProcessMaterial.new()
	proc.direction = Vector3(0.2, 1, 0)
	proc.spread = 20.0
	proc.initial_velocity_min = 2.0
	proc.initial_velocity_max = 5.0
	proc.gravity = Vector3(0.3, 0.4, 0)
	proc.scale_min = 1.5
	proc.scale_max = 3.5
	proc.color = Color(0.25, 0.22, 0.18, 0.55)
	puff.process_material = proc
	var quad := QuadMesh.new()
	quad.size = Vector2(1.4, 1.4)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
	mat.albedo_color = Color(0.4, 0.36, 0.3, 0.5)
	quad.material = mat
	puff.draw_pass_1 = quad
	add_child(puff)
	puff.restart()
	var timer := get_tree().create_timer(3.0)
	timer.timeout.connect(puff.queue_free)

func _drop_collision(id: int) -> void:
	for body in get_tree().get_nodes_in_group("building_body"):
		if int(body.get_meta("osm_id", -1)) == id:
			body.collision_layer = 0
