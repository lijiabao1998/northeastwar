extends Node
class_name HoldNight
## One defend-night on 中央街. It does not replace the evening walk.
## Press H after the alert: scavenge, drag barricades into the street, then
## hold until 06:00. Waves are a recon drone, mortars, a flanking fireteam
## and one vehicle. Civilians are never a score.

const SHOP := Vector3(-268.8, 0.05, -325.2)
const FACING := Vector3(0.427, 0.0, 0.904)

const KINDS := {
	"sandbag": {"zh": "沙袋", "en": "Sandbags", "scrap": 2, "coal": 0},
	"gate": {"zh": "铁门", "en": "Iron gate", "scrap": 3, "coal": 0},
	"radiator": {"zh": "暖气片", "en": "Radiator", "scrap": 1, "coal": 1},
	"car": {"zh": "报废车", "en": "Dead car", "scrap": 4, "coal": 0},
	"concrete": {"zh": "水泥墩", "en": "Concrete barrier", "scrap": 5, "coal": 0},
}

var _phase := ""
var _selected := "sandbag"
var _ghost: MeshInstance3D
var _tooltip: Label
var _title: Control
var _strip: Label
var _squad_label: Label
var _militia: Array = []
var _selected_man := 0
var _barricades: Array = []
var _fires: Array = []
var _lights: Array[SpotLight3D] = []
var _piles: Array = []
var _sos: Array = []
var _marker: MeshInstance3D
var _drone: Node3D
var _done := {}

func _ready() -> void:
	add_to_group("hold_night")
	_ui()

func _process(delta: float) -> void:
	if not GameState.defend_night or GameState.shot_mode:
		return
	_follow_ghost()
	_cold(delta)
	_militia_step(delta)
	_sweep(delta)
	if _phase == "night":
		_clock(delta)
		_waves()
	_refresh_strip()

func _unhandled_input(event: InputEvent) -> void:
	if GameState.shot_mode or not GameState.started_play:
		return
	if event.is_action_pressed("hold_night") and not GameState.defend_night:
		if GameState.alert_stage >= 2:
			begin_build()
		return
	if not GameState.defend_night:
		return
	if event is InputEventKey and event.pressed and not event.echo:
		var typed := event as InputEventKey
		var key := int(typed.keycode)
		if key == KEY_1:
			_selected = "sandbag"
		elif key == KEY_2:
			_selected = "gate"
		elif key == KEY_3:
			_selected = "radiator"
		elif key == KEY_4:
			_selected = "car"
		elif key == KEY_5:
			_selected = "concrete"
		elif key == KEY_G:
			_order("garrison")
		elif key == KEY_B and _phase == "build":
			begin_night()
	if event.is_action_pressed("interact"):
		_scavenge()
		_try_rescue()
		_try_radio()
	if event.is_action_pressed("fire_mission"):
		_request_fire(CombatHooks.aim_point if CombatHooks.aim_point != Vector3.ZERO else _aim())
	if event is InputEventMouseButton and event.pressed and event.button_index == MOUSE_BUTTON_LEFT:
		if _phase == "build":
			_place(_aim())
		elif _phase == "night":
			_order_move(_aim())

func begin_build() -> void:
	GameState.defend_night = true
	GameState.defend_phase = "build"
	GameState.hold_hour = 21.0
	GameState.scrap = 8
	GameState.coal_stock = 3
	GameState.blackout = false
	_phase = "build"
	_show_title(true)
	_seed_piles()
	_seed_militia()
	_seed_sos()
	if _player():
		_player().set_zoom(0.48)
	GameState.show_subtitle("21:00 中央街。守到天亮。", "21:00, Central Street. Hold until dawn.")
	_refresh_strip()

func begin_night() -> void:
	_phase = "night"
	GameState.defend_phase = "night"
	GameState.blackout = true
	GameState.hold_hour = 21.5
	_show_title(false)
	_lamps_off()
	_searchlights()
	_flashlight()
	if _player():
		_player().set_zoom(1.15)
	GameState.show_subtitle("灯灭了。只剩你自己的光。", "The streetlights are out. Only what you light yourself.")

