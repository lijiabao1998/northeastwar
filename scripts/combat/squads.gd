extends Node
class_name Squads
## Russian infantry, in the groups of two to five that open reporting on the
## war in Ukraine describes from 2023 on. Nobody here knows where you are.
## A soldier has to see you: inside his reach for the light there is, inside
## the arc he is facing, with nothing solid between you. Or hear you run.
## Every number comes from data/russia/forces.json through RuForces.
##
## Before the crossing there are no soldiers on this bank at all.

enum { PATROL, CHECK, CHALLENGE, DETAIN, ENGAGE, SEARCH }

const EYE := Vector3(0, 1.55, 0)
const CHEST := Vector3(0, 1.1, 0)
const MAX_GROUPS_NEAR := 3
const MAX_SOLDIERS := 14

class Group extends Node:
	## Not a Node3D on purpose: the men are its children, and a parent that
	## moved would drag them along with it.
	var pos: Vector3 = Vector3.ZERO
	var state: int = 0
	var route: PackedVector3Array = PackedVector3Array()
	var route_i: int = 0
	var route_dir: int = 1
	var meter: float = 0.0
	var last_seen: Vector3 = Vector3.ZERO
	var lost_for: float = 0.0
	var timer: float = 0.0
	var ignore_for: float = 0.0
	var wander: Vector3 = Vector3.ZERO
	var wander_for: float = 0.0
	var shouted: bool = false
	var held_at: Vector3 = Vector3.ZERO

var _groups: Array[Group] = []
var _roads: Array[PackedVector3Array] = []
var _lamps: PackedVector2Array = PackedVector2Array()
var _city: CityData
var _spawn_wait: float = 8.0
var _crossed_at: float = -1.0
var _fire_cool: float = 0.0
var _player_lit: bool = true

func _ready() -> void:
	add_to_group("squads")
	CombatHooks.fire_requested.connect(_player_shot)
	_city = get_parent().get_node_or_null("CityData") as CityData
	if _city:
		_read_city(_city.data)

func _read_city(data: Dictionary) -> void:
	for road in data.get("roads", []):
		var raw = road.get("points", road.get("line", road.get("pts", [])))
		if not raw is Array or raw.size() < 2 or float(road.get("width", 0.0)) < 4.0:
			continue
		var line := PackedVector3Array()
		for p in raw:
			line.append(Vector3(float(p[0]), 0.05, float(p[1])))
		_roads.append(_densify(line, 22.0))
	for lamp in data.get("lamps", []):
		if lamp is Dictionary:
			_lamps.append(Vector2(float(lamp.get("x", 0.0)), float(lamp.get("z", 0.0))))
		elif lamp is Array and lamp.size() >= 2:
			_lamps.append(Vector2(float(lamp[0]), float(lamp[1])))

static func _densify(line: PackedVector3Array, step: float) -> PackedVector3Array:
	var out := PackedVector3Array()
	for i in line.size() - 1:
		var a := line[i]
		var b := line[i + 1]
		var n := maxi(1, int(ceil(a.distance_to(b) / step)))
		for k in n:
			out.append(a.lerp(b, float(k) / float(n)))
	out.append(line[line.size() - 1])
	return out

# ---------------------------------------------------------------- the player's rifle

func _player_shot(point: Vector3) -> void:
	var player := _player()
	if player == null or not GameState.armed:
		return
	var from := player.global_position + Vector3(0, 1.4, 0)
	var flat := point - from
	flat.y = 0
	if flat.length() < 0.2:
		return
	# The shot carries past the spot that was clicked, so a click on the man hits the man.
	var to := point + flat.normalized() * 6.0
	var best: Node3D = null
	var best_t := 2.0
	for soldier in soldiers():
		var chest: Vector3 = soldier.global_position + CHEST
		var seg := to - from
		var t := clampf((chest - from).dot(seg) / seg.length_squared(), 0.0, 1.0)
		if from.lerp(to, t).distance_to(chest) < 0.9 and t < best_t:
			best = soldier
			best_t = t
	_tracer(from, to if best == null else best.global_position + CHEST)
	if best:
		hurt_soldier(best, 28.0)
		_alarm_near(best.global_position, player.global_position)

func _alarm_near(at: Vector3, source: Vector3) -> void:
	for group in _groups:
		if is_instance_valid(group) and group.pos.distance_to(at) < 60.0:
			group.last_seen = source
			if group.state != ENGAGE:
				_set_state(group, SEARCH)

