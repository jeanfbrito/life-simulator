/**
 * UI controls and interaction handlers for the Life Simulator Viewer
 */

import { CONFIG, RESOURCE_SYMBOLS } from './config.js';
import { CoordinateConverter } from './utils/coordinates.js';

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

        // Cache for biomass average to avoid O(n²) calculation on every mousemove
        this.cachedBiomassAverage = null;

        // Throttle tooltip updates to reduce getBoundingClientRect() calls
        this.lastTooltipUpdate = 0;
        this.tooltipThrottleMs = CONFIG.TOOLTIP_THROTTLE_MS; // Update max once per configured interval

        // Keyboard panning state
        this.keysPressed = new Set();
        this.keyPanSpeed = 10; // Pixels per frame when holding a key

        // Last mouse position for zoom centering
        this.lastMousePos = { x: 0, y: 0 };

        // Store bound handlers for cleanup
        this.boundHandlers = {
            mouseMove: (e) => this.handleMouseMove(e),
            mouseLeave: () => this.hideTooltip(),
            mouseDown: (e) => this.handleMouseDown(e),
            drag: (e) => this.handleDrag(e),
            mouseUp: (e) => this.handleMouseUp(e),
            contextMenu: (e) => e.preventDefault(),
            resize: () => this.handleResize(),
            zoomIn: () => this.zoomIn(),
            zoomOut: () => this.zoomOut(),
            resetView: () => this.resetView(),
            toggleGrassDensity: () => this.toggleGrassDensity(),
            wheel: (e) => this.handleWheel(e),
            keyDown: (e) => this.handleKeyDown(e),
            keyUp: (e) => this.handleKeyUp(e)
        };
