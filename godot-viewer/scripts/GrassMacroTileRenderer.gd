extends Node
class_name GrassMacroTileRenderer

## Replicates stone-kingdoms' macro tile system for grass rendering
##
## The system works by:
## 1. Check if a 2x2, 3x3, or 4x4 area has the same terrain type
## 2. If so, use a larger "macro tile" texture covering multiple tiles
## 3. Mark covered tiles as "skip" so they don't render again
## 4. Use weighted random to favor larger macro tiles for performance

# Tile variants available (matching stone-kingdoms)
const GRASS_1X1_VARIANTS = 8  # abundant_grass_1x1 (1) through (8)
const GRASS_2X2_VARIANTS = 4  # abundant_grass_2x2 (1) through (4)
const GRASS_3X3_VARIANTS = 4  # abundant_grass_3x3 (1) through (4)
const GRASS_4X4_VARIANTS = 4  # abundant_grass_4x4 (1) through (4)

# Preload grass textures (cached)
var grass_1x1_textures: Array[Texture2D] = []
var grass_2x2_textures: Array[Texture2D] = []
var grass_3x3_textures: Array[Texture2D] = []
var grass_4x4_textures: Array[Texture2D] = []

# Track which tiles have been covered by macro tiles
var tiles_to_skip: Dictionary = {}  # Format: {chunk_key: {tile_pos: true, ...}}

func _ready():
	_load_grass_textures()

func _load_grass_textures():
	"""Load all grass texture variants into memory."""
	print("🌿 Loading grass textures...")

	# Load 1x1 tiles
	for i in range(1, GRASS_1X1_VARIANTS + 1):
		var path = "res://assets/tiles/grass/abundant_grass_1x1_%02d.png" % i
		var texture = load(path)
		if texture:
			grass_1x1_textures.append(texture)
		else:
			push_warning("Failed to load: " + path)

	# Load 2x2 tiles
	for i in range(1, GRASS_2X2_VARIANTS + 1):
		var path = "res://assets/tiles/grass/abundant_grass_2x2_%02d.png" % i
		var texture = load(path)
		if texture:
			grass_2x2_textures.append(texture)
		else:
			push_warning("Failed to load: " + path)

	# Load 3x3 tiles
	for i in range(1, GRASS_3X3_VARIANTS + 1):
		var path = "res://assets/tiles/grass/abundant_grass_3x3_%02d.png" % i
		var texture = load(path)
		if texture:
			grass_3x3_textures.append(texture)
		else:
			push_warning("Failed to load: " + path)

	# Load 4x4 tiles
	for i in range(1, GRASS_4X4_VARIANTS + 1):
		var path = "res://assets/tiles/grass/abundant_grass_4x4_%02d.png" % i
		var texture = load(path)
		if texture:
			grass_4x4_textures.append(texture)
		else:
			push_warning("Failed to load: " + path)

	print("✅ Grass textures loaded: 1x1=%d, 2x2=%d, 3x3=%d, 4x4=%d" % [
		grass_1x1_textures.size(),
		grass_2x2_textures.size(),
		grass_3x3_textures.size(),
		grass_4x4_textures.size()
	])

func check_max_size_for_terrain(
	chunk_key: String,
	local_x: int,
	local_y: int,
	terrain_type: String,
	chunk_data: Dictionary
) -> int:
	"""
	Check the maximum macro tile size that can fit at this position.
	Returns: 1, 2, 3, or 4 (representing 1x1, 2x2, 3x3, or 4x4)

	This replicates stone-kingdoms' checkMaxSizeBiome() function.
	"""

	# Check if already marked as skip
	if _is_tile_skipped(chunk_key, Vector2i(local_x, local_y)):
		return 1

	# Get the terrain data for this chunk
	var terrain = chunk_data.get("terrain", [])
	if terrain.is_empty():
		return 1

	# Check if we have room for at least 2x2
	if local_x + 1 >= 16 or local_y + 1 >= 16:
		return 1

	# Check 2x2 area (4 tiles)
	if not _check_square(terrain, local_x, local_y, 2, terrain_type):
		return 1

	# Check if we have room for 3x3
	if local_x + 2 >= 16 or local_y + 2 >= 16:
		return 2

	# Check 3x3 area (9 tiles)
	if not _check_square(terrain, local_x, local_y, 3, terrain_type):
		return 2

	# Check if we have room for 4x4
	if local_x + 3 >= 16 or local_y + 3 >= 16:
		return 3

	# Check 4x4 area (16 tiles)
	if not _check_square(terrain, local_x, local_y, 4, terrain_type):
		return 3

	return 4  # Can fit 4x4!

func _check_square(terrain: Array, x: int, y: int, size: int, terrain_type: String) -> bool:
	"""Check if a square area has matching terrain type."""
	for dx in range(size):
		for dy in range(size):
			var check_x = x + dx
			var check_y = y + dy

			if check_y >= terrain.size():
				return false
			if check_x >= terrain[check_y].size():
				return false

			if terrain[check_y][check_x] != terrain_type:
				return false

	return true

