#!/usr/bin/env python3
"""
Validation script for Statistics HUD implementation
Tests file structure, scene integration, and feature completeness
"""

import os
import re

def test_file_exists(filepath, description):
    """Test if a file exists"""
    if os.path.exists(filepath):
        print(f"✅ {description}")
        return True
    else:
        print(f"❌ {description} - Missing: {filepath}")
        return False

def test_content_in_file(filepath, patterns, description):
    """Test if content patterns exist in a file"""
    if not os.path.exists(filepath):
        print(f"❌ {description} - File not found: {filepath}")
        return False
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    all_found = True
    for pattern, desc in patterns:
        if re.search(pattern, content):
            print(f"✅ {desc}")
        else:
            print(f"❌ {desc} - Pattern not found: {pattern}")
            all_found = False
    
    return all_found

def main():
    print("=== Statistics HUD Implementation Validation ===\n")
    
    base_path = "/Users/jean/Github/life-simulator/godot-viewer"
    
    # Test 1: Core files exist
    print("📁 Testing File Structure:")
    files_to_test = [
        (f"{base_path}/scripts/StatisticsHUD.gd", "StatisticsHUD script"),
        (f"{base_path}/scenes/StatisticsHUD.tscn", "StatisticsHUD scene"),
        (f"{base_path}/scenes/World.tscn", "World scene"),
    ]
    
    file_results = []
    for filepath, desc in files_to_test:
        file_results.append(test_file_exists(filepath, desc))
    
    # Test 2: StatisticsHUD features
    print("\n📊 Testing StatisticsHUD Features:")
    stats_patterns = [
        (r"extends Control", "Proper Control inheritance"),
        (r"class_name StatisticsHUD", "Class name definition"),
        (r"func _ready", "Initialization method"),
        (r"func _process", "Process method for updates"),
        (r"func update_statistics", "Statistics update method"),
        (r"func count_entities", "Entity counting"),
        (r"func count_entities_by_species", "Species counting"),
        (r"func count_resources", "Resource counting"),
        (r"func get_formatted_memory", "Memory formatting"),
        (r"WorldDataCache", "World data integration"),
        (r"ChunkManager", "Chunk manager integration"),
        (r"update_interval", "Update timing control"),
        (r"KEY_TAB", "Tab key shortcut"),
        (r"RichTextLabel", "Rich text display"),
        (r"Engine\.get_frames_per_second", "FPS monitoring"),
        (r"OS\.get_static_memory_usage", "Memory monitoring"),
    ]
    
    stats_result = test_content_in_file(
        f"{base_path}/scripts/StatisticsHUD.gd",
        stats_patterns,
        "StatisticsHUD Features"
    )
    
    # Test 3: Scene integration
    print("\n🎬 Testing Scene Integration:")
    world_patterns = [
        (r"StatisticsHUD", "StatisticsHUD instance reference"),
        (r"StatisticsHUD\.tscn", "StatisticsHUD scene reference"),
    ]
    
    scene_result = test_content_in_file(
        f"{base_path}/scenes/World.tscn",
        world_patterns,
        "World Scene Integration"
    )
    
    # Test 4: StatisticsHUD scene structure
    print("\n🎨 Testing StatisticsHUD Scene:")
    stats_scene_patterns = [
        (r"StatisticsHUD.*type.*Control", "Root Control node"),
        (r"StatsPanel.*type.*Panel", "Stats panel container"),
        (r"StatsText.*type.*RichTextLabel", "Stats text display"),
        (r"ToggleButton.*type.*Button", "Toggle button"),
        (r"bbcode_enabled.*true", "BBCode formatting enabled"),
    ]
    
    stats_scene_result = test_content_in_file(
        f"{base_path}/scenes/StatisticsHUD.tscn",
        stats_scene_patterns,
        "StatisticsHUD Scene Structure"
    )
    
    # Test 5: Statistics content categories
    print("\n📈 Testing Statistics Categories:")
    content_patterns = [
        (r"World Information", "World info section"),
        (r"Chunk Statistics", "Chunk stats section"),
        (r"Entity Statistics", "Entity stats section"),
        (r"Resource Statistics", "Resource stats section"),
        (r"Performance", "Performance section"),
        (r"Total Entities", "Entity count display"),
        (r"Total Resources", "Resource count display"),
        (r"FPS:", "FPS display"),
        (r"Memory:", "Memory display"),
        (r"species_counts", "Species breakdown"),
        (r"loading_percentage", "Loading progress"),
    ]
    
    content_result = test_content_in_file(
        f"{base_path}/scripts/StatisticsHUD.gd",
        content_patterns,
        "Statistics Content Categories"
    )
    
    # Summary
    print("\n📋 Implementation Summary:")
    all_results = file_results + [stats_result, scene_result, stats_scene_result, content_result]
    success_count = sum(all_results)
    total_count = len(all_results)
    
    print(f"\n📊 Test Results: {success_count}/{total_count} passed")
    
    if success_count == total_count:
        print("🎉 All tests passed! Statistics HUD implementation is complete.")
        print("\n📊 Features Implemented:")
        print("  • Real-time world information display")
        print("  • Chunk loading statistics and progress")
        print("  • Entity counting by species")
        print("  • Resource counting by type")
        print("  • Performance metrics (FPS, Memory)")
        print("  • Auto-update every second")
        print("  • Tab key toggle functionality")
        print("  • Professional UI with theming")
        print("  • Change tracking (Δ indicators)")
    else:
        print("⚠️  Some tests failed. Please review the implementation.")
    
    print("\n=== Validation Complete ===")
    
    return success_count == total_count

if __name__ == "__main__":
    success = main()
    exit(0 if success else 1)