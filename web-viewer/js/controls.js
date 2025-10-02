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