func pose_build() -> void:
	begin_build()
	var side := Vector3(FACING.z, 0, -FACING.x)
	var street := SHOP + FACING * 4.2
	_put("car", street + side * -2.0)
	_put("gate", street + side * 1.5)
	_put("sandbag", street + side * 3.4)
	_put("concrete", street + side * -4.2)
	_light_barrel(street + side * 5.0)
	if _tooltip:
		_tooltip.visible = true
		_tooltip.text = "铁门    废铁 3\n拖到街面上放下"
		_tooltip.position = Vector2(520, 280)

func pose_wave() -> void:
	if _tooltip:
		_tooltip.visible = false
	if _phase != "night":
		begin_build()
		begin_night()
	GameState.hold_hour = 23.4
	GameState.revealed = true
	var side := Vector3(FACING.z, 0, -FACING.x)
	var street := SHOP + FACING * 3.6
	var squads := get_tree().get_first_node_in_group("squads") as Squads
	if squads:
		squads.pose_firefight(street + side * 2.2, street)
	_searchlights()
	for light in _lights:
		light.rotation_degrees = Vector3(-28, 40, 0)
		light.light_energy = 8.0
	GameState.show_subtitle("江面北側有部隊集結，去江邊那道牆。", "A unit is forming on the north side of the ice. Get to the wall by the river.")
	_refresh_strip()

func pose_drone() -> void:
	pose_wave()
	GameState.hold_hour = 22.1
	var street := SHOP + FACING * 5.0
	var rig := Node3D.new()
	rig.position = street + Vector3(-4, 7, -2)
	var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
	if guns and guns.has_method("show_drone"):
		guns.show_drone(rig)
	else:
		_plain_drone(rig)
	add_child(rig)
	_drone = rig
	if guns:
		guns.barrage(street + Vector3(2, 0, 1), 3)
	GameState.show_subtitle("先是无人机。然后炮。", "First the drone. Then the mortars.")

func pose_fire() -> void:
	if not GameState.radio_unlocked:
		GameState.radio_unlocked = true
	var at := SHOP + FACING * 6.0
	_paint_marker(at)
	var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
	if guns:
		guns.burst(at, true)
		guns.barrage(at, 1)
	var destruction := get_tree().get_first_node_in_group("destruction") as Destruction
	if destruction:
		destruction.damage_at(at + Vector3(0, 4, 0), 10.0, 240.0)
	GameState.show_subtitle("弹着。", "Splash.")
	_refresh_strip()

func pose_sos() -> void:
	_show_title(false)
	if _tooltip:
		_tooltip.visible = false
	_seed_sos()
	for node in _sos:
		if node is Node3D:
			(node as Node3D).visible = true

func pose_cold() -> void:
	if _phase == "":
		begin_build()
		begin_night()
	var at := SHOP + FACING * 2.2
	_light_barrel(at)
	GameState.body_temp = 35.1
	GameState.ambient_c = -24.0
	GameState.revealed = true
	GameState.coal_stock = 2
	_refresh_strip()

func _begin_clock_ok() -> void:
	pass

func _clock(delta: float) -> void:
	GameState.hold_hour += delta * 0.035
	if GameState.hold_hour >= 30.0 and not _done.has("dawn"):
		_done["dawn"] = true
		_phase = "dawn"
		GameState.defend_phase = "dawn"
		GameState.blackout = false
		GameState.show_subtitle("天亮了。街还在。", "Dawn. The street is still here.")

