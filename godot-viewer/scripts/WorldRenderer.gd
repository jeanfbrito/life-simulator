# WorldRenderer.gd - Main world rendering system
# Connects backend chunk data to TileMap rendering with camera-based streaming

extends Node2D

@onready var terrain_tilemap: Node2D = $TerrainTileMap
@onready var camera: Camera2D = $TerrainTileMap/Camera2D
@onready var resource_manager: Node2D = $TerrainTileMap/ResourceManager
@onready var entity_manager: Node2D = $TerrainTileMap/EntityManager

# Grid overlay (will be created dynamically)
var grid_overlay: Node2D = null

# Tooltip overlay (will be created dynamically)
var tooltip_overlay: CanvasLayer = null

# Height marker overlay (will be created dynamically)
var height_marker_overlay: CanvasLayer = null

# UI References
var top_bar: CanvasLayer = null
var statistics_hud: Control = null
var controls_overlay: Control = null

# World state
var world_loaded: bool = false
var current_chunk_keys: Array[String] = []
var chunk_load_radius: int = 5  # Load chunks within this radius
var loading_chunks: bool = false

func _ready():
	print("🌍 WorldRenderer initialized")

	# Connect to chunk manager signals
	ChunkManager.chunks_loaded.connect(_on_chunks_loaded)
	ChunkManager.connection_status_changed.connect(_on_connection_status_changed)
	print("📡 Connected to ChunkManager signals")

	# Test ResourceManager and EntityManager
	print("🧪 Testing visualization components...")
	print("  ResourceManager available: ", resource_manager != null)
	print("  EntityManager available: ", entity_manager != null)

	# Create and initialize grid overlay
	_initialize_grid_overlay()

	# Create and initialize tooltip overlay
	_initialize_tooltip_overlay()

	# Create and initialize height marker overlay
	_initialize_height_marker_overlay()

	# Initialize UI references (wait one frame for UI nodes to be ready)
	await get_tree().process_frame
	_initialize_ui_references()

	# Initialize camera position - center on island area (tile 0,0)
	# Convert tile (0,0) to pixel coordinates in isometric space
	var center_tile = Vector2i(0, 0)
	var center_pixel = terrain_tilemap.map_to_local(center_tile)
	camera.position = center_pixel
	camera.zoom = Vector2(0.2, 0.2)  # 0.2x zoom - zoomed out to see full terrain elevation
	print("📹 Camera positioned at tile ", center_tile, " = pixel ", center_pixel, " with zoom 0.2x (zoomed out view)")

	# Print camera and tilemap info
	print("📹 Camera actual position: ", camera.position, " zoom: ", camera.zoom)
	# TerrainTileMap is now Node2D-based (not TileMap), no tile_set property

	# Start world loading immediately for testing
	print("🚀 Starting world loading immediately (timer bypassed)")
	start_world_loading()

# Start loading the world
func start_world_loading():
	print("🚀 Starting world loading...")

	# Load world info first
	_load_world_info()

# Load world information from backend
func _load_world_info():
	print("📊 Loading world info...")

	# Connect to world info signal if not already connected
	if not ChunkManager.world_info_loaded.is_connected(_on_world_info_loaded):
		ChunkManager.world_info_loaded.connect(_on_world_info_loaded)

	var success = await ChunkManager.load_world_info()
	if success:
		print("✅ World info loading completed successfully")
		# Wait for world info to be processed before loading chunks
		await get_tree().create_timer(0.5).timeout
		await _load_chunks_around_position(Vector2i(0, 0))
		world_loaded = true
		print("✅ World loading completed - viewer should show terrain")
	else:
		print("❌ Failed to load world info")

# Handle world info data
func _on_world_info_loaded(world_info: Dictionary):
	print("🗺️ World info received: ", world_info.get("name", "Unknown"))
	
	# If this is a file-based map, we might want to update the UI or camera
	if world_info.get("source") == "file":
		print("📂 Loaded from file: ", world_info.get("file_path", "Unknown"))
		# You could add specific handling for file-based maps here
		# For example, adjusting camera position based on map size

# Load chunks around a specific position
func _load_chunks_around_position(center_chunk: Vector2i):
	if loading_chunks:
		return

	loading_chunks = true
	print("📍 Loading chunks around: ", center_chunk, " (radius: ", chunk_load_radius, ")")

	# Calculate which chunks to load (respect world bounds)
	var chunks_to_load: Array[String] = []
	for x in range(center_chunk.x - chunk_load_radius, center_chunk.x + chunk_load_radius + 1):
		for y in range(center_chunk.y - chunk_load_radius, center_chunk.y + chunk_load_radius + 1):
			# Only load chunks within world bounds (-3,-3 to 3,3)
			if x >= -3 and x <= 3 and y >= -3 and y <= 3:
				var chunk_key = "%d,%d" % [x, y]
				if not current_chunk_keys.has(chunk_key):
					chunks_to_load.append(chunk_key)

	print("📦 Chunks to load: ", chunks_to_load.size(), " (", chunks_to_load, ")")

	if chunks_to_load.size() > 0:
		# Load chunks in batches
		_load_chunk_batch(chunks_to_load)
	else:
		loading_chunks = false

