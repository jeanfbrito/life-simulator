# TerrainTileMap.gd - Manages isometric terrain rendering
# Handles chunk data conversion to individual Sprite2D nodes with height-based positioning

extends Node2D

const SlopeCalculator = preload("res://scripts/SlopeCalculator.gd")

# RCT2 terrain texture manager for all terrain types (grass, sand, dirt, etc.)
var rct2_terrain_manager = null

# Water texture manager for RCT2 water sprites
var water_texture_manager = null

# Tile sprite container (for Y-sorting)
var tile_container: Node2D = null

# Store all tile sprites by world position for updates/cleanup
var tile_sprites: Dictionary = {}  # Key: "x,y", Value: Sprite2D

# OpenRCT2 coordinate constants - EXACT MATCH
# From: src/openrct2/world/Location.hpp
const COORDS_XY_STEP = 32          # kCoordsXYStep - base coordinate step
const COORDS_Z_STEP = 8            # kCoordsZStep - pixels per Z level
const COORDS_Z_PER_TINY_Z = 16     # kCoordsZPerTinyZ - height division factor

# Isometric tile dimensions - OpenRCT2 EXACT MATCH
const TILE_WIDTH = 64   # 2 * COORDS_XY_STEP = 64 (diamond width)
const TILE_HEIGHT = 32  # COORDS_XY_STEP = 32 (diamond height)

# OpenRCT2 Rendering Scale - CRITICAL FOR ELEVATION VISIBILITY
# OpenRCT2 has sprite scale options: 1x, 2x, 3x, 4x (Settings → Display)
# Default is 2x for modern displays to make elevation dramatic
# This is INDEPENDENT of camera zoom (zoom is for viewport, scale is for rendering)
const RENDERING_SCALE = 2.0  # 2x scale like OpenRCT2 default (try 1.0, 2.0, 3.0, or 4.0)

# Helper TileMap for coordinate conversion only (not rendered)
var coord_helper: TileMap = null

func _ready():
	print("🗺️ TerrainTileMap initialized (Sprite2D-based rendering)")

	# Create tile container with Y-sorting enabled
	tile_container = Node2D.new()
	tile_container.name = "TileContainer"
	tile_container.y_sort_enabled = true
	add_child(tile_container)
	print("📦 Tile container created with Y-sorting enabled")

	# Create helper TileMap for coordinate conversion (not rendered, just for map_to_local)
	coord_helper = TileMap.new()
	coord_helper.tile_set = TileSet.new()
	coord_helper.tile_set.tile_shape = TileSet.TILE_SHAPE_ISOMETRIC
	coord_helper.tile_set.tile_layout = TileSet.TILE_LAYOUT_STACKED
	coord_helper.tile_set.tile_size = Vector2i(TILE_WIDTH, TILE_HEIGHT)
	coord_helper.visible = false  # Don't render it
	add_child(coord_helper)
	print("🧭 Coordinate helper TileMap created")

	# Initialize RCT2 terrain texture manager
	var TerrainManager = load("res://scripts/RCT2TerrainTextureManager.gd")
	rct2_terrain_manager = TerrainManager.new()
	add_child(rct2_terrain_manager)
	print("🌍 RCT2TerrainTextureManager initialized")

	# Initialize water texture manager (simple version for flat water)
	water_texture_manager = _load_water_texture()
	if water_texture_manager:
		print("🌊 Water textures loaded")

# Helper function to convert tile coordinates to pixel position with height
func map_to_local(tile_pos: Vector2i) -> Vector2:
	"""Convert tile coordinates to isometric pixel coordinates using OpenRCT2 EXACT formula."""
	# OpenRCT2 isometric projection formula from Location.hpp:
	# screen_x = (tile_x - tile_y) × kCoordsXYStep
	# screen_y = (tile_x + tile_y) × (kCoordsXYStep / 2)
	var pixel_x = float(tile_pos.x - tile_pos.y) * float(COORDS_XY_STEP)
	var pixel_y = float(tile_pos.x + tile_pos.y) * float(COORDS_XY_STEP / 2)
	return Vector2(pixel_x, pixel_y)

# Helper function for reverse conversion (if needed)
func local_to_map(pixel_pos: Vector2) -> Vector2i:
	"""Convert pixel coordinates to tile coordinates."""
	return coord_helper.local_to_map(pixel_pos)