func _waves() -> void:
	var h := GameState.hold_hour
	if h > 21.7 and not _done.has("drone"):
		_done["drone"] = true
		GameState.show_subtitle("江面上有一架小无人机，在转。", "A small drone is circling over the ice.")
		var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
		if guns:
			guns.fpv_strike(SHOP + FACING * 8.0)
	if h > 22.4 and not _done.has("mortar"):
		_done["mortar"] = true
		GameState.show_subtitle("压制。别站在空地上。", "Suppression. Don't stand in the open.")
		var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
		if guns:
			guns.barrage(SHOP + FACING * 6.0, 4)
	if h > 23.2 and not _done.has("infantry"):
		_done["infantry"] = true
		GameState.show_subtitle("江面北側有部隊集結，去江邊那道牆。", "A unit is forming on the north side of the ice. Get to the wall by the river.")
		var squads := get_tree().get_first_node_in_group("squads") as Squads
		if squads:
			var side := Vector3(FACING.z, 0, -FACING.x)
			squads.spawn_at(SHOP + FACING * 14.0 + side * 8.0, SHOP)
	if h > 26.0 and not _done.has("armour"):
		_done["armour"] = true
		GameState.show_subtitle("一辆车顶着路障过来了。", "A vehicle is pushing through the barricades.")
		var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
		if guns:
			guns.spawn_armour(SHOP + FACING * 16.0, -FACING)

func _place(at: Vector3) -> void:
	var spec: Dictionary = KINDS[_selected]
	if GameState.scrap < int(spec["scrap"]) or GameState.coal_stock < int(spec["coal"]):
		_tooltip.text = "%s  不够。废铁 %d  煤 %d" % [spec["zh"], spec["scrap"], spec["coal"]]
		return
	GameState.scrap -= int(spec["scrap"])
	GameState.coal_stock -= int(spec["coal"])
	_put(_selected, at)

func _put(kind: String, at: Vector3) -> void:
	var body := MeshInstance3D.new()
	var box := BoxMesh.new()
	match kind:
		"sandbag":
			box.size = Vector3(1.6, 0.7, 0.8)
			body.material_override = _color(Color(0.45, 0.38, 0.28))
		"gate":
			box.size = Vector3(2.4, 1.6, 0.12)
			body.material_override = _color(Color(0.2, 0.22, 0.24))
		"radiator":
			box.size = Vector3(1.1, 0.7, 0.18)
			body.material_override = _color(Color(0.55, 0.56, 0.58))
		"car":
			box.size = Vector3(1.7, 1.3, 4.0)
			body.material_override = _color(Color(0.75, 0.62, 0.15))
		_:
			box.size = Vector3(2.2, 0.9, 0.6)
			body.material_override = _color(Color(0.55, 0.55, 0.52))
	body.mesh = box
	body.position = at + Vector3(0, box.size.y * 0.5, 0)
	add_child(body)
	var block := StaticBody3D.new()
	var shape := CollisionShape3D.new()
	var col := BoxShape3D.new()
	col.size = box.size
	shape.shape = col
	block.add_child(shape)
	body.add_child(block)
	body.set_meta("kind", kind)
	_barricades.append(body)

func _seed_piles() -> void:
	var side := Vector3(FACING.z, 0, -FACING.x)
	for i in 4:
		var pile := MeshInstance3D.new()
		var box := BoxMesh.new()
		box.size = Vector3(0.8, 0.35, 0.6)
		pile.mesh = box
		pile.position = SHOP + FACING * (2.0 + float(i)) + side * (2.0 if i % 2 == 0 else -1.5) + Vector3(0, 0.2, 0)
		pile.material_override = _color(Color(0.35, 0.32, 0.28) if i < 2 else Color(0.12, 0.1, 0.09))
		pile.set_meta("kind", "scrap" if i < 2 else "coal")
		add_child(pile)
		_piles.append(pile)

func _scavenge() -> void:
	var player := _player()
	if player == null:
		return
	for pile in _piles:
		if not is_instance_valid(pile) or not pile.visible:
			continue
		if pile.global_position.distance_to(player.global_position) < 2.2:
			if str(pile.get_meta("kind")) == "coal":
				GameState.coal_stock += 1
			else:
				GameState.scrap += 2
			pile.visible = false

