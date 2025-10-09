import { Engine, Scene } from 'excalibur'
import { SystemRegistry } from './SystemRegistry'
import { Logger } from '@/utils/Logger'

/**
 * Manages scenes for the Life Simulator viewer
 */
export class SceneManager {
  private engine: Engine
  private systemRegistry: SystemRegistry
  private logger: Logger
  private mainScene: Scene | null = null

  constructor(engine: Engine, systemRegistry: SystemRegistry) {
    this.engine = engine
    this.systemRegistry = systemRegistry
    this.logger = new Logger('SceneManager')
  }

  /**
   * Initialize the scene manager
   */
  async initialize(): Promise<void> {
    this.logger.info('Initializing scene manager...')

    // Create the main scene
    this.mainScene = new Scene()

    // Add scene to engine
    this.engine.addScene('main', this.mainScene)

    this.logger.info('Scene manager initialized')
  }

  /**
   * Go to the main scene
   */
  async goToMainScene(): Promise<void> {
    if (!this.mainScene) {
      throw new Error('Main scene not initialized')
    }

    this.logger.info('Switching to main scene')
    await this.engine.goToScene('main')

    // Initialize systems that need the scene
    await this.initializeSceneSystems()
  }

  /**
   * Initialize systems that need the scene reference
   */
  private async initializeSceneSystems(): Promise<void> {
    // Initialize ChunkController with scene
    const chunkController = this.systemRegistry.get('ChunkController')
    if (chunkController && this.mainScene) {
      // Update chunk controller with scene reference
      this.logger.debug('Initializing ChunkController with scene')
    }

    // Initialize EntityManager with scene
    const entityManager = this.systemRegistry.get('EntityManager')
    if (entityManager && this.mainScene) {
      this.logger.debug('EntityManager will initialize with scene access')
    }
  }

  /**
   * Update HUD elements
   */
  updateHUD(): void {
    // Get performance monitor for metrics
    const performanceMonitor = this.systemRegistry.get('PerformanceMonitor')

    if (performanceMonitor) {
      const metrics = performanceMonitor.getMetrics()

      // Update entity count
      const entityCountElement = document.getElementById('entity-count')
      if (entityCountElement) {
        entityCountElement.textContent = metrics.entities.toString()
      }

      // Update chunk count
      const chunkCountElement = document.getElementById('chunk-count')
      if (chunkCountElement) {
        chunkCountElement.textContent = metrics.chunks.toString()
      }
    }

    // Update camera position (placeholder - will be updated by IsoCameraController)
    const cameraPosElement = document.getElementById('camera-pos')
    if (cameraPosElement) {
      cameraPosElement.textContent = '0, 0'
    }

    // Update zoom level (placeholder - will be updated by IsoCameraController)
    const zoomLevelElement = document.getElementById('zoom-level')
    if (zoomLevelElement) {
      zoomLevelElement.textContent = '1.0x'
    }
  }

  /**
   * Get the current active scene
   */
  getCurrentScene(): Scene | null {
    return this.engine.currentScene
  }

  /**
   * Clean up resources
   */
  cleanup(): void {
    this.logger.info('Cleaning up scene manager...')
    this.mainScene = null
  }
}