# ---------------------------------------------------------------- the clock

func _process(delta: float) -> void:
	if GameState.shot_mode or not GameState.started_play or GameState.paused or GameState.dead:
		return
	var player := _player()
	if player == null:
		return
	if GameState.alert_stage >= 2 and _crossed_at < 0.0:
		_crossed_at = GameState.hour
	_spawn_wait -= delta
	if _crossed_at >= 0.0 and not GameState.defend_night and _spawn_wait <= 0.0:
		_spawn_wait = randf_range(18.0, 32.0)
		_try_spawn(player.global_position)
	_prune(player.global_position)

func front_depth() -> float:
	## How far south of the bank the leading groups have come, in metres.
	if _crossed_at < 0.0:
		return 0.0
	var minutes := (GameState.hour - _crossed_at) * 60.0
	return 40.0 + minutes * RuForces.num("infantry_group", "advance_m_per_min", 9.0)

func _depth(point: Vector3) -> float:
	if _city == null:
		return 0.0
	return _city.bank_z(point.x) - point.z

func _try_spawn(origin: Vector3) -> void:
	if _roads.is_empty() or soldiers().size() >= MAX_SOLDIERS:
		return
	var near := 0
	for group in _groups:
		if is_instance_valid(group) and _alive(group).size() > 0 and group.pos.distance_to(origin) < 300.0:
			near += 1
	if near >= MAX_GROUPS_NEAR:
		return
	var reach := front_depth()
	# Every street point the front has reached that is out of your sight but not far.
	var spots: Array = []
	for road in _roads:
		for i in road.size():
			var d := road[i].distance_to(origin)
			if d < 75.0 or d > 170.0:
				continue
			var depth := _depth(road[i])
			if depth > reach or depth < 4.0:
				continue
			spots.append([road, i])
	if spots.is_empty():
		_spawn_wait = 6.0
		return
	var pick: Array = spots[randi() % spots.size()]
	var line: PackedVector3Array = pick[0]
	var at_i := int(pick[1])
	var group := _make_group(line[at_i])
	group.route = line
	group.route_i = at_i
	# Walk away from the river first: that is the way the front is moving.
	var ahead := mini(at_i + 1, line.size() - 1)
	group.route_dir = 1 if _depth(line[ahead]) >= _depth(line[at_i]) else -1

func _prune(origin: Vector3) -> void:
	for i in range(_groups.size() - 1, -1, -1):
		var group := _groups[i]
		if not is_instance_valid(group):
			_groups.remove_at(i)
			continue
		if _alive(group).is_empty():
			continue
		if group.state == PATROL and group.pos.distance_to(origin) > 330.0 and not GameState.defend_night:
			_groups.remove_at(i)
			group.queue_free()

# ---------------------------------------------------------------- making soldiers

func spawn_near(origin: Vector3) -> Node:
	var angle := randf() * TAU
	return spawn_at(origin + Vector3(cos(angle), 0, sin(angle)) * randf_range(40.0, 60.0), origin)

func spawn_at(at: Vector3, face: Vector3) -> Node:
	## Used by the defend night: a group that already knows which shop it is after.
	var group := _make_group(at)
	group.last_seen = face
	_set_state(group, SEARCH)
	group.timer = 120.0
	return group

func _make_group(at: Vector3) -> Group:
	var group := Group.new()
	add_child(group)
	group.pos = at
	var count := RuForces.pick_int("infantry_group", "size")
	var spacing := RuForces.num("infantry_group", "spacing_m", 7.0)
	for i in count:
		var body := _soldier(i)
		group.add_child(body)
		body.global_position = at + Vector3(randf_range(-0.6, 0.6), 0.05, -spacing * 0.4 * float(i))
	_groups.append(group)
	return group

func pose_firefight(origin: Vector3, toward: Vector3) -> void:
	var group := spawn_at(origin, toward) as Group
	var i := 0
	for child in group.get_children():
		var flash := child.get_node_or_null("Flash") as OmniLight3D
		if flash:
			flash.light_energy = 8.0 if i % 2 == 0 else 3.0
		var muzzle := child.get_node_or_null("Muzzle") as MeshInstance3D
		if muzzle:
			muzzle.visible = i % 2 == 0
		i += 1
	_set_state(group, ENGAGE)

