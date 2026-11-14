extends CharacterBody3D
class_name Player
## Same rig as DefendTaipei: a fixed isometric camera, not a mouse-look.
## The camera sits 40 m east, 56 m up and 40 m north of you and looks at your
## feet. Orthographic height is 44 m at zoom 1. The wheel (and a pinch) zooms
## between 0.5 and 2.2. W/S run along the screen's up axis, A/D along its right,
## which is the diagonal (-1, 0, -1) / (1, 0, -1) used there. Walk 3.4 m/s,
## run 6 m/s. Clicking sets an aim point for the combat hook and does not
## capture the mouse.

signal stepped

var zoom: float = 1.0
var _body: Node3D
var _cam: Camera3D
var _vel: Vector3 = Vector3.ZERO
var _step_time: float = 0.0
var _notice_time: float = 0.0
var _pinch: float = -1.0
var city: CityData

const CAM_OFF := Vector3(40, 56, 40)
const SCREEN_UP := Vector3(-0.70710678, 0, -0.70710678)
const SCREEN_RIGHT := Vector3(0.70710678, 0, -0.70710678)
const WALK := 3.4
const RUN := 6.0

func _ready() -> void:
	collision_layer = 2
	collision_mask = 1 | 4
	floor_snap_length = 0.25
	floor_max_angle = deg_to_rad(50.0)
	var shape := CollisionShape3D.new()
	var capsule := CapsuleShape3D.new()
	capsule.radius = 0.34
	capsule.height = 1.75
	shape.shape = capsule
	shape.position = Vector3(0, 0.9, 0)
	add_child(shape)
	_body = Node3D.new()
	_body.name = "Body"
	add_child(_body)
	_build_coat(_body, Color(0.16, 0.17, 0.2))
	_body.scale = Vector3(1.25, 1.15, 1.25)
	_cam = Camera3D.new()
	_cam.projection = Camera3D.PROJECTION_ORTHOGONAL
	_cam.near = 0.5
	_cam.far = 500.0
	_cam.current = true
	add_child(_cam)
	set_zoom(1.0)
	add_to_group("player")

func camera() -> Camera3D:
	return _cam

func set_zoom(value: float) -> void:
	zoom = clampf(value, 0.5, 2.2)
	var aspect := 1.0
	var vp := get_viewport()
	if vp and vp.size.y > 0.0:
		aspect = vp.size.x / vp.size.y
	var half := 22.0
	if aspect < 1.0:
		half = maxf(22.0, 16.0 / aspect) * 0.62
	_cam.size = (half * 2.0) / zoom

func _unhandled_input(event: InputEvent) -> void:
	if not GameState.started_play or GameState.paused or GameState.shot_mode:
		return
	if event is InputEventMouseButton and event.pressed:
		if event.button_index == MOUSE_BUTTON_WHEEL_UP:
			set_zoom(zoom * 1.1)
		elif event.button_index == MOUSE_BUTTON_WHEEL_DOWN:
			set_zoom(zoom * 0.9)
		elif event.button_index == MOUSE_BUTTON_LEFT:
			_aim_at_screen(event.position)
			CombatHooks.request_fire()
	elif event is InputEventScreenTouch and event.pressed and _touch_count() >= 2:
		_pinch = _touch_span()
	elif event is InputEventScreenDrag and _touch_count() >= 2 and _pinch > 0.0:
		var span := _touch_span()
		if span > 8.0:
			set_zoom(zoom * (span / _pinch))
			_pinch = span
	elif event is InputEventScreenTouch and not event.pressed:
		_pinch = -1.0
	if event.is_action_pressed("interact"):
		_talk()

