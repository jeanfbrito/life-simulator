#!/bin/bash
# Bevy ECS System Parameter Conflict Checker
# Detects functions that have both &World and Commands/Query<&mut ...> parameters

echo "🔍 Checking for Bevy ECS system parameter conflicts..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

conflicts_found=0

# Find functions with both &World and Commands
echo "Checking for &World + Commands conflicts..."
for file in $(find src -name "*.rs"); do
    # Extract function signatures (simplified approach)
    awk '/^pub fn|^fn/ {
        getline rest
        line = $0 rest
        # Keep reading until we find the opening brace
        while (line !~ /{/ && getline) {
            line = line $0
        }
        # Check if has both &World and Commands
        if (line ~ /&World/ && line ~ /Commands/) {
            print FILENAME ":" NR ": " $0
            exit 1
        }
    }' "$file"

    if [ $? -eq 1 ]; then
        echo "  ⚠️  $file"
        conflicts_found=$((conflicts_found + 1))
    fi
done

# Find functions with both &World and &mut Query
echo ""
echo "Checking for &World + &mut Query conflicts..."
for file in $(find src -name "*.rs"); do
    awk '/^pub fn|^fn/ {
        getline rest
        line = $0 rest
        while (line !~ /{/ && getline) {
            line = line $0
        }
        if (line ~ /&World/ && line ~ /Query<.*&mut/) {
            print FILENAME ":" NR ": " $0
            exit 1
        }
    }' "$file"

    if [ $? -eq 1 ]; then
        echo "  ⚠️  $file"
        conflicts_found=$((conflicts_found + 1))
    fi
done

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $conflicts_found -eq 0 ]; then
    echo "✅ No conflicts detected"
    exit 0
else
    echo "❌ Found $conflicts_found potential conflict(s)"
    echo ""
    echo "💡 Fix: Remove &World parameter and use Query parameters instead"
    echo "   Example: Query<&PackLeader> instead of world.get::<PackLeader>()"
    exit 1
fi
