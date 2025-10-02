/**
 * Rendering engine for the Life Simulator Viewer
 */

import { CONFIG, TERRAIN_COLORS, RESOURCE_CONFIG, RESOURCE_SYMBOLS, DEFAULTS } from './config.js';

export class Renderer {
    constructor(canvas, ctx) {
        this.canvas = canvas;
        this.ctx = ctx;
        this.setupCanvas();
    }

    setupCanvas() {
        // Set canvas rendering properties
        this.canvas.style.imageRendering = 'pixelated';
        this.canvas.style.cursor = 'pointer';
        this.ctx.imageSmoothingEnabled = false;
    }

    setupCanvasSize(dragOffset) {
        const container = this.canvas.parentElement;
        const containerRect = container.getBoundingClientRect();

        // Store old view size for drag offset adjustment
        const oldViewSizeX = CONFIG.VIEW_SIZE_X || 100;
        const oldViewSizeY = CONFIG.VIEW_SIZE_Y || 100;

        // Calculate tile size based on zoom
        CONFIG.TILE_SIZE = Math.max(4, Math.floor(8 * CONFIG.renderScale));

        // Calculate exact view size needed to fill canvas
        const viewWidth = Math.floor(containerRect.width / CONFIG.TILE_SIZE);
        const viewHeight = Math.floor(containerRect.height / CONFIG.TILE_SIZE);

        // Set view size to exactly match container dimensions
        CONFIG.VIEW_SIZE_X = Math.max(10, viewWidth);
        CONFIG.VIEW_SIZE_Y = Math.max(10, viewHeight);

        // Adjust drag offset proportionally to maintain position relative to zoom
        if (oldViewSizeX !== 0 && oldViewSizeY !== 0) {
            const scaleX = CONFIG.VIEW_SIZE_X / oldViewSizeX;
            const scaleY = CONFIG.VIEW_SIZE_Y / oldViewSizeY;
            dragOffset.x *= scaleX;
            dragOffset.y *= scaleY;
        }

        // Set canvas resolution to match container dimensions
        this.canvas.width = containerRect.width;
        this.canvas.height = containerRect.height;

        // Make canvas fill entire container
        this.canvas.style.width = containerRect.width + 'px';
        this.canvas.style.height = containerRect.height + 'px';
        this.canvas.style.left = '0';
        this.canvas.style.top = '0';
        this.canvas.style.transform = 'none';
    }

    render(worldData, dragOffset) {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        // Apply drag offset by translating the canvas
        this.ctx.save();
        this.ctx.translate(dragOffset.x, dragOffset.y);

        // Collect resource data for second pass rendering
        const resourcesToRender = [];

        // First pass: Render terrain layers
        const stats = this.renderTerrain(worldData, resourcesToRender);

        // Second pass: Render resources on top (allows overflow beyond tile boundaries)
        this.renderResources(resourcesToRender);

        this.ctx.restore(); // Restore the drag offset translation

        return stats;
    }

