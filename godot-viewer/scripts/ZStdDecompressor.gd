# ZStdDecompressor.gd - ZStandard decompression utilities for park files
# Provides decompression functionality for compressed OpenRCT2 park file chunks

class_name ZStdDecompressor
extends RefCounted

# Check if ZStd decompression is available
static func is_available() -> bool:
	# For now, we'll use a simple method to check if we have ZStd tools
	# In a production environment, you might want to use a ZStd library
	var output = []
	var result = OS.execute("zstd", ["--version"], output, true, true)
	return result == 0

# Decompress a PackedByteArray using ZStd
static func decompress(compressed_data: PackedByteArray) -> PackedByteArray:
	print("🗜️ Decompressing ", compressed_data.size(), " bytes using ZStd...")

	# Create temporary files for compression/decompression
	var temp_input_path = OS.get_cache_dir() + "/temp_compressed.zst"
	var temp_output_path = OS.get_cache_dir() + "/temp_decompressed.dat"

	# Write compressed data to temp file
	var file = FileAccess.open(temp_input_path, FileAccess.WRITE)
	if file == null:
		print("❌ Failed to create temporary input file")
		return PackedByteArray()

	file.store_buffer(compressed_data)
	file.close()

	# Use system zstd command to decompress
	var output = []
	var result = OS.execute("zstd", ["-d", temp_input_path, "-o", temp_output_path], output, true, true)

	if result != 0:
		print("❌ ZStd decompression failed with exit code: ", result)
		print("   Error output: ", output)
		# Clean up temp files
		_cleanup_temp_files(temp_input_path, temp_output_path)
		return PackedByteArray()

	# Read decompressed data
	file = FileAccess.open(temp_output_path, FileAccess.READ)
	if file == null:
		print("❌ Failed to read decompressed data")
		_cleanup_temp_files(temp_input_path, temp_output_path)
		return PackedByteArray()

	var decompressed_data = file.get_buffer(file.get_length())
	file.close()

	print("✅ Successfully decompressed to ", decompressed_data.size(), " bytes")

	# Clean up temp files
	_cleanup_temp_files(temp_input_path, temp_output_path)

	return decompressed_data

# Decompress a chunk from a file starting at a specific position
static func decompress_chunk(file: FileAccess, compressed_size: int) -> PackedByteArray:
	print("🗜️ Reading and decompressing ", compressed_size, " compressed bytes...")

	# Read compressed data
	var compressed_data = file.get_buffer(compressed_size)
	if compressed_data.size() != compressed_size:
		print("❌ Failed to read compressed data (expected ", compressed_size, ", got ", compressed_data.size(), ")")
		return PackedByteArray()

	return decompress(compressed_data)

# Clean up temporary files
static func _cleanup_temp_files(input_path: String, output_path: String):
	if FileAccess.file_exists(input_path):
		DirAccess.remove_absolute(input_path)
	if FileAccess.file_exists(output_path):
		DirAccess.remove_absolute(output_path)

# Test ZStd availability
static func test_zstd_availability():
	if is_available():
		print("✅ ZStd decompression is available")
		return true
	else:
		print("⚠️ ZStd decompression is not available")
		print("   Install zstd: brew install zstd (macOS) or apt-get install zstd (Linux)")
		return false