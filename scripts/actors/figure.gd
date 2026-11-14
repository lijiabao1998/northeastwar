extends RefCounted
class_name WinterFigure
## Quaternius CC0 characters (Universal / casual pack, redistributed as glb).
## They are real human meshes. The pack does not include a padded Northeast
## winter coat or a modern infantry uniform, so civilians stay in the casual
## outfits and the infantry figure is the worker mesh plus a rifle.

const MALE := "res://assets/models/quaternius/Casual_Male.glb"
const FEMALE := "res://assets/models/quaternius/Casual_Female.glb"
const WORKER := "res://assets/models/quaternius/Worker_Male.glb"

static func attach(parent: Node3D, kind: String, cloth: Color) -> void:
	var path := MALE
	if kind == "soldier":
		path = WORKER
	elif cloth.b > cloth.r:
		path = FEMALE
	var packed: PackedScene = load(path)
	if packed == null:
		return
	var inst := packed.instantiate()
	parent.add_child(inst)
	if kind == "soldier" and inst is Node3D:
		_rifle(inst as Node3D, _height(inst as Node3D))
	if inst is Node3D:
		_fit(inst as Node3D)

static func _fit(inst: Node3D) -> void:
	var height := _height(inst)
	if height < 0.05:
		return
	var scale := 1.78 / height
	inst.scale = Vector3.ONE * scale
	var bottom := _bottom(inst) * scale
	inst.position.y = -bottom

static func _height(node: Node) -> float:
	var box := _merged(node, Transform3D.IDENTITY)
	return box.size.y

static func _bottom(node: Node) -> float:
	var box := _merged(node, Transform3D.IDENTITY)
	return box.position.y

static func _merged(node: Node, parent: Transform3D) -> AABB:
	var here := parent
	if node is Node3D and node != node.get_parent():
		here = parent * (node as Node3D).transform
	var box := AABB()
	var any := false
	if node is MeshInstance3D:
		var mi := node as MeshInstance3D
		if mi.mesh:
			box = here * mi.mesh.get_aabb()
			any = true
	for child in node.get_children():
		var sub := _merged(child, here if node is Node3D else parent)
		if sub.size.length() < 0.001:
			continue
		if not any:
			box = sub
			any = true
		else:
			box = box.merge(sub)
	if not any:
		return AABB()
	return box

static func _rifle(inst: Node3D, height: float) -> void:
	var gun := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(height * 0.035, height * 0.035, height * 0.42)
	gun.mesh = box
	gun.position = Vector3(height * 0.12, height * 0.58, height * 0.16)
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.08, 0.08, 0.09)
	mat.metallic = 0.4
	mat.roughness = 0.45
	gun.material_override = mat
	if inst is Node3D:
		(inst as Node3D).add_child(gun)
