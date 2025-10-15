extends Node2D

## SlopeDiagnostics - Visual test scene for slope corner math validation
## Displays all 19 slope variants in a grid with debug overlays

# References to UI elements
@onready var terrain_container = $TerrainContainer
@onready var debug_overlay = $DebugOverlay
@onready var show_corner_heights_toggle = $UILayer/ControlPanel/ShowCornerHeights
@onready var show_screen_offsets_toggle = $UILayer/ControlPanel/ShowScreenOffsets
@onready var show_corner_markers_toggle = $UILayer/ControlPanel/ShowCornerMarkers
@onready var base_height_slider = $UILayer/ControlPanel/BaseHeightSlider
@onready var base_height_label = $UILayer/ControlPanel/BaseHeightLabel
@onready var regenerate_button = $UILayer/ControlPanel/RegenerateButton

# Test configuration
const GRID_SPACING = Vector2i(4, 4)  # Spacing between test tiles
const NUM_SLOPES = 19  # OpenRCT2 slope indices 0-18
var current_base_height = 0

func _ready():
	# Connect UI signals
	show_corner_heights_toggle.toggled.connect(_on_debug_toggle_changed)
	show_screen_offsets_toggle.toggled.connect(_on_debug_toggle_changed)
	show_corner_markers_toggle.toggled.connect(_on_debug_toggle_changed)
	base_height_slider.value_changed.connect(_on_base_height_changed)
	regenerate_button.pressed.connect(_regenerate_grid)
	
	# Generate initial grid
	_regenerate_grid()

func _regenerate_grid():
	"""Generate a grid of all slope variants at current base height."""
	# Clear existing terrain
	for child in terrain_container.get_children():
		child.queue_free()
	
	# Clear debug overlays
	for child in debug_overlay.get_children():
		child.queue_free()
	
	# Create grid: 5 columns x 4 rows (19 slopes + 1 empty)
	var grid_cols = 5
	var grid_rows = 4
	
	print("=== Slope Diagnostics Grid ===")
	print("Base Height: %d tiny-Z" % current_base_height)
	print("COORDS_Z_STEP: %d, COORDS_Z_PER_TINY_Z: %d" % [Config.COORDS_Z_STEP, Config.COORDS_Z_PER_TINY_Z])
	
	for slope_index in range(NUM_SLOPES):
		var col = slope_index % grid_cols
		var row = slope_index / grid_cols
		var world_pos = Vector2i(col * GRID_SPACING.x, row * GRID_SPACING.y)
		
		_paint_test_tile(world_pos, slope_index)
		_add_debug_markers(world_pos, slope_index)

func _paint_test_tile(world_pos: Vector2i, slope_index: int):
	"""Paint a single test tile using TerrainTileMap logic."""
	# Use a temporary TerrainTileMap to leverage existing rendering
	# Alternatively, duplicate the core logic here for testing
	
	# Get corner heights using static method from SlopeCalculator class
	var slope_calc = load("res://scripts/SlopeCalculator.gd")
	var corner_heights = slope_calc.get_corner_heights(current_base_height, slope_index)
	
	# Calculate screen position (mirroring TerrainTileMap.map_to_local)
	var pixel_x = float(world_pos.x - world_pos.y) * float(Config.COORDS_XY_STEP)
	var pixel_y = float(world_pos.x + world_pos.y) * float(Config.COORDS_XY_STEP / 2)
	var base_pos = Vector2(pixel_x, pixel_y)
	
	# Get north corner offset
	var north_corner_offset = float(corner_heights.top * Config.COORDS_Z_STEP) / float(Config.COORDS_Z_PER_TINY_Z) * Config.render_scale
	
	var final_pos = Vector2(base_pos.x, base_pos.y - north_corner_offset)
	
	# Create sprite
	var sprite = Sprite2D.new()
	sprite.name = "TestTile_Slope%d" % slope_index
	sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST
	sprite.position = final_pos
	sprite.z_index = int(final_pos.y)
	
	# Load test texture (grass slope variant)
	var texture_path = "res://assets/tiles/terrain/openrct2_placeholder/grass_slope_%d.png" % slope_index
	if ResourceLoader.exists(texture_path):
		sprite.texture = load(texture_path)
	else:
		# Fallback: create colored placeholder
		var img = Image.create(64, 32, false, Image.FORMAT_RGB8)
		img.fill(Color(0.3 + slope_index * 0.03, 0.6, 0.3))
		sprite.texture = ImageTexture.create_from_image(img)
	
	terrain_container.add_child(sprite)
	
	# Debug output
	print("  Slope %d at %s → corners(N=%d,E=%d,S=%d,W=%d) → Y_offset=%.1f → screen_pos=(%.1f, %.1f)" %
	      [slope_index, world_pos, corner_heights.top, corner_heights.right, corner_heights.bottom, corner_heights.left,
	       north_corner_offset, final_pos.x, final_pos.y])

