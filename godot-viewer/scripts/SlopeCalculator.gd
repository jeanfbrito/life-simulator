extends RefCounted
class_name SlopeCalculator

## SlopeCalculator - Utilities for working with OpenRCT2 slope indices (0-18)

const CORNER_N := 0b0001
const CORNER_E := 0b0010
const CORNER_S := 0b0100
const CORNER_W := 0b1000

const INDEX_TO_MASK := [
	0b0000,  # 0  flat
	0b0001,  # 1  north corner up
	0b0010,  # 2  east corner up
	0b0011,  # 3  north-east side up
	0b0100,  # 4  south corner up
	0b0101,  # 5  north-south valley
	0b0110,  # 6  south-east side up
	0b0111,  # 7  three corners up (west down)
	0b1000,  # 8  west corner up
	0b1001,  # 9  north-west side up
	0b1010,  # 10 east-west valley
	0b1011,  # 11 three corners up (south down)
	0b1100,  # 12 south-west side up
	0b1101,  # 13 three corners up (east down)
	0b1110,  # 14 three corners up (north down)
	0b1111   # 15 all corners up
]

const SLOPE_NAMES := [
	"Flat",
	"North corner up",
	"East corner up",
	"North-east side up",
	"South corner up",
	"North-south valley",
	"South-east side up",
	"Three corners up (W down)",
	"West corner up",
	"North-west side up",
	"East-west valley",
	"Three corners up (S down)",
	"South-west side up",
	"Three corners up (E down)",
	"Three corners up (N down)",
	"All corners up",
	"Diagonal NE-SW",
	"Diagonal NW-SE",
	"Peak"
]

static func rotate_slope_index(index: int, rotation: int) -> int:
	if index < 0:
		return 0

	rotation = int(rotation) % 4
	if rotation == 0:
		return clamp_index(index)

	if index == 18:
		return 18  # Peaks are rotation invariant
	if index == 16:
		return 16 if rotation % 2 == 0 else 17
	if index == 17:
		return 17 if rotation % 2 == 0 else 16

	if index >= 0 and index < INDEX_TO_MASK.size():
		var mask = INDEX_TO_MASK[index]
		var rotated_mask = mask
		for i in rotation:
			rotated_mask = _rotate_mask_clockwise(rotated_mask)
		return mask_to_index(rotated_mask)

	return clamp_index(index)

static func mask_to_index(mask: int) -> int:
	match mask:
		0b0000: return 0
		0b0001: return 1
		0b0010: return 2
		0b0011: return 3
		0b0100: return 4
		0b0101: return 5
		0b0110: return 6
		0b0111: return 7
		0b1000: return 8
		0b1001: return 9
		0b1010: return 10
		0b1011: return 11
		0b1100: return 12
		0b1101: return 13
		0b1110: return 14
		0b1111: return 15
		_:
			return 0

static func clamp_index(index: int) -> int:
	return clampi(index, 0, SLOPE_NAMES.size() - 1)

static func get_slope_name(index: int) -> String:
	var clamped = clamp_index(index)
	return SLOPE_NAMES[clamped]

static func _rotate_mask_clockwise(mask: int) -> int:
	var rotated := 0
	if mask & CORNER_N:
		rotated |= CORNER_E
	if mask & CORNER_E:
		rotated |= CORNER_S
	if mask & CORNER_S:
		rotated |= CORNER_W
	if mask & CORNER_W:
		rotated |= CORNER_N
	return rotated

## Get relative corner heights for a slope (0-2 units per corner)
## Mirrors OpenRCT2's kSlopeRelativeCornerHeights table from Slope.cpp
## Returns Dictionary with keys: top (north), right (east), bottom (south), left (west)
static func get_relative_corner_heights(slope_index: int) -> Dictionary:
	# OpenRCT2 table: each entry is {top, right, bottom, left} = {north, east, south, west}
	# Indexed by slope & kTileSlopeMask (0-31)
	const RELATIVE_CORNER_HEIGHTS = [
		[0, 0, 0, 0],  # 0  flat
		[0, 0, 1, 0],  # 1  north corner up
		[0, 0, 0, 1],  # 2  east corner up
		[0, 0, 1, 1],  # 3  north-east side up
		[1, 0, 0, 0],  # 4  south corner up
		[1, 0, 1, 0],  # 5  north-south valley
		[1, 0, 0, 1],  # 6  south-east side up
		[1, 0, 1, 1],  # 7  three corners up (west down)
		[0, 1, 0, 0],  # 8  west corner up
		[0, 1, 1, 0],  # 9  north-west side up
		[0, 1, 0, 1],  # 10 east-west valley
		[0, 1, 1, 1],  # 11 three corners up (south down)
		[1, 1, 0, 0],  # 12 south-west side up
		[1, 1, 1, 0],  # 13 three corners up (east down)
		[1, 1, 0, 1],  # 14 three corners up (north down)
		[1, 1, 1, 1],  # 15 all corners up
		[0, 0, 0, 0],  # 16 diagonal NE-SW (base)
		[0, 0, 1, 0],  # 17 diagonal NE-SW variant
		[0, 0, 0, 1],  # 18 diagonal NW-SE
		[0, 0, 1, 1],  # 19 diagonal NW-SE variant
		[1, 0, 0, 0],  # 20
		[1, 0, 1, 0],  # 21
		[1, 0, 0, 1],  # 22
		[1, 0, 1, 2],  # 23 steep diagonal
		[0, 1, 0, 0],  # 24
		[0, 1, 1, 0],  # 25
		[0, 1, 0, 1],  # 26
		[0, 1, 2, 1],  # 27 steep diagonal
		[1, 1, 0, 0],  # 28
		[1, 2, 1, 0],  # 29 steep diagonal
		[2, 1, 0, 1],  # 30 steep diagonal
		[1, 1, 1, 1],  # 31 all corners up (alternate)
	]
	
	var clamped = clampi(slope_index, 0, RELATIVE_CORNER_HEIGHTS.size() - 1)
	var heights = RELATIVE_CORNER_HEIGHTS[clamped]
	
	return {
		"top": heights[0],      # north
		"right": heights[1],    # east
		"bottom": heights[2],   # south
		"left": heights[3]      # west
	}

## Get absolute corner heights in tiny-Z units
## base_height: tile base height (0-255 tiny-Z)
## slope_index: slope index (0-18)
## Returns Dictionary with keys: top, right, bottom, left (absolute tiny-Z values)
static func get_corner_heights(base_height: int, slope_index: int) -> Dictionary:
	const LAND_HEIGHT_STEP = 16  # kCoordsZPerTinyZ - height per slope step
	
	var rel = get_relative_corner_heights(slope_index)
	
	return {
		"top": base_height + rel.top * LAND_HEIGHT_STEP,
		"right": base_height + rel.right * LAND_HEIGHT_STEP,
		"bottom": base_height + rel.bottom * LAND_HEIGHT_STEP,
		"left": base_height + rel.left * LAND_HEIGHT_STEP
	}
