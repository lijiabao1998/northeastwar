extends Node3D
class_name CityView
## Loads the imported glTF and replaces the generic materials with the
## Heihe shaders. One shared material per surface, so night can be driven
## from a single uniform.

var night: float = 0.35
var _night_mats: Array[ShaderMaterial] = []
var cut_materials: Array[ShaderMaterial] = []

func _ready() -> void:
	var packed: PackedScene = load("res://data/heihe/city.gltf")
	if packed == null:
		push_error("Heihe glTF did not import")
		return
	var city := packed.instantiate()
	city.name = "ImportedCity"
	add_child(city)
	add_to_group("city_view")
	var facade := _shader("res://shaders/facade.gdshader", {"floor_h": 3.0, "bay": 3.15})
	var euro := _shader("res://shaders/european.gdshader", {"floor_h": 4.05, "bay": 3.7})
	var glass := _shader("res://shaders/glass.gdshader", {})
	var road := _shader("res://shaders/road.gdshader", {})
	var walk := _shader("res://shaders/sidewalk.gdshader", {})
	var ice := _shader("res://shaders/ice.gdshader", {})
	var snow := _shader("res://shaders/snow.gdshader", {})
	var ground := _shader("res://shaders/ground.gdshader", {})
	var prom := _shader("res://shaders/promenade.gdshader", {})
	var car := _paint(0.45, 0.55, 0.15)
	var car_glass := _paint(0.12, 0.35, 0.0)
	car_glass.albedo_color = Color(0.02, 0.03, 0.04)
	var rail := _paint(0.35, 0.45, 0.55)
	rail.albedo_color = Color(0.1, 0.11, 0.12)
	var prop := _paint(0.7, 0.0, 0.0)
	var assignment := {
		"Facade": facade,
		"European": euro,
		"Glass": glass,
		"Road": road,
		"Sidewalk": walk,
		"Ice": ice,
		"Snow": snow,
		"Roof": _paint(0.85, 0.0, 0.0),
		"Ground": ground,
		"Promenade": prom,
		"Car": car,
		"CarGlass": car_glass,
		"Railing": rail,
		"Prop": prop,
	}
	_assign(city, assignment)
	cut_materials = [facade, euro, glass]
	GameState.quality_changed.connect(apply_quality)
	apply_quality(GameState.quality)

func set_night(value: float) -> void:
	night = value
	for mat in _night_mats:
		mat.set_shader_parameter("night", night)

func apply_quality(level: String) -> void:
	var shadows := GeometryInstance3D.SHADOW_CASTING_SETTING_ON
	if level == "low":
		shadows = GeometryInstance3D.SHADOW_CASTING_SETTING_OFF
	_set_shadows(self, shadows)

func _shader(path: String, params: Dictionary) -> ShaderMaterial:
	var mat := ShaderMaterial.new()
	mat.shader = load(path)
	for key in params:
		mat.set_shader_parameter(key, params[key])
	mat.set_shader_parameter("night", night)
	_night_mats.append(mat)
	return mat

func _paint(rough: float, metal: float, spec: float) -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.vertex_color_use_as_albedo = true
	mat.roughness = rough
	mat.metallic = metal
	mat.metallic_specular = spec
	return mat

func _assign(node: Node, assignment: Dictionary) -> void:
	if node is MeshInstance3D:
		var key := str(node.name)
		if assignment.has(key):
			node.material_override = assignment[key]
			node.gi_mode = GeometryInstance3D.GI_MODE_STATIC
	for child in node.get_children():
		_assign(child, assignment)

func _set_shadows(node: Node, mode: GeometryInstance3D.ShadowCastingSetting) -> void:
	if node is MeshInstance3D and str(node.name) in ["Facade", "European", "Roof", "Car", "Prop"]:
		node.cast_shadow = mode
	for child in node.get_children():
		_set_shadows(child, mode)