func select_grass_tile(
	chunk_key: String,
	local_pos: Vector2i,
	terrain_type: String,
	chunk_data: Dictionary
) -> Dictionary:
	"""
	Select the appropriate grass tile variant and size for this position.
	Returns: {
		"texture": Texture2D,
		"size": int (1-4),
		"variant": int,
		"skip": bool (true if already rendered as part of macro tile)
	}

	This replicates stone-kingdoms' multiTileCalculate() function.
	"""

	# Check if already covered by a macro tile
	if _is_tile_skipped(chunk_key, local_pos):
		return {"skip": true}

	# Check maximum macro tile size that fits
	var max_size = check_max_size_for_terrain(
		chunk_key,
		local_pos.x,
		local_pos.y,
		terrain_type,
		chunk_data
	)

	# Determine upper bound for random selection based on max_size
	# This matches stone-kingdoms' rand system:
	# - 1x1 tiles: rand 1-16
	# - 2x2 tiles: rand 17-20
	# - 3x3 tiles: rand 21-24
	# - 4x4 tiles: rand 25-28
	var upper_border = 16 + (max_size - 1) * 4

	# Weighted random (take max of 3 rolls) - favors larger macro tiles
	var rand = _weighted_random(1, upper_border)

	var result = {
		"skip": false,
		"size": 1,
		"variant": 0,
		"texture": null
	}

	# Select tile based on rand value
	if rand >= 1 and rand <= 16:
		# Use 1x1 tile
		result["size"] = 1
		result["variant"] = rand - 1  # 0-indexed
		result["texture"] = grass_1x1_textures[result["variant"] % grass_1x1_textures.size()]

	elif rand >= 17 and rand <= 20:
		# Use 2x2 macro tile
		result["size"] = 2
		result["variant"] = (20 - rand)  # Maps 17-20 to 3-0
		result["texture"] = grass_2x2_textures[result["variant"] % grass_2x2_textures.size()]
		_mark_macro_tile_skip(chunk_key, local_pos, 2)

	elif rand >= 21 and rand <= 24:
		# Use 3x3 macro tile
		result["size"] = 3
		result["variant"] = (24 - rand)  # Maps 21-24 to 3-0
		result["texture"] = grass_3x3_textures[result["variant"] % grass_3x3_textures.size()]
		_mark_macro_tile_skip(chunk_key, local_pos, 3)

	else:  # rand >= 25
		# Use 4x4 macro tile
		result["size"] = 4
		result["variant"] = (28 - rand) if rand <= 28 else 0  # Maps 25-28 to 3-0
		result["texture"] = grass_4x4_textures[result["variant"] % grass_4x4_textures.size()]
		_mark_macro_tile_skip(chunk_key, local_pos, 4)

	return result

func _weighted_random(min_val: int, max_val: int) -> int:
	"""
	Weighted random that favors higher values.
	Takes the max of 3 random rolls (replicates stone-kingdoms behavior).
	"""
	var rand1 = randi_range(min_val, max_val)
	var rand2 = randi_range(min_val, max_val)
	var rand3 = randi_range(min_val, max_val)
	return max(rand1, max(rand2, rand3))

func _mark_macro_tile_skip(chunk_key: String, origin: Vector2i, size: int):
	"""Mark all tiles covered by a macro tile as 'skip'."""
	if not tiles_to_skip.has(chunk_key):
		tiles_to_skip[chunk_key] = {}

	for dx in range(size):
		for dy in range(size):
			var pos = Vector2i(origin.x + dx, origin.y + dy)
			tiles_to_skip[chunk_key][pos] = true

func _is_tile_skipped(chunk_key: String, pos: Vector2i) -> bool:
	"""Check if a tile should be skipped (already rendered as part of macro tile)."""
	if not tiles_to_skip.has(chunk_key):
		return false
	return tiles_to_skip[chunk_key].get(pos, false)

func clear_skip_data(chunk_key: String):
	"""Clear skip data when a chunk is unloaded."""
	if tiles_to_skip.has(chunk_key):
		tiles_to_skip.erase(chunk_key)

func get_tile_scale_for_size(macro_size: int) -> Vector2:
	"""
	Calculate the scale factor to make macro tiles render correctly.

	Stone-kingdoms tiles:
	- 1x1: 30×18 pixels
	- 2x2: 62×35 pixels (should be 64×32 for exact 2×, but artwork is artistic)
	- 3x3: 94×49 pixels (should be 96×48)
	- 4x4: 126×65 pixels (should be 128×64)

	Your Godot tiles: 128×64 pixels (isometric diamond)

	Scale factors to fit your tile size:
	- 1x1: scale ~4.27× width, ~3.56× height
	- 2x2: scale ~2.06× (covers 2×2 tiles)
	- 3x3: scale ~1.36× (covers 3×3 tiles)
	- 4x4: scale ~1.02× (covers 4×4 tiles, nearly exact!)
	"""

	# Target size is your tile size × macro size
	var target_width = 128.0 * macro_size
	var target_height = 64.0 * macro_size

	# Source sizes from stone-kingdoms
	var source_sizes = {
		1: Vector2(30, 18),
		2: Vector2(62, 35),
		3: Vector2(94, 49),
		4: Vector2(126, 65)
	}

	var source = source_sizes.get(macro_size, Vector2(30, 18))

	return Vector2(
		target_width / source.x,
		target_height / source.y
	)
