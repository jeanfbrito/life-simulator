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

// Entity rendering configuration - loaded from API
// This will be populated by loadSpeciesConfig()
export let ENTITY_CONFIG = {
    'default': {
        emoji: '❓',
        sizeMultiplier: 1.0,
        offsetX: 0.0,
        offsetY: -0.2
    }
};

// Juvenile scaling data - loaded from API
export let JUVENILE_SCALES = {};

// Load species configuration from the backend API
export async function loadSpeciesConfig() {
    try {
        const response = await fetch(`${CONFIG.apiBaseUrl}/api/species`);
        if (!response.ok) {
            throw new Error(`Failed to load species config: ${response.status}`);
        }

        const data = await response.json();

        // Set default entity config from API
        if (data.default_entity) {
            ENTITY_CONFIG.default = data.default_entity;
        }

        // Set species-specific configs
        for (const [speciesName, speciesData] of Object.entries(data.species)) {
            ENTITY_CONFIG[speciesName] = {
                emoji: speciesData.emoji,
                sizeMultiplier: speciesData.viewer_scale,
                offsetX: 0.0,
                offsetY: -0.2  // Default Y offset to keep feet in grid
            };
        }

        // Set juvenile scales
        JUVENILE_SCALES = data.juvenile_scales || {};

        console.log('✅ Species configuration loaded from API:', {
            speciesCount: Object.keys(data.species).length,
            species: Object.keys(data.species),
            juvenileScales: Object.keys(JUVENILE_SCALES)
        });

        return true;
    } catch (error) {
        console.warn('⚠️ Failed to load species configuration from API, using defaults:', error);
        return false;
    }
}

// Default values
export const DEFAULTS = {
    terrainType: 'DeepWater',
    centerChunk: { x: 0, y: 0 },
    dragOffset: { x: 0, y: 0 },
    dragStart: { x: 0, y: 0 }
};
