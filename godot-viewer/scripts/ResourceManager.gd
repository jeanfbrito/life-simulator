# ResourceManager.gd - Renders resource overlays (trees, bushes, flowers, rocks) on terrain
# Uses actual tree textures for trees, emoji sprites for other resources

extends Node2D

# Resource sprite pool for reuse
var resource_sprites: Dictionary = {}  # chunk_key -> Array[Node2D]

# Tree texture manager (loaded dynamically)
var tree_texture_manager = null

signal resources_rendered(chunk_key: String, sprite_count: int)

func _ready():
	print("🌳 ResourceManager initialized")

	# Load RCT2 tree texture manager script and instantiate
	var TreeTextureManagerScript = load("res://scripts/TreeTextureManagerRCT2.gd")
	tree_texture_manager = TreeTextureManagerScript.new()
	add_child(tree_texture_manager)
	# Textures will load in tree_texture_manager's _ready()

# RCT2 trees are static (no animation frames like Stone Kingdoms trees)

# Map backend resource types to specific RCT2 tree species
const TREE_TYPE_MAPPING = {
	"Wood": "scots_pine",           # Default tree type → Scots Pine
	"Pine": "scots_pine",            # Pine → Scots Pine
	"Birch": "black_poplar",         # Birch → Black Poplar (deciduous)
	"Oak": "cedar_lebanon",          # Oak → Cedar of Lebanon (large tree)
	"Fir": "caucasian_fir",          # Fir → Caucasian Fir
	"Spruce": "red_fir",             # Spruce → Red Fir
}

# Paint resources for a chunk
func paint_resources(chunk_key: String, resource_data: Array):
	# Clear existing resources for this chunk
	if resource_sprites.has(chunk_key):
		for container in resource_sprites[chunk_key]:
			container.queue_free()
		resource_sprites.erase(chunk_key)

	var sprites: Array[Node2D] = []
	var chunk_origin = WorldDataCache.chunk_key_to_world_origin(chunk_key)

	# Iterate through 16x16 resource grid
	for y in range(resource_data.size()):
		var row = resource_data[y]
		if not row is Array:
			continue

		for x in range(row.size()):
			var resource_type = row[x]
			if resource_type == "":
				continue

			# Create container for Y-sorting
			var container = Node2D.new()
			container.y_sort_enabled = true

			# Position at tile location (convert to pixel space)
			var tile_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
			var pixel_pos = get_parent().map_to_local(tile_pos)

			# Apply terrain height offset to match elevated terrain (OpenRCT2 EXACT)
			var height = WorldDataCache.get_height_at(tile_pos.x, tile_pos.y)
			if height > 0:
				# Same formula as TerrainTileMap: offset = (height * COORDS_Z_STEP) / COORDS_Z_PER_TINY_Z
				var height_offset = float(height * Config.COORDS_Z_STEP) / float(Config.COORDS_Z_PER_TINY_Z)
				pixel_pos.y -= height_offset  # Move up to match terrain

			# Get resource config for sizing and offset
			var config = Config.get_resource_config(resource_type)

			# Check if this is a tree type (use actual RCT2 texture)
			if _is_tree_resource(resource_type):
				var sprite = Sprite2D.new()

				# Map resource type to specific RCT2 tree species
				var tree_species = _get_tree_species(resource_type)
				var tree_texture = tree_texture_manager.get_tree_texture(tree_species)

				if tree_texture:
					sprite.texture = tree_texture
					sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST

					# RCT2 trees are already centered and properly sized
					# Just apply a small Y-offset to position feet in tile
					sprite.centered = true
					sprite.position = Vector2(0, Config.TILE_SIZE * config.offset_y)

					container.add_child(sprite)
				else:
					# Fallback to emoji if texture not available
					var label = _create_emoji_label(resource_type, config)
					pixel_pos.x += Config.TILE_SIZE * config.offset_x
					pixel_pos.y += Config.TILE_SIZE * config.offset_y
					container.add_child(label)
			else:
				# Use emoji for non-tree resources (rocks, bushes, flowers)
				var label = _create_emoji_label(resource_type, config)
				pixel_pos.x += Config.TILE_SIZE * config.offset_x
				pixel_pos.y += Config.TILE_SIZE * config.offset_y
				container.add_child(label)

			container.position = pixel_pos
			container.z_index = 1  # Above terrain

			# Y-sort uses Y position for depth (handled automatically by y_sort_enabled)

			# Add debug position marker if enabled
			if Config.debug_show_position_markers:
				_add_debug_marker(container)

			add_child(container)
			sprites.append(container)

	resource_sprites[chunk_key] = sprites

	# Count how many trees in this chunk
	var tree_count = 0
	for sprite in sprites:
		if sprite.get_child_count() > 0:
			var child = sprite.get_child(0)
			if child is Sprite2D:
				tree_count += 1

	print("🌳 Rendered ", sprites.size(), " resources for chunk ", chunk_key, " (", tree_count, " RCT2 trees)")
	resources_rendered.emit(chunk_key, sprites.size())