func _seed_militia() -> void:
	var names := ["老周", "小吴", "阿敏", "大刘"]
	var side := Vector3(FACING.z, 0, -FACING.x)
	for i in names.size():
		var man := Node3D.new()
		man.position = SHOP + FACING * 2.5 + side * (float(i) - 1.5) * 1.4
		man.set_meta("who", names[i])
		man.set_meta("goal", man.position)
		WinterFigure.attach(man, "civilian", Color(0.25, 0.28, 0.32))
		add_child(man)
		_militia.append(man)

func _order(what: String) -> void:
	if _militia.is_empty():
		return
	var man: Node3D = _militia[_selected_man % _militia.size()]
	if what == "garrison":
		man.set_meta("goal", SHOP + Vector3(0, 3.2, 0) + FACING * 0.4)
		GameState.show_subtitle("%s 上去守窗。" % str(man.get_meta("who")), "Up to the window.")

func _order_move(at: Vector3) -> void:
	if _militia.is_empty():
		return
	var man: Node3D = _militia[_selected_man % _militia.size()]
	man.set_meta("goal", at)
	_selected_man += 1

func _militia_step(delta: float) -> void:
	for man in _militia:
		if not is_instance_valid(man):
			continue
		var goal: Vector3 = man.get_meta("goal", man.global_position)
		var step: Vector3 = goal - man.global_position
		step.y = 0
		if step.length() > 0.4:
			man.global_position += step.normalized() * 2.2 * delta

func _seed_sos() -> void:
	if not _sos.is_empty():
		return
	_sos.append(_writing(SHOP + FACING * 2.4 + Vector3(0, 0.08, 0), "救命  樓裡還有5人"))
	_sos.append(_writing(SHOP + Vector3(0, 9.5, 0), "救命  樓裡還有5人"))

func _writing(at: Vector3, text: String) -> Label3D:
	var label := Label3D.new()
	label.text = text
	label.font = load("res://assets/fonts/NotoSansSC-VF.ttf")
	label.font_size = 48
	label.pixel_size = 0.028
	label.modulate = Color(0.85, 0.15, 0.12)
	label.outline_size = 8
	label.position = at
	label.rotation_degrees = Vector3(-70, 20, 0)
	add_child(label)
	return label

func _try_rescue() -> void:
	var player := _player()
	if player == null:
		return
	for mark in _sos:
		if mark.global_position.distance_to(player.global_position) < 2.4:
			GameState.neighbours_rescued = true
			GameState.show_subtitle("你把人接下来了。这一夜会记着。", "You brought them down. The night will remember it.")
			return

func _try_radio() -> void:
	if GameState.radio_unlocked:
		return
	var player := _player()
	if player == null or _militia.is_empty():
		return
	if player.global_position.distance_to(_militia[0].global_position) < 2.2:
		GameState.radio_unlocked = true
		GameState.show_subtitle("老周把电台给你。还不能乱叫。", "Zhou hands you the radio. Don't call it in yet.")

func _request_fire(at: Vector3) -> void:
	if not GameState.radio_unlocked:
		GameState.show_subtitle("电台还没接通。", "The radio isn't yours yet.")
		return
	_paint_marker(at)
	GameState.show_subtitle("呼叫火力。等着。", "Fire mission. Wait.")
	var timer := get_tree().create_timer(3.2)
	timer.timeout.connect(func():
		GameState.show_subtitle("弹着。", "Splash.")
		var guns := get_tree().get_first_node_in_group("ordinance") as Ordinance
		if guns:
			guns.burst(at, false)
		var destruction := get_tree().get_first_node_in_group("destruction") as Destruction
		if destruction:
			destruction.damage_at(at + Vector3(0, 3, 0), 9.0, 220.0)
	)

