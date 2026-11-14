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

func _ready() -> void:
	_ensure_actions()
	var cfg := ConfigFile.new()
	if cfg.load("user://settings.cfg") == OK:
		quality = str(cfg.get_value("video", "quality", "medium"))
		if quality not in LEVELS:
			quality = "medium"
	var args := OS.get_cmdline_user_args()
	shot_mode = "--shots" in args or "--combat-shots" in args
	if shot_mode:
		started_play = true
		quality = "medium"

func tick(delta: float) -> void:
	if not started_play or paused or shot_mode:
		return
	hour += delta * TIME_SCALE / 3600.0
	if hour > 22.0:
		hour = 22.0

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
	return "%02d:%02d" % [h, m]

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