# Load chunks in batches to avoid overwhelming the backend
func _load_chunk_batch(chunk_keys: Array[String]):
	const BATCH_SIZE = 10

	var batch = chunk_keys.slice(0, BATCH_SIZE)
	var remaining = chunk_keys.slice(BATCH_SIZE)

	print("🔄 Loading batch of ", batch.size(), " chunks...")

	# Start the batch loading
	var chunk_data = await ChunkManager.load_chunk_batch(batch)
	if chunk_data != null:
		_on_chunks_loaded(chunk_data)

	# Load remaining chunks if any
	if remaining.size() > 0:
		await _load_chunk_batch(remaining)
	else:
		loading_chunks = false
		print("✅ All chunks loaded")

# Handle loaded chunk data
func _on_chunks_loaded(chunk_data: Dictionary):
	print("🎨 Received chunk data: ", chunk_data.chunks.size(), " chunks")

	# Merge chunk data into cache
	WorldDataCache.merge_chunk_data(chunk_data)

	# Update visible chunks on tilemap
	_update_visible_chunks()

# Update visible chunks based on current camera position
func _update_visible_chunks():
	print("🗺️ Updating visible chunks...")

	# Get chunks currently visible to camera
	var visible_chunks = _get_visible_chunks()

	# Remove chunks that are no longer visible
	_remove_invisible_chunks(visible_chunks)

	# Add newly visible chunks (returns list of actually painted chunks)
	var newly_painted = _add_visible_chunks(visible_chunks)

	# Only add chunks that were actually painted to current_chunk_keys
	for chunk_key in newly_painted:
		if not current_chunk_keys.has(chunk_key):
			current_chunk_keys.append(chunk_key)

	print("📊 Total rendered chunks: ", current_chunk_keys.size(), " / ", visible_chunks.size(), " visible")

	# Take debug screenshot when all chunks loaded
	if current_chunk_keys.size() >= 49 and not has_meta("screenshot_taken"):
		set_meta("screenshot_taken", true)
		call_deferred("_take_debug_screenshot")

	# Debug: Print TerrainTileMap state (Sprite2D-based, not TileMap)
	print("📊 TerrainTileMap stats:")
	print("   - Visible: ", terrain_tilemap.visible)
	print("   - Modulate: ", terrain_tilemap.modulate)
	# TerrainTileMap uses Sprite2D-based rendering, not cell-based TileMap

# Get chunks currently visible to the camera
func _get_visible_chunks() -> Array[String]:
	var center_chunk = _world_to_chunk(camera.position)
	var visible_chunks: Array[String] = []

	# Conservative estimate of visible area
	var view_radius = chunk_load_radius + 1

	for x in range(center_chunk.x - view_radius, center_chunk.x + view_radius + 1):
		for y in range(center_chunk.y - view_radius, center_chunk.y + view_radius + 1):
			# Only include chunks within world bounds (-3,-3 to 3,3)
			if x >= -3 and x <= 3 and y >= -3 and y <= 3:
				var chunk_key = "%d,%d" % [x, y]
				visible_chunks.append(chunk_key)

	return visible_chunks

# Remove chunks that are no longer visible
func _remove_invisible_chunks(visible_chunks: Array[String]):
	for chunk_key in current_chunk_keys:
		if not visible_chunks.has(chunk_key):
			terrain_tilemap.clear_chunk(chunk_key)
			resource_manager.clear_resources(chunk_key)
			print("🗑️ Cleared chunk: ", chunk_key)

# Add newly visible chunks - returns array of actually painted chunk keys
func _add_visible_chunks(visible_chunks: Array[String]) -> Array[String]:
	var painted_chunks: Array[String] = []
	for chunk_key in visible_chunks:
		# Always try to paint if chunk data exists in cache
		var terrain_data = WorldDataCache.get_terrain_chunk(chunk_key)
		if terrain_data.size() > 0 and not current_chunk_keys.has(chunk_key):
			# Get height data for slope rendering
			var height_data = WorldDataCache.get_height_chunk(chunk_key)
			var slope_data = WorldDataCache.get_slope_chunk(chunk_key)
			terrain_tilemap.paint_chunk(chunk_key, terrain_data, height_data, slope_data)

			# Paint resources too!
			var resource_data = WorldDataCache.get_resource_chunk(chunk_key)
			if resource_data.size() > 0:
				resource_manager.paint_resources(chunk_key, resource_data)

			painted_chunks.append(chunk_key)

	if painted_chunks.size() > 0:
		print("🎨 Painted ", painted_chunks.size(), " new chunks (total visible: ", visible_chunks.size(), ")")

	return painted_chunks

