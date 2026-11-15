extends RefCounted
class_name RuForces
## Reads data/russia/forces.json. The table is the only place a Russian
## weapon's numbers live. Code asks for them here and never hard-codes one.

const PATH := "res://data/russia/forces.json"

static var _data: Dictionary = {}

static func table() -> Dictionary:
	if _data.is_empty():
		var parsed = JSON.parse_string(FileAccess.get_file_as_string(PATH))
		if parsed is Dictionary:
			_data = parsed
		else:
			push_error("forces.json did not parse")
	return _data

static func section(name: String) -> Dictionary:
	return table().get(name, {})

static func num(part: String, key: String, fallback: float = 0.0) -> float:
	return float(section(part).get(key, fallback))

static func pick(part: String, key: String) -> float:
	## A [low, high] pair in the table, rolled once.
	var pair = section(part).get(key, [0.0, 0.0])
	if pair is Array and pair.size() >= 2:
		return randf_range(float(pair[0]), float(pair[1]))
	return float(pair)

static func pick_int(part: String, key: String) -> int:
	var pair = section(part).get(key, [1, 1])
	if pair is Array and pair.size() >= 2:
		return randi_range(int(pair[0]), int(pair[1]))
	return int(pair)

static func war_minutes(minutes: float) -> float:
	## Minutes on the war clock, as seconds of play.
	return minutes * 60.0 / GameState.TIME_SCALE

static func hit_chance(distance: float) -> float:
	## Straight lines between the points of rifle.hit_p_by_range_m.
	var rows: Array = section("rifle").get("hit_p_by_range_m", [])
	if rows.is_empty():
		return 0.0
	if distance <= float(rows[0][0]):
		return float(rows[0][1])
	for i in range(1, rows.size()):
		var a: Array = rows[i - 1]
		var b: Array = rows[i]
		if distance <= float(b[0]):
			var t := (distance - float(a[0])) / maxf(0.001, float(b[0]) - float(a[0]))
			return lerpf(float(a[1]), float(b[1]), t)
	return 0.0

static func sight(light: String, has_optics: bool) -> float:
	var info := section("infantry_group")
	var by_light: Dictionary = info.get("sight_m", {})
	var reach := float(by_light.get(light, 30.0))
	if has_optics and light.begins_with("night"):
		reach = maxf(reach, float(info.get("night_optics_sight_m", reach)))
	return reach

static func notice_time(distance: float, reach: float) -> float:
	## Seconds of unbroken sight before a soldier is sure of what he sees.
	var span: Dictionary = section("infantry_group").get("notice_s", {})
	var t := clampf(distance / maxf(1.0, reach), 0.0, 1.0)
	return lerpf(float(span.get("near", 0.5)), float(span.get("far", 2.5)), t)
