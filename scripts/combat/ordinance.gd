extends Node
class_name Ordinance
## Shells, an FPV that dives, and one infantry-carrying vehicle.
## The patterns follow open descriptions of the war since 2022: a spotter
## drone, then guns; a disposable quad that flies into the target; armour
## that does not advance without people on foot. Yields here are game
## numbers, not a manual.

var _whistle: AudioStreamPlayer
var _drone: AudioStreamPlayer

func _ready() -> void:
	_whistle = _tone("res://assets/audio/whistle.wav", -4.0)
	_drone = _tone("res://assets/audio/drone.wav", -8.0)
	add_child(_whistle)
	add_child(_drone)

func barrage(center: Vector3, count: int = 4) -> void:
	GameState.show_subtitle("头顶有哨声。炮来了。", "A whistle overhead. Shells.")
	_whistle.play()
	for i in count:
		var delay := 0.45 * float(i)
		var at := center + Vector3(randf_range(-7, 7), 0, randf_range(-7, 7))
		get_tree().create_timer(delay).timeout.connect(func(): _impact(at, 9.0, 28.0, 160.0))

func pose_strike(target: Vector3) -> void:
	var rig := Node3D.new()
	rig.position = target + Vector3(-8, 9, -6)
	_drone_mesh(rig)
	add_child(rig)
	var ball := MeshInstance3D.new()
	var sphere := SphereMesh.new()
	sphere.radius = 3.4
	sphere.height = 6.8
	ball.mesh = sphere
	ball.position = target + Vector3(0, 2.2, 0)
	var glow := StandardMaterial3D.new()
	glow.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	glow.albedo_color = Color(1.0, 0.45, 0.08)
	glow.emission_enabled = true
	glow.emission = Color(1.0, 0.4, 0.05)
	glow.emission_energy_multiplier = 4.0
	ball.material_override = glow
	add_child(ball)
	var flash := OmniLight3D.new()
	flash.position = target + Vector3(0, 3, 0)
	flash.light_color = Color(1, 0.5, 0.15)
	flash.light_energy = 16.0
	flash.omni_range = 28.0
	add_child(flash)
	_impact(target, 6.5, 0.0, 90.0)

func fpv_strike(target: Vector3) -> Node3D:
	GameState.show_subtitle("巡飞弹。贴着楼飞过来了。", "An FPV. It is coming in along the roofs.")
	_drone.play()
	var rig := Node3D.new()
	rig.position = target + Vector3(-18, 16, -12)
	_drone_mesh(rig)
	add_child(rig)
	var tween := create_tween()
	tween.tween_property(rig, "position", target + Vector3(0, 1.2, 0), 1.6)
	tween.tween_callback(func():
		_impact(target, 6.5, 22.0, 120.0)
		rig.queue_free()
	)
	return rig

func spawn_armour(along: Vector3, facing: Vector3) -> void:
	var hull := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(2.6, 2.2, 6.4)
	hull.mesh = box
	hull.position = along + Vector3(0, 1.3, 0)
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.22, 0.26, 0.18)
	mat.roughness = 0.7
	hull.material_override = mat
	add_child(hull)
	var turret := OmniLight3D.new()
	turret.light_color = Color(1, 0.6, 0.25)
	turret.omni_range = 10.0
	turret.light_energy = 0.0
	hull.add_child(turret)
	var goal := along + facing.normalized() * 18.0
	var tween := create_tween()
	tween.tween_property(hull, "position", goal + Vector3(0, 1.3, 0), 6.0)
	get_tree().create_timer(1.2).timeout.connect(func():
		turret.light_energy = 8.0
		_impact(goal + Vector3(4, 2, 0), 8.0, 10.0, 220.0)
	)

func _impact(at: Vector3, radius: float, player_dmg: float, building: float) -> void:
	var squads := get_tree().get_first_node_in_group("squads")
	if squads and squads.has_method("_boom"):
		squads._boom(at, radius, player_dmg, building)
	else:
		GameState.boom.emit()

func _drone_mesh(parent: Node3D) -> void:
	var body := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(1.5, 0.28, 1.5)
	body.mesh = box
	var mat := StandardMaterial3D.new()
	mat.albedo_color = Color(0.12, 0.12, 0.13)
	body.material_override = mat
	parent.add_child(body)
	for x in [-1, 1]:
		for z in [-1, 1]:
			var arm := MeshInstance3D.new()
			var stick := BoxMesh.new()
			stick.size = Vector3(1.7, 0.08, 0.12)
			arm.mesh = stick
			arm.position = Vector3(x * 0.7, 0.05, z * 0.15)
			arm.rotation.y = 0.6 if x == z else -0.6
			parent.add_child(arm)
	parent.scale = Vector3(2.4, 2.4, 2.4)

func _tone(path: String, db: float) -> AudioStreamPlayer:
	var player := AudioStreamPlayer.new()
	player.stream = load(path)
	player.volume_db = db
	return player