func _add_debug_markers(world_pos: Vector2i, slope_index: int):
	"""Add visual debug markers showing corner positions and data."""
	var slope_calc = load("res://scripts/SlopeCalculator.gd")
	var corner_heights = slope_calc.get_corner_heights(current_base_height, slope_index)
	
	# Calculate base screen position
	var pixel_x = float(world_pos.x - world_pos.y) * float(Config.COORDS_XY_STEP)
	var pixel_y = float(world_pos.x + world_pos.y) * float(Config.COORDS_XY_STEP / 2)
	var base_pos = Vector2(pixel_x, pixel_y)
	
	# Corner positions in isometric space (relative to tile center)
	# OpenRCT2 convention: top=north, right=east, bottom=south, left=west
	const CORNER_OFFSETS = {
		"top": Vector2(0, -16),     # North corner
		"right": Vector2(32, 0),    # East corner
		"bottom": Vector2(0, 16),   # South corner
		"left": Vector2(-32, 0)     # West corner
	}
	
	for corner_name in ["top", "right", "bottom", "left"]:
		var corner_height = corner_heights[corner_name]
		var corner_screen_offset = float(corner_height * Config.COORDS_Z_STEP) / float(Config.COORDS_Z_PER_TINY_Z) * Config.render_scale
		var corner_base_pos = base_pos + CORNER_OFFSETS[corner_name]
		var corner_final_pos = Vector2(corner_base_pos.x, corner_base_pos.y - corner_screen_offset)
		
		# Create corner marker (small colored circle)
		if show_corner_markers_toggle.button_pressed:
			var marker = _create_corner_marker(corner_name)
			marker.position = corner_final_pos
			debug_overlay.add_child(marker)
		
		# Add text label showing corner data
		if show_corner_heights_toggle.button_pressed or show_screen_offsets_toggle.button_pressed:
			var label = Label.new()
			label.position = corner_final_pos + Vector2(-20, -10)
			label.add_theme_font_size_override("font_size", 10)
			
			var label_text = ""
			if show_corner_heights_toggle.button_pressed:
				label_text += "h=%d " % corner_height
			if show_screen_offsets_toggle.button_pressed:
				label_text += "y=%.1f" % corner_screen_offset
			
			label.text = label_text
			debug_overlay.add_child(label)
	
	# Add slope index label at center
	var center_label = Label.new()
	center_label.position = base_pos + Vector2(-10, -20)
	center_label.add_theme_font_size_override("font_size", 12)
	center_label.text = "S%d" % slope_index
	center_label.modulate = Color(1, 1, 0, 0.9)
	debug_overlay.add_child(center_label)

func _create_corner_marker(corner_name: String) -> Node2D:
	"""Create a colored marker for a corner."""
	var marker = Node2D.new()
	
	# Draw colored circle
	const COLORS = {
		"top": Color(1, 0, 0),      # Red for north
		"right": Color(0, 1, 0),    # Green for east
		"bottom": Color(0, 0, 1),   # Blue for south
		"left": Color(1, 1, 0)      # Yellow for west
	}
	
	var color = COLORS.get(corner_name, Color.WHITE)
	marker.set_script(preload("res://tests/CornerMarker.gd"))  # Simple draw script
	marker.set_meta("color", color)
	
	return marker

func _on_debug_toggle_changed(_toggled: bool):
	"""Regenerate grid when debug toggles change."""
	_regenerate_grid()

func _on_base_height_changed(value: float):
	"""Update base height and regenerate grid."""
	current_base_height = int(value)
	base_height_label.text = "Base Height: %d" % current_base_height
	_regenerate_grid()

