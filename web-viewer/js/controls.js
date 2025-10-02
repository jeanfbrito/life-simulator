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
        this.dragOffset = { x: 0, y: 0 };

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

    handleMouseDown(e) {
        if (e.button === 1) { // Middle mouse button
            e.preventDefault();
            this.isDragging = true;
            this.dragStart.x = e.clientX - this.dragOffset.x;
            this.dragStart.y = e.clientY - this.dragOffset.y;
            this.canvas.style.cursor = 'grabbing';
        }
    }

    handleDrag(e) {
        if (this.isDragging) {
            e.preventDefault();
            this.dragOffset.x = e.clientX - this.dragStart.x;
            this.dragOffset.y = e.clientY - this.dragStart.y;
            // Trigger chunk loading during drag for smoother experience
            if (this.worldData) {
                this.chunkManager.loadVisibleChunksDebounced(this.dragOffset, this.worldData);
            }
        }
    }

    handleMouseUp(e) {
        if (e.button === 1) { // Middle mouse button
            e.preventDefault();
            this.isDragging = false;
            this.canvas.style.cursor = 'pointer';
            // Load chunks for the new position immediately when dragging stops
            if (this.worldData) {
                this.chunkManager.loadVisibleChunksDebounced(this.dragOffset, this.worldData);
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