# Compute screen Y offsets for each corner (in pixels)
# Returns Dictionary with keys: top, right, bottom, left
func compute_corner_offsets_screen(base_height: int, slope_index: int) -> Dictionary:
	"""Calculate per-corner screen Y offsets using OpenRCT2 corner-height logic."""
	var corner_heights = SlopeCalculator.get_corner_heights(base_height, slope_index)
	
	# Convert each corner's tiny-Z to screen pixels
	# Formula: (tiny_z * kCoordsZStep) / kCoordsZPerTinyZ * RENDERING_SCALE
	# Simplifies to: tiny_z / 2.0 * RENDERING_SCALE
	var offsets = {}
	for corner in ["top", "right", "bottom", "left"]:
		var tiny_z = corner_heights[corner]
		var screen_offset = float(tiny_z * COORDS_Z_STEP) / float(COORDS_Z_PER_TINY_Z) * RENDERING_SCALE
		offsets[corner] = screen_offset
	
	return offsets

# Paint a chunk's terrain using individual Sprite2D nodes
func paint_chunk(chunk_key: String, terrain_data: Array, height_data: Array = [], slope_data: Array = []):
	if terrain_data.size() == 0:
		print("⚠️ No terrain data for chunk ", chunk_key)
		return

	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)
	var has_heights = height_data.size() > 0
	var has_slopes = slope_data.size() > 0
	print("🎨 Painting chunk ", chunk_key, " with origin ", chunk_origin, " (heights: ", has_heights, ")")

	var tiles_painted = 0
	for y in range(terrain_data.size()):
		var row = terrain_data[y]
		if not row is Array:
			continue

		for x in range(row.size()):
			var terrain_type = row[x]
			if terrain_type == "":
				continue

			var world_pos = Vector2i(
				chunk_origin.x + x,
				chunk_origin.y + y
			)

			# Get height value for this tile
			var height = 0
			if has_heights and y < height_data.size() and x < height_data[y].size():
				height = int(height_data[y][x])

			var slope_index = 0
			if has_slopes and y < slope_data.size() and x < slope_data[y].size():
				slope_index = int(slope_data[y][x])

			slope_index = SlopeCalculator.rotate_slope_index(slope_index, Config.slope_rotation)

			paint_terrain_tile(world_pos, terrain_type, slope_index, height)
			tiles_painted += 1

	print("🎨 Painted ", tiles_painted, " terrain tiles for chunk ", chunk_key)
	print("🎨 Total sprites: ", tile_sprites.size())
	
	# Second pass: Add edge faces where height differences exist
	if has_heights and has_slopes:
		_paint_chunk_edges(chunk_key, terrain_data, height_data, slope_data)

# Paint a single terrain tile as Sprite2D with height-based positioning
func paint_terrain_tile(world_pos: Vector2i, terrain_type: String, slope_index: int = 0, height: int = 0):
	# Get texture for this tile
	var texture: Texture2D = null

	if _should_use_rct2_texture(terrain_type) and rct2_terrain_manager and rct2_terrain_manager.has_textures():
		if _is_water_terrain(terrain_type) and water_texture_manager:
			texture = water_texture_manager
		else:
			texture = rct2_terrain_manager.get_terrain_texture(terrain_type, slope_index)

	if not texture:
		print("⚠️ No texture for terrain type: ", terrain_type)
		return

	# Create or get existing sprite for this position
	var tile_key = "%d,%d" % [world_pos.x, world_pos.y]
	var sprite: Sprite2D = null

	if tile_sprites.has(tile_key):
		sprite = tile_sprites[tile_key]
	else:
		sprite = Sprite2D.new()
		sprite.name = "Tile_%d_%d" % [world_pos.x, world_pos.y]
		sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST  # Pixel-perfect
		tile_container.add_child(sprite)
		tile_sprites[tile_key] = sprite

	# Set texture
	sprite.texture = texture

	# Calculate isometric position using OpenRCT2 EXACT formula
	var base_pos = map_to_local(world_pos)  # Uses custom OpenRCT2 formula, NOT Godot's TileMap

	# OpenRCT2 corner-based positioning: anchor sprite at north (top) corner
	# This ensures tiles align seamlessly at shared corners
	var corner_offsets = compute_corner_offsets_screen(height, slope_index)
	var north_corner_offset = corner_offsets["top"]  # North corner = top in our coordinate system
	
	# Sprite Y position: base position minus the north corner height
	# This anchors the sprite so its visual "north corner" sits at the correct screen Y
	var final_pos = Vector2(base_pos.x, base_pos.y - north_corner_offset)

	sprite.position = final_pos

	# Set Z index for Y-sorting based on final Y position
	sprite.z_index = int(final_pos.y)

	# Debug output for first few tiles - corner-based
	if tile_sprites.size() <= 10:
		var slope_info = " slope=%d" % slope_index if slope_index > 0 else ""
		var corner_heights = SlopeCalculator.get_corner_heights(height, slope_index)
		print("🏔️ CORNER DEBUG: tile %s, height=%d, slope=%d → corners(N=%d,E=%d,S=%d,W=%d) → north_offset=%.1f px → final_y=%.1f%s → %s" %
		      [world_pos, height, slope_index, corner_heights.top, corner_heights.right, corner_heights.bottom, corner_heights.left, 
		       north_corner_offset, final_pos.y, slope_info, terrain_type])

