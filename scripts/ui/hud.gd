extends CanvasLayer
class_name Hud

var _font: Font
var _title: Control
var _pause: Control
var _subtitle_zh: Label
var _subtitle_en: Label
var _objective: Label
var _clock: Label
var _temp: Label
var _temp_bar: ColorRect
var _phone: VBoxContainer
var _phone_panel: PanelContainer
var _vignette: ColorRect
var _hint: Label
var _story_line := ""
var _subtitle_until := 0.0
var _status: Label
var _notice: Label
var _prompt: Label
var _death: ColorRect
var _death_text: Label
var _phone_until := 0.0

func _ready() -> void:
	_font = load("res://assets/fonts/NotoSansSC-VF.ttf")
	var latin: Font = load("res://assets/fonts/NotoSans-Regular.ttf")
	if _font is FontFile and latin is FontFile:
		(_font as FontFile).fallbacks = Array([latin])
	layer = 10
	_build_vignette()
	_build_hud()
	_build_title()
	_build_pause()
	_build_phone()
	_build_survival()
	GameState.died.connect(_on_died)
	GameState.night_survived.connect(_on_dawn)
	var stick: Control = load("res://scripts/ui/touch_stick.gd").new()
	stick.name = "TouchStick"
	add_child(stick)
	GameState.subtitle_changed.connect(_on_subtitle)
	GameState.objective_changed.connect(func(zh, en): _objective.text = zh + "\n" + en)
	GameState.phone_message.connect(_on_phone)
	GameState.temperature_changed.connect(_on_temp)
	GameState.paused_changed.connect(func(p): _pause.visible = p)
	GameState.started.connect(func(): _title.visible = false)
	if GameState.shot_mode or GameState.started_play:
		_title.visible = false

func _process(_delta: float) -> void:
	var hold := GameState.defend_night
	_clock.visible = not hold
	_temp.visible = not hold
	_objective.visible = not hold
	_status.visible = GameState.started_play and not hold and not GameState.dead
	_notice.visible = _status.visible
	_prompt.text = "" if hold else GameState.prompt
	if GameState.started_play and not hold:
		_clock.text = GameState.date_text() + "  黑河\n" + GameState.clock_text() + "    北 ↙   南 ↗"
		_status.text = _status_line()
		_notice.text = _notice_line()
	if _phone_until > 0.0 and Time.get_ticks_msec() / 1000.0 > _phone_until:
		_phone_until = 0.0
		_phone_panel.visible = false
	if _subtitle_until > 0.0 and Time.get_ticks_msec() / 1000.0 > _subtitle_until and _story_line == "":
		_subtitle_zh.text = ""
		_subtitle_en.text = ""
	var cold := clampf((35.6 - GameState.body_temp) / 2.0, 0.0, 1.0)
	if _vignette.material is ShaderMaterial:
		(_vignette.material as ShaderMaterial).set_shader_parameter("amount", cold)
	_hint.visible = GameState.started_play and not GameState.paused and GameState.hour < 16.25

func _build_vignette() -> void:
	_vignette = ColorRect.new()
	_vignette.set_anchors_preset(Control.PRESET_FULL_RECT)
	_vignette.mouse_filter = Control.MOUSE_FILTER_IGNORE
	var mat := ShaderMaterial.new()
	mat.shader = load("res://shaders/vignette.gdshader")
	_vignette.material = mat
	add_child(_vignette)

func _status_line() -> String:
	var line := "伤势 %.0f" % GameState.player_hp
	if GameState.bleeding:
		line += "  流血"
	line += "   饱 %.0f   水 %.0f" % [GameState.hunger, GameState.thirst]
	if GameState.coat_bonus > 0.0:
		line += "   厚棉衣"
	line += "\nQ 绷带 %d   Z 吃的 %d   X 水 %d   C 暖宝宝 %d" % [
		GameState.count("bandage"), GameState.count("food"), GameState.count("water"), GameState.count("warmer")]
	return line

func _notice_line() -> String:
	if GameState.under_fire:
		return "他们在朝你开枪"
	if GameState.challenged:
		return "被喝令：站住，或者跑"
	if GameState.drone_sees:
		return "无人机盯着你  进门脸"
	if GameState.notice < 0.04:
		return "头顶有无人机" if GameState.drone_overhead else ""
	var bars := int(ceil(GameState.notice * 5.0))
	return "有人在往这边看  " + "▮".repeat(bars) + "▯".repeat(5 - bars)

func _on_died(cause_zh: String, cause_en: String) -> void:
	var hours := int(GameState.hours_alive)
	var minutes := int((GameState.hours_alive - float(hours)) * 60.0)
	_death_text.text = "%s\n%s\n\n%s %s  ·  活了 %d 小时 %d 分\n\n按 Enter 重新开始" % [
		cause_zh, cause_en, GameState.date_text(), GameState.clock_text(), hours, minutes]
	_death.visible = true

