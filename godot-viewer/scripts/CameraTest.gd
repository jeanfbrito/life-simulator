extends Node

# Test script to validate CameraController functionality
func _ready() -> void:
	print("=== Camera Controller Test ===")
	
	# Test camera controller exists
	var camera = get_node_or_null("/root/World/TerrainTileMap/Camera2D")
	if camera:
		print("✅ Camera2D node found")
		
		# Test if it has our script
		if camera.has_method("_zoom_at_mouse_position"):
			print("✅ CameraController script attached")
		else:
			print("❌ CameraController script not attached")
	else:
		print("❌ Camera2D node not found")
	
	# Test input mapping
	var input_map = InputMap.get_actions()
	print("📋 Available input actions:")
	for action in input_map:
		if "ui" in action or "camera" in action:
			print("  - ", action)
	
	print("=== Camera Controls ===")
	print("🖱️  Mouse Wheel: Zoom in/out")
	print("🖱️  Middle Mouse + Drag: Pan camera")
	print("⌨️  Arrow Keys/WASD: Move camera")
	print("⌨️  +/- Keys: Zoom in/out")
	print("🖱️  Edge Scrolling: Move camera to screen edges")
	
	# Test camera bounds
	if camera:
		print("📐 Camera settings:")
		print("  Position: ", camera.position)
		print("  Zoom: ", camera.zoom)
		print("  Min Zoom: 0.2x")
		print("  Max Zoom: 5.0x")
	
	print("=== Test Complete ===")