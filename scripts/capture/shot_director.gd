extends Node
class_name ShotDirector
## Drives the real scene to a few cameras and writes PNGs.
## This is an actual run of the imported city, not a mockup.

var city: CityData
var player: Player

func _ready() -> void:
	call_deferred("_run")

func _run() -> void:
	DisplayServer.window_set_size(Vector2i(1280, 720))
	DisplayServer.window_set_vsync_mode(DisplayServer.VSYNC_DISABLED)
	await get_tree().process_frame
	await get_tree().process_frame
	var cam := Camera3D.new()
	cam.fov = 58.0
	cam.near = 0.08
	cam.far = 4500.0
	add_child(cam)
	cam.current = true
	if player and player.camera():
		player.camera().current = false
	var spawn: Dictionary = city.spawn()
	var origin := Vector3(float(spawn["x"]), 0.0, float(spawn["z"]))
	var yaw := float(spawn.get("yaw", PI))
	var fwd := Vector3(-sin(yaw), 0.0, -cos(yaw))
	var mother := city.mother()
	var mother_pos := Vector3(mother.x, 0.0, mother.y)
	var bank := city.bank_z(mother.x)

	# Street, late afternoon. Aim at the real 蜜雪冰城 / 肯德基 signs on 中央街.
	GameState.hour = 16.4
	var sign_at := Vector3(-266.0, 2.4, -326.0)
	if player:
		player.global_position = sign_at + Vector3(4.0, -2.35, -6.0)
		player.yaw = yaw
	cam.fov = 55.0
	await _frame(cam, Vector3(-258.0, 1.65, -338.0), sign_at)
	await _save("01_street_signs")

	# Riverfront at dusk, a longer lens so Blagoveshchensk's lights read.
	GameState.hour = 16.75
	cam.fov = 38.0
	if player:
		player.global_position = Vector3(mother.x + 3.0, 0.05, bank - 7.0)
	await _frame(cam, Vector3(mother.x - 2.0, 1.7, bank - 6.0), Vector3(mother.x + 30.0, 8.0, bank + 760.0))
	await _save("02_river_dusk")

	# Sodium lamps on snow, after dark.
	GameState.hour = 17.6
	cam.fov = 50.0
	var lamp_at := origin + fwd * 18.0
	await _frame(cam, lamp_at - fwd * 3.5 + Vector3(0, 1.35, 0), lamp_at + Vector3(0, 4.5, 0) + fwd * 8.0)
	await _save("03_snow_lights")

	# The pink memorial on 王肃街, from the licensed street photo's subject.
	GameState.hour = 16.5
	cam.fov = 42.0
	await _frame(cam, Vector3(-430.0, 1.7, 96.0), Vector3(-454.0, 4.2, 122.0))
	await _save("05_memorial")

	# The alert, from the promenade, skyline filling the lens.
	GameState.hour = 17.5
	cam.fov = 40.0
	GameState.set_alert_stage(2)
	GameState.push_phone(
		"黑河市应急广播",
		"紧急通知：监测到俄军正自封冻的黑龙江越境，向市区推进。请立即远离江岸。",
		"Emergency: Russian troops are crossing the frozen Heilongjiang toward the city. Leave the riverfront."
	)
	GameState.show_subtitle("警报。不是演习。", "The alert. It says this is not a drill.")
	await _frame(cam, Vector3(mother.x + 5.0, 1.65, bank - 3.5), Vector3(mother.x - 15.0, 10.0, bank + 700.0))
	await _save("04_alert_river")
	print("SHOTS_DONE")
	get_tree().quit(0)

func _frame(cam: Camera3D, from: Vector3, at: Vector3) -> void:
	cam.global_position = from
	cam.look_at(at, Vector3.UP)
	for i in 6:
		await RenderingServer.frame_post_draw

func _save(name: String) -> void:
	var image := get_viewport().get_texture().get_image()
	var path := "res://docs/screenshots/%s.png" % name
	var err := image.save_png(path)
	print("saved ", path, " ", err, " ", image.get_width(), "x", image.get_height())
