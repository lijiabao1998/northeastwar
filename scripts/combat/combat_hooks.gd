extends Node
## Combat is intentionally off in this slice. The hook is the same shape
## DefendTaipei uses: an aim point on the ground, a fire request, and a block
## to retake. A follow-up can turn `enabled` on and hang drones, artillery,
## armour, civilians and scavenging off these signals. Nothing here fires.

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
	if not enabled:
		return
	fire_requested.emit(aim_point)

func hurt_player(amount: float) -> void:
	if GameState.shot_mode:
		return
	GameState.under_fire = true
	GameState.player_hp = maxf(0.0, GameState.player_hp - amount)
	if GameState.player_hp <= 0.0:
		GameState.player_hp = 35.0
		GameState.bleeding = false
		GameState.show_subtitle("同伴把你拖进了门洞。", "Someone dragged you into a doorway.")
		return
	if GameState.player_hp < 45.0:
		GameState.bleeding = true
		GameState.show_subtitle("你挂彩了。靠到门脸里停血。", "You're hit. Get into a doorway and stop the bleeding.")

func mark_block(building_id: int) -> void:
	marked_block = building_id
	block_marked.emit(building_id)
