# Windows Sprite Extraction Guide

**Purpose:** Extract OpenRCT2 terrain sprites on Windows machine and transfer to macOS for Godot integration.

**Time Required:** 30-45 minutes

**Prerequisites:**
- Windows PC with web browser
- Access to RollerCoaster Tycoon 2 or RCT Classic g1.dat file
- ~50MB free disk space

---

## 📋 Complete Step-by-Step Instructions

### Step 1: Download Trigger's Graphics Extractor

**Download Link:** [MediaFire - Trigger's Graphics Extractor v1.2.0.2](https://www.mediafire.com/download/wbsathqft8k8oas/Triggers+Graphics+Extractor+v1.2.0.2.zip)

**Instructions:**
1. Click the MediaFire link
2. Download `Triggers Graphics Extractor v1.2.0.2.zip`
3. Extract the ZIP file to a folder (e.g., `C:\RCT2Tools\GraphicsExtractor\`)
4. Inside you'll find `GraphicsExtractor.exe`

**No installation needed** - it's a portable executable.

---

### Step 2: Locate g1.dat File

**Where to find g1.dat:**

**If you have RollerCoaster Tycoon 2 (Steam):**
```
C:\Program Files (x86)\Steam\steamapps\common\Rollercoaster Tycoon 2\Data\g1.dat
```

**If you have RCT Classic (Steam):**
```
C:\Program Files (x86)\Steam\steamapps\common\RollerCoaster Tycoon Classic\Data\g1.dat
```

**If you have GOG version:**
```
C:\Program Files (x86)\GOG Galaxy\Games\RollerCoaster Tycoon 2\Data\g1.dat
```

**If you have OpenRCT2 installed:**
OpenRCT2 shares the same g1.dat - check `Documents\OpenRCT2\object\` or your RCT2 installation.

**To verify you found it:**
- File name: `g1.dat`
- File size: ~16-17 MB
- Right-click → Properties should show "Type: DAT File"

---

### Step 3: Run Graphics Extractor

1. **Launch the tool:**
   - Double-click `GraphicsExtractor.exe`
   - Windows may show security warning - click "More info" → "Run anyway"

2. **Configure extraction:**
   - **Input Directory:** Click "Browse" and navigate to the folder containing `g1.dat`
     - Example: `C:\Program Files (x86)\Steam\steamapps\common\Rollercoaster Tycoon 2\Data\`
   - **Output Directory:** Create a new folder for extracted sprites
     - Recommended: `C:\RCT2Sprites\extracted\`

3. **Extract all graphics:**
   - Click **"Extract"** button
   - Progress bar will show extraction status
   - This extracts ALL ~5000+ sprites from RCT2
   - Takes 2-5 minutes depending on PC speed

4. **Extraction complete:**
   - You'll see thousands of PNG files in the output directory
   - Organized by sprite index numbers
   - Also includes palette files

---

### Step 4: Identify Terrain Sprites

**Terrain sprites are in specific index ranges:**

According to OpenRCT2 sprite list:
- **Grass terrain:** Sprite indices 3419-3437 (19 slopes)
- **Sand terrain:** Sprite indices 3438-3456 (19 slopes)
- **Dirt terrain:** Sprite indices 3457-3475 (19 slopes)
- **Rock/Stone terrain:** Sprite indices 3476-3494 (19 slopes)
- **Grass (dark):** Sprite indices 3495-3513 (19 slopes)
- **Grass (light):** Sprite indices 3514-3532 (19 slopes)

**Create this folder structure:**
```
C:\RCT2Sprites\
  organized\
    grass\
    sand\
    dirt\
    stone\
    forest\
    water\
```

---

### Step 5: Organize Terrain Sprites

**Use this PowerShell script** to automatically organize sprites:

**Save this as `C:\RCT2Sprites\organize_sprites.ps1`:**

```powershell
# Organize RCT2 terrain sprites into folders
# Run from C:\RCT2Sprites\ directory

$extractedPath = "C:\RCT2Sprites\extracted"
$organizedPath = "C:\RCT2Sprites\organized"

# Create organized folder structure
$terrains = @("grass", "sand", "dirt", "stone", "grass_dark", "grass_light")
foreach ($terrain in $terrains) {
    New-Item -ItemType Directory -Force -Path "$organizedPath\$terrain" | Out-Null
}

Write-Host "Organizing terrain sprites..." -ForegroundColor Cyan

# Terrain sprite ranges (19 slopes each)
$terrainRanges = @{
    "grass"       = 3419..3437  # Standard grass
    "sand"        = 3438..3456  # Beach sand
    "dirt"        = 3457..3475  # Brown dirt
    "stone"       = 3476..3494  # Gray rock/stone
    "grass_dark"  = 3495..3513  # Dark green grass (forest floor)
    "grass_light" = 3514..3532  # Light grass (dried)
}

foreach ($terrain in $terrainRanges.Keys) {
    $range = $terrainRanges[$terrain]
    $slopeIndex = 0

    foreach ($spriteNum in $range) {
        $sourceFile = "$extractedPath\$spriteNum.png"
        $destFile = "$organizedPath\$terrain\slope_$('{0:D2}' -f $slopeIndex).png"

        if (Test-Path $sourceFile) {
            Copy-Item $sourceFile $destFile -Force
            Write-Host "  Copied $terrain slope $slopeIndex" -ForegroundColor Green
        } else {
            Write-Host "  WARNING: Missing sprite $spriteNum for $terrain slope $slopeIndex" -ForegroundColor Yellow
        }

        $slopeIndex++
    }
}

Write-Host "`nOrganization complete!" -ForegroundColor Green
Write-Host "Sprites saved to: $organizedPath" -ForegroundColor Cyan
Write-Host "`nNext steps:" -ForegroundColor Yellow
Write-Host "  1. Verify sprites in organized\ folders"
Write-Host "  2. Create atlas files with ImageMagick"
Write-Host "  3. Transfer to macOS project"
```

**Run the script:**
1. Open PowerShell as Administrator
2. Navigate to folder: `cd C:\RCT2Sprites\`
3. Run: `powershell -ExecutionPolicy Bypass -File organize_sprites.ps1`
4. Script will copy and rename all terrain sprites

**Expected result:**
```
C:\RCT2Sprites\organized\
  grass\
    slope_00.png  (flat)
    slope_01.png  (N corner up)
    ...
    slope_18.png  (center peak)
  sand\
    slope_00.png
    ...
```

---

### Step 6: Create Atlas Files

**Install ImageMagick on Windows:**

**Download:** https://imagemagick.org/script/download.php#windows
- Choose: "ImageMagick-7.x.x-Q16-HDRI-x64-dll.exe" (recommended)
- Run installer, check "Install legacy utilities (e.g. convert)" option

**Create atlases with this batch script:**

**Save as `C:\RCT2Sprites\create_atlases.bat`:**

```batch
@echo off
echo Creating terrain atlases...

cd C:\RCT2Sprites\organized

REM Process each terrain type
for %%T in (grass sand dirt stone grass_dark grass_light) do (
    echo.
    echo Processing %%T terrain...
    cd %%T

    REM Create row 0 (slopes 0-9)
    magick montage slope_00.png slope_01.png slope_02.png slope_03.png slope_04.png slope_05.png slope_06.png slope_07.png slope_08.png slope_09.png -tile 10x1 -geometry 32x16+0+0 -background transparent %%T_atlas_row0.png

    REM Create row 1 (slopes 10-18)
    magick montage slope_10.png slope_11.png slope_12.png slope_13.png slope_14.png slope_15.png slope_16.png slope_17.png slope_18.png -tile 9x1 -geometry 32x16+0+0 -background transparent %%T_atlas_row1.png

    REM Combine rows into final atlas (320x32 pixels)
    magick convert %%T_atlas_row0.png %%T_atlas_row1.png -append %%T_atlas.png

    REM Clean up intermediate files
    del %%T_atlas_row0.png
    del %%T_atlas_row1.png

    echo Created %%T_atlas.png (320x32 pixels)

    cd ..
)

echo.
echo All atlases created successfully!
echo Files are in: C:\RCT2Sprites\organized\[terrain]\
pause
```

**Run the batch file:**
1. Double-click `create_atlases.bat`
2. Wait for ImageMagick to process (30-60 seconds)
3. Each terrain folder will now have `[terrain]_atlas.png`

**Verify atlases:**
- Open `grass\grass_atlas.png` in Paint or image viewer
- Should be 320×32 pixels
- Should show 19 tiles arranged in 2 rows (10 in row 0, 9 in row 1)

---

### Step 7: Transfer to macOS

**Files to transfer:**

```
C:\RCT2Sprites\organized\
  grass\
    grass_atlas.png          ← CRITICAL
    slope_00.png - slope_18.png (optional, for reference)
  sand\
    sand_atlas.png           ← CRITICAL
    slope_*.png
  dirt\
    dirt_atlas.png           ← CRITICAL
    slope_*.png
  stone\
    stone_atlas.png          ← CRITICAL
    slope_*.png
  grass_dark\
    grass_dark_atlas.png     ← For forest floor
    slope_*.png
  grass_light\
    grass_light_atlas.png    ← Optional variant
    slope_*.png
```

**Transfer methods:**

**Option A: USB Drive (Recommended)**
1. Copy `C:\RCT2Sprites\organized\` to USB drive
2. Plug USB into Mac
3. Copy to Mac's Downloads folder

**Option B: Cloud Storage**
1. Upload `organized\` folder to Dropbox/Google Drive/iCloud
2. Download on Mac

**Option C: Network Share**
1. Enable file sharing on Windows
2. Connect from Mac via Finder → Network
3. Copy files

**Option D: GitHub (if comfortable with git)**
```bash
# On Windows
cd C:\RCT2Sprites\organized
git init
git add .
git commit -m "Extracted OpenRCT2 terrain sprites"
git remote add origin [your-repo-url]
git push

# On Mac
git pull
```

---

### Step 8: Verify Extraction Quality

**Check each terrain type:**

Open atlases in image viewer and verify:
- [ ] Image is 320×32 pixels
- [ ] 19 tiles visible (10 in top row, 9 in bottom row)
- [ ] First tile (slope_00) is flat
- [ ] Last tile (slope_18) has peaked/raised appearance
- [ ] Colors are correct (grass green, sand tan, dirt brown, stone gray)
- [ ] No black borders or artifacts
- [ ] Transparent background (if applicable)

**Common issues:**

**Issue: Sprites appear black/corrupted**
- Solution: Re-extract with Graphics Extractor, ensure g1.dat is not corrupted

**Issue: Atlas has wrong dimensions**
- Solution: Re-run create_atlases.bat, check ImageMagick installed correctly

**Issue: Missing sprites**
- Solution: Check organize_sprites.ps1 ran without errors, verify sprite index numbers

---

## 📦 Final Deliverable

**You should have:**
- ✅ 6 terrain atlas files (320×32 pixels each)
- ✅ Individual slope sprites (optional backup)
- ✅ All organized in folders by terrain type

**Expected file sizes:**
- Each atlas: ~5-15 KB (PNG with transparency)
- Total: ~100 KB for all atlases

**Next step on macOS:**
Transfer files to:
```
/Users/jean/Github/life-simulator/godot-viewer/assets/tiles/terrain/openrct2_placeholder/
```

Then follow `SETUP_SLOPE_RENDERING.md` Step 4 (Configure Godot TileSet).

---

## 🆘 Troubleshooting

### Graphics Extractor won't run
- **Cause:** Windows security blocking unsigned .exe
- **Solution:** Right-click → Properties → Check "Unblock" → Apply → OK

### PowerShell script won't run
- **Cause:** Execution policy restrictions
- **Solution:** Run PowerShell as Administrator, use `-ExecutionPolicy Bypass` flag

### ImageMagick not found
- **Cause:** Not in PATH or not installed
- **Solution:**
  1. Reinstall ImageMagick with "Add to PATH" option checked
  2. OR use full path: `"C:\Program Files\ImageMagick-7.x.x-Q16-HDRI\magick.exe" montage ...`

### Sprites look wrong (wrong colors/palette)
- **Cause:** Trigger's tool should handle palettes automatically
- **Solution:** This is normal - RCT2 uses indexed colors, extracted PNGs may differ slightly. They'll work fine in Godot.

### Atlas dimensions wrong
- **Cause:** Montage geometry incorrect
- **Solution:** Verify `-geometry 32x16+0+0` is exactly correct (32×16 with no spacing)

---

## 📚 Reference Information

### Terrain Sprite Index Reference

| Terrain Type | Sprite Range | Slopes | Color |
|-------------|--------------|--------|-------|
| Grass (standard) | 3419-3437 | 19 | Green |
| Sand | 3438-3456 | 19 | Tan |
| Dirt | 3457-3475 | 19 | Brown |
| Stone/Rock | 3476-3494 | 19 | Gray |
| Grass (dark) | 3495-3513 | 19 | Dark Green |
| Grass (light) | 3514-3532 | 19 | Light Green |

### Slope Index Meanings

| Index | Description | Corners Raised |
|-------|-------------|----------------|
| 0 | Flat | None |
| 1 | N corner up | N |
| 2 | E corner up | E |
| 3 | NE side up | N, E |
| 4 | S corner up | S |
| 5 | NS valley | N, S |
| 6 | SE side up | S, E |
| 7 | NES corners up | N, E, S |
| 8 | W corner up | W |
| 9 | NW side up | N, W |
| 10 | EW valley | E, W |
| 11 | NEW corners up | N, E, W |
| 12 | SW side up | S, W |
| 13 | NWS corners up | N, W, S |
| 14 | ESW corners up | E, S, W |
| 15 | All corners up | N, E, S, W |
| 16 | Diagonal NE-SW | Diagonal |
| 17 | Diagonal NW-SE | Diagonal |
| 18 | Center peak | Special |

---

## ✅ Completion Checklist

Before leaving Windows machine:

- [ ] Graphics Extractor downloaded and run successfully
- [ ] g1.dat located and extracted (5000+ sprite files)
- [ ] PowerShell organize script completed (6 terrain folders)
- [ ] ImageMagick installed
- [ ] Atlas batch file run successfully
- [ ] 6 atlas files created (grass, sand, dirt, stone, grass_dark, grass_light)
- [ ] Atlases verified (320×32 pixels, 19 tiles visible)
- [ ] Files transferred to USB/cloud storage
- [ ] Backup of organized\ folder created (just in case)

**Ready to continue on macOS!** 🎉

---

**Questions? Issues?**
Document any problems encountered and bring them back to macOS Claude Code session for troubleshooting.

**Alternative terrain types:**
If you want water, snow, or other terrains, the sprite indices are documented at:
https://github.com/OpenRCT2/OpenRCT2/wiki/Sprite-List-g1.dat
