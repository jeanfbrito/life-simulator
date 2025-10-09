import { APIClient } from './APIClient'
import { Logger } from '@/utils/Logger'
import type { PollingOptions } from './types'

/**
 * Polling event types
 */
export enum PollingEventType {
  ENTITIES_UPDATED = 'entities_updated',
  VEGETATION_UPDATED = 'vegetation_updated',
  SPECIES_UPDATED = 'species_updated',
  ERROR = 'error',
  CONNECTION_STATUS_CHANGED = 'connection_status_changed',
}

/**
 * Polling event data
 */
export interface PollingEvent {
  type: PollingEventType
  data: unknown
  timestamp: number
  error?: Error
}

/**
 * Connection status
 */
export enum ConnectionStatus {
  CONNECTED = 'connected',
  DISCONNECTED = 'disconnected',
  CONNECTING = 'connecting',
  ERROR = 'error',
}

/**
 * Manages periodic polling of backend APIs
 * Handles configurable intervals, error recovery, and event dispatching
 */
export class PollingController {
  private apiClient: APIClient
  private logger: Logger
  private options: PollingOptions
  private intervals: Map<string, number> = new Map()
  private isRunning = false
  private connectionStatus: ConnectionStatus = ConnectionStatus.DISCONNECTED
  private eventListeners: Map<PollingEventType, Set<(event: PollingEvent) => void>> = new Map()
  private consecutiveErrors = 0
  private maxConsecutiveErrors = 5
  private backoffMultiplier = 1.5
  private currentBackoff = 1

  constructor(apiClient: APIClient, options: Partial<PollingOptions> = {}) {
    this.apiClient = apiClient
    this.logger = new Logger('PollingController')

    this.options = {
      entities: 200, // 5 times per second
      vegetation: 5000, // Once every 5 seconds
      chunks: 1000, // On-demand (placeholder)
      ...options,
    }

    // Initialize event listener collections
    Object.values(PollingEventType).forEach(eventType => {
      this.eventListeners.set(eventType, new Set())
    })
  }

  /**
   * Start all polling intervals
   */
  start(): void {
    if (this.isRunning) {
      this.logger.warn('Polling controller is already running')
      return
    }

    this.logger.info('Starting polling controller')
    this.isRunning = true
    this.consecutiveErrors = 0
    this.currentBackoff = 1
    this.setConnectionStatus(ConnectionStatus.CONNECTING)

    // Start polling intervals
    this.startPolling('entities', this.options.entities, () => this.pollEntities())
    this.startPolling('vegetation', this.options.vegetation, () => this.pollVegetation())

    // Initial species fetch (one-time)
    this.pollSpecies().catch(error => {
      this.logger.error('Failed to fetch initial species data:', error)
    })
  }

  /**
   * Stop all polling intervals
   */
  stop(): void {
    if (!this.isRunning) {
      return
    }

    this.logger.info('Stopping polling controller')
    this.isRunning = false

    // Clear all intervals
    this.intervals.forEach((interval, name) => {
      clearInterval(interval)
      this.logger.debug(`Stopped polling for ${name}`)
    })
    this.intervals.clear()

    this.setConnectionStatus(ConnectionStatus.DISCONNECTED)
  }

  /**
   * Start a specific polling interval
   */
  private startPolling(name: string, intervalMs: number, pollFunction: () => Promise<void>): void {
    if (this.intervals.has(name)) {
      this.logger.warn(`Polling for ${name} is already running`)
      return
    }

    this.logger.debug(`Starting polling for ${name} every ${intervalMs}ms`)

    // Initial poll
    setTimeout(async () => {
      if (this.isRunning) {
        try {
          await pollFunction()
        } catch (error) {
          this.logger.error(`Initial poll failed for ${name}:`, error)
          this.handleError(error)
        }
      }
    }, 100) // Small delay to avoid immediate flood

    // Set up recurring interval
    const interval = setInterval(() => {
      if (this.isRunning) {
        pollFunction().catch(error => {
          this.logger.error(`Polling failed for ${name}:`, error)
          this.handleError(error)
        })
      }
    }, intervalMs)

    this.intervals.set(name, interval)
  }

  /**
   * Poll entities data
   */
  private async pollEntities(): Promise<void> {
    try {
      const data = await this.apiClient.getEntities()
      this.consecutiveErrors = 0
      this.currentBackoff = 1

      if (this.connectionStatus !== ConnectionStatus.CONNECTED) {
        this.setConnectionStatus(ConnectionStatus.CONNECTED)
      }

      this.dispatchEvent({
        type: PollingEventType.ENTITIES_UPDATED,
        data,
        timestamp: Date.now(),
      })

    } catch (error) {
      throw error // Let the main error handler deal with it
    }
  }

