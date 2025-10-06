/**
 * UI controls and interaction handlers for the Life Simulator Viewer
 */

import { CONFIG, RESOURCE_SYMBOLS } from './config.js';

export class Controls {
    constructor(canvas, renderer, chunkManager) {
        this.canvas = canvas;
        this.renderer = renderer;
        this.chunkManager = chunkManager;

        // Map panning state
        this.isDragging = false;
        this.dragStart = { x: 0, y: 0 };
        // Smoothed camera position used for rendering
        this.dragOffset = { x: 0, y: 0 };
        // Target camera position (follows the mouse while dragging)
        this.targetOffset = { x: 0, y: 0 };
        // Inertia velocity for smooth deceleration after mouseup
        this.inertiaVelocity = { x: 0, y: 0 };
        // Track last mouse position to compute velocity while dragging
        this.lastMouse = { x: 0, y: 0 };

        this.setupEventListeners();
    }

    setupEventListeners() {
        // Mouse move for hover info
        this.canvas.addEventListener('mousemove', (e) => this.handleMouseMove(e));

        // Mouse leave to hide tooltip
        this.canvas.addEventListener('mouseleave', () => this.hideTooltip());

        // Middle mouse button drag functionality
        this.canvas.addEventListener('mousedown', (e) => this.handleMouseDown(e));
        this.canvas.addEventListener('mousemove', (e) => this.handleDrag(e));
        this.canvas.addEventListener('mouseup', (e) => this.handleMouseUp(e));

        // Handle context menu (right click) to prevent interference
        this.canvas.addEventListener('contextmenu', (e) => e.preventDefault());

        // Window resize
        window.addEventListener('resize', () => this.handleResize());

        // Zoom controls
        document.getElementById('zoom-in').addEventListener('click', () => this.zoomIn());
        document.getElementById('zoom-out').addEventListener('click', () => this.zoomOut());
        document.getElementById('reset-view').addEventListener('click', () => this.resetView());

        // Grass density toggle
        document.getElementById('toggle-grass-density').addEventListener('click', () => this.toggleGrassDensity());
    }

    handleMouseMove(e) {
        const rect = this.canvas.getBoundingClientRect();

        // Convert mouse position to canvas coordinates
        const canvasX = e.clientX - rect.left - this.dragOffset.x;
        const canvasY = e.clientY - rect.top - this.dragOffset.y;
        const x = Math.floor(canvasX / CONFIG.TILE_SIZE);
        const y = Math.floor(canvasY / CONFIG.TILE_SIZE);

        if (x >= 0 && x < CONFIG.VIEW_SIZE_X && y >= 0 && y < CONFIG.VIEW_SIZE_Y) {
            this.showTooltip(e, x, y);
        } else {
            this.hideTooltip();
        }
    }

