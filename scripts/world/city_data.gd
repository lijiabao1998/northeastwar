extends Node
class_name CityData
## Parsed slice. Gameplay reads this; the mesh itself is the glTF.

var data: Dictionary = {}
var bank: PackedVector2Array = PackedVector2Array()
var ice_limit: PackedVector2Array = PackedVector2Array()
var walk_min_x: float = -400.0
var walk_max_x: float = 400.0
var walk_min_z: float = -400.0

func _ready() -> void:
	var text := FileAccess.get_file_as_string("res://data/heihe/city.json")
	data = JSON.parse_string(text)
	if data == null:
		push_error("city.json failed to parse")
		data = {}
		return
	bank = _poly(data.get("bank", []))
	ice_limit = _poly(data.get("ice_limit", []))
	var wx: Array = data.get("meta", {}).get("walkable_x", [-400, 400])
	var wz: Array = data.get("meta", {}).get("walkable_z", [-400, 40])
	walk_min_x = float(wx[0]) + 6.0
	walk_max_x = float(wx[1]) - 6.0
	walk_min_z = float(wz[0]) + 4.0

func spawn() -> Dictionary:
	return data.get("meta", {}).get("spawn", {"x": 0, "z": -80, "yaw": PI})

func mother() -> Vector2:
	var m: Dictionary = data.get("mother", {"x": 0, "z": 0})
	return Vector2(float(m["x"]), float(m["z"]))

func bank_z(x: float) -> float:
	return _sample(bank, x, 40.0)

func ice_z(x: float) -> float:
	return _sample(ice_limit, x, bank_z(x) + 80.0)

func _poly(rows: Array) -> PackedVector2Array:
	var out := PackedVector2Array()
	for row in rows:
		out.append(Vector2(float(row[0]), float(row[1])))
	return out

func _sample(poly: PackedVector2Array, x: float, fallback: float) -> float:
	if poly.size() < 2:
		return fallback
	if x <= poly[0].x:
		return poly[0].y
	var last := poly.size() - 1
	if x >= poly[last].x:
		return poly[last].y
	for i in last:
		var a := poly[i]
		var b := poly[i + 1]
		if a.x <= x and x <= b.x:
			var t := (x - a.x) / maxf(0.001, b.x - a.x)
			return lerpf(a.y, b.y, t)
	return fallback