  /**
   * Poll vegetation metrics
   */
  private async pollVegetation(): Promise<void> {
    try {
      const data = await this.apiClient.getVegetationMetrics()
      this.dispatchEvent({
        type: PollingEventType.VEGETATION_UPDATED,
        data,
        timestamp: Date.now(),
      })

    } catch (error) {
      throw error
    }
  }

  /**
   * Poll species data (one-time fetch with refresh capability)
   */
  private async pollSpecies(): Promise<void> {
    try {
      const data = await this.apiClient.getSpecies()
      this.dispatchEvent({
        type: PollingEventType.SPECIES_UPDATED,
        data,
        timestamp: Date.now(),
      })

    } catch (error) {
      throw error
    }
  }

  /**
   * Refresh species data manually
   */
  async refreshSpecies(): Promise<void> {
    await this.pollSpecies()
  }

  /**
   * Handle polling errors with backoff strategy
   */
  private handleError(error: unknown): void {
    this.consecutiveErrors++

    if (this.consecutiveErrors >= this.maxConsecutiveErrors) {
      this.setConnectionStatus(ConnectionStatus.ERROR)

      // Apply exponential backoff
      const backoffDelay = this.currentBackoff * 1000
      this.currentBackoff *= this.backoffMultiplier

      this.logger.warn(`Too many consecutive errors (${this.consecutiveErrors}). Applying backoff: ${backoffDelay}ms`)

      // Stop polling temporarily
      this.stop()

      // Schedule restart with backoff
      setTimeout(() => {
        this.logger.info('Attempting to restart polling after backoff')
        this.start()
      }, backoffDelay)
    }

    this.dispatchEvent({
      type: PollingEventType.ERROR,
      data: error,
      timestamp: Date.now(),
      error: error instanceof Error ? error : new Error('Unknown polling error'),
    })
  }

  /**
   * Set connection status and dispatch event
   */
  private setConnectionStatus(status: ConnectionStatus): void {
    if (this.connectionStatus !== status) {
      this.connectionStatus = status
      this.logger.info(`Connection status changed to: ${status}`)

      this.dispatchEvent({
        type: PollingEventType.CONNECTION_STATUS_CHANGED,
        data: status,
        timestamp: Date.now(),
      })
    }
  }

  /**
   * Dispatch event to all listeners
   */
  private dispatchEvent(event: PollingEvent): void {
    const listeners = this.eventListeners.get(event.type)
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(event)
        } catch (error) {
          this.logger.error(`Error in event listener for ${event.type}:`, error)
        }
      })
    }
  }

  /**
   * Add event listener
   */
  addEventListener(eventType: PollingEventType, listener: (event: PollingEvent) => void): void {
    const listeners = this.eventListeners.get(eventType)
    if (listeners) {
      listeners.add(listener)
    }
  }

  /**
   * Remove event listener
   */
  removeEventListener(eventType: PollingEventType, listener: (event: PollingEvent) => void): void {
    const listeners = this.eventListeners.get(eventType)
    if (listeners) {
      listeners.delete(listener)
    }
  }

  /**
   * Update polling options
   */
  updateOptions(options: Partial<PollingOptions>): void {
    const wasRunning = this.isRunning

    if (wasRunning) {
      this.stop()
    }

    this.options = { ...this.options, ...options }
    this.logger.info('Updated polling options:', this.options)

    if (wasRunning) {
      this.start()
    }
  }

  /**
   * Get current connection status
   */
  getConnectionStatus(): ConnectionStatus {
    return this.connectionStatus
  }

  /**
   * Get current polling options
   */
  getOptions(): Readonly<PollingOptions> {
    return { ...this.options }
  }

  /**
   * Check if polling is currently active
   */
  isActive(): boolean {
    return this.isRunning
  }

  /**
   * Get consecutive error count
   */
  getConsecutiveErrors(): number {
    return this.consecutiveErrors
  }

  /**
   * Reset error count and backoff
   */
  resetErrorState(): void {
    this.consecutiveErrors = 0
    this.currentBackoff = 1
    this.logger.debug('Reset error state')
  }

  /**
   * Clean up resources
   */
  cleanup(): void {
    this.stop()
    this.eventListeners.clear()
    this.logger.info('Polling controller cleaned up')
  }
}