extends Node
class_name Ambience

var _wind: AudioStreamPlayer
var _step: AudioStreamPlayer
var _phone: AudioStreamPlayer
var _boom: AudioStreamPlayer
var _boom_wait: float = 6.0

func _ready() -> void:
	_wind = _player("res://assets/audio/wind.wav", -18.0, true)
	_step = _player("res://assets/audio/crunch.wav", -6.0, false)
	_phone = _player("res://assets/audio/phone.wav", -8.0, false)
	_boom = _player("res://assets/audio/boom.wav", -10.0, false)
	add_child(_wind)
	add_child(_step)
	add_child(_phone)
	add_child(_boom)
	_wind.play()
	GameState.phone_message.connect(func(_t, _a, _b): _phone.play())
	GameState.boom.connect(func(): _boom.play())

func attach_player(player: Player) -> void:
	player.stepped.connect(func():
		if _step.playing:
			return
		_step.pitch_scale = randf_range(0.86, 1.12)
		_step.play()
	)

func _process(delta: float) -> void:
	var target := -16.0 if GameState.on_ice else -22.0
	_wind.volume_db = lerpf(_wind.volume_db, target, 0.02)
	if GameState.alert_stage < 1 or GameState.paused:
		return
	_boom_wait -= delta
	if _boom_wait <= 0.0:
		_boom_wait = randf_range(3.5, 8.5) if GameState.alert_stage == 1 else randf_range(2.2, 5.5)
		_boom.pitch_scale = randf_range(0.75, 1.05)
		GameState.boom.emit()

func _player(path: String, db: float, loop: bool) -> AudioStreamPlayer:
	var player := AudioStreamPlayer.new()
	var stream: AudioStream = load(path)
	if stream is AudioStreamWAV and loop:
		(stream as AudioStreamWAV).loop_mode = AudioStreamWAV.LOOP_FORWARD
	player.stream = stream
	player.volume_db = db
	return player
