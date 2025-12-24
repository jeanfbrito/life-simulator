/**
 * Test suite for CoordinateConverter utility module
 * Tests coordinate conversions between screen, world, and chunk systems
 */

import { CoordinateConverter } from '../js/utils/coordinates.js';

// Mock CONFIG for testing
globalThis.CONFIG = {
    TILE_SIZE: 8,
    VIEW_SIZE_X: 100,
    VIEW_SIZE_Y: 100
};

// Test 1: Screen to World conversion
function testScreenToWorld() {
    console.log('TEST 1: Screen to World conversion');

    const result = CoordinateConverter.screenToWorld(50, 60, 100, 100);

    console.assert(result.x === 0, `Expected screenToWorld(50, 60).x === 0, got ${result.x}`);
    console.assert(result.y === 10, `Expected screenToWorld(50, 60).y === 10, got ${result.y}`);

    // Test with centered coordinates
    const centered = CoordinateConverter.screenToWorld(100, 100, 200, 200);
    console.assert(centered.x === 0, `Expected centered.x === 0, got ${centered.x}`);
    console.assert(centered.y === 0, `Expected centered.y === 0, got ${centered.y}`);

    console.log('✓ Screen to World conversion test passed\n');
}

// Test 2: World to Screen conversion
function testWorldToScreen() {
    console.log('TEST 2: World to Screen conversion');

    const result = CoordinateConverter.worldToScreen(0, 10, 100, 100);

    console.assert(result.x === 50, `Expected worldToScreen(0, 10).x === 50, got ${result.x}`);
    console.assert(result.y === 60, `Expected worldToScreen(0, 10).y === 60, got ${result.y}`);

    // Test with negative world coordinates
    const negative = CoordinateConverter.worldToScreen(-10, -20, 100, 100);
    console.assert(negative.x === 40, `Expected negative.x === 40, got ${negative.x}`);
    console.assert(negative.y === 30, `Expected negative.y === 30, got ${negative.y}`);

    console.log('✓ World to Screen conversion test passed\n');
}

// Test 3: World to Chunk conversion
function testWorldToChunk() {
    console.log('TEST 3: World to Chunk conversion');

    // Test basic chunk conversion
    const chunk1 = CoordinateConverter.worldToChunk(0, 0);
    console.assert(chunk1.chunkX === 0, `Expected chunk1.chunkX === 0, got ${chunk1.chunkX}`);
    console.assert(chunk1.chunkY === 0, `Expected chunk1.chunkY === 0, got ${chunk1.chunkY}`);
    console.assert(chunk1.localX === 0, `Expected chunk1.localX === 0, got ${chunk1.localX}`);
    console.assert(chunk1.localY === 0, `Expected chunk1.localY === 0, got ${chunk1.localY}`);

    // Test with positive world coordinates
    const chunk2 = CoordinateConverter.worldToChunk(20, 20, 16);
    console.assert(chunk2.chunkX === 1, `Expected chunk2.chunkX === 1, got ${chunk2.chunkX}`);
    console.assert(chunk2.chunkY === 1, `Expected chunk2.chunkY === 1, got ${chunk2.chunkY}`);
    console.assert(chunk2.localX === 4, `Expected chunk2.localX === 4, got ${chunk2.localX}`);
    console.assert(chunk2.localY === 4, `Expected chunk2.localY === 4, got ${chunk2.localY}`);

    // Test with negative world coordinates (wrapping)
    const chunk3 = CoordinateConverter.worldToChunk(-5, -5, 16);
    console.assert(chunk3.chunkX === -1, `Expected chunk3.chunkX === -1, got ${chunk3.chunkX}`);
    console.assert(chunk3.chunkY === -1, `Expected chunk3.chunkY === -1, got ${chunk3.chunkY}`);
    console.assert(chunk3.localX === 11, `Expected chunk3.localX === 11, got ${chunk3.localX}`);
    console.assert(chunk3.localY === 11, `Expected chunk3.localY === 11, got ${chunk3.localY}`);

    console.log('✓ World to Chunk conversion test passed\n');
}

