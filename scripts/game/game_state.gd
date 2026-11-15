extends Node
## Shared clock, quality and the signals the rest of the slice listens to.
## This is an autoload so world, player, UI and story stay decoupled.

signal quality_changed(level: String)
signal started
signal paused_changed(is_paused: bool)
signal subtitle_changed(zh: String, en: String)
signal objective_changed(zh: String, en: String)
signal phone_message(title: String, body_zh: String, body_en: String)
signal alert_stage_changed(stage: int)
signal boom
signal temperature_changed(celsius: float)
signal died(cause_zh: String, cause_en: String)
signal items_changed
signal night_survived

const TIME_SCALE := 15.0
const LEVELS := ["low", "medium", "high"]

var hour: float = 16.0
var started_play: bool = false
var paused: bool = false
var quality: String = "medium"
var alert_stage: int = 0
var body_temp: float = 36.8
var ambient_c: float = -24.0
var on_ice: bool = false
var sheltered: bool = false
var move_scale: float = 1.0
var shot_mode: bool = false
var under_fire: bool = false
var player_hp: float = 100.0
var bleeding: bool = false
var defend_night: bool = false
var defend_phase: String = ""
var scrap: int = 0
var coal_stock: int = 0
var radio_unlocked: bool = false
var neighbours_rescued: bool = false
var blackout: bool = false
var revealed: bool = false
var hold_hour: float = 21.0
## Survival layer. One life: when `dead` is set the run is over.
var dead: bool = false
var death_zh: String = ""
var death_en: String = ""
var hunger: float = 100.0
var thirst: float = 100.0
var items: Dictionary = {}
var coat_bonus: float = 0.0
## 0 = nobody has noticed you, 1 = a soldier is sure of what he sees.
var notice: float = 0.0
var challenged: bool = false
var drone_overhead: bool = false
var drone_sees: bool = false
## still, walk or run. Soldiers hear the last two.
var gait: String = "still"
## The line the HUD shows when there is something to do right here.
var prompt: String = ""
## Only true while you hold a weapon (the defend night hands you one).
var armed: bool = false
var hours_alive: float = 0.0
var dawn_seen: bool = false

const END_HOUR := 31.0

func _ready() -> void:
	_ensure_actions()
	var cfg := ConfigFile.new()
	if cfg.load("user://settings.cfg") == OK:
		quality = str(cfg.get_value("video", "quality", "medium"))
		if quality not in LEVELS:
			quality = "medium"
	var args := OS.get_cmdline_user_args()
	shot_mode = "--shots" in args or "--combat-shots" in args or "--fidelity-shots" in args or "--hold-shots" in args
	if shot_mode:
		started_play = true
		quality = "high" if "--fidelity-shots" in args else "medium"

func tick(delta: float) -> void:
	if not started_play or paused or shot_mode:
		return
	if dead:
		return
	var step := delta * TIME_SCALE / 3600.0
	hour = minf(hour + step, END_HOUR)
	hours_alive += step
	if hour >= END_HOUR and not dawn_seen:
		dawn_seen = true
		night_survived.emit()

func begin() -> void:
	if started_play:
		return
	started_play = true
	hour = 16.0
	started.emit()

func set_paused(value: bool) -> void:
	if paused == value:
		return
	paused = value
	paused_changed.emit(paused)
	Input.mouse_mode = Input.MOUSE_MODE_VISIBLE

func set_quality(level: String) -> void:
	if level not in LEVELS or level == quality:
		return
	quality = level
	var cfg := ConfigFile.new()
	cfg.load("user://settings.cfg")
	cfg.set_value("video", "quality", quality)
	cfg.save("user://settings.cfg")
	quality_changed.emit(quality)

func show_subtitle(zh: String, en: String) -> void:
	subtitle_changed.emit(zh, en)

func show_objective(zh: String, en: String) -> void:
	objective_changed.emit(zh, en)

func push_phone(title: String, body_zh: String, body_en: String) -> void:
	phone_message.emit(title, body_zh, body_en)

func set_alert_stage(stage: int) -> void:
	if stage == alert_stage:
		return
	alert_stage = stage
	alert_stage_changed.emit(stage)

func set_temperature(celsius: float, ice: bool, warm: bool, scale: float) -> void:
	body_temp = celsius
	on_ice = ice
	sheltered = warm
	move_scale = scale
	temperature_changed.emit(celsius)

