extends Node
class_name ColdMeter
## Body temperature, hunger, thirst and blood. The heating plant is still
## running on the evening of day 1, but the street is about -24°C and the
## river wind is worse. A warm doorway stops the cold. It does not stop
## bleeding: that takes a bandage.

const DEATH_TEMP := 32.0

var player: Player
var city: CityData
var _sheltered_for: float = 0.0
var _warned := {}

func _process(delta: float) -> void:
	if player == null or city == null or not GameState.started_play or GameState.paused or GameState.dead:
		return
	if GameState.defend_night and GameState.defend_phase == "night":
		# The defend night runs its own cold (fire barrels). Blood and food still count.
		_blood(delta)
		return
	var x := player.global_position.x
	var z := player.global_position.z
	var on_ice := z > city.bank_z(x) + 2.0
	var sheltered := _near_heat(x, z)
	var exposed := 1.0
	# The district heating dies at 21:00. After that a doorway is only out of the wind.
	var heated := GameState.hour < GameState.hm(21, 0)
	if sheltered:
		exposed = 0.0 if heated else 0.22
	elif on_ice:
		exposed = 1.75
	if GameState.gait == "run":
		exposed *= 1.25
	# After dark it is colder, and an empty stomach or a dry mouth makes it worse.
	if GameState.hour > GameState.hm(19, 0):
		exposed *= 1.3
	if GameState.hunger < 25.0:
		exposed *= 1.25
	exposed *= 1.0 - GameState.coat_bonus
	var temp := GameState.body_temp
	if sheltered and heated:
		temp = minf(36.9, temp + 0.014 * delta)
		_sheltered_for += delta
	else:
		temp -= 0.0042 * exposed * delta
		_sheltered_for = 0.0
	temp = clampf(temp, DEATH_TEMP - 0.1, 37.2)
	var scale := 1.0
	if temp < 34.2:
		scale = 0.7
	elif temp < 35.0:
		scale = 0.86
	if GameState.thirst < 20.0:
		scale *= 0.85
	if GameState.player_hp < 40.0:
		scale *= 0.85
	GameState.set_temperature(temp, on_ice, sheltered, scale)
	if temp < 34.2:
		_warn("cold", "你在发抖，手不听使唤了。找个有暖气的门脸。", "You're shaking and your hands won't work. Find a heated doorway.")
	if temp <= DEATH_TEMP:
		GameState.kill("你冻死在黑河的街上。", "You froze to death on a street in Heihe.")
		return
	_needs(delta)
	_blood(delta)

func _needs(delta: float) -> void:
	# A meal lasts about ten hours on the clock, a drink about six.
	var hours := delta * GameState.TIME_SCALE / 3600.0
	GameState.hunger = maxf(0.0, GameState.hunger - hours * 10.0)
	GameState.thirst = maxf(0.0, GameState.thirst - hours * 16.0)
	if GameState.thirst < 20.0:
		_warn("thirst", "嘴唇裂了。得找水。", "Your lips have split. You need water.")
	if GameState.hunger < 20.0:
		_warn("hunger", "饿得发慌，身上更冷了。", "Hunger. The cold bites harder on an empty stomach.")
	if GameState.thirst <= 0.0:
		CombatHooks.hurt_player(1.2 * delta, "你渴死了。", "You died of thirst.", 0.0)

func _blood(delta: float) -> void:
	if not GameState.bleeding:
		return
	CombatHooks.hurt_player(0.55 * delta, "你的血流干了。", "You bled out.", 0.0)

func _warn(key: String, zh: String, en: String) -> void:
	if _warned.has(key):
		return
	_warned[key] = true
	GameState.show_subtitle(zh, en)

func sheltered_seconds() -> float:
	return _sheltered_for

func _near_heat(x: float, z: float) -> bool:
	for heat in city.data.get("heat", []):
		var dx := x - float(heat["x"])
		var dz := z - float(heat["z"])
		var r := float(heat.get("radius", 3.0))
		if dx * dx + dz * dz <= r * r:
			return true
	return false
