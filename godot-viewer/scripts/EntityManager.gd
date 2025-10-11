# EntityManager.gd - Manages entity rendering and polling
# Displays rabbits, humans, and other entities as emoji sprites

extends Node2D

# Entity tracking
var entities: Dictionary = {}  # entity_id -> Label node
var entity_poll_timer: Timer
var last_entity_data: Array = []

signal entities_updated(entity_list)
signal entity_spawned(entity_id: int, entity_data: Dictionary)
signal entity_despawned(entity_id: int)

func _ready():
	print("🐇 EntityManager initialized")

	# Set up polling timer (200ms like web viewer)
	entity_poll_timer = Timer.new()
	entity_poll_timer.wait_time = 0.2
	entity_poll_timer.timeout.connect(_poll_entities)
	add_child(entity_poll_timer)
	entity_poll_timer.start()

	print("🐇 EntityManager: Entity polling started (every 200ms)")

	# Poll immediately on start
	_poll_entities()

# Poll entities from backend API
func _poll_entities():
	var http = HTTPRequest.new()
	add_child(http)

	var error = http.request(Config.api_base_url + "/api/entities")
	if error != OK:
		print("❌ Failed to start entity request: ", error)
		http.queue_free()
		return

	var result = await http.request_completed
	http.queue_free()

	if result[0] != HTTPRequest.RESULT_SUCCESS or result[1] != 200:
		print("❌ Entity request failed: ", result[0], " ", result[1])
		return

	var json = JSON.new()
	if json.parse(result[3].get_string_from_utf8()) != OK:
		print("❌ Failed to parse entity JSON")
		return

	var data = json.data
	if data.has("entities"):
		_update_entities(data.entities)
	else:
		print("⚠️ EntityManager: API response has no 'entities' key")

# Update entity sprites based on latest data
func _update_entities(entity_list: Array):
	last_entity_data = entity_list
	var seen_ids = {}

	# Collect IDs from new data
	for entity_data in entity_list:
		var entity_id = int(entity_data.id)  # Convert to int from JSON float
		seen_ids[entity_id] = true

		if not entities.has(entity_id):
			# Create new entity sprite
			_create_entity(entity_id, entity_data)
			entity_spawned.emit(entity_id, entity_data)
		else:
			# Update existing entity
			_update_entity_position(entity_id, entity_data)

	# Remove entities that no longer exist
	var entities_to_remove = []
	for entity_id in entities.keys():
		if not seen_ids.has(entity_id):
			entities_to_remove.append(entity_id)

	for entity_id in entities_to_remove:
		entities[entity_id].queue_free()
		entities.erase(entity_id)
		entity_despawned.emit(entity_id)

	entities_updated.emit(entity_list)

# Create a new entity sprite
func _create_entity(entity_id: int, data: Dictionary):
	var container = Node2D.new()
	container.y_sort_enabled = true

	var label = Label.new()

	# Get entity configuration
	var entity_type = data.get("entity_type", "default")
	var config = Config.get_entity_config(entity_type)

	# Apply juvenile scaling if applicable
	var size_multiplier = config.size_multiplier
	if data.get("is_juvenile", false) and Config.juvenile_scales.has(entity_type):
		size_multiplier *= Config.juvenile_scales[entity_type]

	var label_size = int(Config.TILE_SIZE * size_multiplier)

	label.text = config.emoji
	label.add_theme_font_size_override("font_size", label_size)

	# Set label color to white for visibility
	label.add_theme_color_override("font_color", Color(1, 1, 1, 1))
	label.add_theme_color_override("font_shadow_color", Color(0, 0, 0, 0.7))
	label.add_theme_constant_override("shadow_offset_x", 1)
	label.add_theme_constant_override("shadow_offset_y", 1)

	# Set label size explicitly so it renders
	label.custom_minimum_size = Vector2(label_size, label_size)
	label.size = Vector2(label_size, label_size)

	# Position entity (with -0.2 Y offset to keep feet in grid!)
	var pos = data.position
	var tile_pos = Vector2i(pos.x, pos.y)
	var pixel_pos = get_parent().map_to_local(tile_pos)

	# For isometric tiles, map_to_local() returns the top point of the diamond
	# We need to offset down by half the tile height to center vertically
	pixel_pos.y += 16  # Half of 32 (tile height)

	pixel_pos.y += Config.TILE_SIZE * config.offset_y  # Apply -0.2 offset

	# Center the emoji on the position by offsetting by half the label size
	label.position = Vector2(-label_size / 2.0, -label_size / 2.0)
	label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER
	label.vertical_alignment = VERTICAL_ALIGNMENT_CENTER

	container.add_child(label)
	container.position = pixel_pos
	container.z_index = 10  # Way above everything else for visibility

	# Make container visible for debugging
	container.modulate = Color(1, 1, 1, 1)
	container.visible = true

	# Y-sort uses Y position for depth (handled automatically by y_sort_enabled)

	# Add debug position marker if enabled
	if Config.debug_show_position_markers:
		_add_debug_marker(container)

	# Add action label if present and zoomed in enough
	if data.has("current_action") and Config.TILE_SIZE >= 8:
		_add_action_label(container, data.current_action)

	add_child(container)
	entities[entity_id] = container

	print("🐇 Spawned entity ", entity_id, " (", entity_type, ") at tile ", tile_pos, " → pixel ", pixel_pos, " (emoji: ", config.emoji, ", size: ", int(Config.TILE_SIZE * size_multiplier), "px)")

