extends CharacterBody3D
class_name Player
## Third person. The January 2026 street photos are full of people seen from
## behind, in black hooded coats, on the brick sidewalk. That is the camera:
## over the shoulder, close enough that the tiled facades still fill the frame.
## First person would hide the coat, which is half of how a Dongbei winter reads.

signal stepped

var yaw: float = PI
var pitch: float = -0.08
var _body: Node3D
var _spring: SpringArm3D
var _cam: Camera3D
var _step_time: float = 0.0
var _notice_time: float = 0.0
var city: CityData

const WALK := 2.35
const RUN := 4.3

func _ready() -> void:
	collision_layer = 2
	collision_mask = 1 | 4
	floor_snap_length = 0.25
	floor_max_angle = deg_to_rad(50.0)
	var shape := CollisionShape3D.new()
	var capsule := CapsuleShape3D.new()
	capsule.radius = 0.32
	capsule.height = 1.72
	shape.shape = capsule
	shape.position = Vector3(0, 0.9, 0)
	add_child(shape)
	_body = Node3D.new()
	_body.name = "Body"
	add_child(_body)
	_build_coat(_body, Color(0.06, 0.06, 0.07))
	_spring = SpringArm3D.new()
	_spring.position = Vector3(0, 1.5, 0.15)
	_spring.spring_length = 3.6
	_spring.collision_mask = 1
	_spring.margin = 0.18
	add_child(_spring)
	_cam = Camera3D.new()
	_cam.fov = 62.0
	_cam.near = 0.08
	_cam.far = 4500.0
	_cam.position = Vector3(0.42, 0.12, 0.15)
	_spring.add_child(_cam)
	add_to_group("player")

func camera() -> Camera3D:
	return _cam

func _unhandled_input(event: InputEvent) -> void:
	if not GameState.started_play or GameState.paused or GameState.shot_mode:
		return
	if event is InputEventMouseMotion and Input.mouse_mode == Input.MOUSE_MODE_CAPTURED:
		yaw -= event.relative.x * 0.0022
		pitch = clampf(pitch - event.relative.y * 0.0017, deg_to_rad(-35.0), deg_to_rad(22.0))
	if event.is_action_pressed("interact"):
		_talk()

func _physics_process(delta: float) -> void:
	if GameState.shot_mode:
		_spring.rotation = Vector3(pitch, yaw, 0)
		return
	if not GameState.started_play or GameState.paused:
		return
	var look_x := Input.get_joy_axis(0, JOY_AXIS_RIGHT_X)
	var look_y := Input.get_joy_axis(0, JOY_AXIS_RIGHT_Y)
	if absf(look_x) < 0.18:
		look_x = 0.0
	if absf(look_y) < 0.18:
		look_y = 0.0
	yaw -= look_x * 2.2 * delta
	pitch = clampf(pitch - look_y * 1.6 * delta, deg_to_rad(-35.0), deg_to_rad(22.0))
	if GameState.body_temp < 35.1:
		var shiver := (35.1 - GameState.body_temp) * 0.004
		pitch += sin(Time.get_ticks_msec() * 0.03) * shiver
	_spring.rotation = Vector3(pitch, yaw, 0)
	var forward := -_spring.global_transform.basis.z
	forward.y = 0.0
	forward = forward.normalized()
	var right := _spring.global_transform.basis.x
	right.y = 0.0
	right = right.normalized()
	var wish := right * (Input.get_action_strength("move_right") - Input.get_action_strength("move_left"))
	wish += forward * (Input.get_action_strength("move_forward") - Input.get_action_strength("move_back"))
	var moving := wish.length() > 0.08
	if moving:
		wish = wish.normalized()
		_body.rotation.y = atan2(-wish.x, -wish.z)
		var bob := sin(Time.get_ticks_msec() * 0.012) * 0.04
		_body.position.y = bob
	var speed := RUN if Input.is_action_pressed("sprint") and moving else WALK
	speed *= GameState.move_scale
	velocity.x = wish.x * speed if moving else move_toward(velocity.x, 0.0, 18.0 * delta)
	velocity.z = wish.z * speed if moving else move_toward(velocity.z, 0.0, 18.0 * delta)
	if not is_on_floor():
		velocity.y -= 22.0 * delta
	else:
		velocity.y = -0.4
	move_and_slide()
	_clamp_world()
	if moving and is_on_floor():
		_step_time -= delta
		var interval := 0.46 if speed < 3.0 else 0.32
		if _step_time <= 0.0:
			_step_time = interval
			stepped.emit()

func _clamp_world() -> void:
	if city == null:
		return
	var x := global_position.x
	var z := global_position.z
	var note := ""
	if x < city.walk_min_x:
		x = city.walk_min_x
		note = "edge"
	elif x > city.walk_max_x:
		x = city.walk_max_x
		note = "edge"
	if z < city.walk_min_z:
		z = city.walk_min_z
		note = "edge"
	var limit := city.ice_z(x)
	if z > limit:
		z = limit
		note = "ice"
	global_position.x = x
	global_position.z = z
	_notice_time -= get_physics_process_delta_time()
	if note != "" and _notice_time <= 0.0:
		_notice_time = 5.0
		if note == "ice":
			if GameState.alert_stage >= 2:
				GameState.show_subtitle("江面上不对劲，不能再往前。", "Something is wrong on the ice. No further.")
			else:
				GameState.show_subtitle("再往前就是封冻的主航道了。", "Past this is the frozen main channel.")
		else:
			GameState.show_subtitle("这一带的街，先走到这儿。", "This block is as far as the evening goes.")

func _talk() -> void:
	var best: Node = null
	var best_d := 3.1
	for node in get_tree().get_nodes_in_group("npc"):
		if not node is Node3D:
			continue
		var d := global_position.distance_to((node as Node3D).global_position)
		if d < best_d:
			best_d = d
			best = node
	if best and best.has_method("speak"):
		best.speak()

func _build_coat(parent: Node3D, cloth: Color) -> void:
	_box(parent, Vector3(0, 0.95, 0), Vector3(0.52, 0.78, 0.32), cloth)
	_box(parent, Vector3(0, 1.42, 0.02), Vector3(0.22, 0.22, 0.22), Color(0.72, 0.58, 0.48))
	_box(parent, Vector3(0, 1.5, 0.04), Vector3(0.3, 0.16, 0.3), Color(0.1, 0.08, 0.07))
	_box(parent, Vector3(0, 1.22, 0.12), Vector3(0.18, 0.08, 0.08), Color(0.62, 0.1, 0.1))
	_box(parent, Vector3(0, 1.58, 0.08), Vector3(0.16, 0.08, 0.18), Color(0.35, 0.26, 0.18))
	_box(parent, Vector3(-0.18, 0.42, 0), Vector3(0.14, 0.55, 0.16), Color(0.08, 0.08, 0.09))
	_box(parent, Vector3(0.18, 0.42, 0), Vector3(0.14, 0.55, 0.16), Color(0.08, 0.08, 0.09))
	_box(parent, Vector3(-0.32, 0.95, 0), Vector3(0.12, 0.48, 0.14), cloth)
	_box(parent, Vector3(0.32, 0.95, 0), Vector3(0.12, 0.48, 0.14), cloth)

func _box(parent: Node3D, pos: Vector3, size: Vector3, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := BoxMesh.new()
	mesh.size = size
	mi.mesh = mesh
	mi.position = pos
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.82
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(mi)