    showTooltip(e, x, y) {
        const tooltip = document.getElementById('tooltip');
        tooltip.style.display = 'block';

        // Position tooltip near cursor with smart positioning
        const offset = 15; // Distance from cursor
        const tooltipRect = tooltip.getBoundingClientRect();
        const viewportWidth = window.innerWidth;
        const viewportHeight = window.innerHeight;

        let left = e.clientX + offset;
        let top = e.clientY + offset;

        // Adjust if tooltip would go off screen
        if (left + tooltipRect.width > viewportWidth) {
            left = e.clientX - tooltipRect.width - offset;
        }
        if (top + tooltipRect.height > viewportHeight) {
            top = e.clientY - tooltipRect.height - offset;
        }

        tooltip.style.left = left + 'px';
        tooltip.style.top = top + 'px';

        // Convert to world coordinates (matching render function)
        const worldX = x - Math.floor(CONFIG.VIEW_SIZE_X / 2);
        const worldY = y - Math.floor(CONFIG.VIEW_SIZE_Y / 2);

        // Get chunk coordinates
        const chunkX = Math.floor(worldX / 16);
        const chunkY = Math.floor(worldY / 16);
        const localX = ((worldX % 16) + 16) % 16;
        const localY = ((worldY % 16) + 16) % 16;

        const chunkKey = `${chunkX},${chunkY}`;
        let terrainType = 'Grass';

        // We need access to worldData - this will be injected from main app
        if (this.worldData) {
            if (this.worldData.chunks[chunkKey] &&
                this.worldData.chunks[chunkKey][localY] &&
                this.worldData.chunks[chunkKey][localY][localX]) {
                terrainType = this.worldData.chunks[chunkKey][localY][localX];
            }

            // Check for resource
            let resourceType = '';
            if (this.worldData.resources[chunkKey] &&
                this.worldData.resources[chunkKey][localY] &&
                this.worldData.resources[chunkKey][localY][localX]) {
                resourceType = this.worldData.resources[chunkKey][localY][localX];
            }

            let tooltipText = `World: (${worldX}, ${worldY})<br>Chunk: (${chunkX}, ${chunkY})<br>Terrain: ${terrainType}`;

            // Add biomass information if grass density overlay is enabled
            if (CONFIG.showGrassDensity && this.renderer.biomassData && (terrainType === 'Grass' || terrainType === 'Forest' || terrainType === 'Dirt')) {
                const biomassLevel = this.getBiomassAtPosition(worldX, worldY);
                if (biomassLevel !== null) {
                    const biomassStatus = this.getBiomassStatus(biomassLevel, terrainType);
                    tooltipText += `<br>🌱 Biomass: ${biomassLevel.toFixed(1)}% ${biomassStatus}`;

                    // Add additional context based on biomass level
                    if (biomassLevel >= 80) {
                        tooltipText += `<br><small style="opacity: 0.7;">(Excellent growth conditions)</small>`;
                    } else if (biomassLevel >= 60) {
                        tooltipText += `<br><small style="opacity: 0.7;">(Healthy vegetation)</small>`;
                    } else if (biomassLevel >= 40) {
                        tooltipText += `<br><small style="opacity: 0.7;">(Moderate growth)</small>`;
                    } else if (biomassLevel >= 20) {
                        tooltipText += `<br><small style="opacity: 0.7;">(Low biomass, may need recovery)</small>`;
                    } else {
                        tooltipText += `<br><small style="opacity: 0.7;">(Very low, depleted area)</small>`;
                    }

                    // Add global vegetation context when biomass overlay is enabled
                    if (this.renderer.biomassData && this.renderer.biomassData.metadata) {
                        const avgBiomass = this.calculateAverageBiomass();
                        const maxBiomass = this.renderer.biomassData.max_biomass || 100;
                        const utilizationRate = (avgBiomass / maxBiomass * 100).toFixed(1);
                        tooltipText += `<br><hr style="opacity: 0.3;">`;
                        tooltipText += `<small style="opacity: 0.8;">🌍 Area biomass utilization: ${utilizationRate}%</small>`;
                    }
                }
            } else if (CONFIG.showGrassDensity) {
                // When overlay is enabled but no biomass data available for this terrain
                tooltipText += `<br><small style="opacity: 0.7;">🌱 No biomass data available</small>`;
            }

            if (resourceType) {
                tooltipText += `<br>Resource: ${resourceType} ${RESOURCE_SYMBOLS[resourceType] || ''}`;
            }
            tooltip.innerHTML = tooltipText;
        } else {
            tooltip.innerHTML = `World: (${worldX}, ${worldY})<br>Chunk: (${chunkX}, ${chunkY})<br>Terrain: Loading...`;
        }
    }

    hideTooltip() {
        document.getElementById('tooltip').style.display = 'none';
    }

    // Get biomass level at specific world coordinates
    getBiomassAtPosition(worldX, worldY) {
        if (!this.renderer.biomassData || !this.renderer.biomassData.heatmap) {
            return null;
        }

        const heatmap = this.renderer.biomassData.heatmap;
        const heatmapSizeX = heatmap.length;
        const heatmapSizeY = heatmap[0]?.length || 0;
        const tileSize = this.renderer.biomassData.tile_size || 16;

        // Convert world tile coordinates to chunk coordinates
        const chunkX = Math.floor(worldX / tileSize);
        const chunkY = Math.floor(worldY / tileSize);

        const offsetX = Math.floor(heatmapSizeX / 2);
        const offsetY = Math.floor(heatmapSizeY / 2);
        const heatmapX = chunkX + offsetX;
        const heatmapY = chunkY + offsetY;

        // Check if we have biomass data for this position
        if (heatmapX >= 0 && heatmapX < heatmapSizeX && heatmapY >= 0 && heatmapY < heatmapSizeY) {
            return heatmap[heatmapX][heatmapY];
        }

        return null;
    }

