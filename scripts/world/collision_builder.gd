extends Node3D
class_name CollisionBuilder
## Static collision from the same footprints the mesh was built with.
## Only the Chinese bank is solid. The far skyline is a view, not a place you can reach.

func build(data: Dictionary) -> void:
	var ground := StaticBody3D.new()
	ground.collision_layer = 1
	ground.collision_mask = 0
	var plane := CollisionShape3D.new()
	var boundary := WorldBoundaryShape3D.new()
	boundary.plane = Plane(Vector3.UP, 0.0)
	plane.shape = boundary
	ground.add_child(plane)
	add_child(ground)

	for building in data.get("buildings", []):
		if not building.get("collision", false):
			continue
		var footprint: Array = building.get("footprint", [])
		if footprint.size() < 3:
			continue
		var height := float(building.get("height", 9.0))
		var faces := PackedVector3Array()
		var count := footprint.size()
		for i in count:
			var a: Array = footprint[i]
			var b: Array = footprint[(i + 1) % count]
			var a0 := Vector3(float(a[0]), 0.0, float(a[1]))
			var b0 := Vector3(float(b[0]), 0.0, float(b[1]))
			var a1 := Vector3(a0.x, height, a0.z)
			var b1 := Vector3(b0.x, height, b0.z)
			faces.append_array([a0, a1, b1, a0, b1, b0])
		# A rough roof so you cannot fall in from above.
		var c: Array = footprint[0]
		var c0 := Vector3(float(c[0]), height, float(c[1]))
		for i in range(1, count - 1):
			var p: Array = footprint[i]
			var q: Array = footprint[i + 1]
			faces.append_array([
				c0,
				Vector3(float(p[0]), height, float(p[1])),
				Vector3(float(q[0]), height, float(q[1])),
			])
		var body := StaticBody3D.new()
		body.collision_layer = 1
		body.collision_mask = 0
		var shape_node := CollisionShape3D.new()
		var shape := ConcavePolygonShape3D.new()
		shape.set_faces(faces)
		shape_node.shape = shape
		body.add_child(shape_node)
		add_child(body)

	for car in data.get("cars", []):
		var body := StaticBody3D.new()
		body.collision_layer = 1
		body.collision_mask = 0
		body.position = Vector3(float(car["x"]), 0.7, float(car["z"]))
		body.rotation.y = float(car["yaw"])
		var shape_node := CollisionShape3D.new()
		var box := BoxShape3D.new()
		box.size = Vector3(1.8, 1.3, 4.4)
		shape_node.shape = box
		body.add_child(shape_node)
		add_child(body)