this.setupEventListeners();
    }

    setupEventListeners() {
        // Mouse move for hover info
        this.canvas.addEventListener('mousemove', this.boundHandlers.mouseMove);

        // Mouse leave to hide tooltip
        this.canvas.addEventListener('mouseleave', this.boundHandlers.mouseLeave);

        // Middle mouse button drag functionality
        this.canvas.addEventListener('mousedown', this.boundHandlers.mouseDown);
        this.canvas.addEventListener('mousemove', this.boundHandlers.drag);
        this.canvas.addEventListener('mouseup', this.boundHandlers.mouseUp);

        // Handle context menu (right click) to prevent interference
        this.canvas.addEventListener('contextmenu', this.boundHandlers.contextMenu);

        // Window resize
        window.addEventListener('resize', this.boundHandlers.resize);

        // Zoom controls
        const zoomInBtn = document.getElementById('zoom-in');
        const zoomOutBtn = document.getElementById('zoom-out');
        const resetViewBtn = document.getElementById('reset-view');
        const toggleGrassBtn = document.getElementById('toggle-grass-density');

        if (zoomInBtn) zoomInBtn.addEventListener('click', this.boundHandlers.zoomIn);
        if (zoomOutBtn) zoomOutBtn.addEventListener('click', this.boundHandlers.zoomOut);
        if (resetViewBtn) resetViewBtn.addEventListener('click', this.boundHandlers.resetView);
        if (toggleGrassBtn) toggleGrassBtn.addEventListener('click', this.boundHandlers.toggleGrassDensity);

        // Scroll wheel zoom
        this.canvas.addEventListener('wheel', this.boundHandlers.wheel, { passive: false });

        // Keyboard controls for panning and zooming
        window.addEventListener('keydown', this.boundHandlers.keyDown);
        window.addEventListener('keyup', this.boundHandlers.keyUp);
    }

    /**
     * Handle mouse movement for hover tooltips
     * Throttled to maximum once per CONFIG.TOOLTIP_THROTTLE_MS to reduce overhead
     * Updates tooltip position smoothly every frame but only updates content periodically
     * @param {MouseEvent} e - Mouse event with clientX/clientY coordinates
     */
    handleMouseMove(e) {
        const rect = this.canvas.getBoundingClientRect();

        // Track mouse position for zoom centering
        this.lastMousePos = { x: e.clientX - rect.left, y: e.clientY - rect.top };

        // Convert mouse position to canvas coordinates
        const canvasX = e.clientX - rect.left - this.dragOffset.x;
        const canvasY = e.clientY - rect.top - this.dragOffset.y;
        const x = Math.floor(canvasX / CONFIG.TILE_SIZE);
        const y = Math.floor(canvasY / CONFIG.TILE_SIZE);

        if (x >= 0 && x < CONFIG.VIEW_SIZE_X && y >= 0 && y < CONFIG.VIEW_SIZE_Y) {
            const now = Date.now();

            // Always update tooltip position for smooth tracking
            const tooltip = document.getElementById('tooltip');
            if (tooltip && tooltip.style.display === 'block') {
                const offset = 15;
                tooltip.style.left = (e.clientX + offset) + 'px';
                tooltip.style.top = (e.clientY + offset) + 'px';
            }

            // But only update content and recalculate bounds every CONFIG.TOOLTIP_THROTTLE_MS
            if (now - this.lastTooltipUpdate >= this.tooltipThrottleMs) {
                this.showTooltip(e, x, y);
                this.lastTooltipUpdate = now;
            }
        } else {
            this.hideTooltip();
        }
    }

    showTooltip(e, x, y) {
        const tooltip = document.getElementById('tooltip');
        tooltip.style.display = 'block';

        // Position tooltip near cursor with smart positioning (throttled - only called every 100ms)
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
        const world = CoordinateConverter.screenToWorld(x, y, CONFIG.VIEW_SIZE_X, CONFIG.VIEW_SIZE_Y);
        const chunk = CoordinateConverter.worldToChunk(world.x, world.y);
        const chunkKey = CoordinateConverter.chunkKey(chunk.chunkX, chunk.chunkY);

        // Use shorthand for readability
        const worldX = world.x;
        const worldY = world.y;
        const chunkX = chunk.chunkX;
        const chunkY = chunk.chunkY;
        const localX = chunk.localX;
        const localY = chunk.localY;
        let terrainType = 'Grass';

        // We need access to worldData - this will be injected from main app
        if (this.worldData) {
            // Use optional chaining for null-safe access to terrain data
            terrainType = this.worldData.chunks?.[chunkKey]?.[localY]?.[localX] ?? 'Grass';

            // Check for resource with null-safe optional chaining
            const resourceType = this.worldData.resources?.[chunkKey]?.[localY]?.[localX] ?? '';

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
        const chunk = CoordinateConverter.worldToChunk(worldX, worldY, tileSize);

        const offsetX = Math.floor(heatmapSizeX / 2);
        const offsetY = Math.floor(heatmapSizeY / 2);
        const heatmapX = chunk.chunkX + offsetX;
        const heatmapY = chunk.chunkY + offsetY;

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
        // Return cached value if available to avoid O(n²) calculation on every mousemove
        if (this.cachedBiomassAverage !== null) {
            return this.cachedBiomassAverage;
        }

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
        // Support both left-click (0) and middle-click (1) for dragging
        if (e.button === 0 || e.button === 1) {
            e.preventDefault();
            this.isDragging = true;
            this.dragButton = e.button; // Track which button started the drag
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
        // Support both left-click (0) and middle-click (1) for dragging
        if ((e.button === 0 || e.button === 1) && this.isDragging && e.button === this.dragButton) {
            e.preventDefault();
            this.isDragging = false;
            this.dragButton = null;
            this.canvas.style.cursor = 'grab';
            // On release, keep current target so inertia moves from current velocity
            // Load chunks for the new position immediately when dragging stops
            if (this.worldData && this.onRender) {
                this.chunkManager.loadVisibleChunksDebounced(this.dragOffset, this.worldData, this.onRender);
            }
        }
    }

    /**
     * Handle scroll wheel for zooming
     * Zooms centered on mouse cursor position
     * @param {WheelEvent} e - Wheel event
     */
    handleWheel(e) {
        e.preventDefault();

        // Determine zoom direction
        const zoomIn = e.deltaY < 0;

        // Get mouse position relative to canvas center
        const rect = this.canvas.getBoundingClientRect();
        const mouseX = e.clientX - rect.left;
        const mouseY = e.clientY - rect.top;

        // Calculate world position under mouse before zoom
        const worldXBefore = (mouseX - this.dragOffset.x) / CONFIG.TILE_SIZE;
        const worldYBefore = (mouseY - this.dragOffset.y) / CONFIG.TILE_SIZE;

        // Apply zoom
        const oldScale = CONFIG.renderScale;
        if (zoomIn) {
            CONFIG.renderScale = Math.min(CONFIG.renderScale * CONFIG.ZOOM_MULTIPLIER, CONFIG.MAX_ZOOM);
        } else {
            CONFIG.renderScale = Math.max(CONFIG.renderScale / CONFIG.ZOOM_MULTIPLIER, CONFIG.MIN_ZOOM);
        }

        // Only adjust if scale actually changed
        if (oldScale !== CONFIG.renderScale) {
            // Recalculate tile size
            this.renderer.setupCanvasSize(this.dragOffset);

            // Calculate world position under mouse after zoom
            const worldXAfter = (mouseX - this.dragOffset.x) / CONFIG.TILE_SIZE;
            const worldYAfter = (mouseY - this.dragOffset.y) / CONFIG.TILE_SIZE;

            // Adjust drag offset to keep the same world position under cursor
            this.dragOffset.x += (worldXAfter - worldXBefore) * CONFIG.TILE_SIZE;
            this.dragOffset.y += (worldYAfter - worldYBefore) * CONFIG.TILE_SIZE;
            this.targetOffset.x = this.dragOffset.x;
            this.targetOffset.y = this.dragOffset.y;

            this.updateZoomDisplay();

            // Trigger re-render
            if (this.onRender) {
                this.onRender();
            }
        }
    }

    /**
     * Handle keyboard key down for panning and zoom controls
     * @param {KeyboardEvent} e - Keyboard event
     */
    handleKeyDown(e) {
        // Ignore if user is typing in an input field
        if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') {
            return;
        }

        const key = e.key.toLowerCase();

        // Pan keys: WASD and Arrow keys
        if (['w', 'a', 's', 'd', 'arrowup', 'arrowdown', 'arrowleft', 'arrowright'].includes(key)) {
            e.preventDefault();
            this.keysPressed.add(key);
        }

        // Zoom keys: + and -
        if (key === '+' || key === '=' || key === 'numpadadd') {
            e.preventDefault();
            this.zoomIn();
        } else if (key === '-' || key === '_' || key === 'numpadsubtract') {
            e.preventDefault();
            this.zoomOut();
        }

        // Reset key: R
        if (key === 'r') {
            e.preventDefault();
            this.resetView();
        }
    }

    /**
     * Handle keyboard key up
     * @param {KeyboardEvent} e - Keyboard event
     */
    handleKeyUp(e) {
        const key = e.key.toLowerCase();
        this.keysPressed.delete(key);
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

    /**
     * Zoom in by multiplying render scale by CONFIG.ZOOM_MULTIPLIER
     * Applies zoom limits defined in CONFIG
     */
    zoomIn() {
        CONFIG.renderScale = Math.min(CONFIG.renderScale * CONFIG.ZOOM_MULTIPLIER, CONFIG.MAX_ZOOM);
        this.renderer.setupCanvasSize(this.dragOffset);
        this.updateZoomDisplay();
        // Trigger re-render via the main app
        if (this.onRender) {
            this.onRender();
        }
    }

    /**
     * Zoom out by dividing render scale by CONFIG.ZOOM_MULTIPLIER
     * Applies zoom limits defined in CONFIG
     */
    zoomOut() {
        CONFIG.renderScale = Math.max(CONFIG.renderScale / CONFIG.ZOOM_MULTIPLIER, CONFIG.MIN_ZOOM);
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
        this.targetOffset = { x: 0, y: 0 };
        this.inertiaVelocity = { x: 0, y: 0 };
        this.renderer.setupCanvasSize(this.dragOffset);
        this.updateZoomDisplay();
        // Trigger re-render via the main app
        if (this.onRender) {
            this.onRender();
        }
    }

    /**
     * Center the view on a specific world tile position
     * @param {number} tileX - X coordinate of the tile in world space
     * @param {number} tileY - Y coordinate of the tile in world space
     */
    centerOnEntity(tileX, tileY) {
        // Get canvas dimensions
        const canvasWidth = this.canvas.width;
        const canvasHeight = this.canvas.height;

        // The renderer uses this formula for entity screen position:
        // screenX = (entityWorldX - cameraOffsetX + VIEW_SIZE_X/2) * TILE_SIZE + TILE_SIZE/2
        // Where cameraOffsetX = floor(-dragOffset.x / TILE_SIZE)
        //
        // After simplification, the actual screen position is:
        // actualScreenX = tileX * TILE_SIZE + dragOffset.x + (VIEW_SIZE_X/2) * TILE_SIZE + TILE_SIZE/2
        //
        // To center the entity (actualScreenX = canvasWidth/2):
        // dragOffset.x = canvasWidth/2 - tileX * TILE_SIZE - (VIEW_SIZE_X/2) * TILE_SIZE - TILE_SIZE/2
        const viewCenterOffsetX = Math.floor(CONFIG.VIEW_SIZE_X / 2) * CONFIG.TILE_SIZE;
        const viewCenterOffsetY = Math.floor(CONFIG.VIEW_SIZE_Y / 2) * CONFIG.TILE_SIZE;
        const tileCenterOffset = CONFIG.TILE_SIZE / 2;

        const targetOffsetX = (canvasWidth / 2) - (tileX * CONFIG.TILE_SIZE) - viewCenterOffsetX - tileCenterOffset;
        const targetOffsetY = (canvasHeight / 2) - (tileY * CONFIG.TILE_SIZE) - viewCenterOffsetY - tileCenterOffset;

        // Set both current and target offset for immediate centering
        this.targetOffset.x = targetOffsetX;
        this.targetOffset.y = targetOffsetY;
        this.dragOffset.x = targetOffsetX;
        this.dragOffset.y = targetOffsetY;

        // Stop any inertia
        this.inertiaVelocity = { x: 0, y: 0 };

        // Force immediate chunk loading for the new position (not debounced)
        // Reset last loaded center to ensure chunks are loaded even if we haven't moved far
        if (this.worldData && this.chunkManager) {
            // Reset to force chunk reload
            this.chunkManager.lastLoadedCenter = { x: -9999, y: -9999 };
            // Load all visible chunks (now properly calculates rectangular visible area)
            this.chunkManager.loadVisibleChunks(this.dragOffset, this.worldData).then(() => {
                if (this.onRender) {
                    this.onRender();
                }
            });
        }

        // Trigger immediate re-render (chunks will re-render when loaded)
        if (this.onRender) {
            this.onRender();
        }

        console.log(`📍 Centered view on tile (${tileX}, ${tileY})`);
    }

    /**
     * Smooth camera update called each animation frame
     * Applies easing to camera movement while dragging and inertia when released
     * Also handles keyboard panning
     * Uses CONFIG constants for smooth pan speed and inertia decay
     */
    update() {
        // Handle keyboard panning
        let keyboardPanX = 0;
        let keyboardPanY = 0;

        if (this.keysPressed.has('w') || this.keysPressed.has('arrowup')) {
            keyboardPanY += this.keyPanSpeed;
        }
        if (this.keysPressed.has('s') || this.keysPressed.has('arrowdown')) {
            keyboardPanY -= this.keyPanSpeed;
        }
        if (this.keysPressed.has('a') || this.keysPressed.has('arrowleft')) {
            keyboardPanX += this.keyPanSpeed;
        }
        if (this.keysPressed.has('d') || this.keysPressed.has('arrowright')) {
            keyboardPanX -= this.keyPanSpeed;
        }

        // Apply keyboard panning directly
        if (keyboardPanX !== 0 || keyboardPanY !== 0) {
            this.dragOffset.x += keyboardPanX;
            this.dragOffset.y += keyboardPanY;
            this.targetOffset.x = this.dragOffset.x;
            this.targetOffset.y = this.dragOffset.y;

            // Load chunks for the new position
            if (this.worldData && this.onRender) {
                this.chunkManager.loadVisibleChunksDebounced(this.dragOffset, this.worldData, this.onRender);
            }
        }

        // Smoothly move current offset toward target while dragging
        if (this.isDragging) {
            this.dragOffset.x += (this.targetOffset.x - this.dragOffset.x) * CONFIG.PAN_SMOOTHING_FACTOR;
            this.dragOffset.y += (this.targetOffset.y - this.dragOffset.y) * CONFIG.PAN_SMOOTHING_FACTOR;
        } else if (keyboardPanX === 0 && keyboardPanY === 0) {
            // Apply inertia when not dragging and not keyboard panning
            if (Math.abs(this.inertiaVelocity.x) > CONFIG.MIN_INERTIA_SPEED || Math.abs(this.inertiaVelocity.y) > CONFIG.MIN_INERTIA_SPEED) {
                this.dragOffset.x += this.inertiaVelocity.x;
                this.dragOffset.y += this.inertiaVelocity.y;
                this.inertiaVelocity.x *= CONFIG.INERTIA_FRICTION;
                this.inertiaVelocity.y *= CONFIG.INERTIA_FRICTION;

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
                // Recalculate and cache average when new data arrives
                this.cachedBiomassAverage = this.calculateAverageBiomass();
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

            // Clear biomass data and invalidate cache
            this.renderer.biomassData = null;
            this.cachedBiomassAverage = null;

            // Update rendering
            if (this.onRender) {
                this.onRender();
            }
        }
    }

    /**
     * Clean up event listeners to prevent memory leaks
     * Call this when destroying the viewer instance
     */
    destroy() {
        // Remove canvas event listeners
        this.canvas.removeEventListener('mousemove', this.boundHandlers.mouseMove);
        this.canvas.removeEventListener('mouseleave', this.boundHandlers.mouseLeave);
        this.canvas.removeEventListener('mousedown', this.boundHandlers.mouseDown);
        this.canvas.removeEventListener('mousemove', this.boundHandlers.drag);
        this.canvas.removeEventListener('mouseup', this.boundHandlers.mouseUp);
        this.canvas.removeEventListener('contextmenu', this.boundHandlers.contextMenu);
        this.canvas.removeEventListener('wheel', this.boundHandlers.wheel);

        // Remove window event listeners
        window.removeEventListener('resize', this.boundHandlers.resize);
        window.removeEventListener('keydown', this.boundHandlers.keyDown);
        window.removeEventListener('keyup', this.boundHandlers.keyUp);

        // Remove button event listeners
        const zoomInBtn = document.getElementById('zoom-in');
        const zoomOutBtn = document.getElementById('zoom-out');
        const resetViewBtn = document.getElementById('reset-view');
        const toggleGrassBtn = document.getElementById('toggle-grass-density');

        if (zoomInBtn) zoomInBtn.removeEventListener('click', this.boundHandlers.zoomIn);
        if (zoomOutBtn) zoomOutBtn.removeEventListener('click', this.boundHandlers.zoomOut);
        if (resetViewBtn) resetViewBtn.removeEventListener('click', this.boundHandlers.resetView);
        if (toggleGrassBtn) toggleGrassBtn.removeEventListener('click', this.boundHandlers.toggleGrassDensity);

        console.log('Controls destroyed, event listeners removed');
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
