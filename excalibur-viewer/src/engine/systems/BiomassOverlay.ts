import { ISystem } from '../SystemRegistry'
import { Actor, Color } from 'excalibur'
import { Logger } from '@/utils/Logger'
import { ChunkController } from './ChunkController'
import { APIClient } from '@/data/APIClient'

/**
 * Biomass data point for heatmap rendering
 */
interface BiomassPoint {
  x: number
  y: number
  biomass: number
  maxBiomass: number
}

/**
 * Biomass heatmap overlay system
 * Renders biomass density as a colored overlay
 */
export class BiomassOverlay implements ISystem {
  readonly name = 'BiomassOverlay'

  private logger: Logger
  private overlayActor: Actor | null = null
  private chunkController: ChunkController | null = null
  private apiClient: APIClient
  private isVisible: boolean = false
  private biomassData: BiomassPoint[] = []
  private lastUpdateTime: number = 0
  private updateInterval: number = 5000 // Update every 5 seconds

  // Heatmap configuration
  private readonly heatmapConfig = {
    cellSize: 16, // Grid cell size in pixels
    colors: [
      { threshold: 0.0, color: Color.fromHex('#000033') },    // Deep blue (no biomass)
      { threshold: 0.2, color: Color.fromHex('#000066') },    // Dark blue
      { threshold: 0.4, color: Color.fromHex('#003399') },    // Medium blue
      { threshold: 0.6, color: Color.fromHex('#0066CC') },    // Light blue
      { threshold: 0.8, color: Color.fromHex('#00CCFF') },    // Cyan
      { threshold: 1.0, color: Color.fromHex('#66FF66') }     // Green (high biomass)
    ],
    opacity: 0.6
  }

  constructor(chunkController: ChunkController) {
    this.logger = new Logger('BiomassOverlay')
    this.chunkController = chunkController
    this.apiClient = new APIClient('http://localhost:54321')
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing biomass overlay...')

    // Create overlay actor
    this.overlayActor = new Actor({
      x: 0,
      y: 0,
      z: 25 // Above terrain but below entities
    })

    // Initially hide the overlay
    this.overlayActor.graphics.visible = false

    // Add to scene
    const engine = (globalThis as any).excaliburEngine
    if (engine && engine.currentScene) {
      engine.currentScene.add(this.overlayActor)
    }

    // Load initial biomass data
    await this.loadBiomassData()

    this.logger.info('Biomass overlay initialized')
  }

  update(deltaTime: number): void {
    const currentTime = Date.now()

    // Update biomass data periodically
    if (currentTime - this.lastUpdateTime > this.updateInterval) {
      this.loadBiomassData()
      this.lastUpdateTime = currentTime
    }

    // Update overlay rendering
    if (this.isVisible && this.overlayGraphics) {
      this.renderHeatmap()
    }
  }

  destroy(): void {
    this.logger.debug('Destroying biomass overlay...')

    if (this.overlayActor) {
      this.overlayActor.kill()
      this.overlayActor = null
    }
    this.biomassData = []
    this.logger.debug('Biomass overlay destroyed')
  }

  /**
   * Toggle overlay visibility
   */
  toggle(): void {
    this.isVisible = !this.isVisible

    if (this.overlayActor) {
      this.overlayActor.graphics.visible = this.isVisible
      this.logger.info(`Biomass overlay ${this.isVisible ? 'shown' : 'hidden'}`)
    }

    if (this.isVisible) {
      this.renderHeatmap()
    }
  }

  /**
   * Show overlay
   */
  show(): void {
    if (!this.isVisible) {
      this.toggle()
    }
  }

  /**
   * Hide overlay
   */
  hide(): void {
    if (this.isVisible) {
      this.toggle()
    }
  }

  /**
   * Check if overlay is visible
   */
  isOverlayVisible(): boolean {
    return this.isVisible
  }

  /**
   * Load biomass data from API or use cached data
   */
  private async loadBiomassData(): Promise<void> {
    try {
      // Try to get vegetation metrics (biomass proxy)
      const response = await this.apiClient.getVegetationMetrics()

      if (response && response.metrics) {
        // Convert vegetation metrics to biomass points
        this.biomassData = this.convertToBiomassPoints(response.metrics)
        this.logger.debug(`Loaded ${this.biomassData.length} biomass points`)
      }
    } catch (error) {
      this.logger.warn('Failed to load biomass data, using mock data:', error)
      this.generateMockBiomassData()
    }
  }

  /**
   * Convert vegetation metrics to biomass points
   */
  private convertToBiomassPoints(metrics: any): BiomassPoint[] {
    const biomassPoints: BiomassPoint[] = []

    // Find max biomass for normalization
    let maxBiomass = 0
    for (const chunkMetrics of Object.values(metrics)) {
      if (chunkMetrics.biomass > maxBiomass) {
        maxBiomass = chunkMetrics.biomass
      }
    }

    // Convert chunk metrics to grid points
    for (const [chunkKey, chunkMetrics] of Object.entries(metrics)) {
      const [chunkX, chunkY] = chunkKey.split(',').map(Number)

      // Convert chunk position to world position
      const worldX = chunkX * 16
      const worldY = chunkY * 16

      // Generate biomass points for this chunk
      const chunkPoints = this.generateChunkBiomassPoints(
        worldX, worldY, chunkMetrics.biomass, maxBiomass
      )

      biomassPoints.push(...chunkPoints)
    }

    return biomassPoints
  }

