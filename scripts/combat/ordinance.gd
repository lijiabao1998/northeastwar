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

func show_drone(parent: Node3D) -> void:
	_drone_mesh(parent)

func pose_strike(target: Vector3) -> void:
	var rig := Node3D.new()
	rig.position = target + Vector3(-6, 8, -4)
	_drone_mesh(rig)
	add_child(rig)
	burst(target, true)
	_impact(target, 6.5, 0.0, 90.0)

func burst(at: Vector3, hold: bool = false) -> void:
	var flash := OmniLight3D.new()
	flash.position = at + Vector3(0, 1.6, 0)
	flash.light_color = Color(1, 0.85, 0.6)
	flash.light_energy = 28.0
	flash.omni_range = 26.0
	add_child(flash)
	var glow := StandardMaterial3D.new()
	glow.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	glow.albedo_color = Color(1.0, 0.5, 0.1)
	glow.emission_enabled = true
	glow.emission = Color(1.0, 0.35, 0.05)
	glow.emission_energy_multiplier = 3.0
	var balls: Array[MeshInstance3D] = []
	for i in 6:
		var ball := MeshInstance3D.new()
		var sphere := SphereMesh.new()
		var radius := 0.45 + float(i % 3) * 0.22
		sphere.radius = radius
		sphere.height = radius * 2.0
		ball.mesh = sphere
		var ang := float(i) * 1.05
		ball.position = at + Vector3(cos(ang) * 0.7, 1.1 + float(i) * 0.28, sin(ang) * 0.7)
		ball.material_override = glow
		add_child(ball)
		balls.append(ball)
	_fire_particles(at)
	var scorch := MeshInstance3D.new()
	var disc := CylinderMesh.new()
	disc.top_radius = 3.2
	disc.bottom_radius = 3.2
	disc.height = 0.06
	scorch.mesh = disc
	scorch.position = at + Vector3(0, 0.08, 0)
	var soot := StandardMaterial3D.new()
	soot.albedo_color = Color(0.12, 0.1, 0.09)
	soot.roughness = 1.0
	scorch.material_override = soot
	add_child(scorch)
	for i in 7:
		var chip := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(0.35, 0.22, 0.45)
		chip.mesh = box
		chip.position = at + Vector3(0, 1.2, 0)
		var mat := StandardMaterial3D.new()
		mat.albedo_color = Color(0.45, 0.4, 0.36)
		mat.roughness = 0.95
		chip.material_override = mat
		add_child(chip)
		var ang := float(i) / 7.0 * TAU
		var land := at + Vector3(cos(ang) * 3.4, 0.2, sin(ang) * 3.4)
		if hold:
			chip.position = land
			chip.rotation = Vector3(0.4, ang, 0.2)
		else:
			var tween := create_tween()
			tween.tween_property(chip, "position", land + Vector3(0, 2.2, 0), 0.18)
			tween.tween_property(chip, "position", land, 0.35)
	_column(at)
	if hold:
		flash.light_energy = 14.0
	else:
		var tween := create_tween()
		tween.tween_property(flash, "light_energy", 0.0, 0.45)
		tween.tween_callback(func():
			flash.queue_free()
			for ball in balls:
				if is_instance_valid(ball):
					ball.queue_free()
		)

