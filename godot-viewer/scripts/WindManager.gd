# WindManager.gd - Global wind simulation for realistic tree animation
# All trees in the world respond to the same wind, creating unified movement

extends Node

# Wind oscillation timer
var wind_time: float = 0.0

# Wind configuration
const WIND_CYCLE_DURATION = 10.0  # Seconds for full sway cycle (slow, natural)
const WIND_WAVE_SPEED = 150.0     # Pixels per second wind travels (creates wave effect)
const WIND_STRENGTH_MIN = 0.3     # Minimum wind (calm)
const WIND_STRENGTH_MAX = 0.7     # Maximum wind (moderate breeze)

# Current wind strength (0.0 = calm, 1.0 = storm)
var wind_strength: float = 0.5

func _ready():
	print("🌬️  WindManager initialized")

func _process(delta: float):
	# Advance wind time
	wind_time += delta

	# Vary wind strength smoothly over time (sine wave for natural variation)
	# This creates periods of calm and stronger breeze
	var strength_cycle = sin(wind_time * 0.15) * 0.5 + 0.5  # 0.0 to 1.0
	wind_strength = lerp(WIND_STRENGTH_MIN, WIND_STRENGTH_MAX, strength_cycle)

func get_wind_frame_for_position(world_pos: Vector2) -> int:
	"""Calculate which animation frame (0-9) a tree should display based on global wind.

	Args:
		world_pos: Tree position in world pixel coordinates

	Returns:
		Frame index (0-9) for tree animation
	"""

	# Calculate base wind phase (0 to 2π, full cycle)
	var base_phase = (wind_time / WIND_CYCLE_DURATION) * TAU

	# Add position-based offset for wave propagation effect
	# Trees further right/down experience wind slightly later
	var distance_offset = (world_pos.x + world_pos.y) / WIND_WAVE_SPEED
	var wind_phase = base_phase + distance_offset

	# Convert sine wave (-1 to +1) to normalized value (0 to 1)
	var wave_value = sin(wind_phase)
	var normalized = (wave_value + 1.0) / 2.0

	# Scale by current wind strength
	# Low strength: trees barely move (frames 0-2)
	# High strength: trees sway fully (frames 0-9)
	var sway_amount = normalized * wind_strength

	# Map to frame index (0-9)
	var frame = int(sway_amount * 9.0)
	return clamp(frame, 0, 9)

func get_wind_info() -> Dictionary:
	"""Get current wind state for debugging."""
	return {
		"time": wind_time,
		"strength": wind_strength,
		"phase": fmod(wind_time / WIND_CYCLE_DURATION, 1.0)
	}
