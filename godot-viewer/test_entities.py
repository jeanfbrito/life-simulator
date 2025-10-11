#!/usr/bin/env python3
"""
test_entities.py - Test entity data and configuration
Validates entity API response and configuration mapping
"""

import json
import subprocess

def get_entity_data():
    """Fetch entity data from backend"""
    try:
        result = subprocess.run(
            ['curl', '-s', 'http://localhost:54321/api/entities'],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return json.loads(result.stdout)
        return None
    except Exception as e:
        print(f"❌ Failed to fetch entity data: {e}")
        return None

def get_species_data():
    """Fetch species configuration from backend"""
    try:
        result = subprocess.run(
            ['curl', '-s', 'http://localhost:54321/api/species'],
            capture_output=True, text=True
        )
        if result.returncode == 0:
            return json.loads(result.stdout)
        return None
    except Exception as e:
        print(f"❌ Failed to fetch species data: {e}")
        return None

def main():
    print("🧪 Testing entity data and configuration...")
    print()
    
    # Test entity data
    print("📡 Fetching entity data...")
    entity_data = get_entity_data()
    
    if not entity_data or 'entities' not in entity_data:
        print("❌ No entity data found")
        return 1
    
    entities = entity_data['entities']
    print(f"✅ Found {len(entities)} entities")
    
    # Analyze entity types
    entity_types = {}
    juvenile_counts = {}
    action_counts = {}
    
    for entity in entities:
        entity_type = entity.get('entity_type', 'unknown')
        entity_types[entity_type] = entity_types.get(entity_type, 0) + 1
        
        if entity.get('is_juvenile', False):
            juvenile_counts[entity_type] = juvenile_counts.get(entity_type, 0) + 1
        
        action = entity.get('current_action', 'Unknown')
        action_counts[action] = action_counts.get(action, 0) + 1
    
    print("📊 Entity types:")
    for entity_type, count in sorted(entity_types.items()):
        juvenile_count = juvenile_counts.get(entity_type, 0)
        print(f"  {entity_type}: {count} total, {juvenile_count} juvenile")
    
    print("\n🎭 Current actions:")
    for action, count in sorted(action_counts.items()):
        print(f"  {action}: {count}")
    
    # Test species configuration
    print("\n📡 Fetching species configuration...")
    species_data = get_species_data()
    
    if not species_data:
        print("❌ No species data found")
        return 1
    
    print("✅ Species configuration loaded")
    
    # Validate species configs
    print("\n🔍 Species configuration:")
    if 'species' in species_data:
        for species_name, species_config in species_data['species'].items():
            emoji = species_config.get('emoji', '❓')
            print(f"  {species_name}: {emoji}")
    
    # Validate juvenile scales
    if 'juvenile_scales' in species_data:
        print("\n👶 Juvenile scales:")
        for species, scale in species_data['juvenile_scales'].items():
            print(f"  {species}: {scale}x")
    
    # Sample entity details
    print("\n🔍 Sample entity details:")
    for i, entity in enumerate(entities[:3]):
        print(f"  Entity {i+1}:")
        print(f"    ID: {entity.get('id')}")
        print(f"    Name: {entity.get('name')}")
        print(f"    Type: {entity.get('entity_type')}")
        print(f"    Position: ({entity.get('position', {}).get('x')}, {entity.get('position', {}).get('y')})")
        print(f"    Action: {entity.get('current_action')}")
        print(f"    Juvenile: {entity.get('is_juvenile')}")
        print(f"    Health: {entity.get('health', 0):.1f}%")
        print(f"    Hunger: {entity.get('hunger', 0):.1f}%")
        print(f"    Thirst: {entity.get('thirst', 0):.1f}%")
    
    print("\n🎉 Entity data validation completed successfully!")
    print("✅ Entity API is working")
    print("✅ Species configuration is available")
    print("✅ EntityManager should be able to display all entities")
    
    return 0

if __name__ == "__main__":
    exit(main())