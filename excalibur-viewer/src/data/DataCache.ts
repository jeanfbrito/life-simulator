import { Logger } from '@/utils/Logger'
import type {
  ChunkLayersResponse,
  EntitiesResponse,
  SpeciesResponse,
  VegetationMetricsResponse,
  CacheOptions,
  Position,
} from './types'

/**
 * Cache entry with metadata
 */
interface CacheEntry<T> {
  data: T
  timestamp: number
  accessCount: number
  lastAccessed: number
}

/**
 * Cache statistics
 */
export interface CacheStats {
  totalEntries: number
  memoryUsage: number // Estimated bytes
  hitRate: number // Percentage
  totalHits: number
  totalMisses: number
}

/**
 * Generic cache with LRU eviction and TTL support
 */
class LRUCache<K, V> {
  private cache = new Map<K, CacheEntry<V>>()
  private maxSize: number
  private ttl: number // Time to live in milliseconds

  constructor(maxSize: number, ttl: number) {
    this.maxSize = maxSize
    this.ttl = ttl
  }

  /**
   * Get value from cache
   */
  get(key: K): V | undefined {
    const entry = this.cache.get(key)

    if (!entry) {
      return undefined
    }

    // Check TTL
    if (Date.now() - entry.timestamp > this.ttl) {
      this.cache.delete(key)
      return undefined
    }

    // Update access metadata
    entry.accessCount++
    entry.lastAccessed = Date.now()

    // Move to end (LRU)
    this.cache.delete(key)
    this.cache.set(key, entry)

    return entry.data
  }

  /**
   * Set value in cache
   */
  set(key: K, value: V): void {
    // Delete existing entry if present
    if (this.cache.has(key)) {
      this.cache.delete(key)
    }

    // Evict oldest if at capacity
    if (this.cache.size >= this.maxSize) {
      const oldestKey = this.cache.keys().next().value
      if (oldestKey !== undefined) {
        this.cache.delete(oldestKey)
      }
    }

    this.cache.set(key, {
      data: value,
      timestamp: Date.now(),
      accessCount: 1,
      lastAccessed: Date.now(),
    })
  }

  /**
   * Check if key exists
   */
  has(key: K): boolean {
    const entry = this.cache.get(key)
    if (!entry) return false

    // Check TTL
    if (Date.now() - entry.timestamp > this.ttl) {
      this.cache.delete(key)
      return false
    }

    return true
  }

  /**
   * Delete entry
   */
  delete(key: K): boolean {
    return this.cache.delete(key)
  }

  /**
   * Clear all entries
   */
  clear(): void {
    this.cache.clear()
  }

  /**
   * Get cache size
   */
  size(): number {
    return this.cache.size
  }

  /**
   * Get all keys
   */
  keys(): K[] {
    return Array.from(this.cache.keys())
  }

  /**
   * Clean up expired entries
   */
  cleanup(): number {
    const now = Date.now()
    const initialSize = this.cache.size

    for (const [key, entry] of this.cache.entries()) {
      if (now - entry.timestamp > this.ttl) {
        this.cache.delete(key)
      }
    }

    return initialSize - this.cache.size
  }

  /**
   * Get cache statistics
   */
  getStats(): { size: number; entries: CacheEntry<V>[] } {
    return {
      size: this.cache.size,
      entries: Array.from(this.cache.values()),
    }
  }
}

/**
 * Comprehensive data cache for Life Simulator
 * Manages chunk data, entities, species config, and vegetation metrics
 */
export class DataCache {
  private logger: Logger
  private options: CacheOptions
  private chunkCache: LRUCache<string, ChunkLayersResponse['chunk_data'][string]>
  private entityCache: LRUCache<string, EntitiesResponse>
  private speciesCache: LRUCache<string, SpeciesResponse>
  private vegetationCache: LRUCache<string, VegetationMetricsResponse>

  // Statistics
  private totalHits = 0
  private totalMisses = 0

  constructor(options: Partial<CacheOptions> = {}) {
    this.logger = new Logger('DataCache')

    this.options = {
      maxChunks: 1000,
      maxEntities: 100,
      ttl: 300000, // 5 minutes
      ...options,
    }

    // Initialize caches with different sizes based on usage patterns
    this.chunkCache = new LRUCache(this.options.maxChunks, this.options.ttl)
    this.entityCache = new LRUCache(this.options.maxEntities, this.options.ttl)
    this.speciesCache = new LRUCache(10, this.options.ttl) // Species changes rarely
    this.vegetationCache = new LRUCache(50, this.options.ttl) // Moderate caching for metrics

    this.logger.info('Data cache initialized', this.options)
  }

  // ============================================================================
  // Chunk Data Caching
  // ============================================================================

  /**
   * Get chunk data for specific coordinates
   */
  getChunk(chunkKey: string): ChunkLayersResponse['chunk_data'][string] | undefined {
    const data = this.chunkCache.get(chunkKey)
    if (data !== undefined) {
      this.totalHits++
    } else {
      this.totalMisses++
    }
    return data
  }

  /**
   * Set chunk data
   */
  setChunk(chunkKey: string, chunkData: ChunkLayersResponse['chunk_data'][string]): void {
    this.chunkCache.set(chunkKey, chunkData)
  }

  /**
   * Get multiple chunks
   */
  getChunks(chunkKeys: string[]): Map<string, ChunkLayersResponse['chunk_data'][string]> {
    const result = new Map<string, ChunkLayersResponse['chunk_data'][string]>()

    for (const key of chunkKeys) {
      const chunk = this.getChunk(key)
      if (chunk) {
        result.set(key, chunk)
      }
    }

    return result
  }

