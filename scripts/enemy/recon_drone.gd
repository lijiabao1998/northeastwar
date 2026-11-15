extends Node3D
class_name ReconDrone
## A Mavic-class quadcopter and the mortar it talks to. In Ukraine the thing
## that kills you is rarely the man who saw you: a drone holds over the spot,
## and three to five minutes later the rounds arrive. Here that is the same
## chain, on the war clock. The buzz is your warning. A doorway is your roof.
## Numbers: data/russia/forces.json, parts recon_drone and mortar_82.

enum { AWAY, ON_STATION }

var player: Player
var _state: int = AWAY
var _wait: float = -1.0
var _left: float = 0.0
var _centre: Vector3 = Vector3.ZERO
var _angle: float = 0.0
var _thermal: bool = false
var _confirm: float = 0.0
var _mission_at: float = -1.0
var _mission_target: Vector3 = Vector3.ZERO
var _rounds_left: int = 0
var _round_wait: float = 0.0
var _rig: Node3D
var _buzz: AudioStreamPlayer

func _ready() -> void:
	add_to_group("recon_drone")
	_rig = Node3D.new()
	_rig.visible = false
	add_child(_rig)
	var guns := get_tree().get_first_node_in_group("ordinance")
	if guns and guns.has_method("show_drone"):
		guns.show_drone(_rig)
	_buzz = AudioStreamPlayer.new()
	var stream = load("res://assets/audio/drone.wav")
	if stream is AudioStream:
		_buzz.stream = stream
	_buzz.volume_db = -30.0
	add_child(_buzz)

func _process(delta: float) -> void:
	if player == null or GameState.shot_mode or not GameState.started_play or GameState.paused or GameState.dead:
		return
	if GameState.alert_stage < 2 or GameState.defend_night:
		return
	if _wait < 0.0:
		_wait = RuForces.war_minutes(randf_range(4.0, 9.0))
	_shells(delta)
	match _state:
		AWAY:
			_wait -= delta
			GameState.drone_overhead = false
			GameState.drone_sees = false
			if _wait <= 0.0:
				_arrive()
		ON_STATION:
			_orbit(delta)

func _arrive() -> void:
	var info := RuForces.section("recon_drone")
	_state = ON_STATION
	_thermal = randf() < float(info.get("thermal_p", 0.3))
	_left = RuForces.war_minutes(float(info.get("on_station_min", 20.0)) * float(info.get("cold_endurance_mul", 0.6)))
	_centre = player.global_position + Vector3(randf_range(-55, 55), 0, randf_range(-55, 55))
	_angle = randf() * TAU
	_confirm = 0.0
	_rig.visible = true
	if _buzz.stream and not _buzz.playing:
		_buzz.play()
	GameState.show_subtitle("头顶有嗡嗡声。一架小无人机。进门脸，别动。", "A buzz overhead. A small drone. Get in a doorway and keep still.")

func _leave() -> void:
	_state = AWAY
	_wait = RuForces.war_minutes(RuForces.pick("recon_drone", "gap_between_sorties_min"))
	_rig.visible = false
	_buzz.stop()
	GameState.drone_overhead = false
	GameState.drone_sees = false

func sight_radius() -> float:
	var by_light: Dictionary = RuForces.section("recon_drone").get("sight_radius_m", {})
	var light := GameState.light()
	if light.begins_with("night"):
		return float(by_light.get("night_thermal" if _thermal else "night_plain", 8.0))
	return float(by_light.get(light, 40.0))

func sees(at: Vector3) -> bool:
	## From above, walls do not help. Only a roof does.
	if GameState.sheltered:
		return false
	var flat := Vector2(at.x - _rig.global_position.x, at.z - _rig.global_position.z)
	var mul := 1.15 if GameState.gait == "run" else (0.8 if GameState.gait == "still" else 1.0)
	return flat.length() <= sight_radius() * mul