func _fire_particles(at: Vector3) -> void:
	var fire := GPUParticles3D.new()
	fire.position = at + Vector3(0, 0.6, 0)
	fire.amount = 48
	fire.lifetime = 1.4
	fire.one_shot = true
	fire.explosiveness = 0.85
	fire.preprocess = 0.2
	var proc := ParticleProcessMaterial.new()
	proc.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_SPHERE
	proc.emission_sphere_radius = 0.4
	proc.direction = Vector3(0, 1, 0)
	proc.spread = 40.0
	proc.initial_velocity_min = 2.0
	proc.initial_velocity_max = 6.0
	proc.gravity = Vector3(0, 1.5, 0)
	proc.scale_min = 0.4
	proc.scale_max = 1.3
	proc.color = Color(1.0, 0.45, 0.08, 0.9)
	fire.process_material = proc
	fire.draw_pass_1 = _puff(0.9, Color(1.0, 0.4, 0.05, 0.85))
	add_child(fire)
	fire.restart()
	var sparks := GPUParticles3D.new()
	sparks.position = at + Vector3(0, 1.0, 0)
	sparks.amount = 30
	sparks.lifetime = 0.8
	sparks.one_shot = true
	sparks.explosiveness = 1.0
	sparks.preprocess = 0.15
	var spark_proc := ParticleProcessMaterial.new()
	spark_proc.direction = Vector3(0, 1, 0)
	spark_proc.spread = 70.0
	spark_proc.initial_velocity_min = 6.0
	spark_proc.initial_velocity_max = 12.0
	spark_proc.gravity = Vector3(0, -8, 0)
	spark_proc.scale_min = 0.05
	spark_proc.scale_max = 0.15
	spark_proc.color = Color(1, 0.85, 0.4, 1)
	sparks.process_material = spark_proc
	sparks.draw_pass_1 = _puff(0.15, Color(1, 0.9, 0.5, 1))
	add_child(sparks)
	sparks.restart()

func _puff(size: float, color: Color) -> QuadMesh:
	var mesh := QuadMesh.new()
	mesh.size = Vector2(size, size)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
	mat.albedo_color = color
	mesh.material = mat
	return mesh

func _column(at: Vector3) -> void:
	var puff := GPUParticles3D.new()
	puff.position = at + Vector3(0, 0.4, 0)
	puff.amount = 28
	puff.lifetime = 3.2
	puff.one_shot = true
	puff.explosiveness = 0.35
	puff.preprocess = 0.8
	var proc := ParticleProcessMaterial.new()
	proc.direction = Vector3(0, 1, 0)
	proc.spread = 12.0
	proc.initial_velocity_min = 3.0
	proc.initial_velocity_max = 7.0
	proc.gravity = Vector3(0.2, -0.2, 0)
	proc.scale_min = 1.4
	proc.scale_max = 3.2
	proc.color = Color(0.25, 0.23, 0.2, 0.7)
	puff.process_material = proc
	var quad := QuadMesh.new()
	quad.size = Vector2(2.4, 2.4)
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.billboard_mode = BaseMaterial3D.BILLBOARD_PARTICLES
	mat.albedo_color = Color(0.35, 0.32, 0.28, 0.55)
	quad.material = mat
	puff.draw_pass_1 = quad
	add_child(puff)
	puff.emitting = true

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
	hull.add_to_group("armour")
	add_child(hull)
	var cabin := MeshInstance3D.new()
	var cabin_mesh := BoxMesh.new()
	cabin_mesh.size = Vector3(1.8, 0.7, 2.2)
	cabin.mesh = cabin_mesh
	cabin.position = Vector3(0, 1.35, -0.6)
	cabin.material_override = mat
	hull.add_child(cabin)
	var nose := MeshInstance3D.new()
	var nose_mesh := BoxMesh.new()
	nose_mesh.size = Vector3(2.4, 0.5, 1.4)
	nose.mesh = nose_mesh
	nose.position = Vector3(0, 0.35, 2.6)
	nose.rotation_degrees = Vector3(-18, 0, 0)
	nose.material_override = mat
	hull.add_child(nose)
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
	var nav := OmniLight3D.new()
	nav.light_color = Color(1.0, 0.12, 0.08)
	nav.light_energy = 5.0
	nav.omni_range = 8.0
	nav.position = Vector3(-0.8, 0.15, -0.2)
	nav.shadow_enabled = false
	parent.add_child(nav)
	var starboard := OmniLight3D.new()
	starboard.light_color = Color(0.15, 1.0, 0.3)
	starboard.light_energy = 5.0
	starboard.omni_range = 8.0
	starboard.position = Vector3(0.8, 0.15, -0.2)
	starboard.shadow_enabled = false
	parent.add_child(starboard)
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
