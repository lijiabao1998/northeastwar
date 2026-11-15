extends Node
## The thin layer between what hurts you and what you are. An aim point on the
## ground, a fire request (only while you hold a weapon), and one way for
## anything in the world to wound the player.

signal aim_changed(point: Vector3)
signal fire_requested(point: Vector3)
signal block_marked(building_id: int)

var enabled: bool = false
var aim_point: Vector3 = Vector3.ZERO
var marked_block: int = -1

func set_aim(point: Vector3) -> void:
	aim_point = point
	aim_changed.emit(point)

func request_fire() -> void:
	if not enabled or not GameState.armed or GameState.dead:
		return
	fire_requested.emit(aim_point)

func hurt_player(amount: float, cause_zh: String = "你伤得太重了。", cause_en: String = "Your wounds were too much.", bleed_p: float = 0.8) -> void:
	## One life. A wound takes health, most wounds bleed, and at zero the run ends.
	if GameState.shot_mode or GameState.dead:
		return
	GameState.player_hp = maxf(0.0, GameState.player_hp - amount)
	if GameState.player_hp <= 0.0:
		GameState.kill(cause_zh, cause_en)
		return
	if not GameState.bleeding and randf() < bleed_p:
		GameState.bleeding = true
		GameState.show_subtitle("你中弹了，在流血。按 Q 用绷带，没有就找药店。", "You're hit and bleeding. Q to bandage. No bandage: find a pharmacy.")

func mark_block(building_id: int) -> void:
	marked_block = building_id
	block_marked.emit(building_id)

func reset() -> void:
	enabled = false
	aim_point = Vector3.ZERO
	marked_block = -1
