/**
 * Configuration and constants for the Life Simulator Viewer
 */

const DEFAULT_API_PORT = 54321;

function determineApiBaseUrl() {
    if (typeof window !== 'undefined' && window.location && window.location.origin) {
        return window.location.origin;
    }
    return `http://localhost:${DEFAULT_API_PORT}`;
}

// Display configuration
export const CONFIG = {
    // Grid and rendering
    TILE_SIZE: 8,           // Pixels per tile (8x8)
    VIEW_SIZE_X: 100,       // Viewport width in tiles
    VIEW_SIZE_Y: 100,       // Viewport height in tiles

    // Chunk system
    CHUNK_SIZE: 16,              // Tiles per chunk (16x16)
    CHUNKS_PER_REQUEST: 10,      // Max chunks to fetch in one HTTP request
    CHUNK_LOAD_DEBOUNCE_MS: 100, // Delay before loading visible chunks (milliseconds)

    // Backwards compatibility with existing property names
    chunkLoadRadius: 5,          // Load radius for chunks around player
    chunkLoadDebounce: 100,      // Alias for CHUNK_LOAD_DEBOUNCE_MS
    chunkBatchSize: 10,          // Alias for CHUNKS_PER_REQUEST
    initialChunkRadius: 5,       // Initial chunk load radius

    // Network and polling
    ENTITY_POLL_INTERVAL_MS: 500,        // How often to fetch entities (milliseconds)
    FETCH_TIMEOUT_MS: 5000,              // Default timeout for API requests (milliseconds)
    CHUNK_FETCH_TIMEOUT_MS: 10000,       // Longer timeout for chunk loading (milliseconds)

    // Performance and throttling
    TOOLTIP_THROTTLE_MS: 100,            // Min time between tooltip updates (milliseconds)
    BIOMASS_FETCH_INTERVAL_MS: 5000,     // How often to fetch biomass data (milliseconds)

    // Circuit breaker
    MAX_FAILURES: 5,                     // Failures before circuit opens
    MAX_BACKOFF_INTERVAL_MS: 10000,      // Max retry interval during backoff (milliseconds)

    // Camera and controls
    PAN_SMOOTHING_FACTOR: 0.2,           // Camera catch-up easing (0-1, lower = faster)
    INERTIA_FRICTION: 0.90,              // Velocity decay after drag release (0-1)
    MIN_INERTIA_SPEED: 0.15,             // Minimum speed before stopping inertia
    ZOOM_MULTIPLIER: 1.25,               // Zoom step size (1.25 = 25% per step)

    // Display and rendering
    renderScale: 1.0,        // Scale factor for rendering (affected by zoom)
    MIN_ZOOM: 0.25,          // Minimum zoom level
    MAX_ZOOM: 4.0,           // Maximum zoom level
    minZoom: 0.25,           // Backwards compatibility alias
    maxZoom: 4.0,            // Backwards compatibility alias
    zoomFactor: 1.25,        // Backwards compatibility alias

    // Performance settings
    targetFPS: 60,           // Target frames per second
    frameDelay: 1000 / 60,   // Milliseconds per frame

    // Panning smoothing (backwards compatibility)
    panSmoothing: 0.2,      // 0..1 how fast the camera catches up to the target
    inertiaFriction: 0.90,  // 0..1 how quickly inertia slows down
    inertiaMinSpeed: 0.15,  // px/frame threshold to stop inertia

    // Grass density visualization
    showGrassDensity: false, // Toggle for grass density overlay

    // Network settings
    apiBaseUrl: determineApiBaseUrl(),
    connectionTimeout: 5000, // API connection timeout (milliseconds)
};

// Terrain colors with improved visibility and contrast
export const TERRAIN_COLORS = {
    'Grass': '#4ade80',        // Keep (good green)
    'Water': '#2563eb',        // Darker blue (was #4a90e2)
    'DeepWater': '#1e40af',    // Keep (dark blue)
    'Sand': '#fbbf24',         // Keep (yellow-orange)
    'Stone': '#78716c',        // Darker brown (was #8b8680)
    'Forest': '#166534',       // Keep (dark green)
    'Mountain': '#d1d5db',     // Much lighter gray (was #a8a8a8)
    'Snow': '#f0f9ff',         // Keep (white)
    'Desert': '#fed7aa',       // Keep (tan)
    'Swamp': '#064e3b',        // Keep (dark teal)
    'Dirt': '#92400e',         // Keep (brown)
    'ShallowWater': '#60a5fa', // Lighter blue (was #5ca7d8)
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
