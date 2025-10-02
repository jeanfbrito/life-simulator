/**
 * Main application controller for the Life Simulator Viewer
 */

import { CONFIG } from './config.js';
import { Renderer } from './renderer.js';
import { ChunkManager } from './chunk-manager.js';
import { Controls, FPSCounter } from './controls.js';
import { NetworkManager } from './network.js';

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

    async initialize() {
        try {
            // Load world information
            const worldInfoLoaded = await this.chunkManager.loadWorldInfo();

            if (worldInfoLoaded) {
                // Load initial chunks
                await this.chunkManager.requestChunks(this.centerCoord);
            }

            // Start the animation loop
            this.startAnimationLoop();

            // Try to connect via WebSocket for real-time updates
            this.networkManager.connect();

        } catch (error) {
            console.error('Failed to initialize viewer:', error);
            // Still start animation loop even if initialization fails
            this.startAnimationLoop();
        }
    }

    render() {
        // Trigger chunk loading for visible area
        this.chunkManager.loadVisibleChunksDebounced(this.controls.getDragOffset());

        // Render the world
        const stats = this.renderer.render(this.worldData, this.controls.getDragOffset());

        // Update statistics display
        this.renderer.updateStatsDisplay(stats);
    }

    startAnimationLoop() {
        const animate = (currentTime) => {
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
    }
}

// Initialize the application when the DOM is loaded
document.addEventListener('DOMContentLoaded', () => {
    // Create global app instance
    window.lifeSimulatorApp = new LifeSimulatorApp();
});

// Export for module usage
export default LifeSimulatorApp;