func _soldier(index: int) -> CharacterBody3D:
	var body := CharacterBody3D.new()
	body.collision_layer = 4
	body.collision_mask = 1
	body.set_meta("hp", 40.0)
	body.set_meta("optics", randf() < RuForces.num("infantry_group", "night_optics_p", 0.2))
	body.add_to_group("soldier")
	var shape := CollisionShape3D.new()
	var cap := CapsuleShape3D.new()
	cap.radius = 0.32
	cap.height = 1.7
	shape.shape = cap
	shape.position = Vector3(0, 0.9, 0)
	body.add_child(shape)
	var visual := Node3D.new()
	visual.name = "Visual"
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

# ---------------------------------------------------------------- seeing and hearing

func light_at(point: Vector3) -> String:
	## The light a watcher has to see someone standing at `point`.
	var light := GameState.light()
	if not light.begins_with("night"):
		return light
	if GameState.revealed:
		return "night_lit"
	if light == "night_dark":
		return light
	var here := Vector2(point.x, point.z)
	for lamp in _lamps:
		if lamp.distance_squared_to(here) < 17.0 * 17.0:
			return "night_lit"
	return "night_dark"

static func exposure(gait: String, sheltered: bool) -> float:
	## How much of a soldier's reach you fill. Still in a doorway is the least.
	var mul := 1.0
	if gait == "run":
		mul = 1.15
	elif gait == "still":
		mul = 0.8
	if sheltered:
		mul *= 0.55
	return mul

static func in_arc(facing_yaw: float, from: Vector3, to: Vector3, half_angle_deg: float) -> bool:
	var forward := Vector3(-sin(facing_yaw), 0, -cos(facing_yaw))
	var want := to - from
	want.y = 0
	if want.length() < 0.5:
		return true
	return rad_to_deg(forward.angle_to(want.normalized())) <= half_angle_deg

func clear_line(from: Vector3, to: Vector3) -> bool:
	var tree := get_tree()
	if tree == null or tree.root == null:
		return true
	var space := tree.root.get_world_3d().direct_space_state
	var query := PhysicsRayQueryParameters3D.create(from, to, 1)
	return space.intersect_ray(query).is_empty()

func _sees(group: Group, body: CharacterBody3D, player: Player, light: String, mul: float) -> bool:
	var from := body.global_position + EYE
	var to := player.global_position + CHEST
	var d := from.distance_to(to)
	var reach := RuForces.sight(light, bool(body.get_meta("optics", false))) * mul
	if d > reach:
		return false
	if group.state == PATROL or group.state == CHECK:
		var yaw := (body.get_node("Visual") as Node3D).rotation.y + body.rotation.y
		if not in_arc(yaw, from, to, RuForces.num("infantry_group", "view_half_angle_deg", 100.0)):
			return false
	return clear_line(from, to)

# ---------------------------------------------------------------- what a group does

func _physics_process(delta: float) -> void:
	if GameState.shot_mode or not GameState.started_play or GameState.paused or GameState.dead:
		return
	var player := _player()
	if player == null:
		return
	var here := player.global_position
	var light := light_at(here)
	var mul := exposure(GameState.gait, GameState.sheltered)
	var top := 0.0
	var challenged := false
	var engaged := false
	for group in _groups:
		if not is_instance_valid(group):
			continue
		var men := _alive(group)
		if men.is_empty():
			continue
		group.pos = men[0].global_position
		_think(group, men, player, light, mul, delta)
		top = maxf(top, group.meter)
		challenged = challenged or group.state == CHALLENGE or group.state == DETAIN
		engaged = engaged or group.state == ENGAGE
	_fire_cool = 8.0 if engaged else maxf(0.0, _fire_cool - delta)
	GameState.notice = 1.0 if (engaged or challenged) else clampf(top, 0.0, 1.0)
	GameState.challenged = challenged
	GameState.under_fire = _fire_cool > 0.0

