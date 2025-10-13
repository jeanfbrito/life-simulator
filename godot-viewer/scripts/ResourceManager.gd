# ResourceManager.gd - Renders resource overlays (trees, bushes, flowers, rocks) on terrain
# Uses actual tree textures for trees, emoji sprites for other resources

extends Node2D

# Resource sprite pool for reuse
var resource_sprites: Dictionary = {}  # chunk_key -> Array[Node2D]

# Tree texture manager (loaded dynamically)
var tree_texture_manager = null

# Tree animation tracking (stateless - trees query global WindManager)
var animated_trees: Array[Dictionary] = []  # Array of {sprite, is_pine, current_frame}

signal resources_rendered(chunk_key: String, sprite_count: int)

func _ready():
	print("🌳 ResourceManager initialized")

	# Load tree texture manager script and instantiate
	var TreeTextureManagerScript = load("res://scripts/TreeTextureManager.gd")
	tree_texture_manager = TreeTextureManagerScript.new()
	add_child(tree_texture_manager)
	# Textures will load in tree_texture_manager's _ready()

func _process(delta: float):
	# Update tree animations based on global wind
	for tree_data in animated_trees:
		var sprite = tree_data["sprite"]

		# Query global wind system for this tree's frame
		var wind_frame = WindManager.get_wind_frame_for_position(sprite.global_position)

		# Only update if frame changed (optimization)
		if tree_data["current_frame"] != wind_frame:
			tree_data["current_frame"] = wind_frame

			# Get texture and offset for this frame
			var new_texture = null
			var new_offset = Vector2.ZERO

			if tree_data["is_pine"]:
				if wind_frame < tree_texture_manager.pine_tree_textures.size():
					new_texture = tree_texture_manager.pine_tree_textures[wind_frame]
					new_offset = tree_texture_manager.pine_offsets[wind_frame]
			else:  # Birch
				if wind_frame < tree_texture_manager.birch_tree_textures.size():
					new_texture = tree_texture_manager.birch_tree_textures[wind_frame]
					new_offset = tree_texture_manager.birch_offsets[wind_frame]

			# Update sprite texture and position
			if new_texture:
				sprite.texture = new_texture

				# Update position with new offset
				var sk_offset_x = tree_texture_manager.TREE_BASE_OFFSET_X + new_offset.x
				var sk_offset_y = tree_texture_manager.TREE_BASE_OFFSET_Y + new_offset.y
				sprite.position = Vector2(sk_offset_x, sk_offset_y)

# Paint resources for a chunk
func paint_resources(chunk_key: String, resource_data: Array):
	# Clear existing resources for this chunk
	if resource_sprites.has(chunk_key):
		for container in resource_sprites[chunk_key]:
			# Remove any tree sprites in this container from animated_trees
			if container.get_child_count() > 0:
				var child = container.get_child(0)
				# Remove from animated_trees if it's a tree sprite
				for i in range(animated_trees.size() - 1, -1, -1):
					if animated_trees[i]["sprite"] == child:
						animated_trees.remove_at(i)
						break
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

			# Get resource config for sizing and offset
			var config = Config.get_resource_config(resource_type)

			# Check if this is a tree type (use actual texture)
			if _is_tree_resource(resource_type):
				var sprite = Sprite2D.new()

				# Determine tree type (pine or birch)
				var is_pine = true
				var type_lower = resource_type.to_lower()
				if type_lower.contains("birch"):
					is_pine = false
				# Default to pine for "wood" or unknown types

				# Start with variant 01 (index 0)
				var tree_texture = null
				var quad_offset = Vector2.ZERO

				if is_pine and tree_texture_manager.pine_tree_textures.size() > 0:
					tree_texture = tree_texture_manager.pine_tree_textures[0]  # tree_pine_large_01
					quad_offset = tree_texture_manager.pine_offsets[0]
				elif not is_pine and tree_texture_manager.birch_tree_textures.size() > 0:
					tree_texture = tree_texture_manager.birch_tree_textures[0]  # tree_birch_large_01
					quad_offset = tree_texture_manager.birch_offsets[0]

				if tree_texture:
					sprite.texture = tree_texture
					sprite.texture_filter = CanvasItem.TEXTURE_FILTER_NEAREST

					# Apply stone-kingdoms offset system
					var sk_offset_x = tree_texture_manager.TREE_BASE_OFFSET_X + quad_offset.x
					var sk_offset_y = tree_texture_manager.TREE_BASE_OFFSET_Y + quad_offset.y

					sprite.centered = false
					sprite.position = Vector2(sk_offset_x, sk_offset_y)

					container.add_child(sprite)

					# Add to animated trees list
					# Trees are stateless - they query global WindManager for their frame
					animated_trees.append({
						"sprite": sprite,
						"is_pine": is_pine,
						"current_frame": 0  # Will be updated by wind system
					})
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

	print("🌳 Rendered ", sprites.size(), " resources for chunk ", chunk_key, " (", tree_count, " animated trees, ", animated_trees.size(), " total)")
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