func clock_text() -> String:
	var h := int(hour)
	var m := int((hour - float(h)) * 60.0)
	return "%02d:%02d" % [h % 24, m]

func date_text() -> String:
	return "2031年1月%d日" % (17 + int(hour) / 24)

static func hm(hours: int, minutes: int) -> float:
	## A time of day as the clock's decimal hours. 17:36 is hm(17, 36), not 17.36.
	return float(hours) + float(minutes) / 60.0

func light() -> String:
	## day, dusk, night_lit or night_dark. Soldiers and drones see by this.
	if hour < hm(16, 30):
		return "day"
	if hour < hm(17, 15):
		return "dusk"
	return "night_dark" if blackout else "night_lit"

func kill(cause_zh: String, cause_en: String) -> void:
	if dead or shot_mode:
		return
	dead = true
	death_zh = cause_zh
	death_en = cause_en
	player_hp = 0.0
	died.emit(cause_zh, cause_en)

func give(item: String, count: int = 1) -> void:
	items[item] = int(items.get(item, 0)) + count
	items_changed.emit()

func take(item: String, count: int = 1) -> bool:
	var have := int(items.get(item, 0))
	if have < count:
		return false
	items[item] = have - count
	items_changed.emit()
	return true

func count(item: String) -> int:
	return int(items.get(item, 0))

func reset_run() -> void:
	## Back to 16:00 on day one. Called before the scene is reloaded.
	hour = 16.0
	started_play = false
	paused = false
	alert_stage = 0
	body_temp = 36.8
	on_ice = false
	sheltered = false
	move_scale = 1.0
	under_fire = false
	player_hp = 100.0
	bleeding = false
	defend_night = false
	defend_phase = ""
	scrap = 0
	coal_stock = 0
	radio_unlocked = false
	neighbours_rescued = false
	blackout = false
	revealed = false
	hold_hour = 21.0
	dead = false
	death_zh = ""
	death_en = ""
	hunger = 100.0
	thirst = 100.0
	items = {}
	coat_bonus = 0.0
	notice = 0.0
	challenged = false
	drone_overhead = false
	drone_sees = false
	gait = "still"
	prompt = ""
	armed = false
	hours_alive = 0.0
	dawn_seen = false

func _ensure_actions() -> void:
	_action("move_forward", [KEY_W, KEY_UP])
	_action("move_back", [KEY_S, KEY_DOWN])
	_action("move_left", [KEY_A, KEY_LEFT])
	_action("move_right", [KEY_D, KEY_RIGHT])
	_joy_axis("move_forward", JOY_AXIS_LEFT_Y, -1.0)
	_joy_axis("move_back", JOY_AXIS_LEFT_Y, 1.0)
	_joy_axis("move_left", JOY_AXIS_LEFT_X, -1.0)
	_joy_axis("move_right", JOY_AXIS_LEFT_X, 1.0)
	_action("sprint", [KEY_SHIFT])
	_joy_button("sprint", JOY_BUTTON_RIGHT_SHOULDER)
	_joy_button("sprint", JOY_BUTTON_LEFT_STICK)
	_action("interact", [KEY_E, KEY_F])
	_joy_button("interact", JOY_BUTTON_A)
	_action("hold_night", [KEY_H])
	_action("use_bandage", [KEY_Q])
	_action("use_food", [KEY_Z])
	_action("use_water", [KEY_X])
	_action("use_warmer", [KEY_C])
	_action("restart", [KEY_ENTER])
	_action("fire_mission", [KEY_R])
	_action("pause", [KEY_ESCAPE])
	_joy_button("pause", JOY_BUTTON_START)

func _action(name: String, keys: Array) -> void:
	if not InputMap.has_action(name):
		InputMap.add_action(name, 0.22)
	for key in keys:
		var ev := InputEventKey.new()
		ev.physical_keycode = key
		InputMap.action_add_event(name, ev)

func _joy_axis(name: String, axis: JoyAxis, value: float) -> void:
	var ev := InputEventJoypadMotion.new()
	ev.axis = axis
	ev.axis_value = value
	InputMap.action_add_event(name, ev)

func _joy_button(name: String, button: JoyButton) -> void:
	var ev := InputEventJoypadButton.new()
	ev.button_index = button
	InputMap.action_add_event(name, ev)