func _think(group: Group, men: Array, player: Player, light: String, mul: float, delta: float) -> void:
	var info := RuForces.section("infantry_group")
	var here := player.global_position
	var seen := false
	var nearest := 1.0e9
	group.ignore_for = maxf(0.0, group.ignore_for - delta)
	for body in men:
		nearest = minf(nearest, body.global_position.distance_to(here))
		if group.ignore_for <= 0.0 and not seen and _sees(group, body, player, light, mul):
			seen = true
	if seen:
		group.last_seen = here
		group.lost_for = 0.0
		var reach := RuForces.sight(light, false) * mul
		var gain := delta / RuForces.notice_time(nearest, reach)
		group.meter = minf(1.0, group.meter + (gain * 3.0 if group.state == SEARCH else gain))
	else:
		group.lost_for += delta
		group.meter = maxf(0.0, group.meter - delta * 0.3)
	var hearing: Dictionary = info.get("hearing_m", {})
	if group.ignore_for <= 0.0 and group.state == PATROL and nearest <= float(hearing.get(GameState.gait, 0.0)):
		group.last_seen = here
		group.meter = maxf(group.meter, 0.4)
		_set_state(group, CHECK)

	match group.state:
		PATROL:
			_walk_route(group, men, delta)
			if group.meter >= 1.0:
				_set_state(group, CHALLENGE)
			elif group.meter >= 0.5:
				_set_state(group, CHECK)
		CHECK:
			_advance(men, group.last_seen, float(info.get("walk_mps", 1.3)), delta)
			group.timer -= delta
			if group.meter >= 1.0:
				_set_state(group, CHALLENGE)
			elif group.timer <= 0.0 and group.meter < 0.2:
				_set_state(group, PATROL)
		CHALLENGE:
			_halt(men, here)
			if not group.shouted:
				group.shouted = true
				group.held_at = here
				GameState.show_subtitle("有人用俄语喊：Стой！站住！站着别动，或者跑。", "A shout in Russian: Stoy! Halt! Stand still, or run.")
			group.timer -= delta
			if group.timer <= 0.0:
				var stood := here.distance_to(group.held_at) < 1.2
				if seen and stood:
					_set_state(group, DETAIN)
				elif seen:
					_set_state(group, ENGAGE)
				else:
					_set_state(group, SEARCH)
		DETAIN:
			if here.distance_to(group.held_at) > 1.6:
				GameState.show_subtitle("你一动，他们就开火了。", "You moved. They opened fire.")
				_set_state(group, ENGAGE)
			else:
				_advance(men, here, float(info.get("walk_mps", 1.3)), delta, 2.4)
				if nearest < 3.0:
					_rob(group)
		ENGAGE:
			for body in men:
				var from: Vector3 = body.global_position + EYE
				if seen and clear_line(from, here + CHEST):
					_halt([body], here)
					_shoot(body, player, light, mul)
				else:
					_move(body, group.last_seen, float(info.get("rush_mps", 3.1)), delta)
			if group.lost_for > float(info.get("lose_sight_s", 4.0)):
				_set_state(group, SEARCH)
		SEARCH:
			group.timer -= delta
			group.wander_for -= delta
			if group.wander_for <= 0.0:
				group.wander_for = randf_range(3.0, 6.0)
				group.wander = group.last_seen + Vector3(randf_range(-12, 12), 0, randf_range(-12, 12))
			var goal := group.last_seen if men[0].global_position.distance_to(group.last_seen) > 9.0 else group.wander
			_advance(men, goal, float(info.get("walk_mps", 1.3)) * 1.5, delta)
			if group.meter >= 1.0:
				_set_state(group, ENGAGE)
			elif group.timer <= 0.0:
				_set_state(group, PATROL)

func _set_state(group: Group, state: int) -> void:
	var info := RuForces.section("infantry_group")
	group.state = state
	match state:
		PATROL:
			group.meter = 0.0
			group.shouted = false
			if group.route.is_empty() and not _roads.is_empty():
				_join_nearest_road(group)
		CHECK:
			group.timer = 14.0
		CHALLENGE:
			group.timer = float(info.get("challenge_s", 1.6))
			group.shouted = false
		ENGAGE:
			group.meter = 1.0
			group.lost_for = 0.0
			# Rifles come up one after another, not as a single volley: that
			# second is the one you get to reach a wall.
			var now := Time.get_ticks_msec() / 1000.0
			for body in _alive(group):
				body.set_meta("next_shot", now + randf_range(0.5, 1.8))
			GameState.show_subtitle("他们朝你开枪了。断开视线。", "They are shooting at you. Break their line of sight.")
		SEARCH:
			group.timer = float(info.get("search_s", 45.0))
			group.wander_for = 0.0

func _rob(group: Group) -> void:
	var taken := 0
	for item in ["food", "water", "bandage", "warmer"]:
		taken += GameState.count(item)
		GameState.items[item] = 0
	GameState.items_changed.emit()
	if taken > 0:
		GameState.show_subtitle("他们搜了你的身，把东西全拿走了，推了你一把。", "They searched you, took everything, and shoved you aside.")
	else:
		GameState.show_subtitle("他们搜了你的身，什么也没找到，挥手让你滚。", "They searched you, found nothing, and waved you off.")
	group.ignore_for = RuForces.num("infantry_group", "calm_after_search_s", 60.0)
	_set_state(group, PATROL)