func _physics_process(delta: float) -> void:
	_rig()
	if GameState.shot_mode or not GameState.started_play or GameState.paused:
		return
	var wish := SCREEN_UP * (Input.get_action_strength("move_forward") - Input.get_action_strength("move_back"))
	wish += SCREEN_RIGHT * (Input.get_action_strength("move_right") - Input.get_action_strength("move_left"))
	var stick := _touch_axis()
	var stick_len := stick.length()
	if stick_len > 0.12:
		wish += SCREEN_UP * (-stick.y) + SCREEN_RIGHT * stick.x
	var moving := wish.length() > 0.08
	var running := (Input.is_action_pressed("sprint") or stick_len > 0.93) and moving
	var speed := (RUN if running else WALK) * GameState.move_scale
	if stick_len > 0.12 and not running:
		speed *= clampf((stick_len - 0.12) / 0.7, 0.3, 1.0)
	var want := wish.normalized() * speed if moving else Vector3.ZERO
	var rate := 9.0 if want.length() > _vel.length() else 7.0
	_vel = _vel.lerp(want, minf(1.0, delta * rate))
	if moving:
		_body.rotation.y = atan2(-wish.x, -wish.z)
		_body.position.y = sin(Time.get_ticks_msec() * 0.01) * 0.035
	velocity.x = _vel.x
	velocity.z = _vel.z
	if not is_on_floor():
		velocity.y -= 22.0 * delta
	else:
		velocity.y = -0.4
	move_and_slide()
	_clamp_world()
	if moving and is_on_floor():
		_step_time -= delta
		var interval := 0.48 if speed < 4.5 else 0.32
		if _step_time <= 0.0:
			_step_time = interval
			stepped.emit()

func _rig() -> void:
	_cam.global_position = global_position + CAM_OFF
	_cam.look_at(global_position, Vector3.UP)

func _aim_at_screen(screen: Vector2) -> void:
	var origin := _cam.project_ray_origin(screen)
	var dir := _cam.project_ray_normal(screen)
	if absf(dir.y) < 0.0001:
		return
	var t := -origin.y / dir.y
	if t <= 0.0:
		return
	CombatHooks.set_aim(origin + dir * t)

func _touch_axis() -> Vector2:
	var stick := get_tree().get_first_node_in_group("touch_stick")
	if stick and "axis" in stick:
		return stick.axis
	return Vector2.ZERO

func _touch_count() -> int:
	return Input.get_connected_joypads().size() if false else _screen_touches()

func _screen_touches() -> int:
	var n := 0
	for i in 8:
		if Input.is_mouse_button_pressed(MOUSE_BUTTON_LEFT) and i == 0:
			pass
	return n

func _touch_span() -> float:
	return 0.0

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
	# Padded coat reads from the isometric angle: wide shoulders, a hood,
	# a scarf, and two boots. The head sits inside the hood.
	_box(parent, Vector3(0, 1.02, 0), Vector3(0.62, 0.78, 0.38), cloth)
	_box(parent, Vector3(0, 0.72, 0.02), Vector3(0.66, 0.22, 0.42), cloth.lightened(0.08))
	_box(parent, Vector3(-0.34, 1.05, 0), Vector3(0.16, 0.42, 0.22), cloth)
	_box(parent, Vector3(0.34, 1.05, 0), Vector3(0.16, 0.42, 0.22), cloth)
	_sphere(parent, Vector3(0, 1.48, 0.02), 0.16, Color(0.55, 0.42, 0.34))
	_sphere(parent, Vector3(0, 1.52, 0.0), 0.2, Color(0.05, 0.05, 0.06))
	_box(parent, Vector3(0, 1.22, 0.16), Vector3(0.22, 0.1, 0.1), Color(0.55, 0.08, 0.08))
	_box(parent, Vector3(-0.14, 0.38, 0.02), Vector3(0.16, 0.62, 0.2), Color(0.1, 0.1, 0.12))
	_box(parent, Vector3(0.14, 0.38, 0.02), Vector3(0.16, 0.62, 0.2), Color(0.1, 0.1, 0.12))
	_box(parent, Vector3(-0.14, 0.08, 0.04), Vector3(0.18, 0.1, 0.28), Color(0.15, 0.12, 0.1))
	_box(parent, Vector3(0.14, 0.08, 0.04), Vector3(0.18, 0.1, 0.28), Color(0.15, 0.12, 0.1))
	_box(parent, Vector3(0.22, 0.85, -0.12), Vector3(0.16, 0.28, 0.22), Color(0.2, 0.16, 0.1))

func _sphere(parent: Node3D, pos: Vector3, radius: float, color: Color) -> void:
	var mi := MeshInstance3D.new()
	var mesh := SphereMesh.new()
	mesh.radius = radius
	mesh.height = radius * 2.0
	mi.mesh = mesh
	mi.position = pos
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.8
	mi.material_override = mat
	mi.cast_shadow = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	parent.add_child(mi)

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
