#!/usr/bin/env python3
"""
validate_resources.py - Validate ResourceManager configuration and data
Tests resource symbols and configurations against actual backend data
"""

import requests
import json
import sys

def fetch_chunk_data(center_x=0, center_y=0, radius=1):
    """Fetch chunk data from backend API"""
    url = f"http://localhost:54321/api/chunks?center_x={center_x}&center_y={center_y}&radius={radius}&layers=true"
    
    try:
        response = requests.get(url)
        response.raise_for_status()
        return response.json()
    except requests.RequestException as e:
        print(f"❌ Failed to fetch data: {e}")
        return None

def extract_unique_resources(chunk_data):
    """Extract all unique resource types from chunk data"""
    resources = set()
    
    if not chunk_data or 'chunk_data' not in chunk_data:
        return resources
    
    for chunk_key, chunk_info in chunk_data['chunk_data'].items():
        if 'resources' in chunk_info:
            for row in chunk_info['resources']:
                for resource in row:
                    if resource:  # Skip empty strings
                        resources.add(resource)
    
    return resources

def validate_resource_config():
    """Validate that all resource types have proper configuration"""
    
    # Expected resource configuration (mirrors Config.gd)
    resource_symbols = {
        "TreeOak": "🌳",
        "TreePine": "🌲", 
        "TreeBirch": "🪾",
        "Rock": "🪨",
        "Bush": "🌳",
        "Flower": "🌸",
        "HazelShrub": "🌳",
        "OakTree": "🌳",
        "PineTree": "🌲",
        "BirchTree": "🪾",
        "Stone": "🪨",
        "BerryBush": "🫐",
        "MushroomPatch": "🍄",
        "WildRoot": "🥜"
    }
    
    resource_config = {
        "TreeOak": {"size_multiplier": 1.4, "offset_x": 0.0, "offset_y": -0.3},
        "TreePine": {"size_multiplier": 1.6, "offset_x": 0.0, "offset_y": -0.5},
        "TreeBirch": {"size_multiplier": 1.4, "offset_x": 0.0, "offset_y": -0.3},
        "Rock": {"size_multiplier": 0.6, "offset_x": 0.0, "offset_y": 0.1},
        "Bush": {"size_multiplier": 0.6, "offset_x": 0.0, "offset_y": 0.1},
        "Flower": {"size_multiplier": 0.4, "offset_x": 0.0, "offset_y": 0.0},
        "HazelShrub": {"size_multiplier": 0.8, "offset_x": 0.0, "offset_y": 0.1},
        "OakTree": {"size_multiplier": 1.4, "offset_x": 0.0, "offset_y": -0.3},
        "PineTree": {"size_multiplier": 1.6, "offset_x": 0.0, "offset_y": -0.5},
        "BirchTree": {"size_multiplier": 1.4, "offset_x": 0.0, "offset_y": -0.3},
        "Stone": {"size_multiplier": 0.6, "offset_x": 0.0, "offset_y": 0.1},
        "BerryBush": {"size_multiplier": 0.7, "offset_x": 0.0, "offset_y": 0.1},
        "MushroomPatch": {"size_multiplier": 0.5, "offset_x": 0.0, "offset_y": 0.0},
        "WildRoot": {"size_multiplier": 0.4, "offset_x": 0.0, "offset_y": 0.0}
    }
    
    return resource_symbols, resource_config

def main():
    print("🧪 Validating ResourceManager configuration...")
    
    # Fetch actual resource data from backend
    print("📡 Fetching resource data from backend...")
    chunk_data = fetch_chunk_data()
    
    if not chunk_data:
        print("❌ Failed to fetch data from backend")
        return 1
    
    # Extract unique resource types
    actual_resources = extract_unique_resources(chunk_data)
    print(f"📊 Found {len(actual_resources)} unique resource types: {sorted(actual_resources)}")
    
    # Get expected configuration
    resource_symbols, resource_config = validate_resource_config()
    
    # Validate each resource type
    missing_symbols = []
    missing_configs = []
    
    for resource_type in actual_resources:
        if resource_type not in resource_symbols:
            missing_symbols.append(resource_type)
            print(f"❌ Missing symbol for resource: {resource_type}")
        else:
            print(f"✅ {resource_type} -> {resource_symbols[resource_type]}")
        
        if resource_type not in resource_config:
            missing_configs.append(resource_type)
            print(f"❌ Missing config for resource: {resource_type}")
        else:
            config = resource_config[resource_type]
            print(f"✅ {resource_type} config: size={config['size_multiplier']}, offset=({config['offset_x']}, {config['offset_y']})")
    
    # Summary
    print("\n📋 Validation Summary:")
    print(f"  Total resource types found: {len(actual_resources)}")
    print(f"  Missing symbols: {len(missing_symbols)}")
    print(f"  Missing configs: {len(missing_configs)}")
    
    if missing_symbols:
        print(f"  ❌ Resources without symbols: {missing_symbols}")
    
    if missing_configs:
        print(f"  ❌ Resources without configs: {missing_configs}")
    
    if not missing_symbols and not missing_configs:
        print("🎉 All resource types are properly configured!")
        return 0
    else:
        print("⚠️ Some resource types need configuration")
        return 1

if __name__ == "__main__":
    sys.exit(main())