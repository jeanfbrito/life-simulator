/**
 * Rendering engine for the Life Simulator Viewer
 */

import { CONFIG, TERRAIN_COLORS, RESOURCE_CONFIG, RESOURCE_SYMBOLS, ENTITY_CONFIG, JUVENILE_SCALES, DEFAULTS } from './config.js';
import { fetchWithTimeout } from './utils/fetch-timeout.js';
import { CoordinateConverter } from './utils/coordinates.js';

export class Renderer {
    constructor(canvas, ctx) {
        this.canvas = canvas;
        this.ctx = ctx;
        this.setupCanvas();
        this.biomassWarningShown = false;
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

    render(worldData, dragOffset, entities = []) {
        this.ctx.clearRect(0, 0, this.canvas.width, this.canvas.height);

        // Apply clipping to prevent content from rendering outside canvas
        this.ctx.save();
        this.ctx.beginPath();
        this.ctx.rect(0, 0, this.canvas.width, this.canvas.height);
        this.ctx.clip();

        // Calculate camera position in world tiles (integer part)
        const cameraOffsetX = Math.floor(-dragOffset.x / CONFIG.TILE_SIZE);
        const cameraOffsetY = Math.floor(-dragOffset.y / CONFIG.TILE_SIZE);

        // Calculate sub-pixel offset for smooth panning (remainder)
        const pixelOffsetX = -dragOffset.x % CONFIG.TILE_SIZE;
        const pixelOffsetY = -dragOffset.y % CONFIG.TILE_SIZE;

        // Apply sub-pixel translation for smooth movement
        this.ctx.translate(pixelOffsetX, pixelOffsetY);

        // Collect resource data for Y-sorted rendering
        const resourcesToRender = [];

        // First pass: Render terrain layers with camera offset
        const stats = this.renderTerrain(worldData, resourcesToRender, cameraOffsetX, cameraOffsetY);

        // Second pass: Y-sorted rendering of entities and resources for proper depth
        this.renderEntitiesAndResourcesSorted(entities, resourcesToRender, cameraOffsetX, cameraOffsetY);

        this.ctx.restore(); // Restore the translation and clipping

        return stats;
    }

    renderTerrain(worldData, resourcesToRender, cameraOffsetX, cameraOffsetY) {
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
                // Calculate world coordinates based on camera position
                const worldX = x + cameraOffsetX - Math.floor(CONFIG.VIEW_SIZE_X / 2);
                const worldY = y + cameraOffsetY - Math.floor(CONFIG.VIEW_SIZE_Y / 2);

                // Get chunk coordinates and local position
                const chunk = CoordinateConverter.worldToChunk(worldX, worldY);
                const chunkKey = CoordinateConverter.chunkKey(chunk.chunkX, chunk.chunkY);
                const chunkX = chunk.chunkX;
                const chunkY = chunk.chunkY;
                const localX = chunk.localX;
                const localY = chunk.localY;

                // Access the actual terrain data from loaded chunks with null-safe optional chaining
                const terrainType = worldData.chunks?.[chunkKey]?.[localY]?.[localX] ?? DEFAULTS.terrainType;

                // Draw tile
                let color = TERRAIN_COLORS[terrainType] || TERRAIN_COLORS[DEFAULTS.terrainType];

                // Apply grass density overlay if enabled and terrain supports grass
                if (CONFIG.showGrassDensity && this.biomassData && (terrainType === 'Grass' || terrainType === 'Forest' || terrainType === 'Dirt')) {
                    color = this.applyGrassDensityOverlay(color, chunkX, chunkY, localX, localY, terrainType);
                }

                this.ctx.fillStyle = color;
                this.ctx.fillRect(x * CONFIG.TILE_SIZE, y * CONFIG.TILE_SIZE, CONFIG.TILE_SIZE, CONFIG.TILE_SIZE);

                // Collect resource data for later rendering with null-safe optional chaining
                const resourceType = worldData.resources?.[chunkKey]?.[localY]?.[localX] ?? '';

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

    renderEntitiesAndResourcesSorted(entities, resourcesToRender, cameraOffsetX, cameraOffsetY) {
        // Create a combined list of entities and resources with Y-coordinates for sorting
        const renderList = [];

        // Add resources to render list with their Y positions
        for (const resource of resourcesToRender) {
            renderList.push({
                type: 'resource',
                data: resource,
                y: resource.y  // Screen Y coordinate
            });
        }

        // Add entities to render list with their Y positions
        for (const entity of entities) {
            if (!entity.position) continue;

            const entityWorldX = entity.position.x;
            const entityWorldY = entity.position.y;

            // Convert world coordinates to screen pixel coordinates
            const screenCoords = CoordinateConverter.worldToScreenPixels(entityWorldX, entityWorldY, cameraOffsetX, cameraOffsetY);

            // Only add if within visible bounds
            const ENTITY_RADIUS = Math.max(2, CONFIG.TILE_SIZE * 0.3);
            if (screenCoords.screenPixelX >= -ENTITY_RADIUS && screenCoords.screenPixelX <= (CONFIG.VIEW_SIZE_X * CONFIG.TILE_SIZE) + ENTITY_RADIUS &&
                screenCoords.screenPixelY >= -ENTITY_RADIUS && screenCoords.screenPixelY <= (CONFIG.VIEW_SIZE_Y * CONFIG.TILE_SIZE) + ENTITY_RADIUS) {

                renderList.push({
                    type: 'entity',
                    data: entity,
                    y: screenCoords.screenTileY,  // Use tile Y for sorting
                    screenX: screenCoords.screenPixelX,
                    screenY: screenCoords.screenPixelY
                });
            }
        }

        // Sort by Y coordinate (back to front)
        renderList.sort((a, b) => a.y - b.y);

        // Render in sorted order
        this.ctx.save();

        for (const item of renderList) {
            if (item.type === 'resource') {
                this.renderSingleResource(item.data);
            } else if (item.type === 'entity') {
                this.renderSingleEntity(item.data, item.screenX, item.screenY);
            }
        }

        this.ctx.restore();
    }

    renderSingleResource(resource) {
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

    renderSingleEntity(entity, screenX, screenY) {
        // Get entity configuration (or use default)
        const entityType = entity.entity_type || 'default';
        const config = ENTITY_CONFIG[entityType] || ENTITY_CONFIG['default'];
        
        // Draw entity as emoji with configured size
        this.ctx.font = `${CONFIG.TILE_SIZE * config.sizeMultiplier}px Arial`;
        this.ctx.textAlign = 'center';
        this.ctx.textBaseline = 'middle';
        
        // Add subtle shadow for better visibility
        this.ctx.shadowColor = 'rgba(0, 0, 0, 0.7)';
        this.ctx.shadowBlur = 2;
        this.ctx.shadowOffsetX = 1;
        this.ctx.shadowOffsetY = 1;
        
        // Calculate position with configured offsets
        const entityX = screenX + (CONFIG.TILE_SIZE * config.offsetX);
        const entityY = screenY + (CONFIG.TILE_SIZE * config.offsetY);
        
        // Map display: keep species emoji on map. Scale down juveniles per species.
        let juvenileScale = 1.0;
        if (entity.is_juvenile && JUVENILE_SCALES[entity.entity_type]) {
            juvenileScale = JUVENILE_SCALES[entity.entity_type];
        }
        const baseSize = CONFIG.TILE_SIZE * config.sizeMultiplier * juvenileScale;

        // Draw species emoji (e.g., 🐇 for rabbits)
        this.ctx.font = `${baseSize}px Arial`;
        this.ctx.fillStyle = '#ffffff';
        this.ctx.fillText(config.emoji, entityX, entityY);
        
        // Reset shadow
        this.ctx.shadowColor = 'transparent';
        this.ctx.shadowBlur = 0;
        this.ctx.shadowOffsetX = 0;
        this.ctx.shadowOffsetY = 0;
        
        // Draw current action label if present (and zoom is sufficient)
        if (entity.current_action && CONFIG.TILE_SIZE >= 8) {
            const fontSize = Math.max(8, CONFIG.TILE_SIZE * 0.5);
            this.ctx.font = `${fontSize}px Arial`;
            this.ctx.textAlign = 'center';
            this.ctx.textBaseline = 'bottom';
            
            // Draw text background for better readability
            const labelY = entityY - (CONFIG.TILE_SIZE * 0.6);
            const textMetrics = this.ctx.measureText(entity.current_action);
            const padding = 2;
            
            this.ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
            this.ctx.fillRect(
                entityX - textMetrics.width / 2 - padding,
                labelY - fontSize - padding,
                textMetrics.width + padding * 2,
                fontSize + padding * 2
            );
            
            // Draw text
            this.ctx.fillStyle = '#ffffff';
            this.ctx.fillText(entity.current_action, entityX, labelY);
        }
    }

    renderEntities(entities, cameraOffsetX, cameraOffsetY) {
        this.ctx.save();

        // Entity rendering configuration
        const ENTITY_RADIUS = Math.max(2, CONFIG.TILE_SIZE * 0.3);
        const ENTITY_COLORS = {
            default: '#ff4444',
            wanderer: '#44ff44',
            animal: '#ffaa44',
            person: '#4444ff'
        };

        for (const entity of entities) {
            if (!entity.position) continue;

            const entityWorldX = entity.position.x;
            const entityWorldY = entity.position.y;

            // Convert world coordinates to screen coordinates
            const screenX = (entityWorldX - cameraOffsetX + Math.floor(CONFIG.VIEW_SIZE_X / 2)) * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
            const screenY = (entityWorldY - cameraOffsetY + Math.floor(CONFIG.VIEW_SIZE_Y / 2)) * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;

            // Only render if entity is within visible bounds
            if (screenX >= -ENTITY_RADIUS && screenX <= (CONFIG.VIEW_SIZE_X * CONFIG.TILE_SIZE) + ENTITY_RADIUS &&
                screenY >= -ENTITY_RADIUS && screenY <= (CONFIG.VIEW_SIZE_Y * CONFIG.TILE_SIZE) + ENTITY_RADIUS) {

                // Draw entity as emoji
                this.ctx.font = `${CONFIG.TILE_SIZE * 1.2}px Arial`;
                this.ctx.textAlign = 'center';
                this.ctx.textBaseline = 'middle';
                
                // Add subtle shadow for better visibility
                this.ctx.shadowColor = 'rgba(0, 0, 0, 0.7)';
                this.ctx.shadowBlur = 2;
                this.ctx.shadowOffsetX = 1;
                this.ctx.shadowOffsetY = 1;
                
                // Render the emoji with Y offset to position feet above
                const entityY = screenY + (CONFIG.TILE_SIZE * -0.2); // Move up 0.2 tiles
                this.ctx.fillText('🧍‍♂️', screenX, entityY);
            }
        }

        // Reset shadow
        this.ctx.shadowColor = 'transparent';
        this.ctx.shadowBlur = 0;
        this.ctx.shadowOffsetX = 0;
        this.ctx.shadowOffsetY = 0;

        this.ctx.restore();
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

    // Fetch biomass data for grass density visualization
    async fetchBiomassData() {
        const url = `${CONFIG.apiBaseUrl}/api/vegetation/biomass`;

        try {
            const response = await fetchWithTimeout(url, {
                mode: 'cors',
                credentials: 'omit'
            }, 5000);

            if (!response.ok) {
                throw new Error(`HTTP ${response.status}`);
            }

            const data = await response.json();

            if (!Array.isArray(data.heatmap) || data.heatmap.length === 0) {
                if (!this.biomassWarningShown) {
                    console.warn('Biomass data response did not include a usable heatmap.');
                    this.biomassWarningShown = true;
                }
                this.biomassData = null;
                return null;
            }

            this.biomassData = data;
            this.biomassWarningShown = false;
            return data;
        } catch (error) {
            if (!this.biomassWarningShown) {
                console.warn('Failed to fetch biomass data:', error);
                this.biomassWarningShown = true;
            }
            this.biomassData = null;
            return null;
        }
    }

    // Apply grass density overlay to terrain colors
    applyGrassDensityOverlay(baseColor, chunkX, chunkY, localX, localY, terrainType) {
        if (!this.biomassData || !this.biomassData.heatmap) {
            return baseColor;
        }

        // Calculate position in the biomass heatmap
        const heatmapWidth = this.biomassData.heatmap.length;
        const heatmapHeight = this.biomassData.heatmap[0]?.length || 0;
        const offsetX = Math.floor(heatmapWidth / 2);
        const offsetY = Math.floor(heatmapHeight / 2);
        const heatmapChunkX = chunkX + offsetX;
        const heatmapChunkY = chunkY + offsetY;

        if (heatmapChunkX >= 0 && heatmapChunkX < heatmapWidth &&
            heatmapChunkY >= 0 && heatmapChunkY < heatmapHeight) {

            const biomassLevel = this.biomassData.heatmap[heatmapChunkX][heatmapChunkY];
            return this.darkenColorByBiomass(baseColor, biomassLevel, terrainType);
        }

        return baseColor;
    }

    // Darken color based on biomass density
    darkenColorByBiomass(color, biomassLevel, terrainType) {
        // Parse hex color to RGB
        const hex = color.replace('#', '');
        const r = parseInt(hex.substr(0, 2), 16);
        const g = parseInt(hex.substr(2, 2), 16);
        const b = parseInt(hex.substr(4, 2), 16);

        // Calculate darkness factor based on biomass (0-100)
        // Higher biomass = darker shade
        let darknessFactor = 0;

        if (terrainType === 'Grass') {
            // Grass: 0% biomass = 0% darkening, 100% biomass = 40% darkening
            darknessFactor = (biomassLevel / 100) * 0.4;
        } else if (terrainType === 'Forest') {
            // Forest: already dark, so less dramatic effect
            // 0% biomass = 0% darkening, 100% biomass = 20% darkening
            darknessFactor = (biomassLevel / 100) * 0.2;
        } else if (terrainType === 'Dirt') {
            // Dirt: can show sparse vegetation
            // 0% biomass = 0% darkening, 100% biomass = 25% darkening
            darknessFactor = (biomassLevel / 100) * 0.25;
        }

        // Apply darkening
        const newR = Math.floor(r * (1 - darknessFactor));
        const newG = Math.floor(g * (1 - darknessFactor));
        const newB = Math.floor(b * (1 - darknessFactor));

        // Convert back to hex
        const toHex = (n) => {
            const hex = n.toString(16);
            return hex.length === 1 ? '0' + hex : hex;
        };

        return `#${toHex(newR)}${toHex(newG)}${toHex(newB)}`;
    }
}