# Update existing entity position
func _update_entity_position(entity_id: int, data: Dictionary):
	var container = entities[entity_id]
	if not container:
		return

	# Update position (discrete jumps like simulation)
	var pos = data.position
	var tile_pos = Vector2i(pos.x, pos.y)
	var pixel_pos = get_parent().map_to_local(tile_pos)

	# For isometric tiles, map_to_local() returns the top point of the diamond
	pixel_pos.y += 16  # Half of 32 (tile height)

	var entity_type = data.get("entity_type", "default")
	var config = Config.get_entity_config(entity_type)
	pixel_pos.y += Config.TILE_SIZE * config.offset_y

	container.position = pixel_pos
	# Y-sort uses Y position for depth (handled automatically by y_sort_enabled)

	# Update action label if present
	_update_action_label(container, data.get("current_action", ""))

# Add action label to entity
func _add_action_label(container: Node2D, action: String):
	if action == "" or action == "Idle":
		return

	var action_label = Label.new()
	action_label.text = action
	action_label.add_theme_font_size_override("font_size", 
		max(8, int(Config.TILE_SIZE * 0.4)))
	action_label.horizontal_alignment = HORIZONTAL_ALIGNMENT_CENTER

	# Position above entity
	action_label.position = Vector2(0, -Config.TILE_SIZE * 0.8)

	# Dark background for readability
	var panel = Panel.new()
	panel.add_child(action_label)
	
	# Make panel semi-transparent
	var style_box = StyleBoxFlat.new()
	style_box.bg_color = Color(0, 0, 0, 0.7)
	style_box.corner_radius_top_left = 4
	style_box.corner_radius_top_right = 4
	style_box.corner_radius_bottom_left = 4
	style_box.corner_radius_bottom_right = 4
	panel.add_theme_stylebox_override("panel", style_box)
	
	container.add_child(panel)

# Update action label on existing entity
func _update_action_label(container: Node2D, action: String):
	# Remove existing action label if any
	for child in container.get_children():
		if child is Panel:
			child.queue_free()
	
	# Add new action label if needed
	if action != "" and action != "Idle":
		_add_action_label(container, action)

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

# Get entity count
func get_entity_count() -> int:
	return entities.size()

# Get entities by type
func get_entities_by_type(entity_type: String) -> Array:
	var result = []
	for entity_id in entities.keys():
		# Find the entity data from last update
		for entity_data in last_entity_data:
			if entity_data.id == entity_id and entity_data.get("entity_type", "") == entity_type:
				result.append(entity_data)
				break
	return result

# Clear all entities
func clear_all_entities():
	for entity_id in entities.keys():
		entities[entity_id].queue_free()
	entities.clear()
	print("🗑️ Cleared all entities")

# Debug information
func debug_print_status():
	print("=== EntityManager Status ===")
	print("Active entities: ", entities.size())
	print("Entity types:")
	var type_counts = {}
	for entity_data in last_entity_data:
		var entity_type = entity_data.get("entity_type", "unknown")
		type_counts[entity_type] = type_counts.get(entity_type, 0) + 1
	
	for entity_type in type_counts.keys():
		print("  ", entity_type, ": ", type_counts[entity_type])
	print("=== End Status ===")

# Get entity data for debugging
func get_entity_data() -> Array:
	return last_entity_data.duplicate()