    renderTerrain(worldData, resourcesToRender) {
        const startX = 0;
        const startY = 0;
        const endX = CONFIG.VIEW_SIZE_X;
        const endY = CONFIG.VIEW_SIZE_Y;

        // Track terrain statistics for display
        let stats = {
            totalTiles: 0,
            walkableTiles: 0,
            resourceCount: 0,
            waterTiles: 0,
            forestTiles: 0
        };

        for (let y = startY; y < endY; y++) {
            for (let x = startX; x < endX; x++) {
                // Simple fixed coordinate system: render area around chunk 0,0
                const worldX = x - Math.floor(CONFIG.VIEW_SIZE_X / 2);
                const worldY = y - Math.floor(CONFIG.VIEW_SIZE_Y / 2);

                // Get chunk coordinates
                const chunkX = Math.floor(worldX / 16);
                const chunkY = Math.floor(worldY / 16);
                const localX = ((worldX % 16) + 16) % 16;
                const localY = ((worldY % 16) + 16) % 16;

                const chunkKey = `${chunkX},${chunkY}`;
                let terrainType = DEFAULTS.terrainType;

                // Access the actual terrain data from loaded chunks
                if (worldData.chunks[chunkKey] &&
                    worldData.chunks[chunkKey][localY] &&
                    worldData.chunks[chunkKey][localY][localX]) {
                    terrainType = worldData.chunks[chunkKey][localY][localX];
                }

                // Draw tile
                const color = TERRAIN_COLORS[terrainType] || TERRAIN_COLORS[DEFAULTS.terrainType];
                this.ctx.fillStyle = color;
                this.ctx.fillRect(x * CONFIG.TILE_SIZE, y * CONFIG.TILE_SIZE, CONFIG.TILE_SIZE, CONFIG.TILE_SIZE);

                // Collect resource data for later rendering
                let resourceType = '';
                if (worldData.resources[chunkKey] &&
                    worldData.resources[chunkKey][localY] &&
                    worldData.resources[chunkKey][localY][localX]) {
                    resourceType = worldData.resources[chunkKey][localY][localX];
                }

                if (resourceType && resourceType !== '') {
                    resourcesToRender.push({
                        type: resourceType,
                        x: x,
                        y: y
                    });
                    stats.resourceCount++;
                }

                // Draw subtle grid lines
                this.ctx.strokeStyle = 'rgba(255, 255, 255, 0.1)';
                this.ctx.lineWidth = 0.3;
                this.ctx.strokeRect(x * CONFIG.TILE_SIZE, y * CONFIG.TILE_SIZE, CONFIG.TILE_SIZE, CONFIG.TILE_SIZE);

                // Count terrain types for statistics
                stats.totalTiles++;
                if (terrainType !== 'Water' && terrainType !== 'DeepWater' && terrainType !== 'Mountain') {
                    stats.walkableTiles++;
                }
                if (terrainType === 'Water' || terrainType === 'DeepWater' || terrainType === 'ShallowWater') {
                    stats.waterTiles++;
                }
                if (terrainType === 'Forest') {
                    stats.forestTiles++;
                }
            }
        }

        return stats;
    }

    renderResources(resourcesToRender) {
        this.ctx.save(); // Save context state
        for (const resource of resourcesToRender) {
            // Get resource configuration or use defaults
            const config = RESOURCE_CONFIG[resource.type] || {
                sizeMultiplier: 0.8,
                offsetX: 0,
                offsetY: 0
            };

            // Calculate position with offsets
            const baseX = resource.x * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
            const baseY = resource.y * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
            const centerX = baseX + (config.offsetX * CONFIG.TILE_SIZE);
            const centerY = baseY + (config.offsetY * CONFIG.TILE_SIZE);

            // Draw resource emoji with dynamic sizing
            this.ctx.font = `${CONFIG.TILE_SIZE * config.sizeMultiplier}px Arial`;
            this.ctx.textAlign = 'center';
            this.ctx.textBaseline = 'middle';

            // Add subtle shadow for better visibility
            this.ctx.shadowColor = 'rgba(0, 0, 0, 0.6)';
            this.ctx.shadowBlur = 1;
            this.ctx.fillStyle = '#ffffff';
            this.ctx.fillText(RESOURCE_SYMBOLS[resource.type] || '•', centerX, centerY);
            this.ctx.shadowBlur = 0; // Reset shadow
        }
        this.ctx.restore(); // Restore context state
    }

    updateStatsDisplay(stats) {
        if (stats.totalTiles > 0) {
            document.getElementById('total-tiles').textContent = stats.totalTiles;
            document.getElementById('walkable-percentage').textContent =
                Math.round((stats.walkableTiles / stats.totalTiles) * 100) + '%';
            document.getElementById('water-percentage').textContent =
                Math.round((stats.waterTiles / stats.totalTiles) * 100) + '%';
            document.getElementById('forest-percentage').textContent =
                Math.round((stats.forestTiles / stats.totalTiles) * 100) + '%';
            document.getElementById('resource-count').textContent = stats.resourceCount;
        }
    }
}