  /**
   * Generate biomass points for a chunk
   */
  private generateChunkBiomassPoints(
    chunkX: number,
    chunkY: number,
    chunkBiomass: number,
    maxBiomass: number
  ): BiomassPoint[] {
    const points: BiomassPoint[] = []
    const variation = 0.3 // 30% variation

    for (let y = 0; y < 16; y++) {
      for (let x = 0; x < 16; x++) {
        const worldX = chunkX + x
        const worldY = chunkY + y

        // Add some randomness to make it look more natural
        const randomFactor = 1 + (Math.random() - 0.5) * variation
        const biomass = chunkBiomass * randomFactor

        points.push({
          x: worldX,
          y: worldY,
          biomass: Math.max(0, biomass),
          maxBiomass
        })
      }
    }

    return points
  }

  /**
   * Generate mock biomass data for testing
   */
  private generateMockBiomassData(): void {
    const points: BiomassPoint[] = []
    const gridSize = 50 // 50x50 grid
    const maxBiomass = 100

    // Generate a gradient pattern with some noise
    for (let y = 0; y < gridSize; y++) {
      for (let x = 0; x < gridSize; x++) {
        // Create gradient from center
        const centerX = gridSize / 2
        const centerY = gridSize / 2
        const distance = Math.sqrt(
          Math.pow(x - centerX, 2) + Math.pow(y - centerY, 2)
        )
        const maxDistance = Math.sqrt(
          Math.pow(centerX, 2) + Math.pow(centerY, 2)
        )

        // Base biomass decreases from center
        let biomass = maxBiomass * (1 - distance / maxDistance)

        // Add some patches of high biomass (vegetation clusters)
        if (Math.random() < 0.1) {
          biomass = Math.min(maxBiomass, biomass + 50)
        }

        // Add noise
        biomass *= (0.8 + Math.random() * 0.4)

        points.push({
          x: x,
          y: y,
          biomass: Math.max(0, biomass),
          maxBiomass
        })
      }
    }

    this.biomassData = points
    this.logger.debug(`Generated ${points.length} mock biomass points`)
  }

  /**
   * Render the heatmap
   */
  private renderHeatmap(): void {
    // TODO: Implement heatmap rendering without Graphics
    // For now, just log that we're trying to render
    if (this.isVisible && this.biomassData.length > 0) {
      this.logger.debug(`Rendering ${this.biomassData.length} biomass points`)
    }
  }

  /**
   * Get color for biomass value
   */
  private getBiomassColor(biomass: number, maxBiomass: number): Color {
    const normalizedValue = maxBiomass > 0 ? biomass / maxBiomass : 0

    // Find appropriate color based on threshold
    for (let i = this.heatmapConfig.colors.length - 1; i >= 0; i--) {
      if (normalizedValue >= this.heatmapConfig.colors[i].threshold) {
        return this.heatmapConfig.colors[i].color
      }
    }

    return this.heatmapConfig.colors[0].color
  }

  /**
   * Update overlay position based on camera
   */
  updatePosition(cameraX: number, cameraY: number, zoom: number): void {
    if (this.overlayActor) {
      // Position overlay relative to camera
      this.overlayActor.pos = { x: -cameraX * zoom, y: -cameraY * zoom }
      this.overlayActor.scale = { x: zoom, y: zoom }
    }
  }

  /**
   * Get overlay statistics
   */
  getOverlayStats(): {
    isVisible: boolean
    pointCount: number
    maxBiomass: number
    averageBiomass: number
    highBiomassAreas: number
  } {
    if (this.biomassData.length === 0) {
      return {
        isVisible: this.isVisible,
        pointCount: 0,
        maxBiomass: 0,
        averageBiomass: 0,
        highBiomassAreas: 0
      }
    }

    const maxBiomass = Math.max(...this.biomassData.map(p => p.maxBiomass))
    const totalBiomass = this.biomassData.reduce((sum, p) => sum + p.biomass, 0)
    const averageBiomass = totalBiomass / this.biomassData.length
    const highBiomassThreshold = maxBiomass * 0.7
    const highBiomassAreas = this.biomassData.filter(p => p.biomass > highBiomassThreshold).length

    return {
      isVisible: this.isVisible,
      pointCount: this.biomassData.length,
      maxBiomass,
      averageBiomass,
      highBiomassAreas
    }
  }

  /**
   * Get overlay configuration for debugging
   */
  getConfiguration(): typeof this.heatmapConfig {
    return { ...this.heatmapConfig }
  }

  /**
   * Set overlay opacity
   */
  setOpacity(opacity: number): void {
    this.heatmapConfig.opacity = Math.max(0, Math.min(1, opacity))
    this.logger.debug(`Biomass overlay opacity set to ${this.heatmapConfig.opacity}`)
  }

  /**
   * Force biomass data refresh
   */
  async refreshData(): Promise<void> {
    this.logger.debug('Forcing biomass data refresh...')
    await this.loadBiomassData()

    if (this.isVisible) {
      this.renderHeatmap()
    }
  }
}