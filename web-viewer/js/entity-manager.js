/**
 * Entity Manager for the Life Simulator Viewer
 * Fetches and manages entities from the server
 */

import { CONFIG } from './config.js';

export class EntityManager {
    constructor() {
        this.entities = [];
        this.isPolling = false;
        this.pollInterval = null;
        this.lastUpdateTime = 0;
    }

    /**
     * Start polling for entities
     * @param {number} intervalMs - Polling interval in milliseconds (default: 1000ms)
     */
    startPolling(intervalMs = 1000) {
        if (this.isPolling) {
            console.warn('Entity polling already started');
            return;
        }

        this.isPolling = true;
        console.log(`🎯 ENTITY_MANAGER: Starting entity polling every ${intervalMs}ms`);

        // Fetch immediately
        this.fetchEntities();

        // Then poll at regular intervals
        this.pollInterval = setInterval(() => {
            this.fetchEntities();
        }, intervalMs);
    }

    /**
     * Stop polling for entities
     */
    stopPolling() {
        if (this.pollInterval) {
            clearInterval(this.pollInterval);
            this.pollInterval = null;
            this.isPolling = false;
            console.log('🎯 ENTITY_MANAGER: Stopped entity polling');
        }
    }

    /**
     * Fetch entities from the server
     */
    async fetchEntities() {
        try {
            const response = await fetch(`${CONFIG.apiBaseUrl}/api/entities`);
            
            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();
            
            if (data.entities && Array.isArray(data.entities)) {
                this.entities = data.entities;
                this.lastUpdateTime = Date.now();
                
                // Log entity count for debugging (only if changed)
                if (this.previousCount !== this.entities.length) {
                    console.log(`🎯 ENTITY_MANAGER: Fetched ${this.entities.length} entities`);
                    this.previousCount = this.entities.length;
                }
            } else {
                console.warn('🎯 ENTITY_MANAGER: Invalid entity data format', data);
                this.entities = [];
            }
        } catch (error) {
            console.error('🎯 ENTITY_MANAGER: Failed to fetch entities:', error);
            // Don't clear entities on error - keep displaying last known state
        }
    }

    /**
     * Get all entities
     * @returns {Array} Array of entity objects
     */
    getEntities() {
        return this.entities;
    }

    /**
     * Get entities within a specific world bounds
     * @param {number} minX - Minimum X coordinate
     * @param {number} minY - Minimum Y coordinate
     * @param {number} maxX - Maximum X coordinate
     * @param {number} maxY - Maximum Y coordinate
     * @returns {Array} Filtered array of entities
     */
    getEntitiesInBounds(minX, minY, maxX, maxY) {
        return this.entities.filter(entity => {
            const x = entity.position?.x || 0;
            const y = entity.position?.y || 0;
            return x >= minX && x <= maxX && y >= minY && y <= maxY;
        });
    }

    /**
     * Get entity count
     * @returns {number} Number of entities
     */
    getEntityCount() {
        return this.entities.length;
    }

    /**
     * Get time since last update
     * @returns {number} Milliseconds since last update
     */
    getTimeSinceUpdate() {
        return Date.now() - this.lastUpdateTime;
    }

    /**
     * Check if polling is active
     * @returns {boolean} True if polling is active
     */
    isActive() {
        return this.isPolling;
    }

    /**
     * Clear all entities and stop polling
     */
    clear() {
        this.stopPolling();
        this.entities = [];
        this.lastUpdateTime = 0;
    }
}
