extends Node3D
## Wires the slice together. Each system lives in its own folder:
## world, environment, player, survival, story, npc, ui, audio.

var player: Player
var city: CityData

func _ready() -> void:
	city = CityData.new()
	city.name = "CityData"
	add_child(city)

	var atmosphere := Atmosphere.new()
	atmosphere.name = "Atmosphere"
	add_child(atmosphere)

	var view := CityView.new()
	view.name = "City"
	add_child(view)

	var collision := CollisionBuilder.new()
	collision.name = "Collision"
	add_child(collision)
	collision.build(city.data)

	var props := CityProps.new()
	props.name = "Props"
	add_child(props)
	props.build(city.data)

	atmosphere.bind(view, props)

	var weather := Weather.new()
	weather.name = "Weather"
	add_child(weather)

	player = Player.new()
	player.name = "Player"
	player.city = city
	var spawn: Dictionary = city.spawn()
	player.position = Vector3(float(spawn["x"]), 0.2, float(spawn["z"]))
	player.yaw = float(spawn.get("yaw", PI))
	add_child(player)

	var cold := ColdMeter.new()
	cold.name = "Cold"
	cold.player = player
	cold.city = city
	player.add_child(cold)

	var people := Pedestrians.new()
	people.name = "People"
	add_child(people)
	people.build(city.data)

	var story := DayOne.new()
	story.name = "Story"
	story.city = city
	story.player = player
	story.cold = cold
	add_child(story)

	var ambience := Ambience.new()
	ambience.name = "Ambience"
	add_child(ambience)
	ambience.attach_player(player)

	var hud := Hud.new()
	hud.name = "Hud"
	add_child(hud)

	if GameState.shot_mode:
		var shots := ShotDirector.new()
		shots.name = "Shots"
		shots.city = city
		shots.player = player
		add_child(shots)

func _process(delta: float) -> void:
	GameState.tick(delta)

func _unhandled_input(event: InputEvent) -> void:
	if GameState.shot_mode:
		return
	if not GameState.started_play:
		var pressed := false
		if event is InputEventMouseButton and event.pressed:
			pressed = true
		elif event is InputEventKey and event.pressed and not event.echo:
			pressed = true
		elif event is InputEventJoypadButton and event.pressed:
			pressed = true
		if pressed:
			GameState.begin()
			Input.mouse_mode = Input.MOUSE_MODE_CAPTURED
		return
	if event.is_action_pressed("pause"):
		GameState.set_paused(not GameState.paused)
		get_viewport().set_input_as_handled()
