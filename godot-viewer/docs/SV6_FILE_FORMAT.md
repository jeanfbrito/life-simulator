# RCT2 SV6 File Format

## Overview

The SV6 format is the original RollerCoaster Tycoon 2 save game format, which is **much simpler** than OpenRCT2's newer `.park` format. This format is well-documented and easier to parse.

## File Structure

SV6 files contain **5 chunks** of compressed data:

```
SV6 File Structure
├── Chunk 0: Header (32 bytes)
├── Chunk 1: [Not present in SV6]
├── Chunk 2: Import Objects (custom DAT files, no header)
├── Chunk 3: Available Items (11536 bytes, 721 x 16-byte structures)
├── Chunk 4: Flags1 (16 bytes)
├── Chunk 5: Game Map ⬅️ **Terrain data is here**
├── Chunk 6: Game Data (3,048,816 bytes - sprites, stats, values, strings)
└── Checksum (4 bytes at end of file)
```

## Chunk Format

Each chunk (except chunk 2) has a **5-byte header**:
- **Byte 0**: Encoding type (see table below)
- **Bytes 1-4**: Number of data bytes in chunk (little-endian uint32)

### Encoding Types

| Code | Meaning |
|------|---------|
| 0x00 | Uncompressed - copy data bytes without change |
| 0x01 | RLE compression |
| 0x02 | RLE + String decompression |
| 0x03 | Bit rotation (1, 3, 5, 7 bits, repeating) |

### String Decompression

After RLE, some chunks use string compression:
- Read byte: if `0xFF`, copy next byte as-is
- Otherwise:
  - Lower 3 bits = length (add 1)
  - Upper 5 bits >> 3 = offset - 32
  - Copy string from `current_pos - offset` for `length` bytes

## Game Map Chunk (Chunk 5)

This is the **most important chunk** for terrain rendering!

### Map Element Structure (8 bytes each)

Each tile position can have **multiple stacked elements**:

```
Offset  Description
------  -----------
0x00    Element type + flags
        Bits 0-3: Element type
          0 = Surface (terrain)
          1 = Path
          2 = Track
          3 = Small Scenery
          4 = Entrance
          5 = Wall
          6 = Large Scenery
          7 = Banner
        Bit 7: Last element flag
        
0x01    Flags + quadrant
        Bit 0-3: Occupied quadrants
        Bit 4: Clearance height extension
        Bit 5: Ghost/hidden
        Bit 6: Scenery quadrant
        Bit 7: Last element in tile
        
0x02    Base height (units of 8)
        
0x03    Clearance height (units of 8)
        
0x04-07 Element-specific data
```

### Surface Element (Type 0)

For terrain tiles, bytes 0x04-0x07 contain:

```
0x04    Terrain type + edge style
        Bits 0-4: Terrain surface type
          0 = Grass
          1 = Sand
          2 = Dirt
          3 = Rock
          4 = Martian
          5 = Checkerboard
          6 = Grass clumps
          7 = Ice
          8 = Grid red
          9 = Grid yellow
          10 = Grid blue
          11 = Grid green
          12 = Sand dark
          13 = Sand light
        Bits 5-7: Edge style (terrain edges - matches SPR_EDGE_* sprite IDs)
          0 = Rock
          1 = Wood Red
          2 = Wood Black  
          3 = Ice
        
0x05    Slope + water
        Bits 0-4: Slope type (0-29, matches OpenRCT2 slope indices)
        Bit 5: Diagonal slope flag
        Bits 6-7: Water height / 16
        
0x06    Grass length + ownership
        Bits 0-2: Grass length (0-7)
        Bits 4-7: Park ownership flags
        
0x07    Water height extension + ownership
```

### Slope Values

The slope byte (0x05, bits 0-4) maps directly to OpenRCT2 slope indices:

```
Value   Description
-----   -----------
0       Flat
1       N corner up
2       E corner up
3       NE side up
4       S corner up
5       NS valley
6       SE side up
7       Three corners up (W down)
8       W corner up
9       NW side up
10      EW valley
11      Three corners up (S down)
12      SW side up
13      Three corners up (E down)
14      Three corners up (N down)
15      All corners up
23      Steep diagonal (varies)
27      Steep diagonal (varies)
29      Steep diagonal (varies)
30      Steep diagonal (varies)
```

