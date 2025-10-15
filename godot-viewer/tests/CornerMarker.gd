extends Node2D

## Simple corner marker for slope diagnostics
## Draws a colored circle at the corner position

const MARKER_RADIUS = 4.0

func _draw():
	var color = get_meta("color", Color.WHITE)
	draw_circle(Vector2.ZERO, MARKER_RADIUS, color)
	draw_arc(Vector2.ZERO, MARKER_RADIUS, 0, TAU, 16, Color.BLACK, 1.0)

