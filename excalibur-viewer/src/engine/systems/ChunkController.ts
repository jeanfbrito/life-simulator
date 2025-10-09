import { ISystem } from '../SystemRegistry'
import { Scene } from 'excalibur'
import { Logger } from '@/utils/Logger'
import { IsoCameraController } from './IsoCameraController'
import { PerformanceMonitor } from './PerformanceMonitor'
import { CoordinateSystem, Point2D } from '@/utils/CoordinateSystem'
import { TerrainType, ResourceType } from '@/data/types'
import { APIClient } from '@/data/APIClient'
import { DataCache } from '@/data/DataCache'
import { TileActor } from './TileActor'

/**
 * Chunk data with rendering information
 */
interface RenderChunk {
  chunkKey: string
  tileActors: Map<string, TileActor>
  lastAccessTime: number
  isLoaded: boolean
}

/**
 * Chunk controller for managing and rendering terrain
 */
export class ChunkController implements ISystem {
  readonly name = 'ChunkController'

  private logger: Logger
  private cameraController: IsoCameraController
  private performanceMonitor: PerformanceMonitor
  private scene: Scene

  // Data sources
  private apiClient: APIClient
  private dataCache: DataCache

  // Chunk management
  private chunks: Map<string, RenderChunk> = new Map()
  private visibleChunks: Set<string> = new Set()
  private lastVisibleTiles: Point2D[] = []

  // Configuration
  private maxLoadedChunks = 50
  private chunkTimeout = 30000 // 30 seconds

  constructor(cameraController: IsoCameraController, performanceMonitor: PerformanceMonitor, scene: Scene) {
    this.logger = new Logger('ChunkController')
    this.cameraController = cameraController
    this.performanceMonitor = performanceMonitor
    this.scene = scene

    // Initialize data sources
    this.apiClient = new APIClient('http://localhost:54321')
    this.dataCache = new DataCache({
      maxChunks: 100,
      maxEntities: 200,
      ttl: 300000 // 5 minutes
    })
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing chunk controller...')

    // Try to connect to backend
    try {
      await this.testBackendConnection()
      this.logger.info('Backend connection successful')
    } catch (error) {
      this.logger.warn('Backend connection failed, using mock data:', error)
    }

    // Start with some mock chunks for immediate visual feedback
    this.loadMockChunks()

    this.logger.debug('Chunk controller initialized')
  }

  update(deltaTime: number): void {
    // Update visible chunks based on camera position
    this.updateVisibleChunks()

    // Update all visible tile actors
    this.updateTileActors()

    // Cleanup old chunks
    this.cleanupOldChunks()

    // Update performance monitor
    this.performanceMonitor.setChunkCount(this.chunks.size)
  }

  destroy(): void {
    this.logger.debug('Destroying chunk controller...')

    // Clean up all tile actors
    for (const chunk of this.chunks.values()) {
      for (const actor of chunk.tileActors.values()) {
        this.scene.remove(actor)
      }
    }

    this.chunks.clear()
    this.logger.debug('Chunk controller destroyed')
  }

  /**
   * Test backend connection
   */
  private async testBackendConnection(): Promise<void> {
    const worldInfo = await this.apiClient.getWorldInfo()
    this.logger.info(`Connected to world: ${worldInfo.name} with ${worldInfo.chunk_count} chunks`)
  }

