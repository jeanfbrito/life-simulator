# WorldRenderer.gd - Main world rendering system
# Connects backend chunk data to TileMap rendering with camera-based streaming

extends Node2D

@onready var terrain_tilemap: TileMap = $TerrainTileMap
@onready var camera: Camera2D = $TerrainTileMap/Camera2D
@onready var resource_manager: Node2D = $TerrainTileMap/ResourceManager
@onready var entity_manager: Node2D = $TerrainTileMap/EntityManager

# Grid overlay (will be created dynamically)
var grid_overlay: Node2D = null

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

	# Initialize UI references (wait one frame for UI nodes to be ready)
	await get_tree().process_frame
	_initialize_ui_references()

	# Initialize camera position - center on island area (tile 0,0)
	# Convert tile (0,0) to pixel coordinates in isometric space
	var center_tile = Vector2i(0, 0)
	var center_pixel = terrain_tilemap.map_to_local(center_tile)
	camera.position = center_pixel
	camera.zoom = Vector2(0.5, 0.5)  # Zoom out to see isometric tiles (128x64 tiles are large)
	print("📹 Camera positioned at tile ", center_tile, " = pixel ", center_pixel, " with zoom 0.5x")

	# Print camera and tilemap info
	print("📹 Camera actual position: ", camera.position, " zoom: ", camera.zoom)
	print("📹 TileMap tile_set: ", "Loaded" if terrain_tilemap.tile_set != null else "NULL")
	if terrain_tilemap.tile_set:
		print("📹 TileSet tile_shape: ", terrain_tilemap.tile_set.tile_shape)
		print("📹 TileSet tile_size: ", terrain_tilemap.tile_set.tile_size)

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

	var success = await ChunkManager.load_world_info()
	if success:
		print("✅ World info loading completed successfully")
		# Since ChunkManager.load_world_info returns bool, we proceed to load chunks
		await _load_chunks_around_position(Vector2i(0, 0))
		world_loaded = true
		print("✅ World loading completed - viewer should show terrain")
	else:
		print("❌ Failed to load world info")

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

	# Debug: Print TileMap state
	print("📊 TileMap stats:")
	print("   - Total cells rendered: ", terrain_tilemap.get_used_cells(0).size())
	print("   - TileSet exists: ", terrain_tilemap.tile_set != null)
	print("   - Visible: ", terrain_tilemap.visible)
	print("   - Modulate: ", terrain_tilemap.modulate)
	if terrain_tilemap.get_used_cells(0).size() > 0:
		var sample_cells = terrain_tilemap.get_used_cells(0).slice(0, min(5, terrain_tilemap.get_used_cells(0).size()))
		print("   - Sample cells: ", sample_cells)
		for cell_pos in sample_cells:
			var pixel_pos = terrain_tilemap.map_to_local(cell_pos)
			print("     Cell ", cell_pos, " -> Pixel ", pixel_pos)

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
			terrain_tilemap.paint_chunk(chunk_key, terrain_data)
			
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
	# Load the GridOverlay script
	var GridOverlay = load("res://scripts/GridOverlay.gd")
	if GridOverlay == null:
		print("⚠️ Failed to load GridOverlay script")
		return

	# Create grid overlay instance
	grid_overlay = GridOverlay.new()
	grid_overlay.set_tilemap(terrain_tilemap)
	grid_overlay.set_camera(camera)

	# Add as child of TerrainTileMap so it inherits transformations
	terrain_tilemap.add_child(grid_overlay)

	print("✅ Grid overlay initialized (Press 'G' to toggle)")

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
		print("✅ UI references set in WorldRenderer and TopBar")

# Reset camera to origin (0,0) with default zoom
func reset_camera_to_origin():
	if camera and terrain_tilemap:
		var center_tile = Vector2i(0, 0)
		var center_pixel = terrain_tilemap.map_to_local(center_tile)
		camera.position = center_pixel
		camera.zoom = Vector2(0.5, 0.5)
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

	# 4. Reload world data
	print("📥 Reloading world...")
	start_world_loading()

	print("✅ Full reload initiated")

# Debug information
func debug_print_status():
	print("=== WorldRenderer Status ===")
	print("World loaded: ", world_loaded)
	print("Current chunks: ", current_chunk_keys.size())
	print("Camera position: ", camera.position)
	print("Camera zoom: ", camera.zoom)
	print("Loading chunks: ", loading_chunks)
	print("=== End Status ===")