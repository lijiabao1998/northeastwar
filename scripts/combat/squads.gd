extends Node
class_name Squads
## Fictional infantry in small packets, the size seen in open reporting of
## the Ukraine war: about four to eight soldiers, not a parade battalion.
## They patrol, then fire and move. They never score a point for a civilian.

const MIN_SOLDIERS := 4
const MAX_SOLDIERS := 8

var _squads: Array[Node3D] = []
var _cooldown: float = 12.0
var _travel: float = 0.0
var _last: Vector3 = Vector3.ZERO

func _ready() -> void:
	add_to_group("squads")
	CombatHooks.fire_requested.connect(_player_shot)

func _player_shot(point: Vector3) -> void:
	var player := _player()
	if player == null:
		return
	var from := player.global_position + Vector3(0, 1.4, 0)
	var best: Node = null
	var best_t := 1.0
	for soldier in soldiers():
		var chest: Vector3 = soldier.global_position + Vector3(0, 1.0, 0)
		var dir := point - from
		var len := dir.length()
		if len < 0.1:
			continue
		dir /= len
		var along := (chest - from).dot(dir)
		var t := clampf(along / len, 0.0, 1.0)
		if from.lerp(point, t).distance_to(chest) < 0.9 and t < best_t:
			best = soldier
			best_t = t
	_tracer(from, point)
	if best:
		hurt_soldier(best, 28.0)

func _process(delta: float) -> void:
	if GameState.shot_mode or not GameState.started_play or GameState.paused:
		return
	var player := _player()
	if player == null:
		return
	_travel += player.global_position.distance_to(_last)
	_last = player.global_position
	_cooldown -= delta
	if _cooldown <= 0.0 and _travel > 28.0 and _squads.size() < 2:
		_cooldown = randf_range(22.0, 40.0)
		_travel = 0.0
		spawn_near(player.global_position)

func spawn_near(origin: Vector3) -> Node3D:
	var angle := randf() * TAU
	var dist := randf_range(36.0, 62.0)
	return spawn_at(origin + Vector3(cos(angle), 0, sin(angle)) * dist, origin)

func spawn_at(at: Vector3, face: Vector3) -> Node3D:
	CombatHooks.enabled = true
	GameState.under_fire = true
	var squad := Node3D.new()
	squad.position = at
	var count := randi_range(MIN_SOLDIERS, MAX_SOLDIERS)
	for i in count:
		var body := _soldier(i)
		var side := 2.2 * float(i - count / 2)
		body.position = Vector3(side, 0.05, float(i % 2))
		squad.add_child(body)
	add_child(squad)
	_squads.append(squad)
	squad.set_meta("face", face)
	squad.set_meta("mode", "patrol")
	squad.set_meta("grenade", 0.0)
	return squad

func pose_firefight(origin: Vector3, toward: Vector3) -> void:
	var squad := spawn_at(origin, toward)
	var i := 0
	for child in squad.get_children():
		child.set_meta("firing", true)
		var flash := child.get_node_or_null("Flash") as OmniLight3D
		if flash:
			flash.light_energy = 8.0 if i % 2 == 0 else 3.0
		var muzzle := child.get_node_or_null("Muzzle") as MeshInstance3D
		if muzzle:
			muzzle.visible = i % 2 == 0
		i += 1
	squad.set_meta("mode", "fight")

func _process_squads(delta: float) -> void:
	var player := _player()
	if player == null:
		return
	for squad in _squads:
		if not is_instance_valid(squad):
			continue
		var mode := str(squad.get_meta("mode", "patrol"))
		var face: Vector3 = squad.get_meta("face", player.global_position)
		var to_player := player.global_position - squad.global_position
		to_player.y = 0
		var dist := to_player.length()
		if mode == "patrol" and dist < 58.0:
			squad.set_meta("mode", "fight")
			mode = "fight"
			GameState.show_subtitle("巷子里有人开火。", "Someone is shooting in the alley.")
		if mode != "fight":
			continue
		var grenade_t := float(squad.get_meta("grenade", 0.0)) - delta
		squad.set_meta("grenade", grenade_t)
		var index := 0
		for child in squad.get_children():
			if not child is CharacterBody3D:
				continue
			if float(child.get_meta("hp", 0)) <= 0.0:
				continue
			var flank := index % 3 == 2
			var goal := player.global_position
			if flank:
				var side := Vector3(to_player.z, 0, -to_player.x).normalized()
				goal = player.global_position + side * 14.0
			elif index % 2 == 0:
				goal = child.global_position
			_step_soldier(child, goal, delta, index % 2 == 0 or flank)
			if grenade_t <= 0.0 and dist < 18.0 and index == 0:
				squad.set_meta("grenade", 9.0)
				_grenade(child.global_position, player.global_position)
			index += 1

func _physics_process(delta: float) -> void:
	if GameState.shot_mode or not GameState.started_play or GameState.paused:
		return
	_process_squads(delta)

func _step_soldier(body: CharacterBody3D, goal: Vector3, delta: float, shooting: bool) -> void:
	var wish := goal - body.global_position
	wish.y = 0
	var moving := wish.length() > 1.4 and not shooting
	if moving:
		wish = wish.normalized()
		body.velocity.x = wish.x * 2.4
		body.velocity.z = wish.z * 2.4
		body.rotation.y = atan2(-wish.x, -wish.z)
	else:
		body.velocity.x = 0
		body.velocity.z = 0
		if shooting:
			_shoot_at_player(body)
	body.velocity.y -= 20.0 * delta
	body.move_and_slide()