# Convert world position to chunk coordinates
func _world_to_chunk(world_pos: Vector2) -> Vector2i:
	# Rough approximation - will need refinement for isometric coordinates
	var tile_size = Config.TILE_SIZE
	var chunk_size = Config.CHUNK_SIZE

	var chunk_x = int(world_pos.x / (tile_size * chunk_size))
	var chunk_y = int(world_pos.y / (tile_size * chunk_size))

	return Vector2i(chunk_x, chunk_y)

# Handle connection status changes
func _on_connection_status_changed(status):
	print("📡 Connection status: ", status)

	# Convert to string if it's not already
	var status_str = str(status)

	match status_str:
		"connected":
			if not world_loaded:
				start_world_loading()
		"disconnected":
			print("⚠️ Lost connection to backend")
		"error":
			print("❌ Backend connection error")
		_:
			print("📡 Unknown status: ", status_str)

# Camera controls
func _unhandled_input(event):
	if event is InputEventKey:
		if event.pressed:
			var move_speed = 500  # pixels per move
			match event.keycode:
				KEY_UP:
					camera.position.y -= move_speed
					_update_visible_chunks()
				KEY_DOWN:
					camera.position.y += move_speed
					_update_visible_chunks()
				KEY_LEFT:
					camera.position.x -= move_speed
					_update_visible_chunks()
				KEY_RIGHT:
					camera.position.x += move_speed
					_update_visible_chunks()
				KEY_PLUS, KEY_EQUAL:
					camera.zoom *= 0.8
				KEY_MINUS:
					camera.zoom *= 1.2
				KEY_G:
					# Toggle grid overlay
					if grid_overlay != null:
						grid_overlay.toggle_grid()
				KEY_T:
					# Toggle tooltip
					if tooltip_overlay != null:
						tooltip_overlay.toggle_tooltip()
				KEY_R:
					# Reload latest map
					reload_latest_map()
				KEY_ESCAPE:
					get_tree().quit()

# Start the world loading process when ready
func _on_timer_timeout():
	print("⏰ Timer triggered - starting world loading...")
	if not world_loaded:
		start_world_loading()
	else:
		print("ℹ️ World already loaded")

# Initialize grid overlay
func _initialize_grid_overlay():
	# Clean up any existing grid overlay first
	if grid_overlay != null:
		print("🧹 Removing existing grid overlay...")
		grid_overlay.queue_free()
		grid_overlay = null

	# Also check for any orphaned GridOverlay nodes
	for child in get_children():
		if child.name == "GridOverlay":
			print("🧹 Removing orphaned GridOverlay...")
			child.queue_free()

	# Load the GridOverlay script
	var GridOverlay = load("res://scripts/GridOverlay.gd")
	if GridOverlay == null:
		print("⚠️ Failed to load GridOverlay script")
		return

	# Create grid overlay instance
	grid_overlay = GridOverlay.new()
	grid_overlay.name = "GridOverlay"  # Set unique name for detection
	grid_overlay.set_tilemap(terrain_tilemap)  # Use TerrainTileMap with OpenRCT2 exact formula
	grid_overlay.set_camera(camera)
	grid_overlay.set_world_data_cache(WorldDataCache)  # Pass cache for height queries

	# Add as direct child of WorldRenderer (NOT TerrainTileMap) to avoid transform inheritance issues
	add_child(grid_overlay)

	print("✅ Grid overlay initialized with slope-following (Press 'G' to toggle)")

# Initialize tooltip overlay
func _initialize_tooltip_overlay():
	# Load the TooltipOverlay script
	var TooltipOverlay = load("res://scripts/TooltipOverlay.gd")
	if TooltipOverlay == null:
		print("⚠️ Failed to load TooltipOverlay script")
		return

	# Create tooltip overlay instance
	tooltip_overlay = TooltipOverlay.new()
	tooltip_overlay.set_tilemap(terrain_tilemap)  # Use TerrainTileMap with OpenRCT2 exact formula
	tooltip_overlay.set_camera(camera)

	# Add as child of root World node to be in screen space (not world space)
	add_child(tooltip_overlay)

	print("✅ Tooltip overlay initialized (Press 'T' to toggle)")