## Header Chunk (Chunk 0)

```
Offset  Description
------  -----------
0x00-01 File type (0 = SV6, 1 = SC6)
0x02-03 Number of custom objects embedded
0x04-07 Version marker (120001 decimal = 0x0001D4C1)
0x08-0B Game version info
0x0C-1F Reserved/unknown
```

## Checksum

Last 4 bytes of file = Simple additive checksum:
```
checksum = 0
for each byte in file (excluding checksum):
    checksum += byte
```

## Converting SV6 to RON Format

### Parsing Strategy

1. **Read file header** (32 bytes)
2. **Skip/parse chunks 2-4** (objects, items, flags)
3. **Parse chunk 5** (Game Map):
   - Read chunk header (5 bytes)
   - Decompress based on encoding type
   - Read 8-byte map elements sequentially
   - Filter for surface elements (type 0)
   - Extract: height, slope, terrain type

4. **Convert to RON**:
   ```ron
   (
       width: 256,
       height: 256,
       terrain: [
           ["grass", "grass", ...],
           ...
       ],
       heights: [
           [0, 0, 1, 2, ...],
           ...
       ],
       slopes: [
           [0, 0, 1, 3, ...],
           ...
       ]
   )
   ```

## Implementation Plan

### Phase 1: SV6 Parser in Rust

Create `src/sv6_parser.rs`:

```rust
pub struct SV6Parser;

impl SV6Parser {
    pub fn parse(path: &Path) -> Result<MapData> {
        // 1. Read and validate header
        // 2. Parse chunks (focus on chunk 5)
        // 3. Decompress Game Map chunk
        // 4. Extract surface elements
        // 5. Convert to internal format
    }
}
```

### Phase 2: Decompression

Implement the 4 encoding types:
- Type 0: Uncompressed (memcpy)
- Type 1: RLE decode
- Type 2: RLE + String decompress
- Type 3: Bit rotation

### Phase 3: CLI Tool

```bash
cargo run --bin sv6-to-ron -- input.sv6 output.ron
```

### Phase 4: Integration

Add to `ChunkManager`:
```gdscript
func detect_file_type(path: String) -> String:
    if path.ends_with(".ron"):
        return "ron"
    elif path.ends_with(".sv6") or path.ends_with(".sc6"):
        return "sv6"
    elif path.ends_with(".park"):
        return "park"
```

## Converting Park to SV6

OpenRCT2 can save in both formats:

```
In OpenRCT2:
1. Load: good-generated-map.park
2. File → Save Game As
3. Choose "Classic (SV6)" format
4. Save as: good-generated-map.sv6
```

## Advantages of SV6 vs Park Format

| Feature | SV6 | Park |
|---------|-----|------|
| **Complexity** | Simple chunk format | Complex Orca stream |
| **Documentation** | Well-documented | OpenRCT2 source only |
| **Parsing** | Straightforward | Requires Orca parser |
| **Size** | Larger (uncompressed) | Smaller (better compression) |
| **Support** | RCT2 + OpenRCT2 | OpenRCT2 only |

**Recommendation: Start with SV6 parsing** - It's much easier to implement and well-documented!

## Next Steps

1. **Convert park to SV6**:
   ```bash
   # Open in OpenRCT2
   open -a OpenRCT2 good-generated-map.park
   # Save as SV6 format
   ```

2. **Implement SV6 parser** in Rust backend

3. **Test with converted file**

4. **Add to auto-load system** in Godot viewer

## References

- [RCT TID: SV6 Format Documentation](http://rct.wikia.com/wiki/SV6_format)
- OpenRCT2 Legacy Importers: `/Users/jean/Github/OpenRCT2/src/openrct2/rct2/S6Importer.cpp`
- Slope definitions: `/Users/jean/Github/OpenRCT2/src/openrct2/world/tile_element/Slope.cpp`

