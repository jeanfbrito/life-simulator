/**
 * Main application controller for the Life Simulator Viewer
 */

import { CONFIG, loadSpeciesConfig } from './config.js';
import { Renderer } from './renderer.js';
import { ChunkManager } from './chunk-manager.js';
import { Controls, FPSCounter } from './controls.js';
import { NetworkManager } from './network.js';
import { EntityManager } from './entity-manager.js';
import { initEntityStats } from './entity-stats.js';

class LifeSimulatorApp {
    constructor() {
        console.log('Loading Life Simulator Viewer v1.0.0 (Modular)');

        // Initialize canvas
        this.canvas = document.getElementById('mapCanvas');
        this.ctx = this.canvas.getContext('2d');

        // Initialize components
        this.renderer = new Renderer(this.canvas, this.ctx);
        this.chunkManager = new ChunkManager();
        this.controls = new Controls(this.canvas, this.renderer, this.chunkManager);
        this.networkManager = new NetworkManager();
        this.fpsCounter = new FPSCounter();
        this.entityManager = new EntityManager();

        // Application state
        this.worldData = {
            chunks: {},
            resources: {},
            centerChunk: { x: 0, y: 0 },
            worldStats: {
                total_chunks: 0,
                total_tiles: 0,
                walkable_tiles: 0,
                terrain_distribution: {}
            }
        };
        this.centerCoord = { x: 0, y: 0 };

        // Biomass data refresh tracking
        this.lastBiomassRefresh = 0;
        this.biomassRefreshInterval = 5000; // Refresh every 5 seconds

        // Setup component connections
        this.setupComponentConnections();

        // Initialize application
        this.initialize();
    }

    setupComponentConnections() {
        // Set world data for controls (for tooltips)
        this.controls.setWorldData(this.worldData);

        // Set render callback for controls
        this.controls.setRenderCallback(() => this.render());

        // Setup network message handlers
        this.networkManager.onMessage((message) => this.handleNetworkMessage(message));

        // Setup connection change handler
        this.networkManager.onConnectionChange((connected) => {
            if (connected) {
                // Request data when connected
                this.networkManager.requestWorldInfo();
                this.networkManager.requestWorldStats();
            }
        });
    }

    handleNetworkMessage(message) {
        switch (message.type) {
            case 'world_info':
                this.updateWorldInfo(message.data);
                break;
            case 'chunk_data':
                this.updateChunkData(message.data);
                break;
            case 'world_stats':
                this.updateWorldStats(message.data);
                break;
            case 'error':
                console.error('Server error:', message.message);
                break;
            default:
                console.log('Unhandled message type:', message.type);
                break;
        }
    }

    updateWorldInfo(data) {
        if (data.center_chunk) {
            this.centerCoord = data.center_chunk;
            this.centerCoord.x = data.center_chunk.x || 0;
            this.centerCoord.y = data.center_chunk.y || 0;
        }

        // Request new chunks around the new center
        this.chunkManager.requestChunks(this.centerCoord);
    }

    updateChunkData(data) {
        if (data.chunk_data) {
            this.worldData.chunks = data.chunk_data;
            this.updateStats();
        }
    }

    updateWorldStats(data) {
        this.worldData.worldStats = data;
        this.updateStats();
    }

    updateStats() {
        this.chunkManager.updateChunkCount(this.worldData.worldStats);
    }

    /**
     * Check for required browser features and provide fallbacks
     */
    checkBrowserFeatures() {
        // Check Canvas support
        const canvas = document.createElement('canvas');
        if (!canvas.getContext || !canvas.getContext('2d')) {
            this.showFatalError('Canvas', 'Your browser does not support HTML5 Canvas. Please use a modern browser like Chrome, Firefox, or Edge.');
            return false;
        }

        // Check for backdrop-filter support
        if (!CSS.supports('backdrop-filter', 'blur(10px)')) {
            console.warn('backdrop-filter not supported, using opaque backgrounds');
            // Apply fallback styles
            document.querySelectorAll('.sidebar, header').forEach(el => {
                el.style.backgroundColor = 'rgba(0, 0, 0, 0.95)';
                el.style.backdropFilter = 'none';
            });
        }

        // Check for ES6+ features
        try {
            eval('const x = () => {}; let y = {...{}}');
        } catch (e) {
            this.showFatalError('JavaScript', 'Your browser does not support modern JavaScript. Please update your browser.');
            return false;
        }

        console.log('✓ Browser feature checks passed');
        return true;
    }