    // Get biomass status emoji based on level and terrain type
    getBiomassStatus(biomassLevel, terrainType) {
        if (biomassLevel >= 80) {
            return '🌿 Dense';
        } else if (biomassLevel >= 60) {
            return '🌿 Lush';
        } else if (biomassLevel >= 40) {
            return '🌱 Moderate';
        } else if (biomassLevel >= 20) {
            return '🌱 Sparse';
        } else {
            return '🍂 Scarce';
        }
    }

    // Calculate average biomass across the entire heatmap
    calculateAverageBiomass() {
        if (!this.renderer.biomassData || !this.renderer.biomassData.heatmap) {
            return 0;
        }

        const heatmap = this.renderer.biomassData.heatmap;
        let totalBiomass = 0;
        let tileCount = 0;

        for (let i = 0; i < heatmap.length; i++) {
            for (let j = 0; j < heatmap[i].length; j++) {
                if (heatmap[i][j] > 0) {
                    totalBiomass += heatmap[i][j];
                    tileCount++;
                }
            }
        }

        return tileCount > 0 ? totalBiomass / tileCount : 0;
    }

    handleMouseDown(e) {
        if (e.button === 1) { // Middle mouse button
            e.preventDefault();
            this.isDragging = true;
            this.dragStart.x = e.clientX - this.dragOffset.x;
            this.dragStart.y = e.clientY - this.dragOffset.y;
            this.lastMouse.x = e.clientX;
            this.lastMouse.y = e.clientY;
            // Stop any existing inertia
            this.inertiaVelocity = { x: 0, y: 0 };
            this.canvas.style.cursor = 'grabbing';
        }
    }

    handleDrag(e) {
        if (this.isDragging) {
            e.preventDefault();
            // Update target offset from mouse position
            this.targetOffset.x = e.clientX - this.dragStart.x;
            this.targetOffset.y = e.clientY - this.dragStart.y;

            // Compute instantaneous velocity for inertia
            const dx = e.clientX - this.lastMouse.x;
            const dy = e.clientY - this.lastMouse.y;
            this.inertiaVelocity.x = dx;
            this.inertiaVelocity.y = dy;
            this.lastMouse.x = e.clientX;
            this.lastMouse.y = e.clientY;

            // Trigger chunk loading during drag for smoother experience
            if (this.worldData && this.onRender) {
                this.chunkManager.loadVisibleChunksDebounced(this.targetOffset, this.worldData, this.onRender);
            }
        }
    }

    handleMouseUp(e) {
        if (e.button === 1) { // Middle mouse button
            e.preventDefault();
            this.isDragging = false;
            this.canvas.style.cursor = 'pointer';
            // On release, keep current target so inertia moves from current velocity
            // Load chunks for the new position immediately when dragging stops
            if (this.worldData && this.onRender) {
                this.chunkManager.loadVisibleChunksDebounced(this.dragOffset, this.worldData, this.onRender);
            }
        }
    }

    handleResize() {
        this.renderer.setupCanvasSize(this.dragOffset);
        // Trigger re-render via the main app
        if (this.onRender) {
            this.onRender();
        }
    }

    updateZoomDisplay() {
        const zoomPercent = Math.round(CONFIG.renderScale * 100);
        document.getElementById('zoom-level').textContent = zoomPercent + '%';
        document.getElementById('tile-size-display').textContent = CONFIG.TILE_SIZE + 'px';
    }

    zoomIn() {
        CONFIG.renderScale = Math.min(CONFIG.renderScale * CONFIG.zoomFactor, CONFIG.maxZoom);
        this.renderer.setupCanvasSize(this.dragOffset);
        this.updateZoomDisplay();
        // Trigger re-render via the main app
        if (this.onRender) {
            this.onRender();
        }
    }

    zoomOut() {
        CONFIG.renderScale = Math.max(CONFIG.renderScale / CONFIG.zoomFactor, CONFIG.minZoom);
        this.renderer.setupCanvasSize(this.dragOffset);
        this.updateZoomDisplay();
        // Trigger re-render via the main app
        if (this.onRender) {
            this.onRender();
        }
    }

