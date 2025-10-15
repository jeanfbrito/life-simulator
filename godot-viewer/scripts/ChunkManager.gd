# ChunkManager.gd - Manages chunk loading, caching, and HTTP requests
# Mirrors JavaScript ChunkManager logic from web-viewer

extends Node

signal chunks_loaded(chunk_data)
signal world_info_loaded(world_info)
signal entities_loaded(entities)
signal connection_status_changed(connected)

# Chunk tracking
var loaded_chunks: Dictionary = {}  # Track which chunks we've loaded
var loading_chunks: Dictionary = {}  # Track chunks currently being loaded
var chunk_load_timer: Timer = null  # Debounce timer for chunk loading
var last_loaded_center: Vector2i = Vector2i(0, 0)  # Track last loaded center

# HTTP requests management
var active_requests: Dictionary = {}  # Track active HTTP requests
var connection_status: bool = false

# Called when the node enters the scene tree for the first time.
func _ready():
	print("ChunkManager initialized")

	# Set up debounce timer
	chunk_load_timer = Timer.new()
	chunk_load_timer.wait_time = Config.chunk_load_debounce / 1000.0  # Convert ms to seconds
	chunk_load_timer.one_shot = true
	add_child(chunk_load_timer)
	chunk_load_timer.timeout.connect(_on_chunk_load_timeout)

# Request initial chunks around the center
func request_chunks(center_coord: Vector2i):
	print("Requesting chunks around center: ", center_coord)
	var radius = Config.initial_chunk_radius
	return await request_chunks_in_area(center_coord.x, center_coord.y, radius)

# Request chunks in a specific area
func request_chunks_in_area(center_x: int, center_y: int, radius: int) -> Dictionary:
	var needed_chunks: Array[String] = []
	var all_loaded_data: Dictionary = {
		"chunks": {},
		"resources": {},
		"heights": {},
		"slope_indices": {}
	}

	# Calculate which chunks we need
	for dx in range(-radius, radius + 1):
		for dy in range(-radius, radius + 1):
			var chunk_x = center_x + dx
			var chunk_y = center_y + dy
			var chunk_key = "%d,%d" % [chunk_x, chunk_y]

			if not loaded_chunks.has(chunk_key) and not loading_chunks.has(chunk_key):
				needed_chunks.append(chunk_key)

	if needed_chunks.size() == 0:
		print("No new chunks to load")
		return all_loaded_data

	# Mark chunks as being loaded
	for chunk_key in needed_chunks:
		loading_chunks[chunk_key] = true

	# Split requests into smaller batches to avoid URL length issues
	print("Loading chunk batches, total needed: ", needed_chunks.size())

	for i in range(0, needed_chunks.size(), Config.chunk_batch_size):
		var batch_end = min(i + Config.chunk_batch_size, needed_chunks.size())
		var batch = needed_chunks.slice(i, batch_end)
		print("Loading batch: ", batch)

		var batch_data = await load_chunk_batch(batch)
		if batch_data:
			# Merge batch data with accumulated data
			for key in batch_data.chunks:
				all_loaded_data.chunks[key] = batch_data.chunks[key]
			for key in batch_data.resources:
				all_loaded_data.resources[key] = batch_data.resources[key]
			for key in batch_data.heights:
				all_loaded_data.heights[key] = batch_data.heights[key]
			if "slope_indices" in batch_data:
				for key in batch_data.slope_indices:
					all_loaded_data.slope_indices[key] = batch_data.slope_indices[key]

	return all_loaded_data

