extends Node
class_name Scavenge
## What you carry and where it comes from. Every shop sign on the map is a
## door you can search once. What is behind it depends on the kind of shop.
## Searching means standing still with your back to the street for a few
## seconds, which is the price.

const SEARCH_SECONDS := 2.6
const REACH := 3.4

const NAMES := {
	"food": ["吃的", "food"],
	"water": ["水", "water"],
	"bandage": ["绷带", "bandage"],
	"warmer": ["暖宝宝", "hand warmer"],
}

## kind -> [[item, low, high], ...]
const TABLES := {
	"pharmacy": [["bandage", 2, 3], ["warmer", 0, 2], ["water", 0, 1]],
	"food": [["food", 1, 3], ["water", 1, 2]],
	"drink": [["water", 2, 3], ["food", 0, 1]],
	"hotel": [["water", 1, 2], ["warmer", 1, 2], ["food", 0, 1], ["bandage", 0, 1]],
	"clothes": [["coat", 1, 1], ["warmer", 0, 1]],
	"office": [["water", 0, 1]],
	"shop": [["food", 0, 2], ["water", 0, 1], ["warmer", 0, 1], ["bandage", 0, 1]],
}

var player: Player
var _doors: Array = []
var _hold: float = 0.0
var _near: int = -1

func build(data: Dictionary) -> void:
	for sign in data.get("signs", []):
		if not sign is Dictionary or str(sign.get("kind", "")) != "shop":
			continue
		var zh := str(sign.get("text", ""))
		var ru := str(sign.get("text_ru", ""))
		_doors.append({
			"x": float(sign.get("x", 0.0)),
			"z": float(sign.get("z", 0.0)),
			"name": zh if zh != "" else ru,
			"kind": kind_of(zh, ru),
			"done": false,
		})

static func kind_of(zh: String, ru: String) -> String:
	## The kind of shop, from what its sign says.
	var ru_low := ru.to_lower()
	if _has(zh, ["药", "牙科", "诊所", "医院", "卫生"]) or _has(ru_low, ["аптек", "стомат"]):
		return "pharmacy"
	if _has(zh, ["酒店", "宾馆", "旅馆", "饭店"]) and not _has(zh, ["饺", "面馆"]):
		return "hotel"
	if _has(zh, ["咖啡", "茶", "蜜雪", "酒吧", "奶"]) or _has(ru_low, ["чай", "бар", "кофе"]):
		return "drink"
	if _has(zh, ["肯德基", "比萨", "餐", "饺", "面", "烧烤", "串", "超市", "便利", "食品", "粮", "果", "肉"]) or _has(ru_low, ["ресторан", "пицц", "пельмен", "продукт", "кафе", "бургер"]):
		return "food"
	if _has(zh, ["以纯", "服", "鞋", "衣", "皮草", "羽绒"]) or _has(ru_low, ["одежд", "шуб", "обув"]):
		return "clothes"
	if _has(zh, ["银行", "邮政", "博物馆", "纪念馆", "文化宫", "台球"]) or _has(ru_low, ["банк"]):
		return "office"
	return "shop"

static func _has(text: String, words: Array) -> bool:
	for word in words:
		if text.find(str(word)) >= 0:
			return true
	return false

static func roll(kind: String) -> Dictionary:
	var out := {}
	for row in TABLES.get(kind, TABLES["shop"]):
		var n := randi_range(int(row[1]), int(row[2]))
		if n > 0:
			out[str(row[0])] = n
	return out

func _process(delta: float) -> void:
	if player == null or not GameState.started_play or GameState.paused or GameState.dead or GameState.defend_night:
		GameState.prompt = ""
		return
	_near = _door_near(player.global_position)
	if _near < 0:
		_hold = 0.0
		GameState.prompt = ""
		return
	var door: Dictionary = _doors[_near]
	var verb := "买" if GameState.alert_stage < 2 else "搜"
	if Input.is_action_pressed("interact") and GameState.gait == "still":
		_hold += delta
		GameState.prompt = "%s %s … %d%%" % [verb, door["name"], int(100.0 * _hold / SEARCH_SECONDS)]
		if _hold >= SEARCH_SECONDS:
			_take(door)
	else:
		_hold = 0.0
		GameState.prompt = "按住 E：%s %s" % [verb, door["name"]]

func _door_near(at: Vector3) -> int:
	var best := -1
	var best_d := REACH * REACH
	for i in _doors.size():
		var door: Dictionary = _doors[i]
		if door["done"]:
			continue
		var dx := at.x - float(door["x"])
		var dz := at.z - float(door["z"])
		var d := dx * dx + dz * dz
		if d < best_d:
			best_d = d
			best = i
	return best

func _take(door: Dictionary) -> void:
	door["done"] = true
	_hold = 0.0
	var found := roll(str(door["kind"]))
	var parts: PackedStringArray = []
	for item in found:
		if item == "coat":
			if GameState.coat_bonus < 0.35:
				GameState.coat_bonus = 0.35
				parts.append("一件厚棉衣（穿上了）")
			continue
		GameState.give(item, int(found[item]))
		parts.append("%s ×%d" % [NAMES[item][0], int(found[item])])
	if parts.is_empty():
		GameState.show_subtitle("%s：柜台是空的。" % door["name"], "Nothing left behind the counter.")
	else:
		GameState.show_subtitle("%s：%s" % [door["name"], "，".join(parts)], "You take what you can carry.")
	GameState.items_changed.emit()

func _unhandled_input(event: InputEvent) -> void:
	if not GameState.started_play or GameState.paused or GameState.dead:
		return
	if event.is_action_pressed("use_bandage"):
		if GameState.bleeding or GameState.player_hp < 100.0:
			if GameState.take("bandage"):
				GameState.bleeding = false
				GameState.player_hp = minf(100.0, GameState.player_hp + 12.0)
				GameState.show_subtitle("你把伤口扎紧了。血止住了。", "You bind the wound tight. The bleeding stops.")
			else:
				GameState.show_subtitle("没有绷带。", "No bandage.")
	elif event.is_action_pressed("use_food"):
		if GameState.take("food"):
			GameState.hunger = minf(100.0, GameState.hunger + 45.0)
			GameState.show_subtitle("你站着把东西吃了。", "You eat standing up.")
	elif event.is_action_pressed("use_water"):
		if GameState.take("water"):
			GameState.thirst = minf(100.0, GameState.thirst + 50.0)
			GameState.show_subtitle("水冰得牙疼。", "The water is cold enough to hurt your teeth.")
	elif event.is_action_pressed("use_warmer"):
		if GameState.take("warmer"):
			GameState.body_temp = minf(36.9, GameState.body_temp + 0.7)
			GameState.show_subtitle("暖宝宝贴在胸口，热起来了。", "The warmer goes against your chest.")
