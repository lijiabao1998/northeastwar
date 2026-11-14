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
	if "--fidelity-shots" in OS.get_cmdline_user_args():
		await _fidelity_shots()
		print("SHOTS_DONE")
		get_tree().quit(0)
		return
	if "--combat-shots" in OS.get_cmdline_user_args():
		await _combat_shots()
		print("SHOTS_DONE")
		get_tree().quit(0)
		return
	if player and player.camera():
		player.camera().current = true
	var mother := city.mother()
	var bank := city.bank_z(mother.x)
	# Signs face northeast. Stand in the street in front of 蜜雪冰城 so the
	# isometric camera, which sits further northeast, looks back at the row.
	var shop := Vector3(-268.8, 0.05, -325.2)
	var facing := Vector3(0.427, 0.0, 0.904)
	var street := shop + facing * 11.0

	GameState.hour = 16.45
	_stand(street)
	_crowd(street, facing)
	player.set_zoom(1.05)
	await _frame()
	await _save("01_street_signs")

	player.set_zoom(2.15)
	_stand(shop + facing * 7.5)
	await _frame()
	await _save("06_shop_row")

	GameState.hour = 16.7
	var prom := Vector3(mother.x, 0.05, bank - 4.0)
	_stand(prom)
	player.set_zoom(0.72)
	await _frame()
	await _save("02_river_dusk")

	GameState.hour = 17.55
	_stand(street)
	_crowd(street, facing)
	player.set_zoom(1.0)
	await _frame()
	await _save("03_snow_lights")

	GameState.hour = 17.45
	_stand(prom + Vector3(6, 0, -2))
	player.set_zoom(0.85)
	GameState.set_alert_stage(2)
	GameState.push_phone(
		"黑河市应急广播",
		"紧急通知：监测到俄军正自封冻的黑龙江越境，向市区推进。请立即远离江岸。",
		"Emergency: Russian troops are crossing the frozen Heilongjiang toward the city. Leave the riverfront."
	)
	GameState.show_subtitle("警报。不是演习。", "The alert. It says this is not a drill.")
	await _frame()
	await _save("04_alert_river")
	print("SHOTS_DONE")
	get_tree().quit(0)

func _fidelity_shots() -> void:
	if player and player.camera():
		player.camera().current = true
	GameState.set_quality("high")
	var shop := Vector3(-268.8, 0.05, -325.2)
	var facing := Vector3(0.427, 0.0, 0.904)
	var street := shop + facing * 12.0
	var mother := city.mother()
	var bank := city.bank_z(mother.x)
	GameState.hour = 15.95
	_stand(street)
	_crowd(street, facing)
	player.set_zoom(0.8)
	await _frame()
	await _save("10_afternoon")
	GameState.hour = 16.55
	_stand(street)
	_crowd(street, facing)
	player.set_zoom(0.8)
	await _frame()
	await _save("11_dusk")
	GameState.hour = 17.7
	_stand(street)
	_crowd(street, facing)
	player.set_zoom(0.8)
	await _frame()
	await _save("12_night")
	player.set_zoom(1.55)
	_stand(shop + facing * 7.2)
	await _frame()
	await _save("13_storefront")
	GameState.set_quality("low")
	GameState.hour = 16.55
	_stand(street)
	_crowd(street, facing)
	player.set_zoom(0.8)
	await _frame()
	await _save("18_low")
	GameState.set_quality("high")
	GameState.hour = 17.05
	# Same isometric pitch, turned to look north across the ice. The gameplay
	# camera always looks southwest, so the Russian bank would stay behind it.
	player.face_river = true
	var prom := Vector3(mother.x, 0.05, bank + 4.0)
	_stand(prom)
	player.set_zoom(0.42)
	await _frame()
	await _save("14_river")
	player.face_river = false
	player.set_zoom(1.0)
	GameState.hour = 17.2
	GameState.under_fire = true
	CombatHooks.enabled = true
	_stand(street)
	player.set_zoom(0.85)
	var squads := get_tree().get_first_node_in_group("squads") as Squads
	if squads:
		squads.pose_firefight(street - facing * 8.0, street)
	await _frame()
	await _save("15_firefight")
	var destruction := get_tree().get_first_node_in_group("destruction") as Destruction
	if destruction:
		destruction.breach_facade(destruction.nearest_id(shop), shop)
	for i in 4:
		await RenderingServer.frame_post_draw
	await _save("16_collapse")
	var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
	if guns:
		guns.pose_strike(street - facing * 2.5)
	await _frame()
	await _save("17_strike")

func _combat_shots() -> void:
	if player and player.camera():
		player.camera().current = true
	var shop := Vector3(-268.8, 0.05, -325.2)
	var facing := Vector3(0.427, 0.0, 0.904)
	var street := shop + facing * 12.0
	GameState.hour = 17.15
	GameState.under_fire = true
	CombatHooks.enabled = true
	_stand(street)
	player.set_zoom(1.2)
	var squads := get_tree().get_first_node_in_group("squads") as Squads
	if squads:
		squads.pose_firefight(street - facing * 9.0, street)
	await _frame()
	await _save("07_firefight")
	var destruction := get_tree().get_first_node_in_group("destruction") as Destruction
	if destruction:
		destruction.breach_facade(destruction.nearest_id(shop), shop)
	for i in 6:
		await RenderingServer.frame_post_draw
	await _save("08_collapse")
	var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
	if guns:
		guns.pose_strike(street - facing * 3.0)
	await _frame()
	await _save("09_drone")

func _stand(at: Vector3) -> void:
	if player:
		player.global_position = at
		player.velocity = Vector3.ZERO

func _crowd(at: Vector3, facing: Vector3) -> void:
	var side := Vector3(facing.z, 0, -facing.x)
	var i := 0
	for node in get_tree().get_nodes_in_group("npc"):
		if i >= 5:
			break
		if node is Node3D:
			(node as Node3D).global_position = at + facing * (3.0 + float(i) * 2.4) + side * (2.2 if i % 2 == 0 else -1.6)
			i += 1

func _frame() -> void:
	if player:
		player.set_zoom(player.zoom)
	for i in 8:
		await RenderingServer.frame_post_draw

func _save(name: String) -> void:
	var image := get_viewport().get_texture().get_image()
	var path := "res://docs/screenshots/%s.png" % name
	var err := image.save_png(path)
	print("saved ", path, " ", err, " ", image.get_width(), "x", image.get_height())
