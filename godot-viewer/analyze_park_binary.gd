#!/usr/bin/env godot
# Analyze the raw binary structure of the park file
extends MainLoop

func _initialize():
	print("=== Park File Binary Analysis ===")
	analyze_park_file_structure()
	return true

func _process(_delta: float) -> bool:
	return true

func analyze_park_file_structure():
	var park_file_path = "res://../good-generated-map.park"

	if not FileAccess.file_exists(park_file_path):
		print("❌ Park file not found: ", park_file_path)
		return

	var file = FileAccess.open(park_file_path, FileAccess.READ)
	if not file:
		print("❌ Failed to open park file")
		return

	print("📁 Analyzing park file: ", park_file_path)
	print("📏 File size: ", file.get_length(), " bytes")

	# Read first 200 bytes for header analysis
	file.seek(0)
	var header_data = file.get_buffer(200)

	print("\n=== HEADER ANALYSIS ===")
	print("First 64 bytes (hex):")
	for i in range(min(64, header_data.size())):
		if i % 16 == 0:
			print("\n%04X: " % i, "")
		print("%02X " % header_data[i], "")

	print("\n\n=== PARK FILE HEADER ===")
	if header_data.size() >= 8:
		print("Magic bytes: ", "%02X %02X %02X %02X" % [header_data[0], header_data[1], header_data[2], header_data[3]])

		# Try to detect common park file signatures
		var potential_header = ""
		for i in range(min(8, header_data.size())):
			if header_data[i] >= 32 and header_data[i] <= 126:  # Printable ASCII
				potential_header += char(header_data[i])
			else:
				potential_header += "."
		print("ASCII header: \"", potential_header, "\"")

	# Look for version information
	if header_data.size() >= 20:
		# Most park files have version around offset 4-8
		var version1 = header_data[4] + (header_data[5] << 8)
		var version2 = header_data[6] + (header_data[7] << 8)
		var version3 = header_data[8] + (header_data[9] << 8)
		var version4 = header_data[10] + (header_data[11] << 8)

		print("Potential version numbers (little endian):")
		print("  Offset 4: ", version1)
		print("  Offset 6: ", version2)
		print("  Offset 8: ", version3)
		print("  Offset 10: ", version4)

	# Look for map size candidates (uint16 values between 10-200)
	print("\n=== MAP SIZE CANDIDATES ===")
	for offset in range(0, min(100, header_data.size() - 1)):
		var size_candidate = header_data[offset] + (header_data[offset + 1] << 8)
		if size_candidate >= 10 and size_candidate <= 200:
			print("  Offset %02X: %d (potential map size)" % [offset, size_candidate])

	# Look for chunk headers (common patterns)
	print("\n=== CHUNK ANALYSIS ===")
	file.seek(0)
	var chunk_headers = []
	var pos = 0

	while pos < file.get_length() - 8:
		file.seek(pos)
		var chunk_id = file.get_32()
		var chunk_size = file.get_32()

		# Look for reasonable chunk sizes
		if chunk_size > 0 and chunk_size < 1000000:  # Reasonable size range
			chunk_headers.append({
				"offset": pos,
				"id": "0x%08X" % chunk_id,
				"size": chunk_size
			})

		pos += 8
		if chunk_headers.size() > 20:  # Limit to first 20 chunks
			break

	print("Found ", chunk_headers.size(), " potential chunks:")
	for chunk in chunk_headers:
		print("  Offset ", chunk.offset, ": ID ", chunk.id, ", Size ", chunk.size, " bytes")

	# Look specifically for map size in common S6 header locations
	print("\n=== S6 HEADER LOCATIONS ===")
	file.seek(0)
	var s6_offsets = [0x0C, 0x0E, 0x10, 0x12, 0x14, 0x16, 0x18, 0x1A, 0x1C, 0x1E]

	for offset in s6_offsets:
		if offset < file.get_length() - 2:
			file.seek(offset)
			var map_size = file.get_16()
			if map_size >= 10 and map_size <= 200:
				print("  Offset %02X: %d (likely MapSize)" % [offset, map_size])

	file.close()
	print("\n✅ Binary analysis complete")