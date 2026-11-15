extends RefCounted
class_name WinterFigure
## Low-poly, but the parts read: long coat, fur hat or helmet, boots, and a
## rifle on infantry. Russians in the street are white winter camouflage.
## Militia and civilians wear dark coats.

static func attach(parent: Node3D, kind: String, cloth: Color) -> void:
	var root := Node3D.new()
	parent.add_child(root)
	if kind == "soldier":
		_soldier(root)
	else:
		_civilian(root, cloth)

static func _civilian(root: Node3D, cloth: Color) -> void:
	_leg(root, -0.11, cloth.darkened(0.35))
	_leg(root, 0.11, cloth.darkened(0.35))
	_coat(root, cloth)
	_arm(root, -0.34, cloth)
	_arm(root, 0.34, cloth)
	_sphere(root, Vector3(0, 1.52, 0.02), 0.11, Color(0.55, 0.42, 0.34))
	_box(root, Vector3(0, 1.66, 0), Vector3(0.3, 0.12, 0.3), Color(0.4, 0.3, 0.18))
	_box(root, Vector3(-0.14, 1.56, 0.02), Vector3(0.08, 0.16, 0.06), Color(0.4, 0.3, 0.18))
	_box(root, Vector3(0.14, 1.56, 0.02), Vector3(0.08, 0.16, 0.06), Color(0.4, 0.3, 0.18))
	_box(root, Vector3(0, 1.28, 0.16), Vector3(0.2, 0.08, 0.05), Color(0.55, 0.08, 0.08))

static func _soldier(root: Node3D) -> void:
	var white := Color(0.86, 0.88, 0.84)
	var grey := Color(0.62, 0.66, 0.62)
	_leg(root, -0.1, grey)
	_leg(root, 0.1, grey)
	_coat(root, white)
	_arm(root, -0.32, white)
	_arm(root, 0.32, white)
	_box(root, Vector3(0, 1.2, 0.14), Vector3(0.42, 0.32, 0.08), grey)
	_sphere(root, Vector3(0, 1.58, 0.02), 0.12, Color(0.7, 0.74, 0.7))
	_cyl(root, Vector3(0.22, 1.2, 0.05), Vector3(0.28, 1.15, -0.55), 0.025, Color(0.08, 0.08, 0.09))

static func _coat(root: Node3D, cloth: Color) -> void:
	_box(root, Vector3(0, 1.05, 0), Vector3(0.46, 0.55, 0.28), cloth)
	_box(root, Vector3(0, 0.62, 0.02), Vector3(0.58, 0.42, 0.34), cloth.darkened(0.06))

static func _leg(root: Node3D, x: float, color: Color) -> void:
	_box(root, Vector3(x, 0.28, 0), Vector3(0.12, 0.42, 0.14), color)
	_box(root, Vector3(x, 0.06, 0.04), Vector3(0.14, 0.08, 0.24), Color(0.08, 0.07, 0.06))

static func _arm(root: Node3D, x: float, cloth: Color) -> void:
	_box(root, Vector3(x, 1.02, 0.02), Vector3(0.1, 0.42, 0.12), cloth.darkened(0.08))

static func _box(parent: Node3D, pos: Vector3, size: Vector3, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := BoxMesh.new()
	mesh.size = size
	mi.mesh = mesh
	mi.position = pos
	mi.material_override = _paint(color)
	parent.add_child(mi)

static func _sphere(parent: Node3D, pos: Vector3, radius: float, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := SphereMesh.new()
	mesh.radius = radius
	mesh.height = radius * 2.0
	mi.mesh = mesh
	mi.position = pos
	mi.material_override = _paint(color)
	parent.add_child(mi)

static func _cyl(parent: Node3D, a: Vector3, b: Vector3, radius: float, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := CylinderMesh.new()
	mesh.top_radius = radius
	mesh.bottom_radius = radius
	mesh.height = a.distance_to(b)
	mi.mesh = mesh
	# Built from the direction itself: a node that is not in the tree yet cannot look_at.
	mi.transform = Transform3D(Basis.looking_at(b - a, Vector3.UP), (a + b) * 0.5)
	mi.rotate_object_local(Vector3.RIGHT, PI * 0.5)
	mi.material_override = _paint(color)
	parent.add_child(mi)

static func _paint(color: Color) -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.78
	return mat