func _orbit(delta: float) -> void:
	var info := RuForces.section("recon_drone")
	var radius := 34.0
	_angle += float(info.get("orbit_mps", 9.0)) / radius * delta
	_rig.global_position = _centre + Vector3(cos(_angle) * radius, float(info.get("altitude_m", 90.0)) * 0.22, sin(_angle) * radius)
	_left -= delta
	var here := player.global_position
	var near := Vector2(here.x - _rig.global_position.x, here.z - _rig.global_position.z).length()
	_buzz.volume_db = clampf(-6.0 - near * 0.28, -40.0, -6.0)
	GameState.drone_overhead = near < 90.0
	var seen := sees(here)
	GameState.drone_sees = seen
	if seen:
		# Once it has you it holds over you, the way an operator would.
		_centre = _centre.lerp(here, minf(1.0, delta * 0.5))
		_confirm += delta
		if _confirm >= float(info.get("confirm_s", 6.0)) and _mission_at < 0.0 and _rounds_left <= 0:
			_call_mortar(here)
	else:
		_confirm = maxf(0.0, _confirm - delta * 0.5)
	if _left <= 0.0 and _rounds_left <= 0 and _mission_at < 0.0:
		_leave()

func _call_mortar(at: Vector3) -> void:
	_mission_target = at
	_mission_at = RuForces.war_minutes(RuForces.pick("mortar_82", "sensor_to_shooter_min"))
	_confirm = 0.0
	GameState.show_subtitle("它停在你头顶不走了。它看见你了，炮几分钟就到。离开这儿，找屋顶。", "It is holding right over you. It has seen you. Shells in minutes. Leave this spot and get under a roof.")

func _shells(delta: float) -> void:
	if _mission_at >= 0.0:
		_mission_at -= delta
		if _mission_at <= 0.0:
			_mission_at = -1.0
			_rounds_left = RuForces.pick_int("mortar_82", "rounds")
			_round_wait = 0.0
			GameState.show_subtitle("头顶有哨声。迫击炮。", "A whistle overhead. Mortars.")
	if _rounds_left <= 0:
		return
	_round_wait -= delta
	if _round_wait > 0.0:
		return
	_round_wait = RuForces.pick("mortar_82", "round_gap_s")
	_rounds_left -= 1
	var info := RuForces.section("mortar_82")
	# A round is corrected only while the drone can still see where you are.
	var watching := _state == ON_STATION and sees(player.global_position)
	var centre := player.global_position if watching else _mission_target
	var error := float(info.get("corrected_error_m" if watching else "first_round_error_m", 20.0))
	var angle := randf() * TAU
	var at := centre + Vector3(cos(angle), 0, sin(angle)) * error * sqrt(randf())
	at.y = 0.0
	land(at)

func land(at: Vector3) -> void:
	var info := RuForces.section("mortar_82")
	var guns := get_tree().get_first_node_in_group("ordinance")
	if guns and guns.has_method("burst") and not GameState.shot_mode:
		guns.burst(at, false)
	var destruction := get_tree().get_first_node_in_group("destruction")
	if destruction and destruction.has_method("damage_at"):
		destruction.damage_at(at + Vector3(0, 3, 0), 8.0, 60.0)
	GameState.boom.emit()
	var d := player.global_position.distance_to(at)
	var harm := blast_harm(d, GameState.sheltered)
	if harm > 0.0:
		CombatHooks.hurt_player(harm, "一发迫击炮弹落在你身边。", "A mortar round landed beside you.", 0.9)

static func blast_harm(distance: float, under_roof: bool) -> float:
	## Health lost to one 82 mm round at this distance.
	var info := RuForces.section("mortar_82")
	var lethal := float(info.get("lethal_m", 6.0))
	var wound := float(info.get("wound_m", 16.0))
	if distance > wound:
		return 0.0
	var pair: Array = info.get("wound_hp", [25, 60])
	var harm := 120.0
	if distance > lethal:
		var t := (distance - lethal) / maxf(0.1, wound - lethal)
		harm = lerpf(float(pair[1]), float(pair[0]), t)
	if under_roof:
		harm *= float(info.get("indoor_mul", 0.25))
	return harm
