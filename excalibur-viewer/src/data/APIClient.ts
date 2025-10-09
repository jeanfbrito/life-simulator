import {
  type WorldInfoResponse,
  type ChunkLayersResponse,
  type ChunkRequestParams,
  type EntitiesResponse,
  type SpeciesResponse,
  type VegetationMetricsResponse,
  isAPIError,
} from './types'
import { Logger } from '@/utils/Logger'

/**
 * API Client for communicating with Life Simulator backend
 * Handles HTTP requests, error handling, and response validation
 */
export class APIClient {
  private baseURL: string
  private logger: Logger
  private abortController: AbortController | null = null

  constructor(baseURL: string = 'http://127.0.0.1:54321') {
    this.baseURL = baseURL
    this.logger = new Logger('APIClient')
  }

  /**
   * Generic HTTP request method with error handling
   */
  private async request<T>(endpoint: string, options: RequestInit = {}): Promise<T> {
    const url = `${this.baseURL}${endpoint}`
    const headers = {
      'Content-Type': 'application/json',
      ...options.headers,
    }

    this.logger.debug(`Making ${options.method || 'GET'} request to ${url}`)

    try {
      const response = await fetch(url, {
        ...options,
        headers,
        signal: this.abortController?.signal,
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const data = await response.json()

      // Check if response is an error
      if (isAPIError(data)) {
        throw new Error(`API Error: ${data.error}${data.message ? ` - ${data.message}` : ''}`)
      }

      this.logger.debug(`Successfully received response from ${endpoint}`)
      return data as T

    } catch (error) {
      if (error instanceof Error && error.name === 'AbortError') {
        this.logger.debug(`Request to ${endpoint} was aborted`)
        throw new Error('Request aborted')
      }

      this.logger.error(`Failed to fetch ${endpoint}:`, error)
      throw error
    }
  }

  /**
   * Get world information
   */
  async getWorldInfo(): Promise<WorldInfoResponse> {
    return this.request<WorldInfoResponse>('/api/world_info')
  }

  /**
   * Get list of available worlds
   */
  async getWorlds(): Promise<{ worlds: string[] }> {
    return this.request<{ worlds: string[] }>('/api/worlds')
  }

  /**
   * Select a specific world
   */
  async selectWorld(worldName: string): Promise<{ success: boolean }> {
    return this.request<{ success: boolean }>('/api/world/select', {
      method: 'POST',
      body: JSON.stringify({ world_name: worldName }),
    })
  }

  /**
   * Get current world details
   */
  async getCurrentWorld(): Promise<WorldInfoResponse> {
    return this.request<WorldInfoResponse>('/api/world/current')
  }

  /**
   * Get chunk data with support for batching and layers
   */
  async getChunks(params: ChunkRequestParams): Promise<ChunkLayersResponse> {
    const searchParams = new URLSearchParams()

    if (params.center_x !== undefined && params.center_y !== undefined && params.radius !== undefined) {
      searchParams.append('center_x', params.center_x.toString())
      searchParams.append('center_y', params.center_y.toString())
      searchParams.append('radius', params.radius.toString())
    }

    if (params.coords && params.coords.length > 0) {
      // Batch coordinates to avoid URL length limits
      const batchSize = 10
      const batches: string[][] = []

      for (let i = 0; i < params.coords.length; i += batchSize) {
        batches.push(params.coords.slice(i, i + batchSize))
      }

      // For now, handle first batch. In a real implementation,
      // we might want to make multiple requests and merge results
      if (batches.length > 0) {
        batches[0].forEach(coord => searchParams.append('coords', coord))
      }
    }

    if (params.layers !== undefined) {
      searchParams.append('layers', params.layers.toString())
    }

    const endpoint = `/api/chunks?${searchParams.toString()}`
    return this.request<ChunkLayersResponse>(endpoint)
  }

  /**
   * Get entity data
   */
  async getEntities(): Promise<EntitiesResponse> {
    return this.request<EntitiesResponse>('/api/entities')
  }

  /**
   * Get species configuration
   */
  async getSpecies(): Promise<SpeciesResponse> {
    return this.request<SpeciesResponse>('/api/species')
  }

  /**
   * Get vegetation metrics
   */
  async getVegetationMetrics(): Promise<VegetationMetricsResponse> {
    return this.request<VegetationMetricsResponse>('/api/vegetation/metrics')
  }

  /**
   * Health check to verify backend is accessible
   */
  async healthCheck(): Promise<boolean> {
    try {
      await this.getWorldInfo()
      return true
    } catch (error) {
      this.logger.warn('Health check failed:', error)
      return false
    }
  }

  /**
   * Abort all ongoing requests
   */
  abortAllRequests(): void {
    if (this.abortController) {
      this.abortController.abort()
    }
    this.abortController = new AbortController()
  }

  /**
   * Reset abort controller (call after aborting requests)
   */
  resetAbortController(): void {
    this.abortController = null
  }

  /**
   * Get the current base URL
   */
  getBaseURL(): string {
    return this.baseURL
  }

  /**
   * Update the base URL (useful for configuration changes)
   */
  setBaseURL(url: string): void {
    this.baseURL = url
    this.logger.info(`Base URL updated to: ${url}`)
  }
}