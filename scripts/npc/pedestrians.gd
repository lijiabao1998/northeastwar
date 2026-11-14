extends Node3D
class_name Pedestrians
## Ordinary people on the mapped sidewalks. Nobody here is a soldier.

const COATS := [
	Color(0.18, 0.19, 0.22),
	Color(0.12, 0.16, 0.26),
	Color(0.28, 0.16, 0.12),
	Color(0.32, 0.1, 0.1),
	Color(0.22, 0.22, 0.24),
]
const BEFORE_ZH := [
	"这风跟刀子似的。",
	"对岸今晚亮得早。",
	"下班了？去江边啊。",
	"店里暖气刚好，一出来就透心凉。",
	"中央街这砖地，雪一化就滑。",
]
const BEFORE_EN := [
	"The wind cuts.",
	"The other bank lit up early tonight.",
	"Off work? Heading to the river?",
	"The shop was warm. Out here it goes straight through you.",
	"This brick gets slick the moment the snow melts.",
]
const AFTER_ZH := [
	"你听见没有？江那边！",
	"别站着，往南走！",
	"手机弹警报了。",
	"回家，把窗户关严。",
	"别往江边去。",
]
const AFTER_EN := [
	"Did you hear that? Across the river!",
	"Don't just stand there. Go south!",
	"There's an alert on my phone.",
	"Go home. Shut the windows tight.",
	"Stay off the riverbank.",
]

var _people: Array[Node] = []

func build(data: Dictionary) -> void:
	var paths: Array = data.get("paths", [])
	var usable: Array = []
	for path in paths:
		if path.size() >= 4:
			usable.append(path)
	if usable.is_empty():
		return
	var count := 10 if GameState.quality == "low" else 16
	for i in count:
		var path: Array = usable[i % usable.size()]
		var person := CharacterBody3D.new()
		person.collision_layer = 4
		person.collision_mask = 1
		var shape := CollisionShape3D.new()
		var capsule := CapsuleShape3D.new()
		capsule.radius = 0.28
		capsule.height = 1.6
		shape.shape = capsule
		shape.position = Vector3(0, 0.85, 0)
		person.add_child(shape)
		var start: Array = path[randi() % path.size()]
		person.position = Vector3(float(start[0]), 0.05, float(start[1]))
		var visual := Node3D.new()
		person.add_child(visual)
		_coat(visual, COATS[i % COATS.size()], 0.92 + (i % 5) * 0.03)
		var walker := Pedestrian.new()
		walker.path = path
		walker.index = randi() % path.size()
		walker.lines_zh = BEFORE_ZH
		walker.lines_en = BEFORE_EN
		person.add_child(walker)
		person.add_to_group("npc")
		add_child(person)
		_people.append(person)

func _coat(parent: Node3D, color: Color, scale_y: float) -> void:
	parent.scale = Vector3(1, scale_y, 1)
	_box(parent, Vector3(0, 0.95, 0), Vector3(0.48, 0.75, 0.3), color)
	_box(parent, Vector3(0, 1.4, 0.02), Vector3(0.2, 0.2, 0.2), Color(0.7, 0.56, 0.46))
	_box(parent, Vector3(0, 1.5, 0.05), Vector3(0.28, 0.14, 0.28), Color(0.08, 0.07, 0.06))
	_box(parent, Vector3(-0.14, 0.4, 0), Vector3(0.13, 0.5, 0.15), Color(0.07, 0.07, 0.08))
	_box(parent, Vector3(0.14, 0.4, 0), Vector3(0.13, 0.5, 0.15), Color(0.07, 0.07, 0.08))

func _box(parent: Node3D, pos: Vector3, size: Vector3, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := BoxMesh.new()
	mesh.size = size
	mi.mesh = mesh
	mi.position = pos
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.85
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(mi)

class Pedestrian extends Node:
	var path: Array = []
	var index: int = 0
	var direction: int = 1
	var lines_zh: Array = []
	var lines_en: Array = []
	var _body: CharacterBody3D

	func _ready() -> void:
		_body = get_parent() as CharacterBody3D

	func speak() -> void:
		var panicked := GameState.alert_stage >= 2
		var pool_zh: Array = Pedestrians.AFTER_ZH if panicked else lines_zh
		var pool_en: Array = Pedestrians.AFTER_EN if panicked else lines_en
		if pool_zh.is_empty():
			return
		var i := randi() % pool_zh.size()
		GameState.show_subtitle(str(pool_zh[i]), str(pool_en[i]))

	func _physics_process(delta: float) -> void:
		if GameState.paused or not GameState.started_play or path.is_empty():
			return
		var speed := 1.15
		var target := Vector3.ZERO
		if GameState.alert_stage >= 2:
			speed = 3.3
			target = _body.global_position + Vector3(randf_range(-0.4, 0.4), 0, -1.0)
		else:
			var pt: Array = path[index]
			target = Vector3(float(pt[0]), _body.global_position.y, float(pt[1]))
			if _body.global_position.distance_to(target) < 1.3:
				index += direction
				if index >= path.size():
					index = path.size() - 2
					direction = -1
				elif index < 0:
					index = 1
					direction = 1
				return
		var wish := target - _body.global_position
		wish.y = 0
		if wish.length() < 0.05:
			return
		wish = wish.normalized()
		_body.velocity.x = wish.x * speed
		_body.velocity.z = wish.z * speed
		_body.velocity.y -= 20.0 * delta
		_body.move_and_slide()
		var visual := _body.get_child(1)
		if visual is Node3D:
			(visual as Node3D).rotation.y = atan2(-wish.x, -wish.z)