func _paint_marker(at: Vector3) -> void:
	if _marker == null:
		_marker = MeshInstance3D.new()
		var disc := CylinderMesh.new()
		disc.top_radius = 3.2
		disc.bottom_radius = 3.2
		disc.height = 0.08
		_marker.mesh = disc
		var mat := StandardMaterial3D.new()
		mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
		mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
		mat.albedo_color = Color(1.0, 0.42, 0.05, 0.55)
		_marker.material_override = mat
		add_child(_marker)
	_marker.position = at + Vector3(0, 0.12, 0)
	_marker.visible = true

func _light_barrel(at: Vector3) -> void:
	var drum := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.7, 0.8, 0.7)
	drum.mesh = box
	drum.position = at + Vector3(0, 0.4, 0)
	drum.material_override = _color(Color(0.18, 0.12, 0.08))
	drum.add_to_group("heat_source")
	add_child(drum)
	var fire := OmniLight3D.new()
	fire.position = drum.position + Vector3(0, 0.6, 0)
	fire.light_color = Color(1.0, 0.45, 0.12)
	fire.light_energy = 6.0
	fire.omni_range = 9.0
	fire.shadow_enabled = false
	add_child(fire)
	_fires.append(fire)

func _cold(delta: float) -> void:
	if _phase != "night":
		return
	var player := _player()
	if player == null:
		return
	var warm := false
	for fire in _fires:
		if is_instance_valid(fire) and fire.global_position.distance_to(player.global_position) < 3.4:
			warm = true
	GameState.revealed = warm
	var temp := GameState.body_temp
	if warm:
		temp = minf(36.8, temp + 0.05 * delta)
	else:
		temp -= 0.025 * delta
	GameState.set_temperature(clampf(temp, 33.6, 37.0), false, warm, 1.0 if temp > 35.0 else 0.75)

func _searchlights() -> void:
	if not _lights.is_empty():
		return
	var side := Vector3(FACING.z, 0, -FACING.x)
	for i in 2:
		var spot := SpotLight3D.new()
		spot.position = SHOP + FACING * (6.0 + float(i) * 4.0) + side * (6.0 if i == 0 else -5.0) + Vector3(0, 6.5, 0)
		spot.light_color = Color(0.85, 0.9, 1.0)
		spot.light_energy = 7.0
		spot.spot_range = 28.0
		spot.spot_angle = 22.0
		spot.shadow_enabled = false
		add_child(spot)
		_lights.append(spot)

func _flashlight() -> void:
	var player := _player()
	if player == null or player.get_node_or_null("Torch"):
		return
	var torch := SpotLight3D.new()
	torch.name = "Torch"
	torch.position = Vector3(0, 1.4, 0)
	torch.rotation_degrees = Vector3(-18, 180, 0)
	torch.light_color = Color(0.9, 0.92, 0.8)
	torch.light_energy = 3.5
	torch.spot_range = 14.0
	torch.spot_angle = 28.0
	torch.shadow_enabled = false
	player.add_child(torch)

func _sweep(delta: float) -> void:
	var t := Time.get_ticks_msec() * 0.001
	for i in _lights.size():
		_lights[i].rotation_degrees = Vector3(-35, sin(t * 0.4 + float(i)) * 40.0, 0)

func _lamps_off() -> void:
	var props := get_tree().get_first_node_in_group("city_props")
	if props and props.has_method("set_lamp_strength"):
		props.set_lamp_strength(0.0)

