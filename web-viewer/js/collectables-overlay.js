/**
 * Collectables Debug Overlay
 *
 * Provides visualization of collectable resources, their statistics,
 * and debugging information for the web viewer.
 */

import { CONFIG } from './config.js';

export class CollectablesOverlay {
    constructor(renderer, chunkManager) {
        this.renderer = renderer;
        this.chunkManager = chunkManager;
        this.isVisible = false;
        this.collectableData = null;
        this.collectableTypes = null;
        this.lastUpdate = 0;
        this.updateInterval = 5000; // Update every 5 seconds
    }

    /**
     * Toggle the collectable overlay visibility
     */
    toggle() {
        this.isVisible = !this.isVisible;
        CONFIG.showCollectableOverlay = this.isVisible;

        if (this.isVisible) {
            this.fetchCollectableData();
        }

        return this.isVisible;
    }

    /**
     * Set the opacity of the overlay
     */
    setOpacity(opacity) {
        CONFIG.collectableOverlayOpacity = Math.max(0.1, Math.min(1.0, opacity));
    }

    /**
     * Fetch collectable data from the API
     */
    async fetchCollectableData() {
        try {
            const [statsResponse, debugResponse, typesResponse] = await Promise.all([
                fetch(`${CONFIG.apiBaseUrl}/api/collectables/stats`),
                fetch(`${CONFIG.apiBaseUrl}/api/collectables/debug`),
                fetch(`${CONFIG.apiBaseUrl}/api/collectables/types`)
            ]);

            const stats = await statsResponse.json();
            const debug = await debugResponse.json();
            const types = await typesResponse.json();

            this.collectableData = { stats, debug };
            this.collectableTypes = types;
            this.lastUpdate = Date.now();

            console.log('🧺 Collectables data updated:', this.collectableData);
        } catch (error) {
            console.error('Failed to fetch collectable data:', error);
            this.collectableData = null;
        }
    }

    /**
     * Update collectable data if needed
     */
    updateIfNeeded() {
        if (!this.isVisible) return;

        const now = Date.now();
        if (now - this.lastUpdate > this.updateInterval) {
            this.fetchCollectableData();
        }
    }

    /**
     * Render the collectable overlay
     */
    render(ctx, viewBounds) {
        if (!this.isVisible || !this.collectableData) {
            return;
        }

        const { stats, debug } = this.collectableData;

        // Save the current context state
        ctx.save();

        // Set overlay opacity
        ctx.globalAlpha = CONFIG.collectableOverlayOpacity;

        // Render collectable hotspots
        this.renderCollectableHotspots(ctx, viewBounds, debug);

        // Render collectable statistics overlay
        this.renderStatisticsOverlay(ctx, stats);

        // Restore the context state
        ctx.restore();
    }

    /**
     * Render collectable hotspots on the map
     */
    renderCollectableHotspots(ctx, viewBounds, debugData) {
        if (!debugData || !debugData.collectables) {
            return;
        }

        const { collectables } = debugData;

        for (const collectable of collectables) {
            const worldX = collectable.position.x;
            const worldY = collectable.position.y;

            // Check if collectable is in view bounds
            if (worldX < viewBounds.minX || worldX > viewBounds.maxX ||
                worldY < viewBounds.minY || worldY > viewBounds.maxY) {
                continue;
            }

            // Convert world coordinates to screen coordinates
            const screenX = (worldX - viewBounds.minX) * CONFIG.TILE_SIZE;
            const screenY = (worldY - viewBounds.minY) * CONFIG.TILE_SIZE;

            // Draw hotspot circle
            ctx.fillStyle = this.getResourceColor(collectable.resource_type);
            ctx.beginPath();
            ctx.arc(screenX, screenY, CONFIG.TILE_SIZE * 0.8, 0, Math.PI * 2);
            ctx.fill();

            // Draw resource symbol
            ctx.font = `${CONFIG.TILE_SIZE * 0.6}px Arial`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillText(this.getResourceSymbol(collectable.resource_type), screenX, screenY);
        }
    }

    /**
     * Render statistics overlay in the corner
     */
    renderStatisticsOverlay(ctx, statsData) {
        if (!statsData || !statsData.statistics) {
            return;
        }

        const padding = 10;
        const lineHeight = 16;
        let y = padding + 40; // Start below world info

        // Draw background
        ctx.fillStyle = 'rgba(0, 0, 0, 0.7)';
        ctx.fillRect(padding, y, 250, Object.keys(statsData.statistics).length * lineHeight + 20);

        // Draw title
        ctx.fillStyle = '#ffffff';
        ctx.font = 'bold 14px Arial';
        ctx.textAlign = 'left';
        ctx.fillText('🧺 Collectables (20 tile radius)', padding + 5, y + 15);

        // Draw statistics
        ctx.font = '12px Arial';
        y += 35;

        for (const [resourceType, stats] of Object.entries(statsData.statistics)) {
            const symbol = this.getResourceSymbol(resourceType);
            const readyText = stats.ready_to_harvest > 0 ? `✅ ${stats.ready_to_harvest}` : `⏳ 0`;

            ctx.fillStyle = '#ffffff';
            ctx.fillText(`${symbol} ${resourceType}: ${stats.count} (${readyText})`, padding + 5, y);
            y += lineHeight;
        }
    }

    /**
     * Get color for a resource type
     */
    getResourceColor(resourceType) {
        const colors = {
            'MushroomPatch': '#d2691e',  // Chocolate
            'WildRoot': '#8b4513',       // Root brown
            'BerryBush': '#8b4513',      // Berry brown
            'HazelShrub': '#654321',     // Hazelnut brown
        };
        return colors[resourceType] || '#666666';
    }

    /**
     * Get symbol for a resource type
     */
    getResourceSymbol(resourceType) {
        const symbols = {
            'MushroomPatch': '🍄',
            'WildRoot': '🥕',
            'BerryBush': '🫐',
            'HazelShrub': '🌿',
        };
        return symbols[resourceType] || '•';
    }

    /**
     * Get collectable type information
     */
    getCollectableTypes() {
        return this.collectableTypes;
    }

    /**
     * Get detailed information about a specific collectable at position
     */
    getCollectableAt(worldX, worldY) {
        if (!this.collectableData || !this.collectableData.debug) {
            return null;
        }

        const collectables = this.collectableData.debug.collectables || [];
        return collectables.find(c =>
            c.position.x === worldX && c.position.y === worldY
        );
    }
}