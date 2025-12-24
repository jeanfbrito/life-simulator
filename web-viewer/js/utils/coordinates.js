/**
 * Coordinate conversion utilities for the Life Simulator web viewer
 * Handles conversions between screen, world, and chunk coordinate systems
 */

import { CONFIG } from '../config.js';

export class CoordinateConverter {
    /**
     * Convert screen coordinates to world coordinates
     * @param {number} screenX - X coordinate in viewport
     * @param {number} screenY - Y coordinate in viewport
     * @param {number} viewSizeX - Viewport width in tiles
     * @param {number} viewSizeY - Viewport height in tiles
     * @returns {{x: number, y: number}} World coordinates
     */
    static screenToWorld(screenX, screenY, viewSizeX, viewSizeY) {
        return {
            x: screenX - Math.floor(viewSizeX / 2),
            y: screenY - Math.floor(viewSizeY / 2)
        };
    }

    /**
     * Convert world coordinates to screen coordinates
     * @param {number} worldX - X coordinate in world space
     * @param {number} worldY - Y coordinate in world space
     * @param {number} viewSizeX - Viewport width in tiles
     * @param {number} viewSizeY - Viewport height in tiles
     * @returns {{x: number, y: number}} Screen coordinates
     */
    static worldToScreen(worldX, worldY, viewSizeX, viewSizeY) {
        return {
            x: worldX + Math.floor(viewSizeX / 2),
            y: worldY + Math.floor(viewSizeY / 2)
        };
    }

    /**
     * Convert world coordinates to chunk coordinates with local tile position
     * @param {number} worldX - X coordinate in world space
     * @param {number} worldY - Y coordinate in world space
     * @param {number} chunkSize - Size of each chunk (default: 16)
     * @returns {{chunkX: number, chunkY: number, localX: number, localY: number}}
     */
    static worldToChunk(worldX, worldY, chunkSize = 16) {
        return {
            chunkX: Math.floor(worldX / chunkSize),
            chunkY: Math.floor(worldY / chunkSize),
            localX: ((worldX % chunkSize) + chunkSize) % chunkSize,
            localY: ((worldY % chunkSize) + chunkSize) % chunkSize
        };
    }

    /**
     * Create chunk key string from chunk coordinates
     * @param {number} chunkX - Chunk X coordinate
     * @param {number} chunkY - Chunk Y coordinate
     * @returns {string} Chunk key in format "x,y"
     */
    static chunkKey(chunkX, chunkY) {
        return `${chunkX},${chunkY}`;
    }

    /**
     * Parse chunk key string into coordinates
     * @param {string} key - Chunk key in format "x,y"
     * @returns {{chunkX: number, chunkY: number}}
     */
    static parseChunkKey(key) {
        const [chunkX, chunkY] = key.split(',').map(Number);
        return { chunkX, chunkY };
    }

    /**
     * Convert canvas pixel coordinates to world coordinates with camera offset
     * @param {number} canvasX - X coordinate in canvas pixels
     * @param {number} canvasY - Y coordinate in canvas pixels
     * @param {number} dragOffset - Current camera drag offset in pixels
     * @returns {{worldX: number, worldY: number, screenX: number, screenY: number}}
     */
    static canvasToWorld(canvasX, canvasY, dragOffset) {
        const screenX = Math.floor(canvasX / CONFIG.TILE_SIZE);
        const screenY = Math.floor(canvasY / CONFIG.TILE_SIZE);

        const world = this.screenToWorld(screenX, screenY, CONFIG.VIEW_SIZE_X, CONFIG.VIEW_SIZE_Y);

        return {
            worldX: world.x,
            worldY: world.y,
            screenX: screenX,
            screenY: screenY
        };
    }

    /**
     * Convert world coordinates to screen pixel coordinates for rendering
     * @param {number} worldX - X coordinate in world space
     * @param {number} worldY - Y coordinate in world space
     * @param {number} cameraOffsetX - Camera X offset in tiles
     * @param {number} cameraOffsetY - Camera Y offset in tiles
     * @returns {{screenPixelX: number, screenPixelY: number, screenTileY: number}}
     */
    static worldToScreenPixels(worldX, worldY, cameraOffsetX, cameraOffsetY) {
        const screenPixelX = (worldX - cameraOffsetX + Math.floor(CONFIG.VIEW_SIZE_X / 2)) * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
        const screenPixelY = (worldY - cameraOffsetY + Math.floor(CONFIG.VIEW_SIZE_Y / 2)) * CONFIG.TILE_SIZE + CONFIG.TILE_SIZE / 2;
        const screenTileY = worldY - cameraOffsetY + Math.floor(CONFIG.VIEW_SIZE_Y / 2);

        return {
            screenPixelX: screenPixelX,
            screenPixelY: screenPixelY,
            screenTileY: screenTileY
        };
    }

    /**
     * Get visible world bounds based on camera position
     * @param {number} dragOffset - Current camera drag offset in pixels
     * @returns {{startX: number, startY: number, endX: number, endY: number}}
     */
    static getVisibleBounds(dragOffset) {
        const startWorldX = Math.floor(-dragOffset / CONFIG.TILE_SIZE);
        const startWorldY = Math.floor(-dragOffset / CONFIG.TILE_SIZE);
        const endWorldX = startWorldX + CONFIG.VIEW_SIZE_X;
        const endWorldY = startWorldY + CONFIG.VIEW_SIZE_Y;

        return {
            startX: startWorldX,
            startY: startWorldY,
            endX: endWorldX,
            endY: endWorldY
        };
    }

    /**
     * Check if world coordinates are within valid bounds
     * @param {number} x - X coordinate in screen space
     * @param {number} y - Y coordinate in screen space
     * @returns {boolean}
     */
    static isWithinViewBounds(x, y) {
        return x >= 0 && x < CONFIG.VIEW_SIZE_X && y >= 0 && y < CONFIG.VIEW_SIZE_Y;
    }
}