  /**
   * Load initial mock chunks for visual feedback
   */
  private loadMockChunks(): void {
    const mockChunks = [
      {
        x: 0,
        y: 0,
        terrain: TerrainType.GRASS,
        resources: [['', 'TreeOak'], ['', 'Bush']]
      },
      {
        x: 1,
        y: 0,
        terrain: TerrainType.GRASS,
        resources: [['Flower', ''], ['', 'Mushroom']]
      },
      {
        x: 0,
        y: 1,
        terrain: TerrainType.GRASS,
        resources: [['TreePine', ''], ['Fruit', '']]
      },
      {
        x: -1,
        y: 0,
        terrain: TerrainType.SHALLOW_WATER,
        resources: [['', ''], ['', '']]
      },
      {
        x: 0,
        y: -1,
        terrain: TerrainType.FOREST,
        resources: [['TreeOak', 'TreePine'], ['TreeBirch', 'TreeOak']]
      },
      {
        x: 1,
        y: 1,
        terrain: TerrainType.SAND,
        resources: [['Rock', 'Stone'], ['', '']]
      },
      {
        x: -1,
        y: -1,
        terrain: TerrainType.MOUNTAINS,
        resources: [['Iron', 'Gold'], ['Stone', 'Copper']]
      },
    ]

    mockChunks.forEach(chunk => {
      this.loadChunkData(chunk.x, chunk.y, [{
        terrain: [[chunk.terrain, chunk.terrain], [chunk.terrain, chunk.terrain]],
        resources: chunk.resources
      }])
    })
  }

  /**
   * Update visible chunks based on camera position
   */
  private updateVisibleChunks(): void {
    const cameraPos = this.cameraController.getWorldPosition()
    const screenSize = { x: window.innerWidth, y: window.innerHeight }
    const zoom = this.cameraController.getZoom()

    // Get visible tiles
    const visibleTiles = CoordinateSystem.getVisibleTiles(cameraPos, screenSize, zoom, 2)

    // Check if we need to update chunks
    const needsUpdate = !this.tilesEqual(visibleTiles, this.lastVisibleTiles)

    if (needsUpdate) {
      this.loadChunksForTiles(visibleTiles)
      this.unloadChunksForTiles(visibleTiles)
      this.lastVisibleTiles = visibleTiles
    }
  }

  /**
   * Update all visible tile actors with current camera position
   */
  private updateTileActors(): void {
    const cameraPos = this.cameraController.getWorldPosition()
    const screenSize = { x: window.innerWidth, y: window.innerHeight }
    const zoom = this.cameraController.getZoom()

    for (const chunk of this.chunks.values()) {
      if (!chunk.isLoaded) continue

      for (const tileActor of chunk.tileActors.values()) {
        // Update tile rendering based on camera position
        tileActor.updateRender(cameraPos, screenSize, zoom)

        // Update visibility - simple check based on screen position
        const screenPos = tileActor.pos
        const isVisible = (
          screenPos.x >= -50 &&
          screenPos.x <= screenSize.x + 50 &&
          screenPos.y >= -50 &&
          screenPos.y <= screenSize.y + 50
        )
        tileActor.graphics.visible = isVisible
      }
    }
  }

  /**
   * Load chunks needed for visible tiles
   */
  private async loadChunksForTiles(visibleTiles: Point2D[]): Promise<void> {
    // Get unique chunk positions needed
    const chunkPositions = new Set<string>()

    for (const tile of visibleTiles) {
      const chunkX = Math.floor(tile.x / 16)
      const chunkY = Math.floor(tile.y / 16)
      chunkPositions.add(`${chunkX},${chunkY}`)
    }

    // Load each chunk
    for (const chunkKey of chunkPositions) {
      if (!this.chunks.has(chunkKey)) {
        const [x, y] = chunkKey.split(',').map(Number)
        await this.loadChunk(x, y)
      }

      // Update last access time
      const chunk = this.chunks.get(chunkKey)
      if (chunk) {
        chunk.lastAccessTime = Date.now()
        chunk.isLoaded = true
      }
    }
  }

  /**
   * Unload chunks no longer needed
   */
  private unloadChunksForTiles(visibleTiles: Point2D[]): void {
    // Get chunks currently needed
    const neededChunks = new Set<string>()

    for (const tile of visibleTiles) {
      const chunkX = Math.floor(tile.x / 16)
      const chunkY = Math.floor(tile.y / 16)
      neededChunks.add(`${chunkX},${chunkY}`)
    }

    // Unload chunks that are not needed
    for (const [chunkKey, chunk] of this.chunks.entries()) {
      if (!neededChunks.has(chunkKey) && chunk.isLoaded) {
        this.unloadChunk(chunkKey)
      }
    }
  }

