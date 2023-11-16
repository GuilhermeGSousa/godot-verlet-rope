@tool
extends Line2D

@export var spawn_rotation = 0.0
@export var point_count = 100
@export var segment_length = 3.0

func _process(delta):
	
	if not Engine.is_editor_hint():
		return
		
	if len(points) < point_count:
		for i in range(point_count - len(points)):
			add_point(Vector2.ZERO)
	
	var dir = Vector2.from_angle(deg_to_rad(spawn_rotation))
	for i in range(len(points)):
		points[i] = i * dir * segment_length
