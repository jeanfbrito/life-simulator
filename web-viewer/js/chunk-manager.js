/**
 * Chunk management and loading system for the Life Simulator Viewer
 */

import { CONFIG } from './config.js';
import { fetchWithTimeout } from './utils/fetch-timeout.js';
import { CoordinateConverter } from './utils/coordinates.js';

export class ChunkManager {
    constructor() {
        this.loadedChunks = new Set(); // Track which chunks we've loaded
        this.loadingChunks = new Set(); // Track chunks currently being loaded
        this.chunkLoadTimeout = null; // Debounce timer for chunk loading
        this.lastLoadedCenter = { x: 0, y: 0 }; // Track last loaded center to avoid duplicate requests
    }

    async requestChunks(centerCoord) {
        // Request initial chunks around the center
        const radius = CONFIG.initialChunkRadius; // Load 7x7 chunks (from -3 to +3)
        return await this.requestChunksInArea(centerCoord.x, centerCoord.y, radius);
    }

    async requestChunksInArea(centerX, centerY, radius) {
        const neededChunks = new Set();
        const allLoadedData = { chunks: {}, resources: {} };

        // Calculate which chunks we need
        for (let dx = -radius; dx <= radius; dx++) {
            for (let dy = -radius; dy <= radius; dy++) {
                const chunkX = centerX + dx;
                const chunkY = centerY + dy;
                const chunkKey = CoordinateConverter.chunkKey(chunkX, chunkY);

                if (!this.loadedChunks.has(chunkKey) && !this.loadingChunks.has(chunkKey)) {
                    neededChunks.add(chunkKey);
                }
            }
        }

        if (neededChunks.size === 0) {
            return allLoadedData; // No new chunks to load
        }

        // Mark chunks as being loaded
        neededChunks.forEach(chunkKey => this.loadingChunks.add(chunkKey));

        // Split requests into smaller batches to avoid URL length issues
        const chunkArray = Array.from(neededChunks);

        for (let i = 0; i < chunkArray.length; i += CONFIG.CHUNKS_PER_REQUEST) {
            const batch = chunkArray.slice(i, i + CONFIG.CHUNKS_PER_REQUEST);
            const batchData = await this.loadChunkBatch(batch);
            if (batchData) {
                // Merge batch data with accumulated data
                Object.assign(allLoadedData.chunks, batchData.chunks);
                Object.assign(allLoadedData.resources, batchData.resources);
            }
        }

        return allLoadedData;
    }

    /**
     * Request all chunks in a rectangular area (more efficient for screen-aligned views)
     * @param {number} startX - Start chunk X coordinate
     * @param {number} startY - Start chunk Y coordinate
     * @param {number} endX - End chunk X coordinate (inclusive)
     * @param {number} endY - End chunk Y coordinate (inclusive)
     */
    async requestChunksInRect(startX, startY, endX, endY) {
        const neededChunks = new Set();
        const allLoadedData = { chunks: {}, resources: {} };

        // Calculate which chunks we need in the rectangular area
        for (let chunkX = startX; chunkX <= endX; chunkX++) {
            for (let chunkY = startY; chunkY <= endY; chunkY++) {
                const chunkKey = CoordinateConverter.chunkKey(chunkX, chunkY);

                if (!this.loadedChunks.has(chunkKey) && !this.loadingChunks.has(chunkKey)) {
                    neededChunks.add(chunkKey);
                }
            }
        }

        if (neededChunks.size === 0) {
            return allLoadedData; // No new chunks to load
        }

        // Mark chunks as being loaded
        neededChunks.forEach(chunkKey => this.loadingChunks.add(chunkKey));

        // Split requests into smaller batches to avoid URL length issues
        const chunkArray = Array.from(neededChunks);

        for (let i = 0; i < chunkArray.length; i += CONFIG.CHUNKS_PER_REQUEST) {
            const batch = chunkArray.slice(i, i + CONFIG.CHUNKS_PER_REQUEST);
            const batchData = await this.loadChunkBatch(batch);
            if (batchData) {
                // Merge batch data with accumulated data
                Object.assign(allLoadedData.chunks, batchData.chunks);
                Object.assign(allLoadedData.resources, batchData.resources);
            }
        }

        return allLoadedData;
    }