  /**
   * Set multiple chunks
   */
  setChunks(chunks: Record<string, ChunkLayersResponse['chunk_data'][string]>): void {
    for (const [key, data] of Object.entries(chunks)) {
      this.setChunk(key, data)
    }
  }

  /**
   * Get chunks in a radius around a center point
   */
  getChunksInRadius(centerX: number, centerY: number, radius: number): Map<string, ChunkLayersResponse['chunk_data'][string]> {
    const result = new Map<string, ChunkLayersResponse['chunk_data'][string]>()

    for (let x = centerX - radius; x <= centerX + radius; x++) {
      for (let y = centerY - radius; y <= centerY + radius; y++) {
        const key = `${x},${y}`
        const chunk = this.getChunk(key)
        if (chunk) {
          result.set(key, chunk)
        }
      }
    }

    return result
  }

  // ============================================================================
  // Entity Data Caching
  // ============================================================================

  /**
   * Get current entity data
   */
  getEntities(): EntitiesResponse | undefined {
    const data = this.entityCache.get('current')
    if (data !== undefined) {
      this.totalHits++
    } else {
      this.totalMisses++
    }
    return data
  }

  /**
   * Set entity data
   */
  setEntities(entities: EntitiesResponse): void {
    this.entityCache.set('current', entities)
  }

  /**
   * Get entities by type
   */
  getEntitiesByType(entityType: string): EntitiesResponse['entities'] {
    const entitiesData = this.getEntities()
    if (!entitiesData) {
      return []
    }

    return entitiesData.entities.filter(entity => entity.entity_type === entityType)
  }

  /**
   * Get entities in a specific area
   */
  getEntitiesInArea(center: Position, radius: number): EntitiesResponse['entities'] {
    const entitiesData = this.getEntities()
    if (!entitiesData) {
      return []
    }

    return entitiesData.entities.filter(entity => {
      const distance = Math.sqrt(
        Math.pow(entity.position.x - center.x, 2) +
        Math.pow(entity.position.y - center.y, 2)
      )
      return distance <= radius
    })
  }

  // ============================================================================
  // Species Configuration Caching
  // ============================================================================

  /**
   * Get species configuration
   */
  getSpecies(): SpeciesResponse | undefined {
    const data = this.speciesCache.get('current')
    if (data !== undefined) {
      this.totalHits++
    } else {
      this.totalMisses++
    }
    return data
  }

  /**
   * Set species configuration
   */
  setSpecies(species: SpeciesResponse): void {
    this.speciesCache.set('current', species)
  }

  // ============================================================================
  // Vegetation Metrics Caching
  // ============================================================================

  /**
   * Get vegetation metrics
   */
  getVegetationMetrics(): VegetationMetricsResponse | undefined {
    const data = this.vegetationCache.get('current')
    if (data !== undefined) {
      this.totalHits++
    } else {
      this.totalMisses++
    }
    return data
  }

  /**
   * Set vegetation metrics
   */
  setVegetationMetrics(metrics: VegetationMetricsResponse): void {
    this.vegetationCache.set('current', metrics)
  }

  // ============================================================================
  // Cache Management
  // ============================================================================

  /**
   * Clean up expired entries
   */
  cleanup(): number {
    const chunkCleaned = this.chunkCache.cleanup()
    const entityCleaned = this.entityCache.cleanup()
    const speciesCleaned = this.speciesCache.cleanup()
    const vegetationCleaned = this.vegetationCache.cleanup()

    const totalCleaned = chunkCleaned + entityCleaned + speciesCleaned + vegetationCleaned
    if (totalCleaned > 0) {
      this.logger.debug(`Cleaned up ${totalCleaned} expired cache entries`)
    }

    return totalCleaned
  }

  /**
   * Clear all caches
   */
  clear(): void {
    this.chunkCache.clear()
    this.entityCache.clear()
    this.speciesCache.clear()
    this.vegetationCache.clear()
    this.totalHits = 0
    this.totalMisses = 0

    this.logger.info('All caches cleared')
  }

  /**
   * Get comprehensive cache statistics
   */
  getStats(): CacheStats {
    const totalEntries =
      this.chunkCache.size() +
      this.entityCache.size() +
      this.speciesCache.size() +
      this.vegetationCache.size()

    // Rough memory estimation (each entry ~1KB average)
    const memoryUsage = totalEntries * 1024

    const totalRequests = this.totalHits + this.totalMisses
    const hitRate = totalRequests > 0 ? (this.totalHits / totalRequests) * 100 : 0

    return {
      totalEntries,
      memoryUsage,
      hitRate,
      totalHits: this.totalHits,
      totalMisses: this.totalMisses,
    }
  }

  /**
   * Get detailed statistics for each cache type
   */
  getDetailedStats(): {
    chunks: { size: number; entries: CacheEntry<unknown>[] }
    entities: { size: number; entries: CacheEntry<unknown>[] }
    species: { size: number; entries: CacheEntry<unknown>[] }
    vegetation: { size: number; entries: CacheEntry<unknown>[] }
  } {
    return {
      chunks: this.chunkCache.getStats(),
      entities: this.entityCache.getStats(),
      species: this.speciesCache.getStats(),
      vegetation: this.vegetationCache.getStats(),
    }
  }

  /**
   * Update cache options
   */
  updateOptions(options: Partial<CacheOptions>): void {
    this.options = { ...this.options, ...options }
    this.logger.info('Cache options updated', this.options)
  }

  /**
   * Get current cache options
   */
  getOptions(): Readonly<CacheOptions> {
    return { ...this.options }
  }

  /**
   * Force cleanup and return statistics
   */
  performMaintenance(): CacheStats {
    this.cleanup()
    return this.getStats()
  }
}