func _on_dawn() -> void:
	_death_text.text = "天亮了。你还活着。\nDawn. You are still alive.\n\n1月18日 07:00  ·  第一夜\n\n按 Enter 再来一次"
	_death.visible = true
	GameState.dead = true

func _build_survival() -> void:
	_status = _label(16)
	_status.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	_status.position = Vector2(16, -92)
	_status.size = Vector2(560, 56)
	_status.modulate = Color(0.92, 0.94, 0.9)
	add_child(_status)
	_notice = _label(20)
	_notice.set_anchors_preset(Control.PRESET_CENTER_TOP)
	_notice.position = Vector2(-300, 24)
	_notice.size = Vector2(600, 30)
	_notice.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_notice.modulate = Color(1.0, 0.82, 0.5)
	add_child(_notice)
	_prompt = _label(20)
	_prompt.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	_prompt.position = Vector2(-400, -200)
	_prompt.size = Vector2(800, 30)
	_prompt.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_prompt.modulate = Color(1.0, 0.92, 0.6)
	add_child(_prompt)
	_death = ColorRect.new()
	_death.visible = false
	_death.set_anchors_preset(Control.PRESET_FULL_RECT)
	_death.color = Color(0.02, 0.02, 0.03, 0.78)
	_death.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_death_text = _centered("", 24)
	_death_text.set_anchors_preset(Control.PRESET_CENTER)
	_death_text.position = Vector2(-420, -150)
	_death_text.size = Vector2(840, 300)
	_death_text.mouse_filter = Control.MOUSE_FILTER_IGNORE
	_death.add_child(_death_text)
	add_child(_death)

func _build_hud() -> void:
	_clock = _label(22)
	_clock.position = Vector2(28, 22)
	_clock.text = "2031年1月17日  黑河"
	add_child(_clock)
	_objective = _label(16)
	_objective.position = Vector2(28, 92)
	_objective.size = Vector2(620, 90)
	_objective.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	_objective.modulate = Color(0.85, 0.9, 0.95)
	add_child(_objective)
	_temp = _label(20)
	_temp.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_temp.position = Vector2(-280, 22)
	_temp.size = Vector2(250, 70)
	_temp.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_temp.text = "体温 36.8°C\n室外 -24°C"
	add_child(_temp)
	_temp_bar = ColorRect.new()
	_temp_bar.set_anchors_preset(Control.PRESET_TOP_RIGHT)
	_temp_bar.position = Vector2(-250, 96)
	_temp_bar.size = Vector2(220, 8)
	_temp_bar.color = Color(0.85, 0.4, 0.25)
	add_child(_temp_bar)
	_subtitle_zh = _label(26)
	_subtitle_zh.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	_subtitle_zh.position = Vector2(-460, -150)
	_subtitle_zh.size = Vector2(920, 40)
	_subtitle_zh.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	add_child(_subtitle_zh)
	_subtitle_en = _label(16)
	_subtitle_en.set_anchors_preset(Control.PRESET_CENTER_BOTTOM)
	_subtitle_en.position = Vector2(-460, -112)
	_subtitle_en.size = Vector2(920, 28)
	_subtitle_en.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	_subtitle_en.modulate = Color(0.8, 0.84, 0.9)
	add_child(_subtitle_en)
	var credit := _label(13)
	credit.text = "地图数据 © OpenStreetMap 贡献者（ODbL）"
	credit.set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	credit.position = Vector2(16, -28)
	credit.modulate = Color(1, 1, 1, 0.7)
	add_child(credit)
	_hint = _label(14)
	_hint.text = "WASD 移动 · 滚轮 缩放 · Shift 跑（会被听见） · E 交谈 / 按住 搜    手柄：左摇杆 / 右摇杆 / RB 跑 / A 交谈 / Start 暂停"
	_hint.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_hint.position = Vector2(-760, -28)
	_hint.size = Vector2(740, 22)
	_hint.horizontal_alignment = HORIZONTAL_ALIGNMENT_RIGHT
	_hint.modulate = Color(1, 1, 1, 0.75)
	add_child(_hint)

func _build_title() -> void:
	_title = ColorRect.new()
	_title.set_anchors_preset(Control.PRESET_FULL_RECT)
	_title.color = Color(0.03, 0.04, 0.06, 0.72)
	var box := VBoxContainer.new()
	box.set_anchors_preset(Control.PRESET_CENTER)
	box.position = Vector2(-320, -180)
	box.size = Vector2(640, 360)
	box.add_theme_constant_override("separation", 12)
	box.add_child(_centered("东北：俄罗斯入侵", 36))
	box.add_child(_centered("黑河 · 2031年1月17日 · 傍晚", 20))
	box.add_child(_centered("你不是军人。你刚从中央街的店里下班。", 18))
	box.add_child(_centered("Northeast: The Russian Invasion", 16))
	box.add_child(_centered("A work of fiction. The places are real. The people and the war are not.", 14))
	box.add_child(_centered("点击鼠标，或按任意键 / 手柄 A", 18))
	_title.add_child(box)
	add_child(_title)