# ---------------------------------------------------------------- moving

func _walk_route(group: Group, men: Array, delta: float) -> void:
	if group.route.is_empty():
		return
	var speed := RuForces.num("infantry_group", "walk_mps", 1.3)
	var lead: CharacterBody3D = men[0]
	var goal := group.route[group.route_i]
	if lead.global_position.distance_to(goal) < 2.2:
		var next := group.route_i + group.route_dir
		if next < 0 or next >= group.route.size():
			_turn_at_end(group, lead.global_position)
		else:
			group.route_i = next
		goal = group.route[group.route_i]
	_move(lead, goal, speed, delta)
	var spacing := RuForces.num("infantry_group", "spacing_m", 7.0)
	for i in range(1, men.size()):
		var ahead: CharacterBody3D = men[i - 1]
		_move(men[i], ahead.global_position, speed * 1.15, delta, spacing)

func _turn_at_end(group: Group, at: Vector3) -> void:
	# Step onto a crossing street if one starts here. Otherwise walk back.
	var options: Array = []
	for road in _roads:
		if road == group.route:
			continue
		for i in road.size():
			if road[i].distance_to(at) < 20.0:
				options.append([road, i])
				break
	if options.is_empty():
		group.route_dir = -group.route_dir
		group.route_i = clampi(group.route_i + group.route_dir, 0, group.route.size() - 1)
		return
	var pick: Array = options[randi() % options.size()]
	group.route = pick[0]
	group.route_i = int(pick[1])
	group.route_dir = 1 if randf() < 0.5 else -1
	if group.route_i + group.route_dir < 0 or group.route_i + group.route_dir >= group.route.size():
		group.route_dir = -group.route_dir

func _join_nearest_road(group: Group) -> void:
	var best := 1.0e9
	for road in _roads:
		for i in road.size():
			var d := road[i].distance_to(group.pos)
			if d < best:
				best = d
				group.route = road
				group.route_i = i

func _advance(men: Array, goal: Vector3, speed: float, delta: float, stop: float = 1.2) -> void:
	for i in men.size():
		var spread := Vector3(float(i % 2) * 2.4 - 1.2, 0, -float(i) * 1.6)
		_move(men[i], goal + (spread if i > 0 else Vector3.ZERO), speed, delta, stop)

func _halt(men: Array, face: Vector3) -> void:
	for body in men:
		body.velocity = Vector3(0, body.velocity.y, 0)
		var want: Vector3 = face - body.global_position
		if want.length() > 0.3:
			(body.get_node("Visual") as Node3D).rotation.y = atan2(-want.x, -want.z)

func _move(body: CharacterBody3D, goal: Vector3, speed: float, delta: float, stop: float = 0.8) -> void:
	var wish := goal - body.global_position
	wish.y = 0
	if wish.length() > stop:
		wish = wish.normalized()
		# Sidestep when a wall has held him for a while.
		var stuck := float(body.get_meta("stuck", 0.0))
		if stuck > 1.2:
			wish = wish.rotated(Vector3.UP, deg_to_rad(75.0) * float(body.get_meta("side", 1.0)))
			if stuck > 2.6:
				body.set_meta("stuck", 0.0)
				body.set_meta("side", -float(body.get_meta("side", 1.0)))
		var before := body.global_position
		body.velocity.x = wish.x * speed
		body.velocity.z = wish.z * speed
		(body.get_node("Visual") as Node3D).rotation.y = atan2(-wish.x, -wish.z)
		body.velocity.y -= 20.0 * delta
		body.move_and_slide()
		var moved := body.global_position.distance_to(before)
		body.set_meta("stuck", stuck + delta if moved < speed * delta * 0.35 else maxf(0.0, stuck - delta * 2.0))
	else:
		body.velocity.x = 0
		body.velocity.z = 0
		body.velocity.y -= 20.0 * delta
		body.move_and_slide()

# ---------------------------------------------------------------- shooting