func _should_use_rct2_texture(terrain_type: String) -> bool:
	"""Check if this terrain type should use RCT2 textures."""
	# Enable RCT2 textures for ALL terrain types that have RCT2 mappings
	var supported_terrains = ["Grass", "Forest", "Sand", "Desert", "Dirt",
							  "Stone", "Mountain", "Snow", "Ice", "Swamp",
							  "DeepWater", "ShallowWater"]
	return terrain_type in supported_terrains

func _is_water_terrain(terrain_type: String) -> bool:
	"""Check if this terrain type is water."""
	return terrain_type in ["DeepWater", "ShallowWater"]

func _load_water_texture() -> Texture2D:
	"""Load flat RCT2 water texture."""
	var file_path = "assets/tiles/water/rct2/water_mask_00.png"
	var image = Image.new()
	var error = image.load(file_path)

	if error == OK:
		var texture = ImageTexture.create_from_image(image)
		if texture:
			print("✅ Loaded RCT2 water texture: ", file_path)
			return texture

	push_warning("🌊 Could not load water texture: " + file_path)
	return null

# Paint edge faces for a chunk (called after base terrain)
func _paint_chunk_edges(chunk_key: String, terrain_data: Array, height_data: Array, slope_data: Array):
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)
	
	# Create edge renderer if not exists
	if not has_node("EdgeRenderer"):
		var edge_renderer = EdgeRenderer.new()
		edge_renderer.name = "EdgeRenderer"
		add_child(edge_renderer)
	
	var edge_renderer = get_node("EdgeRenderer")
	var edges_painted = 0
	
	for y in range(terrain_data.size()):
		if not terrain_data[y] is Array:
			continue
			
		for x in range(terrain_data[y].size()):
			var world_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
			
			# Get this tile's corner heights
			var height = int(height_data[y][x]) if (y < height_data.size() and x < height_data[y].size()) else 0
			var slope_index = int(slope_data[y][x]) if (y < slope_data.size() and x < slope_data[y].size()) else 0
			slope_index = SlopeCalculator.rotate_slope_index(slope_index, Config.slope_rotation)
			
			var tile_corners = SlopeCalculator.get_corner_heights(height, slope_index)
			
			# Get neighbor corner heights
			var neighbors = _get_neighbor_corners(world_pos, height_data, slope_data)
			
			# Paint edge faces for this tile
			if neighbors.size() > 0:
				edge_renderer.paint_edge_faces(tile_container, world_pos, tile_corners, neighbors)
				edges_painted += 1
	
	if edges_painted > 0:
		print("🏔️ Painted edge faces for %d tiles in chunk %s" % [edges_painted, chunk_key])

# Get corner heights for neighboring tiles
func _get_neighbor_corners(tile_pos: Vector2i, height_data: Array, slope_data: Array) -> Dictionary:
	var neighbors = {}
	
	# North neighbor (y-1)
	var north_pos = tile_pos + Vector2i(0, -1)
	neighbors["north"] = _get_tile_corners_from_world(north_pos, height_data, slope_data)
	
	# East neighbor (x+1)
	var east_pos = tile_pos + Vector2i(1, 0)
	neighbors["east"] = _get_tile_corners_from_world(east_pos, height_data, slope_data)
	
	# South neighbor (y+1)
	var south_pos = tile_pos + Vector2i(0, 1)
	neighbors["south"] = _get_tile_corners_from_world(south_pos, height_data, slope_data)
	
	# West neighbor (x-1)
	var west_pos = tile_pos + Vector2i(0, -1)
	neighbors["west"] = _get_tile_corners_from_world(west_pos, height_data, slope_data)
	
	return neighbors