    resetView() {
        CONFIG.renderScale = 1.0;
        this.dragOffset = { x: 0, y: 0 };
        this.renderer.setupCanvasSize(this.dragOffset);
        this.updateZoomDisplay();
        // Trigger re-render via the main app
        if (this.onRender) {
            this.onRender();
        }
    }

    // Smooth update called each animation frame
    update() {
        // Smoothly move current offset toward target while dragging
        if (this.isDragging) {
            this.dragOffset.x += (this.targetOffset.x - this.dragOffset.x) * CONFIG.panSmoothing;
            this.dragOffset.y += (this.targetOffset.y - this.dragOffset.y) * CONFIG.panSmoothing;
        } else {
            // Apply inertia when not dragging
            if (Math.abs(this.inertiaVelocity.x) > CONFIG.inertiaMinSpeed || Math.abs(this.inertiaVelocity.y) > CONFIG.inertiaMinSpeed) {
                this.dragOffset.x += this.inertiaVelocity.x;
                this.dragOffset.y += this.inertiaVelocity.y;
                this.inertiaVelocity.x *= CONFIG.inertiaFriction;
                this.inertiaVelocity.y *= CONFIG.inertiaFriction;

                // While inertia is moving, keep loading chunks
                if (this.worldData && this.onRender) {
                    this.chunkManager.loadVisibleChunksDebounced(this.dragOffset, this.worldData, this.onRender);
                }
            } else {
                // Stop inertia when velocity is small
                this.inertiaVelocity = { x: 0, y: 0 };
            }
        }
    }

    // Set world data reference for tooltips
    setWorldData(worldData) {
        this.worldData = worldData;
    }

    // Set render callback
    setRenderCallback(callback) {
        this.onRender = callback;
    }

    // Get current drag offset (for other components)
    getDragOffset() {
        return this.dragOffset;
    }

    // Toggle grass density visualization
    async toggleGrassDensity() {
        const button = document.getElementById('toggle-grass-density');
        const status = document.getElementById('grass-density-status');

        // Toggle the config setting
        CONFIG.showGrassDensity = !CONFIG.showGrassDensity;

        if (CONFIG.showGrassDensity) {
            // Enable grass density visualization
            button.style.background = 'rgba(34, 197, 94, 0.4)';
            button.style.borderColor = 'rgba(34, 197, 94, 0.6)';
            button.innerHTML = '🌱 Hide Grass Density';
            status.innerHTML = '<span style="color: #fbbf24;">⏳ Loading biomass data...</span>';

            // Fetch biomass data
            const biomassData = await this.renderer.fetchBiomassData();

            if (biomassData) {
                status.innerHTML = '<span style="color: #22c55e;">✅ Grass density overlay active</span>';
            } else {
                status.innerHTML = '<span style="color: #f87171;">⚠️ Biomass data unavailable — check that the simulator is running</span>';
                CONFIG.showGrassDensity = false;
                button.style.background = 'rgba(34, 197, 94, 0.2)';
                button.style.borderColor = 'rgba(34, 197, 94, 0.4)';
                button.innerHTML = '🌱 Show Grass Density';
            }

            // Update rendering
            if (this.onRender) {
                this.onRender();
            }
        } else {
            // Disable grass density visualization
            button.style.background = 'rgba(34, 197, 94, 0.2)';
            button.style.borderColor = 'rgba(34, 197, 94, 0.4)';
            button.innerHTML = '🌱 Show Grass Density';
            status.innerHTML = '<span style="color: #fbbf24;">🔍 Click to show grass growth patterns</span>';

            // Clear biomass data
            this.renderer.biomassData = null;

            // Update rendering
            if (this.onRender) {
                this.onRender();
            }
        }
    }
}

export class FPSCounter {
    constructor() {
        this.lastTime = 0;
        this.frameCount = 0;
        this.fpsTime = 0;
    }

    update(currentTime) {
        if (!currentTime) currentTime = 0;
        const deltaTime = currentTime - this.lastTime;

        if (deltaTime >= CONFIG.frameDelay) {
            this.lastTime = currentTime - (deltaTime % CONFIG.frameDelay);

            // Update FPS counter
            this.frameCount++;
            if (currentTime - this.fpsTime >= 1000) {
                document.getElementById('fps').textContent = this.frameCount;
                this.frameCount = 0;
                this.fpsTime = currentTime;
            }

            return true; // Should render
        }

        return false; // Skip frame
    }

  }
