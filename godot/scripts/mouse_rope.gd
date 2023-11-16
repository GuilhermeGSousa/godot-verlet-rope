extends Node2D

@export var rope : Rope2D

func _ready():
	rope.bind_to_node(self, 0)

func _process(delta):
	global_position = get_global_mouse_position()