# Get corner heights for a tile at world position
func _get_tile_corners_from_world(world_pos: Vector2i, height_data: Array, slope_data: Array) -> Dictionary:
	# Get height from cache
	var height = WorldDataCache.get_height_at(world_pos)
	if height == null:
		return {} # No data
	
	# Get slope from cache
	var slope_index = WorldDataCache.get_slope_index_at(world_pos)
	if slope_index == null:
		slope_index = 0
	
	# Apply rotation
	slope_index = SlopeCalculator.rotate_slope_index(slope_index, Config.slope_rotation)
	
	# Return corner heights
	return SlopeCalculator.get_corner_heights(height, slope_index)

# Clear a chunk's tiles from the TileMap
func clear_chunk(chunk_key: String):
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

	for y in range(Config.CHUNK_SIZE):
		for x in range(Config.CHUNK_SIZE):
			var world_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
			var tile_key = "%d,%d" % [world_pos.x, world_pos.y]

			# Remove sprite if it exists
			if tile_sprites.has(tile_key):
				var sprite = tile_sprites[tile_key]
				if sprite and is_instance_valid(sprite):
					sprite.queue_free()
				tile_sprites.erase(tile_key)

	# Clear terrain rectangles in this chunk area
	_clear_terrain_rects_in_chunk(chunk_origin)

# Clear terrain rectangles in a specific chunk area
func _clear_terrain_rects_in_chunk(chunk_origin: Vector2i):
	if not has_meta("terrain_rects"):
		return

	var rects = get_meta("terrain_rects")
	var rects_to_keep = []

	for rect in rects:
		if rect and is_instance_valid(rect):
			# Check if rectangle is within chunk bounds (simple approximation)
			# For now, we'll clear all rects since we're doing complete chunk replacement
			rect.queue_free()
		else:
			rects_to_keep.append(rect)

	set_meta("terrain_rects", rects_to_keep)

# Clear terrain sprites in a specific chunk area (legacy)
func _clear_terrain_sprites_in_chunk(chunk_origin: Vector2i):
	if not has_meta("terrain_sprites"):
		return

	var sprites = get_meta("terrain_sprites")
	var sprites_to_keep = []

	for sprite in sprites:
		if sprite and is_instance_valid(sprite):
			sprite.queue_free()

	set_meta("terrain_sprites", sprites_to_keep)

# Clear all tiles (sprites)
func clear_all_tiles():
	# Clear all tile sprites
	for tile_key in tile_sprites.keys():
		var sprite = tile_sprites[tile_key]
		if sprite and is_instance_valid(sprite):
			sprite.queue_free()
	tile_sprites.clear()

	# Clear all terrain sprites (legacy)
	if has_meta("terrain_sprites"):
		var sprites = get_meta("terrain_sprites")
		for sprite in sprites:
			if sprite and is_instance_valid(sprite):
				sprite.queue_free()
		set_meta("terrain_sprites", [])

# Get the terrain color for a tile position
func get_terrain_at_tile(tile_pos: Vector2i) -> String:
	# This would need to be implemented based on our data structure
	# For now, return default terrain
	return Config.DEFAULT_TERRAIN_TYPE

# Update multiple chunks efficiently
func update_chunks(chunk_keys: Array[String]):
	for chunk_key in chunk_keys:
		var terrain_data = WorldDataCache.get_terrain_chunk(chunk_key)
		if terrain_data.size() > 0:
			paint_chunk(chunk_key, terrain_data)

# Debug function to print terrain rendering info
func debug_print_info():
	print("=== TerrainTileMap Debug Info ===")
	print("Rendering mode: Sprite2D-based with height positioning")
	print("Total tile sprites: ", tile_sprites.size())
	print("Tile container children: ", tile_container.get_child_count() if tile_container else 0)
	print("Y-sorting enabled: ", tile_container.y_sort_enabled if tile_container else false)
	print("=== End Debug Info ===")
