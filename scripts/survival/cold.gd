extends Node
class_name ColdMeter
## Body temperature. The heating plant is still running on the evening of
## day 1, but the street is about -24°C and the river wind is worse.

var player: Player
var city: CityData
var _sheltered_for: float = 0.0

func _process(delta: float) -> void:
	if player == null or city == null or not GameState.started_play or GameState.paused:
		return
	var x := player.global_position.x
	var z := player.global_position.z
	var on_ice := z > city.bank_z(x) + 2.0
	var sheltered := _near_heat(x, z)
	var exposed := 1.0
	if sheltered:
		exposed = 0.0
	elif on_ice:
		exposed = 1.75
	if Input.is_action_pressed("sprint") and player.velocity.length() > 2.0:
		exposed *= 1.25
	var temp := GameState.body_temp
	if sheltered:
		temp = minf(36.9, temp + 0.014 * delta)
		_sheltered_for += delta
	else:
		temp -= 0.0042 * exposed * delta
		_sheltered_for = 0.0
	temp = clampf(temp, 33.6, 37.2)
	var scale := 1.0
	if temp < 34.2:
		scale = 0.7
	elif temp < 35.0:
		scale = 0.86
	GameState.set_temperature(temp, on_ice, sheltered, scale)

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
