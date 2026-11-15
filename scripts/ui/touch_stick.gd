extends Control
class_name TouchStick
## Left-side stick, same contract as DefendTaipei's joystick: x is right,
## y is down, magnitude 1 is a full run. Hidden until a touch lands on it,
## so a mouse player never sees it.

var axis: Vector2 = Vector2.ZERO
var _id: int = -1
var _center: Vector2 = Vector2.ZERO
const RADIUS := 55.0

func _ready() -> void:
	add_to_group("touch_stick")
	mouse_filter = Control.MOUSE_FILTER_STOP
	set_anchors_preset(Control.PRESET_BOTTOM_LEFT)
	offset_left = 28
	offset_top = -168
	offset_right = 168
	offset_bottom = -28
	visible = false

func _gui_input(event: InputEvent) -> void:
	if event is InputEventScreenTouch:
		if event.pressed:
			visible = true
			_id = event.index
			_center = size * 0.5
			_set_from(event.position)
		elif event.index == _id:
			_id = -1
			axis = Vector2.ZERO
			queue_redraw()
	elif event is InputEventScreenDrag and event.index == _id:
		_set_from(event.position)

func _set_from(pos: Vector2) -> void:
	var d := pos - _center
	var length := d.length()
	if length > RADIUS:
		d = d * (RADIUS / length)
	axis = Vector2(d.x / RADIUS, d.y / RADIUS)
	queue_redraw()

func _draw() -> void:
	var c := size * 0.5
	draw_circle(c, RADIUS, Color(1, 1, 1, 0.12))
	draw_arc(c, RADIUS, 0, TAU, 48, Color(1, 1, 1, 0.45), 2.0)
	draw_circle(c + axis * RADIUS, 22, Color(1, 1, 1, 0.55))
