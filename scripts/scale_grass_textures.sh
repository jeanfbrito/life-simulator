#!/bin/bash
# Scale grass textures to match Godot viewer's 128×64 tile size
# This ensures pixel-perfect rendering in the isometric view

INPUT_DIR="godot-viewer/assets/tiles/grass"
OUTPUT_DIR="godot-viewer/assets/tiles/grass_scaled"

# Check for ImageMagick
if ! command -v convert &> /dev/null; then
    echo "❌ Error: ImageMagick not found"
    echo "   Install with: brew install imagemagick"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "📏 Scaling grass textures to match 128×64 tile size..."
echo ""

# Scale 1×1 tiles: 30×18 → 128×64
echo "Scaling 1×1 tiles (30×18 → 128×64)..."
count=0
for f in "$INPUT_DIR"/abundant_grass_1x1_*.png; do
    if [ -f "$f" ]; then
        basename=$(basename "$f" .png)
        convert "$f" -interpolate Nearest -filter point -resize 128x64! \
            "$OUTPUT_DIR/${basename}.png" 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "  ✓ ${basename}.png"
            ((count++))
        fi
    fi
done
echo "  Scaled $count 1×1 tiles"
echo ""

# Scale 2×2 tiles: 62×35 → 256×128
echo "Scaling 2×2 tiles (62×35 → 256×128)..."
count=0
for f in "$INPUT_DIR"/abundant_grass_2x2_*.png; do
    if [ -f "$f" ]; then
        basename=$(basename "$f" .png)
        convert "$f" -interpolate Nearest -filter point -resize 256x128! \
            "$OUTPUT_DIR/${basename}.png" 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "  ✓ ${basename}.png"
            ((count++))
        fi
    fi
done
echo "  Scaled $count 2×2 tiles"
echo ""

# Scale 3×3 tiles: 94×49 → 384×192
echo "Scaling 3×3 tiles (94×49 → 384×192)..."
count=0
for f in "$INPUT_DIR"/abundant_grass_3x3_*.png; do
    if [ -f "$f" ]; then
        basename=$(basename "$f" .png)
        convert "$f" -interpolate Nearest -filter point -resize 384x192! \
            "$OUTPUT_DIR/${basename}.png" 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "  ✓ ${basename}.png"
            ((count++))
        fi
    fi
done
echo "  Scaled $count 3×3 tiles"
echo ""

# Scale 4×4 tiles: 126×65 → 512×256
echo "Scaling 4×4 tiles (126×65 → 512×256)..."
count=0
for f in "$INPUT_DIR"/abundant_grass_4x4_*.png; do
    if [ -f "$f" ]; then
        basename=$(basename "$f" .png)
        convert "$f" -interpolate Nearest -filter point -resize 512x256! \
            "$OUTPUT_DIR/${basename}.png" 2>/dev/null
        if [ $? -eq 0 ]; then
            echo "  ✓ ${basename}.png"
            ((count++))
        fi
    fi
done
echo "  Scaled $count 4×4 tiles"
echo ""

# Generate size report
echo "✅ Grass textures scaled successfully!"
echo ""
echo "📊 Output directory: $OUTPUT_DIR"
echo ""
echo "Size comparison:"
echo "┌─────────┬──────────────┬───────────────┬─────────────┐"
echo "│ Size    │ Original     │ Scaled        │ Covers      │"
echo "├─────────┼──────────────┼───────────────┼─────────────┤"
echo "│ 1×1     │ 30×18 px     │ 128×64 px     │ 1 tile      │"
echo "│ 2×2     │ 62×35 px     │ 256×128 px    │ 4 tiles     │"
echo "│ 3×3     │ 94×49 px     │ 384×192 px    │ 9 tiles     │"
echo "│ 4×4     │ 126×65 px    │ 512×256 px    │ 16 tiles    │"
echo "└─────────┴──────────────┴───────────────┴─────────────┘"
echo ""
echo "💡 Next steps:"
echo "   1. Update GrassMacroTileRenderer.gd to load from grass_scaled/ directory"
echo "   2. Or replace original files with scaled versions"
echo "   3. Test in Godot viewer to verify pixel-perfect alignment"
echo ""
echo "To replace originals (backup recommended):"
echo "   cp -r $OUTPUT_DIR/* $INPUT_DIR/"
