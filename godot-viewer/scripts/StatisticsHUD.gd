extends Control
class_name StatisticsHUD

# UI Elements
@onready var stats_panel: Panel = $StatsPanel
@onready var stats_text: RichTextLabel = $StatsPanel/StatsText
@onready var toggle_button: Button = $ToggleButton

# Data sources
var world_data_cache: WorldDataCache
var chunk_manager: ChunkManager

# Update timing
var update_interval: float = 1.0  # Update every second
var time_since_update: float = 0.0

# Statistics tracking
var last_entity_count: int = 0
var last_chunk_count: int = 0
var last_resource_count: int = 0

func _ready() -> void:
	# Get references to global systems
	world_data_cache = WorldDataCache.get_instance()
	chunk_manager = ChunkManager.get_instance()
	
	# Setup UI
	stats_panel.position = Vector2(10, 10)
	toggle_button.position = Vector2(10, stats_panel.size.y + 15)
	
	# Connect toggle button
	toggle_button.pressed.connect(_on_toggle_pressed)
	
	# Initial update
	update_statistics()

func _process(delta: float) -> void:
	time_since_update += delta
	if time_since_update >= update_interval:
		update_statistics()
		time_since_update = 0.0

func update_statistics() -> void:
	var stats_lines = []

	# Get current counts for delta calculations
	var current_loaded_chunks = 0
	var current_entity_count = 0

	# World Information
	stats_lines.append("[b]World Information[/b]")
	if world_data_cache and world_data_cache.world_info:
		var world_info = world_data_cache.world_info
		stats_lines.append("Name: " + str(world_info.get("name", "Unknown")))
		stats_lines.append("Seed: " + str(world_info.get("seed", "Unknown")))
		stats_lines.append("Center: " + str(world_info.get("center_chunk", "Unknown")))
		stats_lines.append("Radius: " + str(world_info.get("radius", "Unknown")) + " chunks")
	else:
		stats_lines.append("World: Not loaded")

	stats_lines.append("")

	# Chunk Statistics
	stats_lines.append("[b]Chunk Statistics[/b]")
	if chunk_manager:
		current_loaded_chunks = chunk_manager.get_loaded_chunk_count()
		var total_chunks = chunk_manager.get_total_chunk_count()
		stats_lines.append("Loaded: " + str(current_loaded_chunks) + " / " + str(total_chunks))

		# Calculate loading percentage
		var loading_percentage = 0.0
		if total_chunks > 0:
			loading_percentage = (float(current_loaded_chunks) / float(total_chunks)) * 100.0
		stats_lines.append("Progress: " + str("%.1f" % loading_percentage) + "%")
	else:
		stats_lines.append("Chunk Manager: Not available")

	stats_lines.append("")

	# Entity Statistics
	stats_lines.append("[b]Entity Statistics[/b]")
	current_entity_count = count_entities()
	stats_lines.append("Total Entities: " + str(current_entity_count))
	
	# Count by species
	var species_counts = count_entities_by_species()
	for species in species_counts:
		stats_lines.append("  " + species + ": " + str(species_counts[species]))
	
	stats_lines.append("")
	
	# Resource Statistics
	stats_lines.append("[b]Resource Statistics[/b]")
	var resource_stats = count_resources()
	stats_lines.append("Total Resources: " + str(resource_stats.total))
	
	for resource_type in resource_stats.types:
		stats_lines.append("  " + resource_type + ": " + str(resource_stats.types[resource_type]))
	
	last_resource_count = resource_stats.total
	
	stats_lines.append("")
	
	# Performance Statistics
	stats_lines.append("[b]Performance[/b]")
	stats_lines.append("FPS: " + str(Engine.get_frames_per_second()))
	stats_lines.append("Memory: " + get_formatted_memory())

	# Changes since last update
	if last_entity_count > 0:
		var entity_delta = current_entity_count - last_entity_count
		if entity_delta != 0:
			stats_lines.append("Entity Δ: " + str(entity_delta))
	if last_chunk_count > 0:
		var chunk_delta = current_loaded_chunks - last_chunk_count
		if chunk_delta != 0:
			stats_lines.append("Chunk Δ: " + str(chunk_delta))

	# Update last values for next delta calculation
	last_entity_count = current_entity_count
	last_chunk_count = current_loaded_chunks

	# Update display
	stats_text.text = "\n".join(stats_lines)

func count_entities() -> int:
	var count = 0
	if world_data_cache and world_data_cache.entities:
		count = world_data_cache.entities.size()
	return count

func count_entities_by_species() -> Dictionary:
	var species_counts = {}
	
	if world_data_cache and world_data_cache.entities:
		for entity in world_data_cache.entities.values():
			var species = entity.get("species", "Unknown")
			if species in species_counts:
				species_counts[species] += 1
			else:
				species_counts[species] = 1
	
	return species_counts

func count_resources() -> Dictionary:
	var resource_stats = {
		"total": 0,
		"types": {}
	}
	
	if chunk_manager:
		var loaded_chunks = chunk_manager.get_loaded_chunks()
		for chunk_data in loaded_chunks.values():
			if chunk_data and chunk_data.has("resources"):
				var resources = chunk_data["resources"]
				for resource in resources:
					var resource_type = resource.get("type", "Unknown")
					resource_stats.total += 1
					
					if resource_type in resource_stats.types:
						resource_stats.types[resource_type] += 1
					else:
						resource_stats.types[resource_type] = 1
	
	return resource_stats

func get_formatted_memory() -> String:
	# Get static memory usage (Godot 4.x)
	var memory_bytes = OS.get_static_memory_usage()

	# Convert to MB
	var memory_mb = float(memory_bytes) / (1024.0 * 1024.0)
	return str("%.1f" % memory_mb) + " MB"

func _on_toggle_pressed() -> void:
	stats_panel.visible = !stats_panel.visible
	if stats_panel.visible:
		toggle_button.text = "Hide Stats (Tab)"
	else:
		toggle_button.text = "Show Stats (Tab)"

func _input(event: InputEvent) -> void:
	# Toggle with Tab key
	if event is InputEventKey and event.is_pressed():
		if event.keycode == KEY_TAB:
			_on_toggle_pressed()

# Public methods for external control
func set_update_interval(interval: float) -> void:
	update_interval = max(0.1, interval)  # Minimum 0.1 seconds

func force_update() -> void:
	update_statistics()
	time_since_update = 0.0