# TooltipOverlay.gd - Shows tile information on mouse hover
# Displays world coordinates, chunk coordinates, terrain type, and resources

extends Control

# References
var tilemap: TileMap = null
var camera: Camera2D = null

# Tooltip label
var tooltip_label: Label = null

# Visibility state
var tooltip_visible: bool = true

# Offset from cursor
const CURSOR_OFFSET = Vector2(15, 15)

func _ready():
	# Ensure this Control fills the viewport
	set_anchors_preset(Control.PRESET_FULL_RECT)
	mouse_filter = Control.MOUSE_FILTER_IGNORE  # Don't block mouse events

	# Create tooltip label
	tooltip_label = Label.new()
	tooltip_label.add_theme_font_size_override("font_size", 14)
	tooltip_label.visible = false
	tooltip_label.mouse_filter = Control.MOUSE_FILTER_IGNORE

	# Style the tooltip with a dark background
	var style_box = StyleBoxFlat.new()
	style_box.bg_color = Color(0, 0, 0, 0.95)
	style_box.border_color = Color(0.3, 0.3, 0.3, 1.0)
	style_box.border_width_left = 1
	style_box.border_width_right = 1
	style_box.border_width_top = 1
	style_box.border_width_bottom = 1
	style_box.corner_radius_top_left = 4
	style_box.corner_radius_top_right = 4
	style_box.corner_radius_bottom_left = 4
	style_box.corner_radius_bottom_right = 4
	style_box.content_margin_left = 8
	style_box.content_margin_right = 8
	style_box.content_margin_top = 6
	style_box.content_margin_bottom = 6

	# Create a PanelContainer for the styled background
	var panel = PanelContainer.new()
	panel.add_theme_stylebox_override("panel", style_box)
	panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	panel.add_child(tooltip_label)

	add_child(panel)

	print("🖱️ TooltipOverlay initialized")

func set_tilemap(p_tilemap: TileMap):
	tilemap = p_tilemap

func set_camera(p_camera: Camera2D):
	camera = p_camera

func toggle_tooltip():
	tooltip_visible = not tooltip_visible
	if not tooltip_visible:
		tooltip_label.get_parent().visible = false
	print("🖱️ Tooltip ", "enabled" if tooltip_visible else "disabled")

func _input(event):
	if event is InputEventMouseMotion and tooltip_visible:
		_update_tooltip(event.position)
	elif event is InputEventMouseButton:
		# Hide tooltip when clicking to avoid interference
		if event.pressed:
			tooltip_label.get_parent().visible = false

func _update_tooltip(screen_pos: Vector2):
	if tilemap == null or camera == null:
		return

	# Convert screen position to world position
	var world_pos = camera.get_screen_center_position() + (screen_pos - get_viewport_rect().size / 2) / camera.zoom

	# Convert world position to tile coordinates using isometric conversion
	var tile_pos = tilemap.local_to_map(world_pos)

	# Get chunk coordinates
	var chunk_x = floori(float(tile_pos.x) / 16.0)
	var chunk_y = floori(float(tile_pos.y) / 16.0)
	var chunk_key = "%d,%d" % [chunk_x, chunk_y]

	# Get terrain type from cache
	var terrain_type = "Unknown"
	var resource_type = ""

	if WorldDataCache.has_terrain_chunk(chunk_key):
		var chunk_data = WorldDataCache.get_terrain_chunk(chunk_key)
		var local_x = ((tile_pos.x % 16) + 16) % 16
		var local_y = ((tile_pos.y % 16) + 16) % 16

		if local_y < chunk_data.size() and local_x < chunk_data[local_y].size():
			terrain_type = chunk_data[local_y][local_x]

	# Get resource type from cache
	if WorldDataCache.has_resource_chunk(chunk_key):
		var chunk_data = WorldDataCache.get_resource_chunk(chunk_key)
		var local_x = ((tile_pos.x % 16) + 16) % 16
		var local_y = ((tile_pos.y % 16) + 16) % 16

		if local_y < chunk_data.size() and local_x < chunk_data[local_y].size():
			var res = chunk_data[local_y][local_x]
			if res != "":
				resource_type = res

	# Build tooltip text
	var text = "World: (%d, %d)\n" % [tile_pos.x, tile_pos.y]
	text += "Chunk: (%d, %d)\n" % [chunk_x, chunk_y]
	text += "Terrain: %s" % terrain_type

	if resource_type != "":
		var resource_symbol = Config.get_resource_symbol(resource_type)
		text += "\nResource: %s %s" % [resource_type, resource_symbol]

	# Update label
	tooltip_label.text = text

	# Position tooltip near cursor with smart positioning
	var panel = tooltip_label.get_parent()
	panel.visible = true

	# Force update to get correct size
	panel.reset_size()
	await get_tree().process_frame

	var tooltip_size = panel.size
	var viewport_size = get_viewport_rect().size

	var pos = screen_pos + CURSOR_OFFSET

	# Adjust if tooltip would go off screen
	if pos.x + tooltip_size.x > viewport_size.x:
		pos.x = screen_pos.x - tooltip_size.x - CURSOR_OFFSET.x
	if pos.y + tooltip_size.y > viewport_size.y:
		pos.y = screen_pos.y - tooltip_size.y - CURSOR_OFFSET.y

	panel.position = pos