func _build_pause() -> void:
	_pause = ColorRect.new()
	_pause.visible = false
	_pause.set_anchors_preset(Control.PRESET_FULL_RECT)
	_pause.color = Color(0.02, 0.03, 0.05, 0.66)
	var box := VBoxContainer.new()
	box.set_anchors_preset(Control.PRESET_CENTER)
	box.position = Vector2(-220, -160)
	box.size = Vector2(440, 340)
	box.add_theme_constant_override("separation", 8)
	box.add_child(_centered("暂停", 28))
	box.add_child(_centered("画质  Quality", 16))
	var row := HBoxContainer.new()
	row.alignment = BoxContainer.ALIGNMENT_CENTER
	for level in ["low", "medium", "high"]:
		var button := Button.new()
		button.text = level
		button.custom_minimum_size = Vector2(100, 36)
		button.pressed.connect(func(): GameState.set_quality(level))
		row.add_child(button)
	box.add_child(row)
	box.add_child(_centered("低：少灯光，可破坏建筑更少。\n高：体积雾，更多楼层会被炸开。", 14))
	box.add_child(_centered("地图 © OpenStreetMap 贡献者，ODbL 1.0\n字体：Noto Sans / Noto Sans SC，OFL\n© 2026 李家宝", 13))
	var resume := Button.new()
	resume.text = "继续  Resume"
	resume.pressed.connect(func(): GameState.set_paused(false))
	box.add_child(resume)
	_pause.add_child(box)
	add_child(_pause)

func _build_phone() -> void:
	_phone_panel = PanelContainer.new()
	_phone_panel.visible = false
	_phone_panel.set_anchors_preset(Control.PRESET_BOTTOM_RIGHT)
	_phone_panel.position = Vector2(-360, -330)
	_phone_panel.size = Vector2(330, 0)
	var margin := MarginContainer.new()
	margin.add_theme_constant_override("margin_left", 12)
	margin.add_theme_constant_override("margin_right", 12)
	margin.add_theme_constant_override("margin_top", 10)
	margin.add_theme_constant_override("margin_bottom", 10)
	_phone = VBoxContainer.new()
	_phone.add_theme_constant_override("separation", 8)
	var header := _label(16)
	header.text = "短信"
	_phone.add_child(header)
	margin.add_child(_phone)
	_phone_panel.add_child(margin)
	add_child(_phone_panel)

func _on_subtitle(zh: String, en: String) -> void:
	_story_line = zh
	_subtitle_zh.text = zh
	_subtitle_en.text = en
	_subtitle_until = Time.get_ticks_msec() / 1000.0 + 7.0
	# Let the line fade unless a newer one replaces it.
	_story_line = ""

func _on_phone(title: String, zh: String, en: String) -> void:
	_phone_panel.visible = true
	var block := Label.new()
	block.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	block.add_theme_font_override("font", _font)
	block.add_theme_font_size_override("font_size", 15)
	block.text = title + "\n" + zh + "\n" + en
	block.custom_minimum_size = Vector2(300, 0)
	# One message at a time, and it goes away by itself.
	while _phone.get_child_count() > 1:
		var old := _phone.get_child(1)
		_phone.remove_child(old)
		old.queue_free()
	_phone.add_child(block)
	_phone_until = Time.get_ticks_msec() / 1000.0 + 14.0

func _on_temp(celsius: float) -> void:
	_temp.text = "体温 %.1f°C\n室外 -24°C" % celsius
	var u := clampf((celsius - 33.6) / (37.2 - 33.6), 0.05, 1.0)
	_temp_bar.size.x = 220.0 * u
	_temp_bar.position.x = -30.0 - _temp_bar.size.x
	_temp_bar.color = Color(0.35, 0.55, 0.9).lerp(Color(0.9, 0.38, 0.22), u)
	if celsius < 35.0:
		_temp.modulate = Color(0.7, 0.85, 1.0)
	else:
		_temp.modulate = Color(1, 1, 1)

func _label(size: int) -> Label:
	var label := Label.new()
	label.add_theme_font_override("font", _font)
	label.add_theme_font_size_override("font_size", size)
	# A dark edge, so white text still reads over snow.
	label.add_theme_constant_override("outline_size", 5)
	label.add_theme_color_override("font_outline_color", Color(0.02, 0.03, 0.05, 0.7))
	return label

func _centered(text: String, size: int) -> Label:
	var label := _label(size)
	label.text = text
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	return label