  /**
   * Load a chunk from API or cache
   */
  private async loadChunk(chunkX: number, chunkY: number): Promise<void> {
    const chunkKey = `${chunkX},${chunkY}`

    try {
      // Try cache first
      let chunkData = this.dataCache.getChunk(chunkKey)

      if (!chunkData) {
        // Load from API
        const response = await this.apiClient.getChunks({
          coords: [chunkKey],
          layers: true
        })

        chunkData = response.chunk_data[chunkKey]

        if (chunkData) {
          this.dataCache.setChunk(chunkKey, chunkData)
        }
      }

      if (chunkData) {
        this.loadChunkData(chunkX, chunkY, [chunkData])
      }

    } catch (error) {
      this.logger.error(`Failed to load chunk ${chunkKey}:`, error)

      // Create empty chunk as fallback
      this.loadChunkData(chunkX, chunkY, [{
        terrain: [['DeepWater', 'DeepWater'], ['DeepWater', 'DeepWater']],
        resources: [['', ''], ['', '']]
      }])
    }
  }

  /**
   * Load chunk data and create tile actors
   */
  private loadChunkData(chunkX: number, chunkY: number, chunks: any[]): void {
    const chunkKey = `${chunkX},${chunkY}`

    const renderChunk: RenderChunk = {
      chunkKey,
      tileActors: new Map(),
      lastAccessTime: Date.now(),
      isLoaded: true
    }

    // Process each chunk layer
    for (const chunkData of chunks) {
      if (!chunkData.terrain) continue

      // Create tile actors for each tile in the chunk
      for (let y = 0; y < chunkData.terrain.length; y++) {
        for (let x = 0; x < chunkData.terrain[y].length; x++) {
          const terrainTypeStr = chunkData.terrain[y][x] as string
          const resourceTypeStr = chunkData.resources?.[y]?.[x] as string | undefined

          // Parse terrain type
          let terrainType: TerrainType
          try {
            terrainType = this.parseTerrainType(terrainTypeStr)
          } catch {
            this.logger.warn(`Unknown terrain type: ${terrainTypeStr}, using Grass`)
            terrainType = TerrainType.GRASS
          }

          // Parse resource type (optional)
          let resourceType: ResourceType | null = null
          if (resourceTypeStr && resourceTypeStr.trim() !== '') {
            try {
              resourceType = this.parseResourceType(resourceTypeStr)
            } catch {
              this.logger.warn(`Unknown resource type: ${resourceTypeStr}, ignoring`)
            }
          }

          const worldX = chunkX * 16 + x
          const worldY = chunkY * 16 + y
          const tilePos = { x: worldX, y: worldY }

          // Create enhanced tile actor with terrain and resource
          const tileActor = new TileActor(tilePos, terrainType, resourceType)
          renderChunk.tileActors.set(`${x},${y}`, tileActor)
          this.scene.add(tileActor)
        }
      }
    }

    this.chunks.set(chunkKey, renderChunk)
    this.logger.debug(`Loaded chunk ${chunkKey} with ${renderChunk.tileActors.size} tiles`)
  }

  /**
   * Parse terrain type string to enum
   */
  private parseTerrainType(terrainStr: string): TerrainType {
    const terrainMap: Record<string, TerrainType> = {
      'DeepWater': TerrainType.DEEP_WATER,
      'ShallowWater': TerrainType.SHALLOW_WATER,
      'Sand': TerrainType.SAND,
      'Grass': TerrainType.GRASS,
      'Forest': TerrainType.FOREST,
      'Dirt': TerrainType.DIRT,
      'Desert': TerrainType.DESERT,
      'Mountains': TerrainType.MOUNTAINS,
      'Snow': TerrainType.SNOW,
      'Stone': TerrainType.STONE,
      'Swamp': TerrainType.SWAMP,
      'Water': TerrainType.SHALLOW_WATER // Legacy mapping
    }

    return terrainMap[terrainStr] || TerrainType.GRASS
  }