func _ui() -> void:
	var layer := CanvasLayer.new()
	add_child(layer)
	_title = Control.new()
	_title.visible = false
	_title.set_anchors_preset(Control.PRESET_FULL_RECT)
	var dim := ColorRect.new()
	dim.color = Color(0.02, 0.03, 0.05, 0.45)
	dim.set_anchors_preset(Control.PRESET_FULL_RECT)
	_title.add_child(dim)
	var card := Label.new()
	card.text = "21:00  中央街\n守到天亮"
	card.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	card.set_anchors_preset(Control.PRESET_CENTER)
	card.position = Vector2(-220, -80)
	card.size = Vector2(440, 160)
	card.add_theme_font_size_override("font_size", 42)
	card.add_theme_color_override("font_color", Color(0.95, 0.9, 0.8))
	_title.add_child(card)
	layer.add_child(_title)
	_strip = Label.new()
	_strip.position = Vector2(24, 24)
	_strip.add_theme_font_size_override("font_size", 18)
	_strip.add_theme_color_override("font_color", Color(0.9, 0.92, 0.88))
	layer.add_child(_strip)
	_squad_label = Label.new()
	_squad_label.position = Vector2(24, 120)
	_squad_label.add_theme_font_size_override("font_size", 16)
	_squad_label.add_theme_color_override("font_color", Color(0.8, 0.84, 0.78))
	layer.add_child(_squad_label)
	_tooltip = Label.new()
	_tooltip.visible = false
	_tooltip.add_theme_font_size_override("font_size", 20)
	_tooltip.add_theme_color_override("font_color", Color(1, 0.9, 0.55))
	layer.add_child(_tooltip)
	_ghost = MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(1.4, 0.6, 0.8)
	_ghost.mesh = box
	var mat := StandardMaterial3D.new()
	mat.transparency = BaseMaterial3D.TRANSPARENCY_ALPHA
	mat.albedo_color = Color(0.9, 0.8, 0.3, 0.35)
	_ghost.material_override = mat
	_ghost.visible = false
	add_child(_ghost)

func _show_title(on: bool) -> void:
	if _title:
		_title.visible = on

func _refresh_strip() -> void:
	if _strip == null:
		return
	_strip.visible = GameState.defend_night
	var h := GameState.hold_hour
	var clock_h := int(h) % 24
	var clock_m := int((h - float(int(h))) * 60.0)
	var spec: Dictionary = KINDS[_selected]
	_strip.text = "%02d:%02d   体温 %.1f°   室外 -24°\n废铁 %d   煤 %d\n放置 %s  废铁%d 煤%d" % [
		clock_h, clock_m, GameState.body_temp, GameState.scrap, GameState.coal_stock,
		spec["zh"], spec["scrap"], spec["coal"]]
	if _squad_label:
		var line := "民兵"
		for man in _militia:
			if is_instance_valid(man):
				line += "  " + str(man.get_meta("who"))
		if GameState.radio_unlocked:
			line += "\n电台已接通  R 呼叫火力"
		if GameState.neighbours_rescued:
			line += "\n楼里的人接下来了"
		_squad_label.text = line
		_squad_label.visible = GameState.defend_night

func _follow_ghost() -> void:
	if _ghost == null:
		return
	var show := _phase == "build"
	_ghost.visible = show
	if _tooltip and not GameState.shot_mode:
		_tooltip.visible = show
	if not show:
		return
	var at := _aim()
	_ghost.position = at + Vector3(0, 0.4, 0)
	if _tooltip and not GameState.shot_mode:
		var spec: Dictionary = KINDS[_selected]
		_tooltip.text = "%s\n废铁 %d   煤 %d" % [spec["zh"], spec["scrap"], spec["coal"]]
		_tooltip.position = get_viewport().get_mouse_position() + Vector2(16, 16)

func _aim() -> Vector3:
	var player := _player()
	if player == null:
		return SHOP
	var cam := player.camera()
	var mouse := get_viewport().get_mouse_position()
	var origin := cam.project_ray_origin(mouse)
	var dir := cam.project_ray_normal(mouse)
	if absf(dir.y) < 0.0001:
		return player.global_position
	var t := -origin.y / dir.y
	return origin + dir * t

func _player() -> Player:
	var node := get_tree().get_first_node_in_group("player")
	return node as Player

func _color(color: Color) -> StandardMaterial3D:
	var mat := StandardMaterial3D.new()
	mat.albedo_color = color
	mat.roughness = 0.8
	return mat

func _plain_drone(parent: Node3D) -> void:
	var body := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(1.4, 0.25, 1.4)
	body.mesh = box
	body.material_override = _color(Color(0.12, 0.12, 0.13))
	parent.add_child(body)

func show_drone_fallback() -> void:
	pass
