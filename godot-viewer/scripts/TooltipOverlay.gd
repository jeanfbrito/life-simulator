# TooltipOverlay.gd - Shows tile information on mouse hover
# Displays world coordinates, chunk coordinates, terrain type, and resources

extends CanvasLayer

# References
var tilemap: TileMap = null
var camera: Camera2D = null

# Tooltip container
var tooltip_panel: PanelContainer = null
var tooltip_label: Label = null

# Visibility state
var tooltip_visible: bool = true

# Offset from cursor
const CURSOR_OFFSET = Vector2(15, 15)

func _ready():
	# Set layer to be above everything
	layer = 100

	# Create a Control that fills the screen for input handling
	var control_container = Control.new()
	control_container.set_anchors_preset(Control.PRESET_FULL_RECT)
	control_container.mouse_filter = Control.MOUSE_FILTER_IGNORE
	add_child(control_container)

	# Create tooltip label with white text
	tooltip_label = Label.new()
	tooltip_label.add_theme_font_size_override("font_size", 14)
	tooltip_label.add_theme_color_override("font_color", Color(1, 1, 1, 1))  # White text
	tooltip_label.mouse_filter = Control.MOUSE_FILTER_IGNORE

	# Style the tooltip with a dark background
	var style_box = StyleBoxFlat.new()
	style_box.bg_color = Color(0, 0, 0, 0.95)
	style_box.border_color = Color(0.4, 0.4, 0.4, 1.0)
	style_box.border_width_left = 1
	style_box.border_width_right = 1
	style_box.border_width_top = 1
	style_box.border_width_bottom = 1
	style_box.corner_radius_top_left = 4
	style_box.corner_radius_top_right = 4
	style_box.corner_radius_bottom_left = 4
	style_box.corner_radius_bottom_right = 4
	style_box.content_margin_left = 10
	style_box.content_margin_right = 10
	style_box.content_margin_top = 8
	style_box.content_margin_bottom = 8

	# Create a PanelContainer for the styled background
	tooltip_panel = PanelContainer.new()
	tooltip_panel.add_theme_stylebox_override("panel", style_box)
	tooltip_panel.mouse_filter = Control.MOUSE_FILTER_IGNORE
	tooltip_panel.visible = false  # Start hidden
	tooltip_panel.add_child(tooltip_label)

	control_container.add_child(tooltip_panel)

	print("🖱️ TooltipOverlay initialized")

func set_tilemap(p_tilemap: TileMap):
	tilemap = p_tilemap

func set_camera(p_camera: Camera2D):
	camera = p_camera

func toggle_tooltip():
	tooltip_visible = not tooltip_visible
	if not tooltip_visible:
		tooltip_panel.visible = false
	print("🖱️ Tooltip ", "enabled" if tooltip_visible else "disabled")

func _input(event):
	if event is InputEventMouseMotion and tooltip_visible:
		_update_tooltip(event.position)
	elif event is InputEventMouseButton:
		# Hide tooltip when clicking to avoid interference
		if event.pressed:
			tooltip_panel.visible = false

func _update_tooltip(screen_pos: Vector2):
	if tilemap == null or camera == null:
		return

	# Convert screen position to world position
	var viewport_size = get_viewport().get_visible_rect().size
	var world_pos = camera.get_screen_center_position() + (screen_pos - viewport_size / 2) / camera.zoom

	# Convert world position to tile coordinates using isometric conversion
	var tile_pos = tilemap.local_to_map(world_pos)

	# Get chunk coordinates
	var chunk_x = floori(float(tile_pos.x) / 16.0)
	var chunk_y = floori(float(tile_pos.y) / 16.0)
	var chunk_key = "%d,%d" % [chunk_x, chunk_y]

	# Get terrain type from cache using WorldDataCache's built-in method
	var terrain_type = WorldDataCache.get_terrain_at(tile_pos.x, tile_pos.y)

	# Get resource type from cache using WorldDataCache's built-in method
	var resource_type = WorldDataCache.get_resource_at(tile_pos.x, tile_pos.y)

	# Build tooltip text
	var text = "World: (%d, %d)\n" % [tile_pos.x, tile_pos.y]
	text += "Chunk: (%d, %d)\n" % [chunk_x, chunk_y]
	text += "Terrain: %s" % terrain_type

	if resource_type != "":
		var resource_symbol = Config.get_resource_symbol(resource_type)
		text += "\nResource: %s %s" % [resource_type, resource_symbol]

	# Update label
	tooltip_label.text = text

	# Show tooltip and position it
	tooltip_panel.visible = true

	# Position immediately (CanvasLayer is in screen space)
	var pos = screen_pos + CURSOR_OFFSET
	tooltip_panel.position = pos