func _shoot_at_player(body: Node3D) -> void:
	var ready_at := float(body.get_meta("next_shot", 0.0))
	if Time.get_ticks_msec() / 1000.0 < ready_at:
		return
	body.set_meta("next_shot", Time.get_ticks_msec() / 1000.0 + randf_range(0.7, 1.5))
	var flash := body.get_node_or_null("Flash") as OmniLight3D
	if flash:
		flash.light_energy = 8.0
	var muzzle := body.get_node_or_null("Muzzle") as MeshInstance3D
	if muzzle:
		muzzle.visible = true
		get_tree().create_timer(0.06).timeout.connect(func():
			if is_instance_valid(muzzle):
				muzzle.visible = false
		)
	var player := _player()
	if player == null:
		return
	var aim := player.global_position + Vector3(randf_range(-1.2, 1.2), 1.0, randf_range(-1.2, 1.2))
	if body.global_position.distance_to(player.global_position) < 42.0 and randf() < 0.55:
		var cover := 0.55 if _near_building(player.global_position) else 1.0
		CombatHooks.hurt_player(randf_range(8.0, 16.0) * cover)
	_tracer(body.global_position + Vector3(0, 1.3, 0), aim)

func hurt_soldier(body: Node, amount: float) -> void:
	var hp := float(body.get_meta("hp", 40.0)) - amount
	body.set_meta("hp", hp)
	if hp <= 0.0:
		body.visible = false
		if body is CollisionObject3D:
			(body as CollisionObject3D).collision_layer = 0

func soldiers() -> Array:
	return get_tree().get_nodes_in_group("soldier")

func _grenade(from: Vector3, to: Vector3) -> void:
	var bomb := MeshInstance3D.new()
	var sphere := SphereMesh.new()
	sphere.radius = 0.12
	bomb.mesh = sphere
	bomb.position = from + Vector3(0, 1.4, 0)
	add_child(bomb)
	var tween := create_tween()
	var mid := (from + to) * 0.5 + Vector3(0, 6, 0)
	tween.tween_property(bomb, "position", mid, 0.35)
	tween.tween_property(bomb, "position", to + Vector3(0, 0.2, 0), 0.35)
	tween.tween_callback(func():
		bomb.queue_free()
		_boom(to, 7.0, 36.0, 70.0)
	)

func _boom(at: Vector3, radius: float, player_dmg: float, building_dmg: float) -> void:
	var guns := get_tree().get_first_node_in_group("ordinance")
	if guns and guns.has_method("burst") and not GameState.shot_mode:
		guns.burst(at, false)
	var destruction := _destruction()
	if destruction:
		destruction.damage_at(at + Vector3(0, 3, 0), radius + 4.0, building_dmg)
	var player := _player()
	if player and player.global_position.distance_to(at) < radius:
		CombatHooks.hurt_player(player_dmg)
	for soldier in soldiers():
		if soldier.global_position.distance_to(at) < radius:
			hurt_soldier(soldier, 80.0)
	GameState.boom.emit()

func _tracer(a: Vector3, b: Vector3) -> void:
	var mesh := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.04, 0.04, a.distance_to(b))
	mesh.mesh = box
	mesh.position = (a + b) * 0.5
	mesh.look_at(b, Vector3.UP)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = Color(1, 0.78, 0.35)
	mat.emission_enabled = true
	mat.emission = Color(1, 0.6, 0.2)
	mat.emission_energy_multiplier = 3.0
	mesh.material_override = mat
	add_child(mesh)
	var timer := get_tree().create_timer(0.12)
	timer.timeout.connect(mesh.queue_free)

func _soldier(index: int) -> CharacterBody3D:
	var body := CharacterBody3D.new()
	body.collision_layer = 4
	body.collision_mask = 1
	body.set_meta("hp", 40.0)
	body.add_to_group("soldier")
	var shape := CollisionShape3D.new()
	var cap := CapsuleShape3D.new()
	cap.radius = 0.32
	cap.height = 1.7
	shape.shape = cap
	shape.position = Vector3(0, 0.9, 0)
	body.add_child(shape)
	var visual := Node3D.new()
	body.add_child(visual)
	WinterFigure.attach(visual, "soldier", Color(0.34, 0.36, 0.26) if index % 2 == 0 else Color(0.42, 0.4, 0.32))
	var muzzle := MeshInstance3D.new()
	muzzle.name = "Muzzle"
	var flash_mesh := SphereMesh.new()
	flash_mesh.radius = 0.12
	flash_mesh.height = 0.24
	muzzle.mesh = flash_mesh
	muzzle.position = Vector3(0.16, 1.12, -0.62)
	var flash_mat := StandardMaterial3D.new()
	flash_mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	flash_mat.albedo_color = Color(1, 0.85, 0.45)
	muzzle.material_override = flash_mat
	muzzle.visible = false
	body.add_child(muzzle)
	var flash := OmniLight3D.new()
	flash.name = "Flash"
	flash.position = Vector3(0, 1.35, -0.4)
	flash.light_color = Color(1, 0.7, 0.3)
	flash.omni_range = 7.0
	flash.light_energy = 0.0
	body.add_child(flash)
	return body

func _player() -> Player:
	var node := get_tree().get_first_node_in_group("player")
	return node as Player

func _destruction() -> Destruction:
	return get_tree().get_first_node_in_group("destruction") as Destruction

func _near_building(point: Vector3) -> bool:
	var destruction := _destruction()
	if destruction == null:
		return false
	return destruction.nearest_id(point) >= 0 and point.distance_to(Vector3(point.x, 0, point.z)) < 100.0