  /**
   * Parse resource type string to enum
   */
  private parseResourceType(resourceStr: string): ResourceType {
    const resourceMap: Record<string, ResourceType> = {
      'TreePine': ResourceType.TREE_PINE,
      'TreeOak': ResourceType.TREE_OAK,
      'TreeBirch': ResourceType.TREE_BIRCH,
      'TreePalm': ResourceType.TREE_PALM,
      'Bush': ResourceType.BUSH,
      'Flower': ResourceType.FLOWER,
      'Rock': ResourceType.ROCK,
      'Stone': ResourceType.STONE,
      'Gold': ResourceType.GOLD,
      'Copper': ResourceType.COPPER,
      'Iron': ResourceType.IRON,
      'Fruit': ResourceType.FRUIT,
      'Vegetables': ResourceType.VEGETABLES,
      'Herb': ResourceType.HERB,
      'Mushroom': ResourceType.MUSHROOM
    }

    const resourceType = resourceMap[resourceStr]
    if (!resourceType) {
      throw new Error(`Unknown resource type: ${resourceStr}`)
    }
    return resourceType
  }

  /**
   * Unload a chunk
   */
  private unloadChunk(chunkKey: string): void {
    const chunk = this.chunks.get(chunkKey)
    if (!chunk) return

    // Remove all tile actors from scene
    for (const tileActor of chunk.tileActors.values()) {
      this.scene.remove(tileActor)
    }

    // Mark as unloaded but keep for potential reuse
    chunk.isLoaded = false
    this.logger.debug(`Unloaded chunk ${chunkKey}`)
  }

  /**
   * Cleanup old chunks
   */
  private cleanupOldChunks(): void {
    const now = Date.now()
    const chunksToRemove: string[] = []

    // Find chunks that are old and not loaded
    for (const [chunkKey, chunk] of this.chunks.entries()) {
      if (!chunk.isLoaded && now - chunk.lastAccessTime > this.chunkTimeout) {
        chunksToRemove.push(chunkKey)
      }
    }

    // Remove old chunks
    for (const chunkKey of chunksToRemove) {
      const chunk = this.chunks.get(chunkKey)
      if (chunk) {
        for (const tileActor of chunk.tileActors.values()) {
          this.scene.remove(tileActor)
        }
      }
      this.chunks.delete(chunkKey)
    }

    if (chunksToRemove.length > 0) {
      this.logger.debug(`Cleaned up ${chunksToRemove.length} old chunks`)
    }

    // Enforce max loaded chunks
    const loadedChunks = Array.from(this.chunks.entries())
      .filter(([_, chunk]) => chunk.isLoaded)
      .sort(([_, a], [__, b]) => a.lastAccessTime - b.lastAccessTime)

    if (loadedChunks.length > this.maxLoadedChunks) {
      const toUnload = loadedChunks.slice(this.maxLoadedChunks)
      for (const [chunkKey] of toUnload) {
        this.unloadChunk(chunkKey)
      }
    }
  }

  /**
   * Check if two tile arrays are equal
   */
  private tilesEqual(tiles1: Point2D[], tiles2: Point2D[]): boolean {
    if (tiles1.length !== tiles2.length) return false

    const set1 = new Set(tiles1.map(t => `${t.x},${t.y}`))
    const set2 = new Set(tiles2.map(t => `${t.x},${t.y}`))

    if (set1.size !== set2.size) return false

    for (const key of set1) {
      if (!set2.has(key)) return false
    }

    return true
  }

  /**
   * Get loaded chunk count
   */
  getLoadedChunkCount(): number {
    return Array.from(this.chunks.values()).filter(c => c.isLoaded).length
  }

  /**
   * Get visible tile count
   */
  getVisibleTileCount(): number {
    let count = 0
    for (const chunk of this.chunks.values()) {
      if (chunk.isLoaded) {
        count += chunk.tileActors.size
      }
    }
    return count
  }

  /**
   * Get API client for external access
   */
  getAPIClient(): APIClient {
    return this.apiClient
  }

  /**
   * Get data cache for external access
   */
  getDataCache(): DataCache {
    return this.dataCache
  }
}