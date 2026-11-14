extends Node3D
class_name Weather
## Snow that stays around whoever is looking, so the slice never runs out of weather.

var _particles: GPUParticles3D

func _ready() -> void:
	_particles = GPUParticles3D.new()
	_particles.amount = 1600
	_particles.lifetime = 7.0
	_particles.local_coords = false
	_particles.transform_align = GPUParticles3D.TRANSFORM_ALIGN_Z_BILLBOARD
	_particles.visibility_aabb = AABB(Vector3(-40, -20, -40), Vector3(80, 50, 80))
	var proc := ParticleProcessMaterial.new()
	proc.emission_shape = ParticleProcessMaterial.EMISSION_SHAPE_BOX
	proc.emission_box_extents = Vector3(26, 10, 26)
	proc.direction = Vector3(0.35, -1.0, 0.12)
	proc.spread = 18.0
	proc.initial_velocity_min = 0.5
	proc.initial_velocity_max = 1.5
	proc.gravity = Vector3(0.15, -0.35, 0.05)
	proc.scale_min = 0.4
	proc.scale_max = 1.0
	proc.color = Color(0.95, 0.96, 0.98, 0.85)
	_particles.process_material = proc
	var quad := QuadMesh.new()
	quad.size = Vector2(0.035, 0.035)
	var mat := ShaderMaterial.new()
	mat.shader = load("res://shaders/flake.gdshader")
	quad.material = mat
	_particles.draw_pass_1 = quad
	add_child(_particles)
	GameState.quality_changed.connect(_apply_quality)
	_apply_quality(GameState.quality)

func _process(_delta: float) -> void:
	var cam := get_viewport().get_camera_3d()
	if cam:
		global_position = cam.global_position + Vector3(0, 6, 0)

func _apply_quality(level: String) -> void:
	match level:
		"low":
			_particles.amount = 450
		"high":
			_particles.amount = 3200
		_:
			_particles.amount = 1500
