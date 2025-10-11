# ResourceManager.gd - Renders resource overlays (trees, bushes, flowers, rocks) on terrain
# Creates emoji sprites for resources positioned on top of terrain tiles

extends Node2D

# Resource sprite pool for reuse
var resource_sprites: Dictionary = {}  # chunk_key -> Array[Node2D]

signal resources_rendered(chunk_key: String, sprite_count: int)

func _ready():
	print("🌳 ResourceManager initialized")

# Paint resources for a chunk
func paint_resources(chunk_key: String, resource_data: Array):
	# Clear existing resources for this chunk
	if resource_sprites.has(chunk_key):
		for sprite in resource_sprites[chunk_key]:
			sprite.queue_free()
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

			# Create emoji label for this resource
			var label = Label.new()
			label.text = Config.get_resource_symbol(resource_type)

			# Get resource config for sizing and offset
			var config = Config.get_resource_config(resource_type)
			var label_size = int(Config.TILE_SIZE * config.size_multiplier)
			label.add_theme_font_size_override("font_size", label_size)

			# Position at tile location (convert to pixel space)
			var tile_pos = Vector2i(chunk_origin.x + x, chunk_origin.y + y)
			var pixel_pos = get_parent().map_to_local(tile_pos)

			# For isometric tiles, map_to_local() returns the top point of the diamond
			# We need to offset down by half the tile height to center vertically
			pixel_pos.y += 16  # Half of 32 (tile height)

			# Apply resource offset
			pixel_pos.x += Config.TILE_SIZE * config.offset_x
			pixel_pos.y += Config.TILE_SIZE * config.offset_y

			# Center the emoji on the position by offsetting by half the label size
			label.position = Vector2(-label_size / 2.0, -label_size / 2.0)
			label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
			label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER
			label.custom_minimum_size = Vector2(label_size, label_size)

			container.add_child(label)
			container.position = pixel_pos
			container.z_index = 1  # Above terrain

			# Y-sort uses Y position for depth (handled automatically by y_sort_enabled)

			add_child(container)
			sprites.append(container)

	resource_sprites[chunk_key] = sprites
	print("🌳 Rendered ", sprites.size(), " resources for chunk ", chunk_key)
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