# Load chunks individually (workaround for backend batch issue)
func load_chunk_batch(batch: Array[String]) -> Dictionary:
	print("📦 CHUNK_MANAGER: Loading ", batch.size(), " chunks individually")

	var new_world_data: Dictionary = {
		"chunks": {},
		"resources": {},
		"heights": {},
		"slope_indices": {}
	}

	for chunk_key in batch:
		print("📦 Loading individual chunk: ", chunk_key)
		# NOTE: Don't URL-encode coords because backend has a bug with URL-encoded commas
		var endpoint = "/api/chunks?coords=" + chunk_key + "&layers=true"
		var full_url = Config.api_base_url + endpoint

		# Add small delay to avoid backend concurrency issues
		await get_tree().create_timer(0.1).timeout

		var http_request = HTTPRequest.new()
		add_child(http_request)

		var error = http_request.request(full_url)

		if error != OK:
			print("❌ Failed to start request for chunk ", chunk_key)
			loading_chunks.erase(chunk_key)
			http_request.queue_free()
			continue

		# Wait for request to complete
		var result = await http_request.request_completed
		http_request.queue_free()

		if result[0] != HTTPRequest.RESULT_SUCCESS:
			print("❌ HTTP request failed for chunk ", chunk_key, ": ", result[0])
			loading_chunks.erase(chunk_key)
			continue

		var response_code = result[1]
		if response_code != 200:
			print("❌ Bad response code for chunk ", chunk_key, ": ", response_code)
			loading_chunks.erase(chunk_key)
			continue

		var body = result[3]
		var json = JSON.new()
		var parse_result = json.parse(body.get_string_from_utf8())

		if parse_result != OK:
			print("❌ Failed to parse JSON for chunk ", chunk_key)
			loading_chunks.erase(chunk_key)
			continue

		var data = json.data

		# Debug only for one specific chunk to see what's happening
		if chunk_key == "1,1":
			print("🔍 DEBUG 1,1: Response = ", data)

		if data.has("chunk_data") and data.chunk_data.has(chunk_key):
			var chunk_data = data.chunk_data[chunk_key]

			# Store the chunk data
			if chunk_data.has("terrain"):
				new_world_data.chunks[chunk_key] = chunk_data.terrain
			if chunk_data.has("resources"):
				new_world_data.resources[chunk_key] = chunk_data.resources
			if chunk_data.has("heights"):
				var heights_int = []
				for row in chunk_data.heights:
					var row_int = []
					for height_str in row:
						row_int.append(int(height_str))
					heights_int.append(row_int)
				new_world_data.heights[chunk_key] = heights_int

			if chunk_data.has("slope_indices"):
				var slopes_int = []
				for row in chunk_data.slope_indices:
					var row_int = []
					for slope_str in row:
						row_int.append(int(slope_str))
					slopes_int.append(row_int)
				new_world_data.slope_indices[chunk_key] = slopes_int

			# Mark as loaded
			loaded_chunks[chunk_key] = true
			loading_chunks.erase(chunk_key)

			print("✅ Loaded chunk: ", chunk_key)
		else:
			print("❌ No data for chunk ", chunk_key)

	print("📦 CHUNK_MANAGER: Total loaded chunks: ", new_world_data.chunks.keys())
	update_connection_status(true)
	return new_world_data

# Load chunks around the visible area (debounced)
func load_visible_chunks_debounced(drag_offset: Vector2, world_data: Dictionary, on_chunks_loaded: Callable):
	# Clear existing timeout
	if chunk_load_timer.time_left > 0:
		chunk_load_timer.stop()

	# Store callback for when timer fires
	_load_visible_chunks_callback = on_chunks_loaded

	# Set new timeout
	chunk_load_timer.start()

# Callback storage for debounced loading
var _load_visible_chunks_callback: Callable

# Timer timeout handler
func _on_chunk_load_timeout():
	if _load_visible_chunks_callback.is_valid():
		var loaded = await load_visible_chunks(_last_drag_offset, _last_world_data)
		# Trigger callback if chunks were loaded
		if loaded:
			_load_visible_chunks_callback.call()
		_load_visible_chunks_callback = Callable()

# Store parameters for debounced loading
var _last_drag_offset: Vector2 = Vector2.ZERO
var _last_world_data: Dictionary = {}

# Load chunks around the visible area
func load_visible_chunks(drag_offset: Vector2, world_data: Dictionary) -> bool:
	_last_drag_offset = drag_offset
	_last_world_data = world_data

	# Calculate the center of the current view in world coordinates
	var view_center_world_x = int(-drag_offset.x / Config.TILE_SIZE) + int(Config.VIEW_SIZE_X / 2)
	var view_center_world_y = int(-drag_offset.y / Config.TILE_SIZE) + int(Config.VIEW_SIZE_Y / 2)

	var center_chunk_x = int(floor(float(view_center_world_x) / 16.0))
	var center_chunk_y = int(floor(float(view_center_world_y) / 16.0))

	# Only load if we've moved significantly from last loaded center
	var distance_x = abs(center_chunk_x - last_loaded_center.x)
	var distance_y = abs(center_chunk_y - last_loaded_center.y)

	if distance_x < 1 and distance_y < 1:
		return false  # Not enough movement to trigger new loading

	last_loaded_center = Vector2i(center_chunk_x, center_chunk_y)

	# Calculate the actual visible area in world coordinates
	var view_start_world_x = int(-drag_offset.x / Config.TILE_SIZE)
	var view_start_world_y = int(-drag_offset.y / Config.TILE_SIZE)
	var view_end_world_x = view_start_world_x + Config.VIEW_SIZE_X
	var view_end_world_y = view_start_world_y + Config.VIEW_SIZE_Y

	# Convert to chunk coordinates and add buffer
	var start_chunk_x = int(floor(float(view_start_world_x) / 16.0)) - 1  # Add 1 chunk buffer
	var start_chunk_y = int(floor(float(view_start_world_y) / 16.0)) - 1
	var end_chunk_x = int(floor(float(view_end_world_x) / 16.0)) + 1  # Add 1 chunk buffer
	var end_chunk_y = int(floor(float(view_end_world_y) / 16.0)) + 1

	# Calculate radius from the bounds
	var radius_x = abs(center_chunk_x - start_chunk_x)
	var radius_y = abs(center_chunk_y - start_chunk_y)
	var visible_radius = max(max(radius_x, radius_y), 3)  # Minimum radius of 3

	print("📦 Loading chunks around (%d, %d) with radius %d" % [center_chunk_x, center_chunk_y, visible_radius])
	var new_data = await request_chunks_in_area(center_chunk_x, center_chunk_y, visible_radius)

	# Merge newly loaded chunks into worldData if provided
	if new_data.chunks.size() > 0:
		print("✅ Loaded %d new chunks, merging into worldData" % new_data.chunks.size())
		merge_chunk_data(new_data, world_data)
		chunks_loaded.emit(new_data)
		return true  # Return true to indicate chunks were loaded

	return false

