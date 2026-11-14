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
	if GameState.started_play:
		var hp := ""
		if CombatHooks.enabled or GameState.under_fire:
			hp = "  伤势 %.0f" % GameState.player_hp
		_clock.text = "2031年1月17日  黑河\n" + GameState.clock_text() + hp
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

func _build_hud() -> void:
	_clock = _label(22)
	_clock.position = Vector2(28, 22)
	_clock.text = "2031年1月17日  黑河"
	add_child(_clock)
	_objective = _label(16)
	_objective.position = Vector2(28, 84)
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
	_hint.text = "WASD 移动 · 鼠标 环顾 · Shift 跑 · E 交谈    手柄：左摇杆 / 右摇杆 / RB 跑 / A 交谈 / Start 暂停"
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
	_phone_panel.position = Vector2(-360, -460)
	_phone_panel.size = Vector2(330, 420)
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
	_phone.add_child(block)
	if _phone.get_child_count() > 5:
		_phone.get_child(1).queue_free()

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
	return label

func _centered(text: String, size: int) -> Label:
	var label := _label(size)
	label.text = text
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.autowrap_mode = TextServer.AUTOWRAP_WORD_SMART
	return label
