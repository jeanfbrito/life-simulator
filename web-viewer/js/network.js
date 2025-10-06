/**
 * WebSocket and network communication for the Life Simulator Viewer
 */

import { CONFIG } from './config.js';

function determineWebSocketUrl() {
    if (typeof window !== 'undefined' && window.location) {
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        return `${protocol}//${window.location.host}/ws`;
    }
    const defaultHost = CONFIG.apiBaseUrl.replace(/^https?:\/\//, '');
    return `ws://${defaultHost}/ws`;
}

export class NetworkManager {
    constructor() {
        this.ws = null;
        this.reconnectAttempts = 0;
        this.maxReconnectAttempts = 5;
        this.reconnectDelay = 1000;
        this.isConnected = false;
        this.onMessageCallbacks = [];
        this.onConnectionChangeCallbacks = [];
    }

    connect() {
        try {
            this.ws = new WebSocket(determineWebSocketUrl());

            this.ws.onopen = () => {
                console.log('WebSocket connected');
                this.isConnected = true;
                this.reconnectAttempts = 0;
                this.updateConnectionStatus(true);
                this.notifyConnectionChange(true);
            };

            this.ws.onmessage = (event) => {
                try {
                    const message = JSON.parse(event.data);
                    this.handleMessage(message);
                } catch (error) {
                    console.error('Error parsing WebSocket message:', error);
                }
            };

            this.ws.onclose = () => {
                console.log('WebSocket disconnected');
                this.isConnected = false;
                this.updateConnectionStatus(false);
                this.notifyConnectionChange(false);
                this.attemptReconnect();
            };

            this.ws.onerror = (error) => {
                console.error('WebSocket error:', error);
                this.updateConnectionStatus(false);
            };

        } catch (error) {
            console.error('Failed to create WebSocket connection:', error);
            this.updateConnectionStatus(false);
        }
    }

    attemptReconnect() {
        if (this.reconnectAttempts < this.maxReconnectAttempts) {
            this.reconnectAttempts++;
            console.log(`Attempting to reconnect (${this.reconnectAttempts}/${this.maxReconnectAttempts})...`);

            setTimeout(() => {
                this.connect();
            }, this.reconnectDelay * this.reconnectAttempts);
        } else {
            console.log('Max reconnection attempts reached');
        }
    }

    handleMessage(message) {
        console.log('Received message:', message);

        // Notify all registered callbacks
        this.onMessageCallbacks.forEach(callback => {
            try {
                callback(message);
            } catch (error) {
                console.error('Error in message callback:', error);
            }
        });
    }

    updateConnectionStatus(connected) {
        const status = document.getElementById('connection-status');
        if (connected) {
            status.className = 'status-item connected';
            status.innerHTML = '<span class="status-dot">🟢</span><span>Connected</span>';
        } else {
            status.className = 'status-item disconnected';
            status.innerHTML = '<span class="status-dot">🔴</span><span>Disconnected</span>';
        }
    }

    // Message handling for different types
    static handleWorldInfoMessage(message, onWorldInfoUpdate) {
        if (message.type === 'world_info' && onWorldInfoUpdate) {
            onWorldInfoUpdate(message.data);
        }
    }

    static handleChunkDataMessage(message, onChunkDataUpdate) {
        if (message.type === 'chunk_data' && onChunkDataUpdate) {
            onChunkDataUpdate(message.data);
        }
    }

    static handleWorldStatsMessage(message, onWorldStatsUpdate) {
        if (message.type === 'world_stats' && onWorldStatsUpdate) {
            onWorldStatsUpdate(message.data);
        }
    }

    static handleErrorMessage(message) {
        if (message.type === 'error') {
            console.error('Server error:', message.message);
        }
    }

    // Register callbacks
    onMessage(callback) {
        this.onMessageCallbacks.push(callback);
    }

    onConnectionChange(callback) {
        this.onConnectionChangeCallbacks.push(callback);
    }

    notifyConnectionChange(connected) {
        this.onConnectionChangeCallbacks.forEach(callback => {
            try {
                callback(connected);
            } catch (error) {
                console.error('Error in connection change callback:', error);
            }
        });
    }

    // Send messages
    sendMessage(message) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
            this.ws.send(JSON.stringify(message));
        } else {
            console.warn('WebSocket not connected, cannot send message:', message);
        }
    }

    // Request specific data
    requestWorldInfo() {
        this.sendMessage({ type: 'get_world_info' });
    }

    requestChunkData(chunkX, chunkY) {
        this.sendMessage({
            type: 'get_chunk_data',
            chunk_x: chunkX,
            chunk_y: chunkY
        });
    }

    requestWorldStats() {
        this.sendMessage({ type: 'get_world_stats' });
    }

    // Disconnect
    disconnect() {
        if (this.ws) {
            this.ws.close();
            this.ws = null;
        }
        this.isConnected = false;
        this.updateConnectionStatus(false);
    }

    // Get connection status
    getConnectionStatus() {
        return this.isConnected;
    }
}
