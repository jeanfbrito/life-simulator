/**
 * Entity Manager for the Life Simulator Viewer
 * Fetches and manages entities from the server
 */

import { CONFIG } from './config.js';
import { fetchWithTimeout } from './utils/fetch-timeout.js';

export class EntityManager {
    constructor() {
        this.entities = [];
        this.isPolling = false;
        this.pollInterval = null;
        this.lastUpdateTime = 0;
        this.previousCount = 0;

        // Circuit breaker properties
        this.failureCount = 0;
        this.maxFailures = CONFIG.MAX_FAILURES;
        this.currentInterval = CONFIG.ENTITY_POLL_INTERVAL_MS; // Will be set by startPolling
        this.baseInterval = CONFIG.ENTITY_POLL_INTERVAL_MS; // Store base interval for reset
        this.maxBackoffInterval = CONFIG.MAX_BACKOFF_INTERVAL_MS; // Max backoff interval between retries
        this.circuitOpen = false;
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
        this.baseInterval = intervalMs;
        this.currentInterval = intervalMs;
        console.log(`🎯 ENTITY_MANAGER: Starting entity polling every ${intervalMs}ms`);

        // Fetch immediately
        this.fetchEntities();

        // Schedule next poll with interval that may be backed off
        this.scheduleNextPoll();
    }

    /**
     * Stop polling for entities
     */
    stopPolling() {
        if (this.pollInterval) {
            clearTimeout(this.pollInterval);
            this.pollInterval = null;
            this.isPolling = false;
            this.circuitOpen = false;
            this.failureCount = 0;
            this.currentInterval = this.baseInterval;
            console.log('🎯 ENTITY_MANAGER: Stopped entity polling');
        }
    }

    /**
     * Fetch entities from the server with circuit breaker and exponential backoff
     */
    async fetchEntities() {
        try {
            const response = await fetchWithTimeout(`${CONFIG.apiBaseUrl}/api/entities`, {}, CONFIG.FETCH_TIMEOUT_MS);

            if (!response.ok) {
                throw new Error(`HTTP error! status: ${response.status}`);
            }

            const data = await response.json();

            if (data.entities && Array.isArray(data.entities)) {
                this.entities = data.entities;
                this.lastUpdateTime = Date.now();

                // Success - reset failure count and interval
                this.resetCircuitBreaker();

                // Log entity count for debugging (only if changed)
                if (this.previousCount !== this.entities.length) {
                    console.log(`🎯 ENTITY_MANAGER: Fetched ${this.entities.length} entities`);
                    this.previousCount = this.entities.length;
                }
            } else {
                console.warn('🎯 ENTITY_MANAGER: Invalid entity data format', data);
                this.handleFetchFailure('Invalid entity data format');
            }
        } catch (error) {
            this.handleFetchFailure(error.message);
        }
    }

    /**
     * Handle fetch failure with circuit breaker logic
     * @param {string} errorMessage - Error message to log
     */
    handleFetchFailure(errorMessage) {
        // Don't clear entities on error - keep displaying last known state
        this.failureCount++;

        console.warn(`🎯 ENTITY_MANAGER: Fetch failed (${this.failureCount}/${this.maxFailures}): ${errorMessage}`);

        // Check circuit breaker threshold
        if (this.failureCount >= this.maxFailures && !this.circuitOpen) {
            this.circuitOpen = true;
            console.error(`🎯 ENTITY_MANAGER: Circuit breaker OPENED after ${this.failureCount} failures`);
            this.showErrorUI();
        }

        // Apply exponential backoff if circuit is open
        if (this.circuitOpen) {
            this.currentInterval = Math.min(
                this.currentInterval * 2,
                this.maxBackoffInterval
            );
            console.log(`🎯 ENTITY_MANAGER: Exponential backoff - next retry in ${this.currentInterval}ms`);
        }
    }

    /**
     * Display error UI to user
     */
    showErrorUI() {
        // Try to find the entity list container or stats area
        const entityStatsElement = document.getElementById('entity-count');
        const pageTitle = document.querySelector('h1') || document.querySelector('title');

        // Add visual indicator to entity stats if available
        if (entityStatsElement) {
            const parent = entityStatsElement.parentElement;
            if (parent && !parent.querySelector('.connection-error')) {
                const errorIndicator = document.createElement('div');
                errorIndicator.className = 'connection-error';
                errorIndicator.style.cssText = `
                    margin-top: 0.5rem;
                    padding: 0.5rem;
                    background-color: #fee2e2;
                    border: 1px solid #fecaca;
                    border-radius: 0.25rem;
                    color: #dc2626;
                    font-size: 0.85rem;
                    text-align: center;
                `;
                errorIndicator.innerHTML = `
                    <strong>⚠️ Connection Issues</strong><br>
                    <span style="opacity: 0.8;">API unreachable. Retrying with exponential backoff...</span>
                `;
                parent.appendChild(errorIndicator);
            }
        }
    }

    /**
     * Reset circuit breaker on successful request
     */
    resetCircuitBreaker() {
        if (this.circuitOpen || this.failureCount > 0) {
            console.log(`🎯 ENTITY_MANAGER: Circuit breaker CLOSED - connection restored`);
            this.circuitOpen = false;
            this.failureCount = 0;
            this.currentInterval = this.baseInterval;

            // Remove error indicator if present
            const errorIndicator = document.querySelector('.connection-error');
            if (errorIndicator) {
                errorIndicator.remove();
            }
        }
    }

    /**
     * Schedule the next poll with current interval (may be backed off)
     */
    scheduleNextPoll() {
        if (!this.isPolling) return;

        this.pollInterval = setTimeout(() => {
            this.fetchEntities();
            this.scheduleNextPoll();
        }, this.currentInterval);
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
     * Manually reset circuit breaker (e.g., after API comes back online)
     */
    manualReset() {
        console.log('🎯 ENTITY_MANAGER: Manual circuit breaker reset');
        this.resetCircuitBreaker();
        // Immediately try to fetch
        this.fetchEntities();
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