    async loadChunkBatch(batch) {
        const coordsQuery = batch.map(c => `coords=${c}`).join('&');

        console.log('📦 CHUNK_MANAGER: Loading batch:', batch);

        try {
            const data = await this.fetchData(`/api/chunks?${coordsQuery}&layers=true`);
            console.log('📦 CHUNK_MANAGER: Received data:', data);

            if (data && data.chunk_data) {
                const newWorldData = { chunks: {}, resources: {} };

                for (const [chunkKey, chunkData] of Object.entries(data.chunk_data)) {
                    // Store the chunk data
                    if (chunkData.terrain) {
                        newWorldData.chunks[chunkKey] = chunkData.terrain;
                    }
                    if (chunkData.resources) {
                        newWorldData.resources[chunkKey] = chunkData.resources;
                    }

                    // Mark as loaded
                    this.loadedChunks.add(chunkKey);
                    this.loadingChunks.delete(chunkKey);
                }

                console.log('📦 CHUNK_MANAGER: Loaded chunks:', Object.keys(newWorldData.chunks));
                return newWorldData;
            }
        } catch (error) {
            console.error('❌ CHUNK_MANAGER: Error loading chunks:', error);
            // Remove from loading set on error
            batch.forEach(chunkKey => this.loadingChunks.delete(chunkKey));
        }

        return null;
    }

    /**
     * HTTP-based data fetching with CORS workaround and timeout protection
     * Uses longer timeout for chunk loading due to larger data size
     * @param {string} endpoint - API endpoint to fetch from
     * @returns {Object} Parsed JSON response data
     * @throws {Error} If fetch fails or times out
     */
    async fetchData(endpoint) {
        console.log(`Fetching: ${CONFIG.apiBaseUrl}${endpoint}`);

        try {
            // Try fetch first with timeout protection
            const response = await fetchWithTimeout(`${CONFIG.apiBaseUrl}${endpoint}`, {
                mode: 'cors',
                credentials: 'omit'
            }, CONFIG.CHUNK_FETCH_TIMEOUT_MS);

            console.log(`Response status: ${response.status}`);
            if (!response.ok) {
                this.updateConnectionStatus(false);
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();
            console.log('Successfully received data:', data);
            this.updateConnectionStatus(true);
            return data;
        } catch (error) {
            this.updateConnectionStatus(false);
            throw error;
        }
    }

    /**
     * Load chunks around visible area with debounce to avoid excessive API calls
     * Delays loading by CONFIG.CHUNK_LOAD_DEBOUNCE_MS to coalesce rapid movement requests
     * @param {Object} dragOffset - Current camera offset {x, y}
     * @param {Object} worldData - Current world data object to merge into
     * @param {Function} onChunksLoaded - Callback to trigger render after loading
     */
    loadVisibleChunksDebounced(dragOffset, worldData, onChunksLoaded) {
        // Clear existing timeout
        if (this.chunkLoadTimeout) {
            clearTimeout(this.chunkLoadTimeout);
        }

        // Set new timeout
        this.chunkLoadTimeout = setTimeout(async () => {
            const loaded = await this.loadVisibleChunks(dragOffset, worldData);
            // Trigger a render if chunks were loaded
            if (loaded && onChunksLoaded) {
                onChunksLoaded();
            }
        }, CONFIG.CHUNK_LOAD_DEBOUNCE_MS);
    }

    // Function to load chunks around the visible area
    async loadVisibleChunks(dragOffset, worldData) {
        // Calculate camera offset (matches renderer's coordinate system)
        const cameraOffsetX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE);
        const cameraOffsetY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE);

        // Calculate the actual visible area in world coordinates
        // This matches the renderer's formula: worldX = screenX + cameraOffsetX - floor(VIEW_SIZE_X/2)
        const viewStartWorldX = cameraOffsetX - Math.floor(CONFIG.VIEW_SIZE_X / 2);
        const viewStartWorldY = cameraOffsetY - Math.floor(CONFIG.VIEW_SIZE_Y / 2);
        const viewEndWorldX = cameraOffsetX + Math.ceil(CONFIG.VIEW_SIZE_X / 2);
        const viewEndWorldY = cameraOffsetY + Math.ceil(CONFIG.VIEW_SIZE_Y / 2);