# Clear resources for a chunk
func clear_resources(chunk_key: String):
	if resource_sprites.has(chunk_key):
		for sprite in resource_sprites[chunk_key]:
			sprite.queue_free()
		resource_sprites.erase(chunk_key)
		print("🗑️ Cleared resources for chunk ", chunk_key)

# Clear all resources
func clear_all_resources():
	for chunk_key in resource_sprites.keys():
		clear_resources(chunk_key)

# Get resource count for a chunk
func get_resource_count(chunk_key: String) -> int:
	if resource_sprites.has(chunk_key):
		return resource_sprites[chunk_key].size()
	return 0

# Get total resource count
func get_total_resource_count() -> int:
	var total = 0
	for chunk_key in resource_sprites.keys():
		total += get_resource_count(chunk_key)
	return total

# Add debug cross marker at container origin
func _add_debug_marker(container: Node2D):
	var cross_size = 5.0
	var cross_color = Color(1.0, 0.0, 0.0, 1.0)  # Red

	# Horizontal line
	var h_line = Line2D.new()
	h_line.add_point(Vector2(-cross_size, 0))
	h_line.add_point(Vector2(cross_size, 0))
	h_line.default_color = cross_color
	h_line.width = 2.0
	h_line.z_index = 100
	container.add_child(h_line)

	# Vertical line
	var v_line = Line2D.new()
	v_line.add_point(Vector2(0, -cross_size))
	v_line.add_point(Vector2(0, cross_size))
	v_line.default_color = cross_color
	v_line.width = 2.0
	v_line.z_index = 100
	container.add_child(v_line)

# Helper: Check if resource type is a tree
func _is_tree_resource(resource_type: String) -> bool:
	var type_lower = resource_type.to_lower()
	return (type_lower.contains("tree") or
			type_lower.contains("wood") or
			type_lower.contains("pine") or
			type_lower.contains("birch") or
			type_lower.contains("oak"))

# Helper: Map resource type to RCT2 tree species
func _get_tree_species(resource_type: String) -> String:
	# Check if we have a direct mapping
	if TREE_TYPE_MAPPING.has(resource_type):
		return TREE_TYPE_MAPPING[resource_type]

	# Check for substring matches
	var type_lower = resource_type.to_lower()
	for key in TREE_TYPE_MAPPING.keys():
		if type_lower.contains(key.to_lower()):
			return TREE_TYPE_MAPPING[key]

	# Default to scots pine for unmapped tree types
	return "scots_pine"

# Helper: Create emoji label for non-tree resources
func _create_emoji_label(resource_type: String, config: Dictionary) -> Label:
	var label = Label.new()
	label.text = Config.get_resource_symbol(resource_type)

	var label_size = int(Config.TILE_SIZE * config.size_multiplier)
	label.add_theme_font_size_override("font_size", label_size)

	# Center the emoji on the position by offsetting by half the label size
	label.position = Vector2(-label_size / 2.0, -label_size / 2.0)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
	label.custom_minimum_size = Vector2(label_size, label_size)

	return label

# Update resources from cached data for multiple chunks
func update_from_cache(chunk_keys: Array[String]):
	print("🌳 Updating resources from cache for ", chunk_keys.size(), " chunks")

	var chunks_updated = 0
	for chunk_key in chunk_keys:
		var resource_data = WorldDataCache.get_resource_chunk(chunk_key)
		if resource_data.size() > 0:
			paint_resources(chunk_key, resource_data)
			chunks_updated += 1

	print("✅ Updated resources for ", chunks_updated, " chunks")

# Debug information
func debug_print_status():
	print("=== ResourceManager Status ===")
	print("Chunks with resources: ", resource_sprites.keys().size())
	var total_resources = 0
	for chunk_key in resource_sprites.keys():
		var count = get_resource_count(chunk_key)
		total_resources += count
		print("  Chunk ", chunk_key, ": ", count, " resources")
	print("Total resources rendered: ", total_resources)
	print("=== End Status ===")