    /**
     * Show fatal error overlay
     */
    showFatalError(feature, message) {
        const overlay = document.createElement('div');
        overlay.style.cssText = `
            position: fixed;
            top: 0;
            left: 0;
            right: 0;
            bottom: 0;
            background: #1a1a1a;
            color: white;
            display: flex;
            align-items: center;
            justify-content: center;
            z-index: 10000;
            font-family: system-ui, sans-serif;
        `;

        overlay.innerHTML = `
            <div style="text-align: center; max-width: 500px; padding: 2rem;">
                <div style="font-size: 4rem; margin-bottom: 1rem;">⚠️</div>
                <h2 style="margin-bottom: 1rem; color: #f87171;">Unsupported Browser</h2>
                <p style="margin-bottom: 1rem; line-height: 1.6;">${message}</p>
                <p style="opacity: 0.7; font-size: 0.9rem;">Feature missing: ${feature}</p>
            </div>
        `;

        document.body.appendChild(overlay);
    }

    async initialize() {
        try {
            console.log('🚀 APP: Initializing viewer...');

            // Feature detection
            if (!this.checkBrowserFeatures()) {
                return; // Stop initialization if critical features missing
            }

            // Setup initial canvas size
            this.renderer.setupCanvasSize(this.controls.getDragOffset());
            this.controls.updateZoomDisplay();

            // Load species configuration from backend
            const speciesConfigLoaded = await loadSpeciesConfig();
            console.log('🦎 APP: Species config loaded:', speciesConfigLoaded);

            // Load world information
            const worldInfoLoaded = await this.chunkManager.loadWorldInfo();
            console.log('📊 APP: World info loaded:', worldInfoLoaded);

            if (worldInfoLoaded) {
                // Load initial chunks around center, and also expand to show full world
                console.log('📦 APP: Requesting chunks around center:', this.centerCoord);
                const chunkData = await this.chunkManager.requestChunks(this.centerCoord);
                console.log('📦 APP: Chunk data received:', chunkData);

                if (chunkData) {
                    console.log('🔗 APP: Merging chunk data into worldData...');
                    this.chunkManager.mergeChunkData(chunkData, this.worldData);
                    console.log('🗺️ APP: World data after merge:', {
                        chunksCount: Object.keys(this.worldData.chunks).length,
                        resourcesCount: Object.keys(this.worldData.resources).length
                    });
                    // Update stats after loading initial chunks
                    this.updateStats();
                    // Force an initial render
                    this.render();
                }
            }

            // Start the animation loop
            console.log('🎬 APP: Starting animation loop...');
            this.startAnimationLoop();

            // Start entity polling
            console.log('🎯 APP: Starting entity polling...');
            this.entityManager.startPolling(200); // Poll every 0.2 seconds for smoother visualization
            
            // Start entity stats display
            console.log('📊 APP: Starting entity stats display...');
            initEntityStats();

            // Note: WebSocket is not supported by the simple web server
            // The viewer works perfectly with HTTP-only mode
            // this.networkManager.connect();

        } catch (error) {
            console.error('❌ APP: Failed to initialize viewer:', error);
            // Still start animation loop even if initialization fails
            this.startAnimationLoop();
        }
    }

    render() {
        // Get entities from entity manager
        const entities = this.entityManager.getEntities();

        // Render the world with entities
        const stats = this.renderer.render(this.worldData, this.controls.getDragOffset(), entities);

        // Update statistics display
        this.renderer.updateStatsDisplay(stats);
        
        // Update entity count display
        const entityCountElement = document.getElementById('entity-count');
        if (entityCountElement) {
            entityCountElement.textContent = entities.length;
        }
        
        // Trigger chunk loading for visible area (after first render)
        // Pass render callback so new chunks trigger a re-render
        this.chunkManager.loadVisibleChunksDebounced(
            this.controls.getDragOffset(), 
            this.worldData,
            () => this.render()
        );
    }

    startAnimationLoop() {
        const animate = (currentTime) => {
            // Always update controls (for smoothing/inertia)
            this.controls.update();

            // Refresh biomass data periodically if grass density is enabled
            if (CONFIG.showGrassDensity && (currentTime - this.lastBiomassRefresh) > this.biomassRefreshInterval) {
                this.renderer.fetchBiomassData();
                this.lastBiomassRefresh = currentTime;
            }

            // Check if we should render this frame (FPS limiting)
            if (this.fpsCounter.update(currentTime)) {
                // Render the map
                this.render();
            }

            // Continue animation loop
            requestAnimationFrame(animate);
        };

        // Start the animation loop
        requestAnimationFrame(animate);
    }

    // Public methods for external control
    zoomIn() {
        this.controls.zoomIn();
    }

    zoomOut() {
        this.controls.zoomOut();
    }

    resetView() {
        this.controls.resetView();
    }

    // Cleanup method
    destroy() {
        this.networkManager.disconnect();
        this.chunkManager.clear();
        this.entityManager.clear();
    }
}

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    // Create global app instance
    window.lifeSimulatorApp = new LifeSimulatorApp();
});

// Export for module usage
export default LifeSimulatorApp;