// Test 4: Chunk key creation and parsing
function testChunkKeyOperations() {
    console.log('TEST 4: Chunk key operations');

    // Test chunk key creation
    const key = CoordinateConverter.chunkKey(5, -3);
    console.assert(key === '5,-3', `Expected '5,-3', got '${key}'`);

    // Test chunk key parsing
    const parsed = CoordinateConverter.parseChunkKey('5,-3');
    console.assert(parsed.chunkX === 5, `Expected parsed.chunkX === 5, got ${parsed.chunkX}`);
    console.assert(parsed.chunkY === -3, `Expected parsed.chunkY === -3, got ${parsed.chunkY}`);

    // Test round-trip conversion
    const original = { x: 10, y: -7 };
    const keyRoundTrip = CoordinateConverter.chunkKey(original.x, original.y);
    const parsedRoundTrip = CoordinateConverter.parseChunkKey(keyRoundTrip);
    console.assert(parsedRoundTrip.chunkX === original.x, 'Round trip failed for chunkX');
    console.assert(parsedRoundTrip.chunkY === original.y, 'Round trip failed for chunkY');

    console.log('✓ Chunk key operations test passed\n');
}

// Test 5: Canvas to World conversion
function testCanvasToWorld() {
    console.log('TEST 5: Canvas to World conversion');

    const result = CoordinateConverter.canvasToWorld(400, 480, { x: 0, y: 0 });

    console.assert(result.screenX === 50, `Expected screenX === 50, got ${result.screenX}`);
    console.assert(result.screenY === 60, `Expected screenY === 60, got ${result.screenY}`);
    console.assert(result.worldX === 0, `Expected worldX === 0, got ${result.worldX}`);
    console.assert(result.worldY === 10, `Expected worldY === 10, got ${result.worldY}`);

    console.log('✓ Canvas to World conversion test passed\n');
}

// Test 6: World to Screen Pixels conversion
function testWorldToScreenPixels() {
    console.log('TEST 6: World to Screen Pixels conversion');

    const result = CoordinateConverter.worldToScreenPixels(0, 0, 0, 0);

    console.assert(result.screenPixelX === 404, `Expected screenPixelX === 404, got ${result.screenPixelX}`);
    console.assert(result.screenPixelY === 404, `Expected screenPixelY === 404, got ${result.screenPixelY}`);
    console.assert(result.screenTileY === 50, `Expected screenTileY === 50, got ${result.screenTileY}`);

    console.log('✓ World to Screen Pixels conversion test passed\n');
}

// Test 7: Bounds checking
function testIsWithinViewBounds() {
    console.log('TEST 7: Is within view bounds');

    console.assert(CoordinateConverter.isWithinViewBounds(0, 0) === true, 'Should be in bounds at (0, 0)');
    console.assert(CoordinateConverter.isWithinViewBounds(50, 50) === true, 'Should be in bounds at (50, 50)');
    console.assert(CoordinateConverter.isWithinViewBounds(99, 99) === true, 'Should be in bounds at (99, 99)');
    console.assert(CoordinateConverter.isWithinViewBounds(100, 100) === false, 'Should NOT be in bounds at (100, 100)');
    console.assert(CoordinateConverter.isWithinViewBounds(-1, 50) === false, 'Should NOT be in bounds at (-1, 50)');
    console.assert(CoordinateConverter.isWithinViewBounds(50, -1) === false, 'Should NOT be in bounds at (50, -1)');

    console.log('✓ Is within view bounds test passed\n');
}

// Test 8: Consistency across conversions
function testConsistency() {
    console.log('TEST 8: Consistency across conversions');

    // Test that screen->world->screen round trip is consistent
    const screenCoords = { x: 25, y: 75 };
    const world = CoordinateConverter.screenToWorld(screenCoords.x, screenCoords.y, 100, 100);
    const screenAgain = CoordinateConverter.worldToScreen(world.x, world.y, 100, 100);

    console.assert(screenAgain.x === screenCoords.x, 'Screen X round trip failed');
    console.assert(screenAgain.y === screenCoords.y, 'Screen Y round trip failed');

    // Test that chunk coordinates are within valid bounds
    const chunk = CoordinateConverter.worldToChunk(100, 100);
    console.assert(chunk.localX >= 0 && chunk.localX < 16, 'Local X out of bounds');
    console.assert(chunk.localY >= 0 && chunk.localY < 16, 'Local Y out of bounds');

    console.log('✓ Consistency test passed\n');
}

// Run all tests
console.log('=== Coordinate Converter Test Suite ===\n');

try {
    testScreenToWorld();
    testWorldToScreen();
    testWorldToChunk();
    testChunkKeyOperations();
    testCanvasToWorld();
    testWorldToScreenPixels();
    testIsWithinViewBounds();
    testConsistency();

    console.log('=== All tests passed! ===');
} catch (error) {
    console.error('Test failed with error:', error);
}