# Initialize height marker overlay
func _initialize_height_marker_overlay():
	# Load the HeightMarkerOverlay script
	var HeightMarkerOverlay = load("res://scripts/HeightMarkerOverlay.gd")
	if HeightMarkerOverlay == null:
		print("⚠️ Failed to load HeightMarkerOverlay script")
		return

	# Create height marker overlay instance
	height_marker_overlay = HeightMarkerOverlay.new()
	height_marker_overlay.set_terrain_tilemap(terrain_tilemap)
	height_marker_overlay.set_world_data_cache(WorldDataCache)
	height_marker_overlay.set_camera(camera)

	# Add as child of root World node to be in screen space (not world space)
	add_child(height_marker_overlay)

	print("✅ Height marker overlay initialized (Press 'M' or click 📏 button to toggle)")

# Initialize UI component references
func _initialize_ui_references():
	# Wait additional frame to ensure all nodes are in scene tree
	await get_tree().process_frame
	await get_tree().process_frame

	# Get references from parent World node - try multiple times
	for attempt in range(5):
		var world_node = get_tree().root.get_node_or_null("World")
		if not world_node:
			world_node = get_parent()

		if world_node:
			top_bar = world_node.get_node_or_null("TopBar")
			statistics_hud = world_node.get_node_or_null("StatisticsHUD")
			controls_overlay = world_node.get_node_or_null("ControlsOverlay")

			if top_bar and statistics_hud and controls_overlay:
				set_ui_references(top_bar, statistics_hud, controls_overlay)
				print("✅ All UI components found on attempt ", attempt + 1)
				return
			elif attempt == 4:
				print("⚠️ Some UI components not found after 5 attempts:")
				print("  TopBar: ", top_bar != null)
				print("  StatisticsHUD: ", statistics_hud != null)
				print("  ControlsOverlay: ", controls_overlay != null)
		else:
			print("⚠️ Could not get World node")

		# Wait before retrying
		await get_tree().create_timer(0.1).timeout

# Set UI component references (called from World node)
func set_ui_references(p_top_bar: CanvasLayer, p_statistics_hud: Control, p_controls_overlay: Control):
	top_bar = p_top_bar
	statistics_hud = p_statistics_hud
	controls_overlay = p_controls_overlay

	# Pass references to TopBar
	if top_bar:
		top_bar.set_world_renderer(self)
		top_bar.set_statistics_hud(statistics_hud)
		top_bar.set_controls_overlay(controls_overlay)
		top_bar.set_height_marker_overlay(height_marker_overlay)
		print("✅ UI references set in WorldRenderer and TopBar")

# Reset camera to origin (0,0) with default zoom
func reset_camera_to_origin():
	if camera and terrain_tilemap:
		var center_tile = Vector2i(0, 0)
		var center_pixel = terrain_tilemap.map_to_local(center_tile)
		camera.position = center_pixel
		camera.zoom = Vector2(0.5, 0.5)  # OpenRCT2 64×32 tiles
		print("📹 Camera reset to origin (0,0)")
		_update_visible_chunks()

# Force refresh all visible chunks - Full reload of everything
func force_refresh_chunks():
	print("🔄 Starting full reload...")

	# 1. Clear all existing data
	print("🗑️ Clearing entities...")
	if entity_manager:
		entity_manager.clear_all_entities()

	print("🗑️ Clearing resources...")
	if resource_manager:
		resource_manager.clear_all_resources()

	print("🗑️ Clearing terrain tiles...")
	if terrain_tilemap:
		terrain_tilemap.clear_all_tiles()

	print("🗑️ Clearing world data cache...")
	WorldDataCache.clear_cache()

	# 2. Reset state
	current_chunk_keys.clear()
	world_loaded = false
	loading_chunks = false

	# 3. Reload species configuration
	print("📥 Reloading species config...")
	await Config.load_species_config()

	# 4. Reload world data (will automatically load latest map)
	print("📥 Reloading world...")
	start_world_loading()

	print("✅ Full reload initiated")

# Reload the latest map specifically
func reload_latest_map():
	print("🗺️ Reloading latest map...")
	
	# Clear existing data first
	force_refresh_chunks()
	
	# The latest map will be loaded automatically in start_world_loading()

# Debug: Take screenshot for debugging
func _take_debug_screenshot():
	await RenderingServer.frame_post_draw
	var viewport = get_viewport()
	var img = viewport.get_texture().get_image()
	var screenshot_path = "/tmp/godot_terrain_debug.png"
	var error = img.save_png(screenshot_path)
	if error == OK:
		print("📸 Screenshot saved to: ", screenshot_path)
	else:
		print("❌ Failed to save screenshot: ", error)

# Debug information
func debug_print_status():
	print("=== WorldRenderer Status ===")
	print("World loaded: ", world_loaded)
	print("Current chunks: ", current_chunk_keys.size())
	print("Camera position: ", camera.position)
	print("Camera zoom: ", camera.zoom)
	print("Loading chunks: ", loading_chunks)
	print("=== End Status ===")