func _shoot(body: CharacterBody3D, player: Player, light: String, mul: float) -> void:
	var now := Time.get_ticks_msec() / 1000.0
	if now < float(body.get_meta("next_shot", 0.0)):
		return
	body.set_meta("next_shot", now + RuForces.pick("rifle", "burst_gap_s"))
	var flash := body.get_node_or_null("Flash") as OmniLight3D
	if flash:
		flash.light_energy = 8.0
		create_tween().tween_property(flash, "light_energy", 0.0, 0.12)
	var muzzle := body.get_node_or_null("Muzzle") as MeshInstance3D
	if muzzle:
		muzzle.visible = true
		get_tree().create_timer(0.06).timeout.connect(func():
			if is_instance_valid(muzzle):
				muzzle.visible = false
		)
	var rifle := RuForces.section("rifle")
	var distance := body.global_position.distance_to(player.global_position)
	var chance := RuForces.hit_chance(distance)
	if GameState.gait != "still":
		chance *= float(rifle.get("moving_target_mul", 0.6))
	if GameState.sheltered:
		chance *= float(rifle.get("cover_mul", 0.45))
	if light == "night_dark" and not bool(body.get_meta("optics", false)):
		chance *= float(rifle.get("dark_mul", 0.5))
	var aim := player.global_position + Vector3(randf_range(-1.2, 1.2), 1.0, randf_range(-1.2, 1.2))
	if randf() < chance:
		aim = player.global_position + CHEST
		CombatHooks.hurt_player(RuForces.pick("rifle", "wound_hp"), "你被步枪打中，倒在了雪里。", "A rifle round put you down in the snow.", float(rifle.get("bleed_p", 0.85)))
	_tracer(body.global_position + Vector3(0, 1.3, 0), aim)

func hurt_soldier(body: Node, amount: float) -> void:
	var hp := float(body.get_meta("hp", 40.0)) - amount
	body.set_meta("hp", hp)
	if hp > 0.0:
		return
	body.remove_from_group("soldier")
	body.add_to_group("corpse")
	if body is CollisionObject3D:
		(body as CollisionObject3D).collision_layer = 0
		(body as CollisionObject3D).collision_mask = 0
	var visual := body.get_node_or_null("Visual") as Node3D
	if visual:
		# He stays where he fell.
		visual.rotation.x = -PI * 0.5
		visual.position.y = 0.25
	var flash := body.get_node_or_null("Flash") as OmniLight3D
	if flash:
		flash.light_energy = 0.0

func soldiers() -> Array:
	return get_tree().get_nodes_in_group("soldier")

func _alive(group: Group) -> Array:
	var out: Array = []
	for child in group.get_children():
		if child is CharacterBody3D and child.is_in_group("soldier"):
			out.append(child)
	return out

func _boom(at: Vector3, radius: float, player_dmg: float, building_dmg: float) -> void:
	var guns := get_tree().get_first_node_in_group("ordinance")
	if guns and guns.has_method("burst") and not GameState.shot_mode:
		guns.burst(at, false)
	var destruction := _destruction()
	if destruction:
		destruction.damage_at(at + Vector3(0, 3, 0), radius + 4.0, building_dmg)
	var player := _player()
	if player and player_dmg > 0.0:
		var d := player.global_position.distance_to(at)
		if d < radius:
			var share := 1.0 - 0.6 * (d / radius)
			if GameState.sheltered:
				share *= 0.4
			CombatHooks.hurt_player(player_dmg * share * 2.2, "一发炮弹落在你身边。", "A shell landed next to you.", 0.9)
	for soldier in soldiers():
		if soldier.global_position.distance_to(at) < radius:
			hurt_soldier(soldier, 80.0)
	_fire_cool = 8.0
	GameState.boom.emit()

func _tracer(a: Vector3, b: Vector3) -> void:
	if a.distance_to(b) < 0.2:
		return
	var mesh := MeshInstance3D.new()
	var box := BoxMesh.new()
	box.size = Vector3(0.04, 0.04, a.distance_to(b))
	mesh.mesh = box
	var mat := StandardMaterial3D.new()
	mat.shading_mode = BaseMaterial3D.SHADING_MODE_UNSHADED
	mat.albedo_color = Color(1, 0.78, 0.35)
	mat.emission_enabled = true
	mat.emission = Color(1, 0.6, 0.2)
	mat.emission_energy_multiplier = 3.0
	mesh.material_override = mat
	add_child(mesh)
	# In the tree first, then turned: a node outside the tree cannot look at anything.
	mesh.look_at_from_position((a + b) * 0.5, b, Vector3.UP if absf((b - a).normalized().y) < 0.98 else Vector3.RIGHT)
	get_tree().create_timer(0.12).timeout.connect(mesh.queue_free)

func _player() -> Player:
	return get_tree().get_first_node_in_group("player") as Player

func _destruction() -> Destruction:
	return get_tree().get_first_node_in_group("destruction") as Destruction