        // Calculate the center of the current view in world coordinates
        const viewCenterWorldX = Math.floor((viewStartWorldX + viewEndWorldX) / 2);
        const viewCenterWorldY = Math.floor((viewStartWorldY + viewEndWorldY) / 2);

        // Convert to chunk coordinates
        const centerChunk = CoordinateConverter.worldToChunk(viewCenterWorldX, viewCenterWorldY);
        const centerChunkX = centerChunk.chunkX;
        const centerChunkY = centerChunk.chunkY;

        // Only load if we've moved significantly from last loaded center
        const distanceX = Math.abs(centerChunkX - this.lastLoadedCenter.x);
        const distanceY = Math.abs(centerChunkY - this.lastLoadedCenter.y);

        if (distanceX < 1 && distanceY < 1) {
            return false; // Not enough movement to trigger new loading
        }

        this.lastLoadedCenter = { x: centerChunkX, y: centerChunkY };

        // Convert visible bounds to chunk coordinates and add buffer
        const startChunk = CoordinateConverter.worldToChunk(viewStartWorldX, viewStartWorldY);
        const endChunk = CoordinateConverter.worldToChunk(viewEndWorldX, viewEndWorldY);
        const startChunkX = startChunk.chunkX - 1; // Add 1 chunk buffer
        const startChunkY = startChunk.chunkY - 1;
        const endChunkX = endChunk.chunkX + 1; // Add 1 chunk buffer
        const endChunkY = endChunk.chunkY + 1;

        // Load all chunks in the rectangular visible area (not just circular radius)
        const newData = await this.requestChunksInRect(startChunkX, startChunkY, endChunkX, endChunkY);

        console.log(`📦 Loading chunks in rect (${startChunkX},${startChunkY}) to (${endChunkX},${endChunkY})`);
        
        // Merge newly loaded chunks into worldData if provided
        if (newData && worldData) {
            const chunkCount = Object.keys(newData.chunks || {}).length;
            if (chunkCount > 0) {
                console.log(`✅ Loaded ${chunkCount} new chunks, merging into worldData`);
                this.mergeChunkData(newData, worldData);
                return true; // Return true to indicate chunks were loaded
            }
        }
        return false;
    }

    async loadWorldInfo() {
        try {
            // Load current world info
            const currentData = await this.fetchData('/api/world/current');
            if (currentData) {
                document.getElementById('current-world').textContent = currentData.name;
                document.getElementById('current-seed').textContent = currentData.seed;
            }

            // Load additional world info
            const worldData = await this.fetchData('/api/world_info');
            if (worldData) {
                document.getElementById('total-chunks').textContent = worldData.chunk_count;
            }

            return true;
        } catch (error) {
            console.error('Failed to load world info:', error);
            document.getElementById('current-world').textContent = 'Error';
            document.getElementById('current-seed').textContent = 'Error';
            document.getElementById('total-chunks').textContent = 'Error';
            return false;
        }
    }

    updateChunkCount(worldStats) {
        if (worldStats) {
            document.getElementById('chunk-count').textContent =
                this.loadedChunks.size + '/' + worldStats.total_chunks;
        }
    }

    mergeChunkData(newData, existingWorldData) {
        if (newData && newData.chunks) {
            Object.assign(existingWorldData.chunks, newData.chunks);
        }
        if (newData && newData.resources) {
            Object.assign(existingWorldData.resources, newData.resources);
        }
    }

    updateConnectionStatus(connected) {
        const status = document.getElementById('connection-status');
        if (connected) {
            status.className = 'status-item connected';
            status.innerHTML = '<span class="status-dot">🟢</span><span>Connected (HTTP)</span>';
        } else {
            status.className = 'status-item disconnected';
            status.innerHTML = '<span class="status-dot">🔴</span><span>Disconnected</span>';
        }
    }

    clear() {
        this.loadedChunks.clear();
        this.loadingChunks.clear();
        if (this.chunkLoadTimeout) {
            clearTimeout(this.chunkLoadTimeout);
            this.chunkLoadTimeout = null;
        }
        this.lastLoadedCenter = { x: 0, y: 0 };
    }
}
