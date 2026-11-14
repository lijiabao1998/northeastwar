extends RefCounted
class_name WinterFigure
## Low-poly on purpose. The readable part is the silhouette: a long quilted
## coat and a fur hat for civilians, white winter camouflage, a helmet and a
## rifle for infantry. Polygon count is not the point.

static func attach(parent: Node3D, kind: String, cloth: Color) -> void:
	var root := Node3D.new()
	root.scale = Vector3(1.2, 1.2, 1.2)
	parent.add_child(root)
	if kind == "soldier":
		_soldier(root)
	else:
		_civilian(root, cloth)

static func _civilian(root: Node3D, cloth: Color) -> void:
	_box(root, Vector3(0, 0.95, 0), Vector3(0.62, 1.05, 0.38), cloth)
	_box(root, Vector3(0, 0.55, 0.02), Vector3(0.66, 0.28, 0.42), cloth.lightened(0.08))
	_box(root, Vector3(-0.28, 1.05, 0), Vector3(0.16, 0.55, 0.22), cloth)
	_box(root, Vector3(0.28, 1.05, 0), Vector3(0.16, 0.55, 0.22), cloth)
	_box(root, Vector3(-0.12, 0.32, 0), Vector3(0.16, 0.5, 0.2), cloth.darkened(0.25))
	_box(root, Vector3(0.12, 0.32, 0), Vector3(0.16, 0.5, 0.2), cloth.darkened(0.25))
	_box(root, Vector3(-0.12, 0.08, 0.05), Vector3(0.18, 0.1, 0.28), Color(0.1, 0.09, 0.08))
	_box(root, Vector3(0.12, 0.08, 0.05), Vector3(0.18, 0.1, 0.28), Color(0.1, 0.09, 0.08))
	_box(root, Vector3(0, 1.64, 0), Vector3(0.36, 0.2, 0.36), Color(0.42, 0.32, 0.2))
	_box(root, Vector3(-0.16, 1.52, 0), Vector3(0.1, 0.16, 0.08), Color(0.35, 0.28, 0.2))
	_box(root, Vector3(0.16, 1.52, 0), Vector3(0.1, 0.16, 0.08), Color(0.35, 0.28, 0.2))
	_box(root, Vector3(0, 1.48, 0.02), Vector3(0.12, 0.12, 0.12), Color(0.55, 0.42, 0.34))
	_box(root, Vector3(0, 1.22, 0.16), Vector3(0.22, 0.1, 0.06), Color(0.55, 0.08, 0.08))

static func _soldier(root: Node3D) -> void:
	var white := Color(0.82, 0.84, 0.8)
	var grey := Color(0.62, 0.66, 0.64)
	_box(root, Vector3(0, 0.98, 0), Vector3(0.52, 0.85, 0.32), white)
	_box(root, Vector3(0, 1.15, 0.12), Vector3(0.46, 0.4, 0.12), grey)
	_box(root, Vector3(-0.24, 1.05, 0), Vector3(0.12, 0.45, 0.16), white)
	_box(root, Vector3(0.24, 1.05, 0), Vector3(0.12, 0.45, 0.16), white)
	_box(root, Vector3(-0.11, 0.32, 0), Vector3(0.14, 0.52, 0.16), grey)
	_box(root, Vector3(0.11, 0.32, 0), Vector3(0.14, 0.52, 0.16), grey)
	_box(root, Vector3(0, 1.62, 0), Vector3(0.24, 0.16, 0.26), Color(0.55, 0.58, 0.54))
	_box(root, Vector3(0.16, 1.15, -0.28), Vector3(0.05, 0.05, 0.7), Color(0.08, 0.08, 0.09))

static func _box(parent: Node3D, pos: Vector3, size: Vector3, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := BoxMesh.new()
	mesh.size = size
	mi.mesh = mesh
	mi.position = pos
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.8
	mi.material_override = mat
	parent.add_child(mi)
