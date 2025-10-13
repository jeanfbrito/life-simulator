extends TileMap
class_name GrassTerrainIntegration

## Example integration of GrassMacroTileRenderer with TerrainTileMap
##
## This shows how to use the macro tile system for grass rendering.
## You can adapt this to work with your existing TerrainTileMap.gd

@onready var grass_renderer: GrassMacroTileRenderer = GrassMacroTileRenderer.new()

func _ready():
	add_child(grass_renderer)
	print("🌿 GrassTerrainIntegration ready")

func paint_chunk_with_macro_tiles(chunk_key: String, chunk_data: Dictionary):
	"""
	Paint a chunk using the macro tile system.
	This replaces the standard per-tile rendering for grass terrain.
	"""

	var terrain_data = chunk_data.get("terrain", [])
	if terrain_data.is_empty():
		push_warning("No terrain data for chunk: " + chunk_key)
		return

	# Get chunk origin in world tile coordinates
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

	# Iterate through all tiles in the chunk
	for local_y in range(16):
		for local_x in range(16):
			# Get terrain type at this position
			var terrain_type = terrain_data[local_y][local_x] if local_y < terrain_data.size() and local_x < terrain_data[local_y].size() else "Unknown"

			# Only use macro tiles for grass terrains
			if not _is_grass_terrain(terrain_type):
				# Use standard colored tile for non-grass
				_paint_standard_tile(chunk_origin, local_x, local_y, terrain_type)
				continue

			# Calculate world position
			var world_pos = Vector2i(
				chunk_origin.x + local_x,
				chunk_origin.y + local_y
			)

			# Get the appropriate grass tile from macro tile renderer
			var tile_info = grass_renderer.select_grass_tile(
				chunk_key,
				Vector2i(local_x, local_y),
				terrain_type,
				chunk_data
			)

			# Skip if this tile is covered by a macro tile painted earlier
			if tile_info.get("skip", false):
				continue

			# Paint the tile based on its size
			_paint_grass_tile(world_pos, tile_info)

	print("🌿 Painted chunk %s with macro tiles" % chunk_key)

func _is_grass_terrain(terrain_type: String) -> bool:
	"""Check if terrain type should use grass textures."""
	return terrain_type in ["Grass", "Forest"]  # Add more as needed

func _paint_grass_tile(world_pos: Vector2i, tile_info: Dictionary):
	"""
	Paint a grass tile using the macro tile texture.

	For macro tiles (2x2, 3x3, 4x4), we need to render a sprite
	because TileMap doesn't support multi-tile textures directly.
	"""

	var size = tile_info.get("size", 1)
	var texture = tile_info.get("texture")

	if not texture:
		push_warning("No texture for grass tile at " + str(world_pos))
		return

	if size == 1:
		# For 1x1 tiles, we can use TileMap's standard rendering
		# Create a source with this texture
		var source_id = _get_or_create_texture_source(texture)
		set_cell(0, world_pos, source_id, Vector2i(0, 0))

	else:
		# For macro tiles (2x2, 3x3, 4x4), render as Sprite2D
		_render_macro_tile_sprite(world_pos, texture, size)

func _render_macro_tile_sprite(world_pos: Vector2i, texture: Texture2D, size: int):
	"""
	Render a macro tile as a Sprite2D.

	This is necessary because Godot TileMap expects each cell to be a single tile.
	Macro tiles cover multiple cells, so we render them as sprites on top.
	"""

	# Convert tile position to pixel position
	var pixel_pos = map_to_local(world_pos)

	# Offset to center the macro tile over its covered area
	# Macro tiles are positioned at their top-left tile, but should render centered
	var offset = Vector2(
		(size - 1) * 64,  # Half-width offset in pixels
		(size - 1) * 32   # Half-height offset in pixels
	)
	pixel_pos += offset / 2.0

	# Create sprite
	var sprite = Sprite2D.new()
	sprite.texture = texture
	sprite.position = pixel_pos

	# Scale the sprite to match your tile size
	var scale = grass_renderer.get_tile_scale_for_size(size)
	sprite.scale = scale

	# Set z-index so it renders below entities but above base terrain
	sprite.z_index = -10

	# Add metadata for cleanup
	sprite.set_meta("chunk_origin", world_pos)
	sprite.set_meta("macro_size", size)
	sprite.set_meta("is_grass_macro_tile", true)

	# Add to scene
	add_child(sprite)

func _get_or_create_texture_source(texture: Texture2D) -> int:
	"""
	Get or create a TileSet source for a specific texture.
	This allows using the grass textures in the TileMap.
	"""

	# Check if we already have this texture as a source
	var texture_path = texture.resource_path
	if has_meta("texture_sources") and get_meta("texture_sources").has(texture_path):
		return get_meta("texture_sources")[texture_path]

	# Create new source
	var source = TileSetAtlasSource.new()
	source.texture = texture
	source.texture_region_size = Vector2i(128, 64)  # Your tile size
	source.create_tile(Vector2i(0, 0))

	var source_id = tile_set.add_source(source)

	# Cache the source ID
	if not has_meta("texture_sources"):
		set_meta("texture_sources", {})
	get_meta("texture_sources")[texture_path] = source_id

	return source_id

func _paint_standard_tile(chunk_origin: Vector2i, local_x: int, local_y: int, terrain_type: String):
	"""Paint a standard colored tile (for non-grass terrain)."""
	var world_pos = Vector2i(
		chunk_origin.x + local_x,
		chunk_origin.y + local_y
	)

	var color = Config.get_terrain_color(terrain_type)
	var source_id = _get_or_create_colored_source(terrain_type, color)

	set_cell(0, world_pos, source_id, Vector2i(0, 0))

func _get_or_create_colored_source(terrain_type: String, color: Color) -> int:
	"""Create a colored tile source (existing TerrainTileMap logic)."""
	# Your existing implementation from TerrainTileMap.gd
	# This is just a placeholder
	return 0

func clear_chunk_grass_tiles(chunk_key: String):
	"""Clear grass macro tile sprites for a chunk when it unloads."""

	# Clear skip data
	grass_renderer.clear_skip_data(chunk_key)

	# Remove macro tile sprites
	for child in get_children():
		if child is Sprite2D and child.has_meta("is_grass_macro_tile"):
			var sprite_chunk_origin = child.get_meta("chunk_origin")
			var chunk_coords = WorldDataCache.get_chunk_key(sprite_chunk_origin.x, sprite_chunk_origin.y)

			if chunk_coords == chunk_key:
				child.queue_free()