# Load world info from API
func load_world_info() -> bool:
	print("Loading world info...")

	# Load current world info
	var current_data = await fetch_data("/api/world/current")
	if current_data:
		print("✅ Current world loaded: ", current_data.get("name", "Unknown"))
		world_info_loaded.emit(current_data)

	# Load additional world info
	var world_data = await fetch_data("/api/world_info")
	if world_data:
		print("✅ World metadata loaded: ", world_data.get("chunk_count", 0), " chunks")
		world_info_loaded.emit(world_data)
		return true

	print("❌ Failed to load world info")
	return false

# Generic data fetching method
func fetch_data(endpoint: String) -> Dictionary:
	print("Fetching: ", Config.api_base_url + endpoint)

	var http_request = HTTPRequest.new()
	add_child(http_request)

	var error = http_request.request(Config.api_base_url + endpoint)
	if error != OK:
		print("❌ Failed to start HTTP request: ", error)
		update_connection_status(false)
		return {}

	# Wait for request to complete
	var result = await http_request.request_completed
	http_request.queue_free()

	if result[0] != HTTPRequest.RESULT_SUCCESS:
		print("❌ HTTP request failed: ", result[0])
		update_connection_status(false)
		return {}

	var response_code = result[1]
	if response_code != 200:
		print("❌ Bad response code: ", response_code)
		update_connection_status(false)
		return {}

	var body = result[3]
	var json = JSON.new()
	var parse_result = json.parse(body.get_string_from_utf8())

	if parse_result != OK:
		print("❌ Failed to parse JSON response")
		update_connection_status(false)
		return {}

	var data = json.data
	print("✅ Successfully received data from: ", endpoint)
	update_connection_status(true)
	return data

# Merge new chunk data into existing world data
func merge_chunk_data(new_data: Dictionary, existing_world_data: Dictionary):
	if not existing_world_data.has("chunks"):
		existing_world_data["chunks"] = {}
	if not existing_world_data.has("resources"):
		existing_world_data["resources"] = {}
	if not existing_world_data.has("heights"):
		existing_world_data["heights"] = {}
	if not existing_world_data.has("slope_indices"):
		existing_world_data["slope_indices"] = {}

	if new_data.has("chunks"):
		for key in new_data.chunks:
			existing_world_data.chunks[key] = new_data.chunks[key]

	if new_data.has("resources"):
		for key in new_data.resources:
			existing_world_data.resources[key] = new_data.resources[key]

	if new_data.has("heights"):
		for key in new_data.heights:
			existing_world_data.heights[key] = new_data.heights[key]

	if new_data.has("slope_indices"):
		for key in new_data.slope_indices:
			existing_world_data.slope_indices[key] = new_data.slope_indices[key]

# Update connection status
func update_connection_status(connected: bool):
	if connection_status != connected:
		connection_status = connected
		print("Connection status changed: ", "Connected" if connected else "Disconnected")
		connection_status_changed.emit(connected)

# Get chunk count for UI
func get_chunk_count() -> String:
	return "%d loaded" % loaded_chunks.size()

# Clear all loaded chunks
func clear():
	print("Clearing all loaded chunks")
	loaded_chunks.clear()
	loading_chunks.clear()
	if chunk_load_timer.time_left > 0:
		chunk_load_timer.stop()
	last_loaded_center = Vector2i(0, 0)

	# Cancel any active requests
	for request_id in active_requests:
		var request_data = active_requests[request_id]
		if request_data and request_data.http_request:
			request_data.http_request.cancel_request()
			request_data.http_request.queue_free()
	active_requests.clear()

# Check if a chunk is loaded
func is_chunk_loaded(chunk_key: String) -> bool:
	return loaded_chunks.has(chunk_key)

# Get loaded chunk keys
func get_loaded_chunk_keys() -> Array[String]:
	return loaded_chunks.keys()

# Get number of loaded chunks (for TopBar/UI)
func get_loaded_chunk_count() -> int:
	return loaded_chunks.size()

# Get total expected chunks based on world radius (for TopBar/UI)
func get_total_chunk_count() -> int:
	# Assume radius of 5 chunks = 11x11 grid = 121 chunks
	# This will match the world being loaded
	return 121
