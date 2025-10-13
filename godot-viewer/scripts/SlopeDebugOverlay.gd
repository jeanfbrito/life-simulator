extends CanvasLayer

## Debug overlay showing slope indices and height information
##
## Toggle with F3 key. Shows:
## - Current tile position (world coordinates)
## - Chunk and local coordinates
## - Tile height (0-255)
## - Slope index (0-18) and name
## - Neighbor heights
##
## Reference: /GODOT_SLOPE_RENDERING_IMPLEMENTATION.md

@onready var debug_label: Label = $DebugPanel/VBoxContainer/DebugLabel
@onready var debug_panel: Panel = $DebugPanel

var visible_state = false
var camera: Camera2D = null
var world_cache: Node = null


func _ready():
	# Start hidden
	visible = false

	# Get references
	world_cache = get_node_or_null("/root/WorldDataCache")
	if world_cache == null:
		push_warning("SlopeDebugOverlay: WorldDataCache not found")

	# Create debug panel if not in scene tree
	if not has_node("DebugPanel"):
		_create_debug_ui()


func _create_debug_ui():
	"""Create debug UI elements programmatically if not in scene"""
	var panel = Panel.new()
	panel.name = "DebugPanel"
	panel.position = Vector2(10, 10)
	panel.custom_minimum_size = Vector2(350, 200)
	add_child(panel)

	var vbox = VBoxContainer.new()
	vbox.name = "VBoxContainer"
	panel.add_child(vbox)

	var title = Label.new()
	title.text = "=== SLOPE DEBUG (F3) ==="
	title.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	vbox.add_child(title)

	var label = Label.new()
	label.name = "DebugLabel"
	label.text = "No data"
	label.add_theme_font_size_override("font_size", 12)
	vbox.add_child(label)

	debug_label = label
	debug_panel = panel


func _input(event):
	"""Toggle overlay with F3 key"""
	if event is InputEventKey and event.pressed and event.keycode == KEY_F3:
		visible_state = !visible_state
		visible = visible_state

		if visible_state:
			print("🔍 Slope debug overlay enabled")
		else:
			print("🔍 Slope debug overlay disabled")


func _process(_delta):
	"""Update debug info every frame"""
	if not visible_state or world_cache == null:
		return

	# Get camera reference
	if camera == null:
		camera = get_viewport().get_camera_2d()
		if camera == null:
			return

	# Get tile under camera center
	var camera_pos = camera.global_position
	var tile_pos = _world_pos_to_tile(camera_pos)

	# Calculate chunk and local coordinates
	var chunk_coord = Vector2i(
		int(floor(float(tile_pos.x) / 16.0)),
		int(floor(float(tile_pos.y) / 16.0))
	)
	var local_pos = Vector2i(
		tile_pos.x - (chunk_coord.x * 16),
		tile_pos.y - (chunk_coord.y * 16)
	)

	# Clamp local pos to valid range
	local_pos.x = clampi(local_pos.x, 0, 15)
	local_pos.y = clampi(local_pos.y, 0, 15)

	# Get chunk data
	var chunk_key = "%d,%d" % [chunk_coord.x, chunk_coord.y]
	var chunk_data = world_cache.get_chunk(chunk_key)

	if chunk_data == null:
		debug_label.text = "Chunk not loaded: %s" % chunk_key
		return

	if not chunk_data.has("heights"):
		debug_label.text = "Chunk missing height data: %s" % chunk_key
		return

	# Get height info
	var heights = chunk_data["heights"]
	var current_height = heights[local_pos.y][local_pos.x]

	# Get terrain info
	var terrain = "Unknown"
	if chunk_data.has("terrain"):
		terrain = chunk_data["terrain"][local_pos.y][local_pos.x]

	# Calculate slope
	var slope_idx = SlopeCalculator.calculate_slope_index(
		heights,
		local_pos,
		chunk_coord,
		world_cache
	)
	var slope_name = SlopeCalculator.get_slope_name(slope_idx)

	# Get neighbor heights
	var h_n = _get_safe_height(heights, local_pos, Vector2i(0, -1), chunk_coord)
	var h_e = _get_safe_height(heights, local_pos, Vector2i(1, 0), chunk_coord)
	var h_s = _get_safe_height(heights, local_pos, Vector2i(0, 1), chunk_coord)
	var h_w = _get_safe_height(heights, local_pos, Vector2i(-1, 0), chunk_coord)

	# Build debug text
	var debug_text = """
Tile World: (%d, %d)
Chunk: (%d, %d)
Local: (%d, %d)

Terrain: %s
Height: %d / 255

Slope: %d - %s

Neighbor Heights:
  N: %d  (%+d)
  E: %d  (%+d)
  S: %d  (%+d)
  W: %d  (%+d)
""" % [
		tile_pos.x, tile_pos.y,
		chunk_coord.x, chunk_coord.y,
		local_pos.x, local_pos.y,
		terrain,
		current_height,
		slope_idx, slope_name,
		h_n, h_n - current_height,
		h_e, h_e - current_height,
		h_s, h_s - current_height,
		h_w, h_w - current_height,
	]

	debug_label.text = debug_text.strip_edges()


func _world_pos_to_tile(world_pos: Vector2) -> Vector2i:
	"""Convert world pixel position to tile coordinates"""
	# Isometric conversion: (128×64 tiles)
	# This is a simplified conversion - adjust based on your actual tile size
	return Vector2i(
		int(floor(world_pos.x / 128.0)),
		int(floor(world_pos.y / 64.0))
	)


func _get_safe_height(heights: Array, local_pos: Vector2i, offset: Vector2i, chunk_coord: Vector2i) -> int:
	"""Get neighbor height safely using SlopeCalculator"""
	return SlopeCalculator.get_neighbor_height(
		heights,
		local_pos,
		offset,
		chunk_coord,
		world_cache
	)
