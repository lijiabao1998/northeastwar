extends RefCounted
class_name WinterFigure
## A person built for the isometric camera: a coat with shoulders, a hood or
## a helmet, separate legs and boots. Civilians get a scarf. Infantry get a
## vest and a rifle. The colour is the coat, not a flat capsule.

static func attach(parent: Node3D, kind: String, cloth: Color) -> void:
	var mi := MeshInstance3D.new()
	mi.mesh = _build(kind, cloth)
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.roughness = 0.72
	mat.metallic = 0.02
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_ON
	parent.add_child(mi)

static func _build(kind: String, cloth: Color) -> ArrayMesh:
	var st := SurfaceTool.new()
	st.begin(Mesh.PRIMITIVE_TRIANGLES)
	var boot := Color(0.08, 0.07, 0.06)
	var trouser := cloth.darkened(0.35)
	var skin := Color(0.55, 0.42, 0.34)
	# Legs, apart, so the hem of the coat does not read as one column.
	_cyl(st, Vector3(-0.12, 0.08, 0.02), Vector3(-0.11, 0.72, 0), 0.07, trouser)
	_cyl(st, Vector3(0.12, 0.08, 0.02), Vector3(0.11, 0.72, 0), 0.07, trouser)
	_box(st, Vector3(-0.12, 0.06, 0.05), Vector3(0.16, 0.1, 0.3), boot)
	_box(st, Vector3(0.12, 0.06, 0.05), Vector3(0.16, 0.1, 0.3), boot)
	# Coat, wider at the shoulders and the hem.
	_taper(st, Vector3(0, 0.78, 0), Vector3(0, 1.38, 0), Vector2(0.5, 0.3), Vector2(0.58, 0.32), cloth)
	_box(st, Vector3(0, 0.74, 0.02), Vector3(0.56, 0.16, 0.34), cloth.lightened(0.06))
	_cyl(st, Vector3(-0.32, 1.22, 0), Vector3(-0.4, 0.78, 0.04), 0.055, cloth)
	_cyl(st, Vector3(0.32, 1.22, 0), Vector3(0.4, 0.78, 0.04), 0.055, cloth)
	_box(st, Vector3(0, 1.58, 0.02), Vector3(0.16, 0.18, 0.16), skin)
	if kind == "soldier":
		var suit := cloth
		var helm := Color(0.2, 0.22, 0.16)
		var vest := Color(0.4, 0.36, 0.26)
		_box(st, Vector3(0, 1.68, 0.02), Vector3(0.22, 0.12, 0.24), helm)
		_box(st, Vector3(0, 1.6, 0.04), Vector3(0.2, 0.08, 0.2), helm.darkened(0.15))
		_box(st, Vector3(0, 1.18, 0.12), Vector3(0.42, 0.36, 0.12), vest)
		# Rifle held across the front, barrel toward -Z.
		var rifle := Color(0.06, 0.06, 0.07)
		_box(st, Vector3(0.16, 1.12, -0.28), Vector3(0.05, 0.05, 0.72), rifle)
		_box(st, Vector3(0.16, 1.05, -0.02), Vector3(0.05, 0.14, 0.16), rifle.lightened(0.08))
	else:
		var hood := Color(0.06, 0.06, 0.07)
		_box(st, Vector3(0, 1.66, -0.02), Vector3(0.26, 0.16, 0.26), hood)
		_box(st, Vector3(0, 1.28, 0.16), Vector3(0.24, 0.08, 0.08), Color(0.62, 0.1, 0.1))
		_box(st, Vector3(0.22, 0.95, -0.16), Vector3(0.16, 0.22, 0.18), cloth.darkened(0.2))
	st.generate_normals()
	return st.commit()

static func _box(st: SurfaceTool, c: Vector3, s: Vector3, color: Color) -> void:
	var h := s * 0.5
	var p := [
		c + Vector3(-h.x, -h.y, -h.z), c + Vector3(h.x, -h.y, -h.z),
		c + Vector3(h.x, h.y, -h.z), c + Vector3(-h.x, h.y, -h.z),
		c + Vector3(-h.x, -h.y, h.z), c + Vector3(h.x, -h.y, h.z),
		c + Vector3(h.x, h.y, h.z), c + Vector3(-h.x, h.y, h.z),
	]
	_face(st, p[0], p[1], p[2], p[3], color)
	_face(st, p[5], p[4], p[7], p[6], color)
	_face(st, p[4], p[0], p[3], p[7], color)
	_face(st, p[1], p[5], p[6], p[2], color)
	_face(st, p[3], p[2], p[6], p[7], color)
	_face(st, p[4], p[5], p[1], p[0], color)

static func _taper(st: SurfaceTool, a: Vector3, b: Vector3, sa: Vector2, sb: Vector2, color: Color) -> void:
	var ax := sa.x * 0.5
	var az := sa.y * 0.5
	var bx := sb.x * 0.5
	var bz := sb.y * 0.5
	var p := [
		a + Vector3(-ax, 0, -az), a + Vector3(ax, 0, -az), a + Vector3(ax, 0, az), a + Vector3(-ax, 0, az),
		b + Vector3(-bx, 0, -bz), b + Vector3(bx, 0, -bz), b + Vector3(bx, 0, bz), b + Vector3(-bx, 0, bz),
	]
	_face(st, p[0], p[1], p[5], p[4], color)
	_face(st, p[3], p[7], p[6], p[2], color)
	_face(st, p[0], p[4], p[7], p[3], color)
	_face(st, p[1], p[2], p[6], p[5], color)
	_face(st, p[4], p[5], p[6], p[7], color)
	_face(st, p[3], p[2], p[1], p[0], color)

static func _cyl(st: SurfaceTool, a: Vector3, b: Vector3, radius: float, color: Color) -> void:
	var axis := b - a
	var len := axis.length()
	if len < 0.001:
		return
	axis /= len
	var up := Vector3.UP
	if absf(axis.dot(up)) > 0.9:
		up = Vector3.RIGHT
	var side := axis.cross(up).normalized()
	var bin := side.cross(axis).normalized()
	var n := 6
	for i in n:
		var t0 := float(i) / float(n) * TAU
		var t1 := float(i + 1) / float(n) * TAU
		var d0 := side * cos(t0) + bin * sin(t0)
		var d1 := side * cos(t1) + bin * sin(t1)
		_face(st, a + d0 * radius, a + d1 * radius, b + d1 * radius, b + d0 * radius, color)

static func _face(st: SurfaceTool, p0: Vector3, p1: Vector3, p2: Vector3, p3: Vector3, color: Color) -> void:
	st.set_color(color)
	st.add_vertex(p0)
	st.set_color(color)
	st.add_vertex(p1)
	st.set_color(color)
	st.add_vertex(p2)
	st.set_color(color)
	st.add_vertex(p0)
	st.set_color(color)
	st.add_vertex(p2)
	st.set_color(color)
	st.add_vertex(p3)
