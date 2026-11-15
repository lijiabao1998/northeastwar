extends Node
class_name DayOne
## Evening of 17 January 2031, about eight real minutes. The war is a clock.
## You can walk wherever the streets allow. You cannot stop what crosses the river.

var city: CityData
var player: Player
var cold: ColdMeter
var _fired := {}
var _saw_river := false
var _closed := false

func _process(delta: float) -> void:
	if GameState.shot_mode or not GameState.started_play or GameState.paused:
		return
	var h := GameState.hour
	_beat(16.04, "open", "你在中央街下了班。天已经擦黑了。", "You're off work on Central Street. The light is already going.")
	_beat(16.20, "walk", "顺着街往北，江边风硬，可你说好去看看。", "North, toward the river. The wind is sharp, but you said you'd go.")
	if h >= 16.50 and not _fired.has("friend"):
		_fired["friend"] = true
		GameState.push_phone(
			"赵小北",
			"我在母亲广场这边。江风挺硬，你还来不？对岸灯都亮了。",
			"I'm by Mother Square. The river wind is nasty. You still coming? Their lights are all on."
		)
		GameState.show_subtitle("手机震了一下。赵小北在江边等你。", "Your phone buzzes. Zhao Xiaobei is waiting by the river.")
	if h >= 17.22 and not _fired.has("unease"):
		_fired["unease"] = true
		GameState.show_subtitle("江对岸有人在喊，听不清。天上像有闷雷。", "Shouting on the far bank, too far to make out. A dull crack in the sky.")
		GameState.set_alert_stage(1)
	if h >= 17.36 and not _fired.has("alert"):
		_fired["alert"] = true
		GameState.set_alert_stage(2)
		GameState.push_phone(
			"黑河市应急广播",
			"紧急通知：监测到俄军正自封冻的黑龙江越境，向市区推进。请立即远离江岸和江面，就近躲避，注意保暖。请勿围观。",
			"Emergency: Russian troops are crossing the frozen Heilongjiang toward the city. Leave the riverfront and the ice. Take shelter. Keep warm. Do not gather to watch."
		)
		GameState.show_subtitle("警报。不是演习。", "The alert. It says this is not a drill.")
		GameState.show_objective("离开江面。若留下守街，按 H。你也可以继续走。", "Get off the ice. Press H to hold the street, or keep walking.")
	if h >= 17.52 and not _fired.has("follow"):
		_fired["follow"] = true
		GameState.push_phone(
			"黑河市应急广播",
			"供电可能在今夜中断。远离黑龙江公园。照看好身边的人。",
			"Power may fail tonight. Stay away from Heilongjiang Park. Look after whoever is with you."
		)
	_river_line()
	if cold and GameState.alert_stage >= 2 and cold.sheltered_seconds() > 12.0:
		_close("你在门脸底下站住，手机还在震。这一夜才刚刚开始。", "You stop under the shop sign. The phone is still buzzing. The night has only started.")
	elif h >= 18.08:
		_close("街灯还亮着。江面那头的闪光没有停。第一日，傍晚。", "The streetlights are still on. The flashes across the ice haven't stopped. Day one, evening.")

func _beat(hour: float, id: String, zh: String, en: String) -> void:
	if GameState.hour < hour or _fired.has(id):
		return
	_fired[id] = true
	GameState.show_subtitle(zh, en)
	if id == "open":
		GameState.show_objective("往江边走。母亲广场在北边。", "Walk to the river. Mother Square is to the north.")

func _river_line() -> void:
	if _saw_river or player == null or city == null:
		return
	if player.global_position.z > city.bank_z(player.global_position.x) - 18.0:
		_saw_river = true
		GameState.show_subtitle("江对岸就是布拉戈维申斯克。灯一盏一盏的。", "Blagoveshchensk is just across the ice. Light after light.")
		if GameState.alert_stage < 2:
			GameState.show_objective("在江边站一会儿。", "Stay by the river a moment.")

func _close(zh: String, en: String) -> void:
	if _closed:
		return
	_closed = true
	GameState.show_subtitle(zh, en)
	GameState.show_objective("你可以继续走。这一段到这儿。", "You can keep walking. This is the end of the slice.")
