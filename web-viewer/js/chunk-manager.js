/**
 * Chunk management and loading system for the Life Simulator Viewer
 */

import { CONFIG } from './config.js';

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
                const chunkKey = `${chunkX},${chunkY}`;

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

        for (let i = 0; i < chunkArray.length; i += CONFIG.chunkBatchSize) {
            const batch = chunkArray.slice(i, i + CONFIG.chunkBatchSize);
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

    // HTTP-based data fetching with CORS workaround
    async fetchData(endpoint) {
        console.log(`Fetching: ${CONFIG.apiBaseUrl}${endpoint}`);

        try {
            // Try fetch first, fallback to our proxy if CORS fails
            const response = await fetch(`${CONFIG.apiBaseUrl}${endpoint}`, {
                mode: 'cors',
                credentials: 'omit'
            });

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

    // Function to load chunks around the visible area (debounced)
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
        }, CONFIG.chunkLoadDebounce);
    }

    // Function to load chunks around the visible area
    async loadVisibleChunks(dragOffset, worldData) {
        // Calculate the center of the current view in world coordinates
        const viewCenterWorldX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE) + Math.floor(CONFIG.VIEW_SIZE_X / 2);
        const viewCenterWorldY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE) + Math.floor(CONFIG.VIEW_SIZE_Y / 2);

        const centerChunkX = Math.floor(viewCenterWorldX / 16);
        const centerChunkY = Math.floor(viewCenterWorldY / 16);

        // Only load if we've moved significantly from last loaded center
        const distanceX = Math.abs(centerChunkX - this.lastLoadedCenter.x);
        const distanceY = Math.abs(centerChunkY - this.lastLoadedCenter.y);

        if (distanceX < 1 && distanceY < 1) {
            return false; // Not enough movement to trigger new loading
        }

        this.lastLoadedCenter = { x: centerChunkX, y: centerChunkY };

        // Calculate the actual visible area in world coordinates
        const viewStartWorldX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE);
        const viewStartWorldY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE);
        const viewEndWorldX = viewStartWorldX + CONFIG.VIEW_SIZE_X;
        const viewEndWorldY = viewStartWorldY + CONFIG.VIEW_SIZE_Y;

        // Convert to chunk coordinates and add buffer
        const startChunkX = Math.floor(viewStartWorldX / 16) - 1; // Add 1 chunk buffer
        const startChunkY = Math.floor(viewStartWorldY / 16) - 1;
        const endChunkX = Math.floor(viewEndWorldX / 16) + 1; // Add 1 chunk buffer
        const endChunkY = Math.floor(viewEndWorldY / 16) + 1;

        // Calculate radius from the bounds
        const radiusX = Math.abs(centerChunkX - startChunkX);
        const radiusY = Math.abs(centerChunkY - startChunkY);
        const visibleRadius = Math.max(radiusX, radiusY, 3); // Minimum radius of 3

        console.log(`📦 Loading chunks around (${centerChunkX}, ${centerChunkY}) with radius ${visibleRadius}`);
        const newData = await this.requestChunksInArea(centerChunkX, centerChunkY, visibleRadius);
        
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
