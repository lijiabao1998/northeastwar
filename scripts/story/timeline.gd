extends Node
class_name DayOne
## 17 January 2031 and the night after it. The war is a clock: it does what it
## does at the minute it does it, whoever you are. Until 17:36 Heihe is an
## ordinary winter evening and the shops are open. After that the only goal
## is to be alive at 07:00.

var city: CityData
var player: Player
var cold: ColdMeter
var _fired := {}
var _saw_river := false

func _ready() -> void:
	GameState.night_survived.connect(_dawn)

func _process(_delta: float) -> void:
	if GameState.shot_mode or not GameState.started_play or GameState.paused or GameState.dead:
		return
	var h := GameState.hour
	if _due(h, 16, 4, "open"):
		GameState.show_subtitle("你在中央街下了班。天已经擦黑了。", "You're off work on Central Street. The light is already going.")
		GameState.show_objective("趁店还开着，备点吃的、水和绷带（店招底下按住 E）。\n赵小北在江边等你，母亲广场在北边。", "While the shops are open, get food, water and bandages (hold E under a shop sign).\nZhao Xiaobei is waiting by the river. Mother Square is north.")
	if _due(h, 16, 20, "walk"):
		GameState.show_subtitle("顺着街往北，江边风硬，可你说好去看看。", "North, toward the river. The wind is sharp, but you said you'd go.")
	if _due(h, 16, 50, "friend"):
		GameState.push_phone(
			"赵小北",
			"我在母亲广场这边。江风挺硬，你还来不？对岸灯都亮了。",
			"I'm by Mother Square. The river wind is nasty. You still coming? Their lights are all on."
		)
		GameState.show_subtitle("手机震了一下。赵小北在江边等你。", "Your phone buzzes. Zhao Xiaobei is waiting by the river.")
	if _due(h, 17, 22, "unease"):
		GameState.show_subtitle("江对岸有人在喊，听不清。天上像有闷雷。", "Shouting on the far bank, too far to make out. A dull crack in the sky.")
		GameState.set_alert_stage(1)
	if _due(h, 17, 36, "alert"):
		GameState.set_alert_stage(2)
		GameState.push_phone(
			"黑河市应急广播",
			"紧急通知：监测到俄军正自封冻的黑龙江越境，向市区推进。请立即远离江岸和江面，就近躲避，注意保暖。请勿围观。",
			"Emergency: Russian troops are crossing the frozen Heilongjiang toward the city. Leave the riverfront and the ice. Take shelter. Keep warm. Do not gather to watch."
		)
		GameState.show_subtitle("警报。不是演习。", "The alert. It says this is not a drill.")
		GameState.show_objective("活到天亮（07:00）。别让他们看见你：别跑，躲开路灯，让楼挡在中间。\n被喝令时：站住会被搜身，跑会挨枪。  H：留在中央街守一夜。", "Be alive at 07:00. Don't be seen: don't run, stay out of the lamplight, keep a building between you.\nWhen challenged: stand still and be searched, or run and be shot at.  H: hold Central Street instead.")
	if _due(h, 17, 52, "follow"):
		GameState.push_phone(
			"黑河市应急广播",
			"供电可能在今夜中断。远离黑龙江公园。照看好身边的人。",
			"Power may fail tonight. Stay away from Heilongjiang Park. Look after whoever is with you."
		)
	if _due(h, 19, 30, "blackout") and not GameState.defend_night:
		GameState.blackout = true
		GameState.show_subtitle("全城的灯一起灭了。黑暗现在是你的。", "Every light in the city goes out at once. The dark is yours now.")
	if _due(h, 21, 0, "heat") and not GameState.defend_night:
		GameState.show_subtitle("暖气片凉了。供热停了，门脸里只剩下挡风。", "The radiators have gone cold. Doorways only keep the wind off now.")
	_river_line()

func _due(now: float, hours: int, minutes: int, id: String) -> bool:
	## True once, at hours:minutes on the clock. 17:36 means 17:36.
	if _fired.has(id) or now < GameState.hm(hours, minutes):
		return false
	_fired[id] = true
	return true

func _river_line() -> void:
	if _saw_river or player == null or city == null:
		return
	if player.global_position.z > city.bank_z(player.global_position.x) - 18.0:
		_saw_river = true
		GameState.show_subtitle("江对岸就是布拉戈维申斯克。灯一盏一盏的。", "Blagoveshchensk is just across the ice. Light after light.")

func _dawn() -> void:
	GameState.show_subtitle("天亮了。你还活着。", "Dawn. You are still alive.")
	GameState.show_objective("第一夜过去了。", "The first night is over.")
