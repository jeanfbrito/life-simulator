/**
 * Configuration and constants for the Life Simulator Viewer
 */

// Display configuration
export const CONFIG = {
    TILE_SIZE: 8, // Dynamic tile size for zoom functionality
    renderScale: 1.0, // Scale factor for rendering
    VIEW_SIZE_X: 100, // Dynamic view width based on container
    VIEW_SIZE_Y: 100, // Dynamic view height based on container

    // Performance settings
    targetFPS: 60,
    frameDelay: 1000 / 60,

    // Panning smoothing
    panSmoothing: 0.2,      // 0..1 how fast the camera catches up to the target
    inertiaFriction: 0.90,  // 0..1 how quickly inertia slows down
    inertiaMinSpeed: 0.15,  // px/frame threshold to stop inertia

    // Chunk loading settings
    chunkLoadRadius: 5,
    chunkLoadDebounce: 100, // ms delay for chunk loading
    chunkBatchSize: 10,
    initialChunkRadius: 3,

    // Zoom settings
    minZoom: 0.25,
    maxZoom: 4.0,
    zoomFactor: 1.25,

    // Network settings
    apiBaseUrl: 'http://localhost:54321',
    connectionTimeout: 5000,
};

// Terrain colors with improved visibility and contrast
export const TERRAIN_COLORS = {
    'Grass': '#3a7f47',      // Brighter grass green
    'Stone': '#8b8680',       // Lighter stone gray
    'Sand': '#f4d58f',       // Brighter sand yellow
    'Water': '#4a90e2',      // Brighter water blue
    'Dirt': '#8b6239',       // Richer dirt brown
    'Snow': '#f0f0f0',       // Slightly off-white snow
    'Forest': '#2d5a2d',     // Darker forest green
    'Mountain': '#a8a8a8',   // Lighter mountain gray
    'DeepWater': '#1e3a5f',  // Darker deep water
    'ShallowWater': '#5ca7d8', // Lighter shallow water
    'Swamp': '#5a6b3c',      // Brighter swamp green
    'Desert': '#d4a76a'      // Brighter desert tan
};

// Resource colors and symbols
export const RESOURCE_COLORS = {
    'TreeOak': '#0d4d0d',
    'TreePine': '#0d3d0d',
    'TreeBirch': '#1d5d1d',
    'Rock': '#5a5a5a',
    'Bush': '#2d4d2d',
    'Flower': '#ff69b4'
};

// Resource symbols for rendering
export const RESOURCE_SYMBOLS = {
    'TreeOak': '🌳',
    'TreePine': '🌲',
    'TreeBirch': '🪾',
    'Rock': '🪨',
    'Bush': '🌳',
    'Flower': '🌸'
};

// Resource rendering configuration with size and positioning
export const RESOURCE_CONFIG = {
    'TreeOak': {
        sizeMultiplier: 1.4,
        offsetX: 0,
        offsetY: -0.3
    },
    'TreePine': {
        sizeMultiplier: 1.6, // Double size (0.8 * 2.0 = 1.6)
        offsetX: 0,
        offsetY: -0.5 // Perfectly centered the base of the tree on the grid
    },
    'TreeBirch': {
        sizeMultiplier: 1.4,
        offsetX: 0,
        offsetY: -0.3
    },
    'Rock': {
        sizeMultiplier: 0.6,
        offsetX: 0,
        offsetY: 0.1
    },
    'Bush': {
        sizeMultiplier: 0.6,
        offsetX: 0,
        offsetY: 0.1
    },
    'Flower': {
        sizeMultiplier: 0.4,
        offsetX: 0,
        offsetY: 0
    }
};

// Entity rendering configuration with size and positioning
// sizeMultiplier: Size relative to tile (1.0 = tile size, 0.5 = half tile, etc.)
// offsetX/Y: Position offset in tiles (-0.2 = move up 20% of a tile)
// Adjust these values to make entities look right at different scales!
export const ENTITY_CONFIG = {
    'Human': {
        emoji: '🧍‍♂️',
        sizeMultiplier: 1.2,  // Standard human size
        offsetX: 0,
        offsetY: -0.2  // Move up to keep feet in grid
    },
    'Rabbit': {
        emoji: '🐇',
        sizeMultiplier: 0.5,  // Smaller than humans
        offsetX: 0,
        offsetY: -0.05  // Slightly less offset for smaller creature
    },
'Deer': {
        emoji: '🦌',
        sizeMultiplier: 0.9,  // Smaller for clearer testing
        offsetX: 0,
        offsetY: -0.18
    },
'Raccoon': {
        emoji: '🦝',
        sizeMultiplier: 0.65,
        offsetX: 0,
        offsetY: -0.12
    },
    'Wolf': {
        emoji: '🐺',
        sizeMultiplier: 1.0,  // Similar to human
        offsetX: 0,
        offsetY: -0.2
    },
    // Default for unknown entity types
    'default': {
        emoji: '❓',
        sizeMultiplier: 1.0,
        offsetX: 0,
        offsetY: -0.2
    }
};

// Default values
export const DEFAULTS = {
    terrainType: 'DeepWater',
    centerChunk: { x: 0, y: 0 },
    dragOffset: { x: 0, y: 0 },
    dragStart: { x: 0, y: 0 }
};
