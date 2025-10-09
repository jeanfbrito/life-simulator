import { Engine, Color } from 'excalibur'
import { SceneManager } from './SceneManager'
import { SystemRegistry, ISystem } from './SystemRegistry'
import { PerformanceMonitor } from './systems/PerformanceMonitor'
import { InputSystem } from './systems/InputSystem'
import { IsoCameraController } from './systems/IsoCameraController'
import { InputBindings } from './systems/InputBindings'
import { ChunkController } from './systems/ChunkController'
import { EntityManager } from './systems/EntityManager'
import { MovementSystem } from './systems/MovementSystem'
import { BiomassOverlay } from './systems/BiomassOverlay'
import { UIOverlay } from './systems/UIOverlay'
import { AdaptivePollingManager } from './systems/AdaptivePollingManager'
import { PollingController } from '@/data/PollingController'
import { APIClient } from '@/data/APIClient'
import { Logger } from '@/utils/Logger'

/**
 * Engine configuration interface
 */
export interface EngineConfig {
  enableDebug?: boolean
  logLevel?: 'debug' | 'info' | 'warn' | 'error'
  targetFPS?: number
  enableHiDPI?: boolean
}

/**
 * Main engine class for the Life Simulator viewer
 */
export class LifeSimulatorEngine {
  private engine: Engine
  private sceneManager: SceneManager
  private systemRegistry: SystemRegistry
  private performanceMonitor: PerformanceMonitor
  private inputSystem: InputSystem
  private cameraController: IsoCameraController
  private inputBindings!: InputBindings
  private chunkController: ChunkController | null = null
  private entityManager: EntityManager | null = null
  private movementSystem: MovementSystem | null = null
  private biomassOverlay: BiomassOverlay | null = null
  private uiOverlay: UIOverlay | null = null
  private adaptivePollingManager: AdaptivePollingManager | null = null
  private apiClient: APIClient
  private pollingController: PollingController
  private logger: Logger
  private config: Required<EngineConfig>
  private isInitialized = false
  private isRunning = false
  private frameCount = 0

  constructor(config: EngineConfig = {}) {
    this.config = {
      enableDebug: config.enableDebug ?? false,
      logLevel: config.logLevel ?? 'info',
      targetFPS: config.targetFPS ?? 60,
      enableHiDPI: config.enableHiDPI ?? true,
    }

    // Set global log level
    Logger.setGlobalLevel(this.config.logLevel)

    this.logger = new Logger('LifeSimulatorEngine')

    // Initialize system registry first
    this.systemRegistry = new SystemRegistry()

    // Calculate display scale for HiDPI support
    const displayScale = this.calculateDisplayScale()

    // Get container element
    const containerElement = document.getElementById('game-container')
    if (!containerElement) {
      throw new Error('Game container element not found')
    }

    // Initialize Excalibur engine
    this.engine = new Engine({
      width: window.innerWidth * displayScale,
      height: window.innerHeight * displayScale,
      canvasElement: containerElement,
      backgroundColor: Color.fromHex('#1a1a1a'),
      antialiasing: true,
      snapToPixel: false, // Disable for subpixel rendering
      displayMode: 'fit' as any, // Scale to fit container
      pixelRatio: displayScale,
    })

    // Apply display scale to canvas
    this.applyDisplayScale(displayScale)

    // Initialize API client and polling controller
    this.apiClient = new APIClient('http://localhost:54321')
    this.pollingController = new PollingController(this.apiClient)

    this.sceneManager = new SceneManager(this.engine, this.systemRegistry)
    this.performanceMonitor = new PerformanceMonitor()
    this.inputSystem = new InputSystem(this.engine.canvas)
    this.cameraController = new IsoCameraController(this.inputSystem)
    this.entityManager = new EntityManager()
    this.movementSystem = new MovementSystem()
    this.chunkController = new ChunkController(this.cameraController, this.performanceMonitor, this.sceneManager.getMainScene())
    this.biomassOverlay = new BiomassOverlay(this.chunkController)
    this.uiOverlay = new UIOverlay(this, this.performanceMonitor, this.cameraController)
    this.adaptivePollingManager = new AdaptivePollingManager(this.performanceMonitor, this.pollingController)

    // Register core systems
    this.registerCoreSystems()
  }

  /**
   * Register core systems
   */
  private registerCoreSystems(): void {
    this.systemRegistry.register(this.performanceMonitor)
    this.systemRegistry.register(this.inputSystem)
    this.systemRegistry.register(this.cameraController)

    // Input bindings depends on both input system and camera controller
    this.inputBindings = new InputBindings(this.inputSystem, this.cameraController)
    this.systemRegistry.register(this.inputBindings)

    // Register entity manager
    if (this.entityManager) {
      this.systemRegistry.register(this.entityManager)
    }

    // Register movement system
    if (this.movementSystem) {
      this.systemRegistry.register(this.movementSystem)
    }

    // Register biomass overlay
    if (this.biomassOverlay) {
      this.systemRegistry.register(this.biomassOverlay)
    }

    // Register UI overlay
    if (this.uiOverlay) {
      this.systemRegistry.register(this.uiOverlay)
    }

    // Register adaptive polling manager
    if (this.adaptivePollingManager) {
      this.systemRegistry.register(this.adaptivePollingManager)
    }

    // Connect systems
    this.connectSystems()

    this.logger.debug(`Registered ${this.systemRegistry.count} core systems`)
  }

  /**
   * Connect systems that depend on each other
   */
  private connectSystems(): void {
    if (this.entityManager && this.movementSystem) {
      this.entityManager.setMovementSystem(this.movementSystem)
    }
  }

  /**
   * Calculate display scale for HiDPI support
   */
  private calculateDisplayScale(): number {
    if (!this.config.enableHiDPI) {
      return 1
    }

    const devicePixelRatio = window.devicePixelRatio || 1
    const ctx = document.createElement('canvas').getContext('2d')

    if (ctx) {
      const backingStoreRatio = (
        ctx.webkitBackingStorePixelRatio ||
        ctx.mozBackingStorePixelRatio ||
        ctx.msBackingStorePixelRatio ||
        ctx.oBackingStorePixelRatio ||
        ctx.backingStorePixelRatio ||
        1
      )

      return Math.min(devicePixelRatio / backingStoreRatio, 2) // Cap at 2x for performance
    }

    return Math.min(devicePixelRatio, 2)
  }

  /**
   * Apply display scale to canvas
   */
  private applyDisplayScale(scale: number): void {
    const canvas = this.engine.canvas
    if (!canvas) return

    canvas.style.width = '100%'
    canvas.style.height = '100%'
    canvas.style.imageRendering = scale > 1 ? 'auto' : 'pixelated'
  }

  /**
   * Initialize the engine and all systems
   */
  async initialize(): Promise<void> {
    if (this.isInitialized) {
      this.logger.warn('Engine already initialized')
      return
    }

    try {
      this.logger.info('Initializing engine...')

      // Initialize all systems
      await this.systemRegistry.initializeAll()

      // Initialize scene manager
      await this.sceneManager.initialize()

      // Set up engine event handlers
      this.engine.on('initialize', () => {
        this.logger.debug('Excalibur engine initialized')
        this.lastFrameTime = performance.now()
      })

      this.engine.on('postupdate', (event) => {
        this.handlePostUpdate(event.delta)
      })

      // Set up debug mode if enabled
      if (this.config.enableDebug) {
        this.setupDebugMode()
      }

      this.isInitialized = true
      this.logger.info('Engine initialization complete')

    } catch (error) {
      this.logger.error('Failed to initialize engine:', error)
      throw error
    }
  }

  /**
   * Start the engine
   */
  async start(): Promise<void> {
    if (!this.isInitialized) {
      throw new Error('Engine must be initialized before starting')
    }

    if (this.isRunning) {
      this.logger.warn('Engine already running')
      return
    }

    try {
      this.logger.info('Starting engine...')

      // Start the Excalibur engine
      await this.engine.start()

      // Start polling controller
      this.pollingController.start()

      // Go to the main scene
      await this.sceneManager.goToMainScene()

      this.isRunning = true
      this.logger.info('Engine started successfully')

    } catch (error) {
      this.logger.error('Failed to start engine:', error)
      throw error
    }
  }

  /**
   * Pause the engine
   */
  pause(): void {
    if (!this.isRunning) return

    this.logger.debug('Pausing engine')
    this.engine.stop()
    this.pollingController.stop()
    this.isRunning = false
  }

  /**
   * Resume the engine
   */
  async resume(): Promise<void> {
    if (this.isRunning) return

    this.logger.debug('Resuming engine')
    await this.engine.start()
    this.pollingController.start()
    this.isRunning = true
  }

  /**
   * Handle post-update events
   */
  private handlePostUpdate(deltaTime: number): void {
    // Update all systems
    this.systemRegistry.updateAll(deltaTime)

    // Update frame counter
    this.frameCount++
  }

  /**
   * Handle window resize
   */
  handleResize(): void {
    this.logger.debug(`Resizing engine to ${window.innerWidth}x${window.innerHeight}`)

    // Recalculate display scale
    const displayScale = this.calculateDisplayScale()

    // Update engine dimensions
    this.engine.screen.viewport.width = window.innerWidth * displayScale
    this.engine.screen.viewport.height = window.innerHeight * displayScale

    // Apply display scale changes
    this.applyDisplayScale(displayScale)

    // Notify systems of resize
    this.notifySystemsResize()
  }

  /**
   * Notify systems of window resize
   */
  private notifySystemsResize(): void {
    // This could be expanded to notify all systems that implement a resize method
    this.logger.debug('Notified systems of resize')
  }

  /**
   * Setup debug mode
   */
  private setupDebugMode(): void {
    this.logger.info('Debug mode enabled')

    // Add debug information to console
    setInterval(() => {
      const metrics = this.performanceMonitor.getMetrics()
      const report = this.performanceMonitor.getPerformanceReport()
      this.logger.debug(`Performance Report:\n${report}`)
    }, 5000)

    // Add keyboard shortcuts for debugging
    this.inputSystem.on('keydown', (event: KeyboardEvent) => {
      // Handle system shortcuts (not Ctrl/Meta)
      if (!event.ctrlKey && !event.metaKey && this.uiOverlay) {
        this.uiOverlay.handleKeyPress(event.key)
      }

      // Handle debug shortcuts (Ctrl/Meta)
      if (event.ctrlKey || event.metaKey) {
        switch (event.key) {
          case 'd':
            event.preventDefault()
            this.logger.info('Debug: Toggle debug panel')
            break
          case 'p':
            event.preventDefault()
            this.logger.info('Debug: Performance report')
            console.log(this.performanceMonitor.getPerformanceReport())
            break
          case 'r':
            event.preventDefault()
            this.logger.info('Debug: Reloading scene')
            this.sceneManager.goToMainScene()
            break
        }
      }
    })
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    return this.performanceMonitor.getMetrics()
  }

  /**
   * Get input state
   */
  getInputState() {
    return this.inputSystem.getState()
  }

  /**
   * Register a custom system
   */
  registerSystem(system: ISystem): void {
    this.systemRegistry.register(system)
    this.logger.info(`Registered custom system: ${system.name}`)
  }

  /**
   * Get a system by name
   */
  getSystem<T extends ISystem>(name: string): T | undefined {
    return this.systemRegistry.get<T>(name)
  }

  /**
   * Get engine configuration
   */
  getConfig(): Readonly<Required<EngineConfig>> {
    return this.config
  }

  /**
   * Check if engine is in debug mode
   */
  isDebugEnabled(): boolean {
    return this.config.enableDebug
  }

  /**
   * Get current display scale
   */
  getDisplayScale(): number {
    return this.engine.screen.pixelRatio || 1
  }

  /**
   * Shutdown the engine
   */
  shutdown(): void {
    this.logger.info('Shutting down engine...')

    if (this.isRunning) {
      this.engine.stop()
      this.pollingController.stop()
      this.isRunning = false
    }

    // Destroy all systems
    this.systemRegistry.destroyAll()

    // Clean up polling controller
    this.pollingController.cleanup()

    // Clean up scene manager
    this.sceneManager.cleanup()

    this.isInitialized = false

    this.logger.info('Engine shutdown complete')
  }

  /**
   * Get the underlying Excalibur engine (for advanced usage)
   */
  getExcaliburEngine(): Engine {
    return this.engine
  }

  /**
   * Get the system registry
   */
  getSystemRegistry(): SystemRegistry {
    return this.systemRegistry
  }

  /**
   * Get the scene manager
   */
  getSceneManager(): SceneManager {
    return this.sceneManager
  }

  /**
   * Get the performance monitor
   */
  getPerformanceMonitor(): PerformanceMonitor {
    return this.performanceMonitor
  }

  /**
   * Get the input system
   */
  getInputSystem(): InputSystem {
    return this.inputSystem
  }

  /**
   * Get the camera controller
   */
  getCameraController(): IsoCameraController {
    return this.cameraController
  }

  /**
   * Get the input bindings
   */
  getInputBindings(): InputBindings {
    return this.inputBindings
  }

  /**
   * Toggle biomass overlay visibility
   */
  toggleBiomassOverlay(): void {
    if (this.biomassOverlay) {
      this.biomassOverlay.toggle()
      this.logger.info('Biomass overlay toggled')
    }
  }

  /**
   * Show biomass overlay
   */
  showBiomassOverlay(): void {
    if (this.biomassOverlay) {
      this.biomassOverlay.show()
      this.logger.info('Biomass overlay shown')
    }
  }

  /**
   * Hide biomass overlay
   */
  hideBiomassOverlay(): void {
    if (this.biomassOverlay) {
      this.biomassOverlay.hide()
      this.logger.info('Biomass overlay hidden')
    }
  }

  /**
   * Check if biomass overlay is visible
   */
  isBiomassOverlayVisible(): boolean {
    return this.biomassOverlay ? this.biomassOverlay.isOverlayVisible() : false
  }

  /**
   * Get biomass overlay statistics
   */
  getBiomassOverlayStats() {
    return this.biomassOverlay ? this.biomassOverlay.getOverlayStats() : {
      isVisible: false,
      pointCount: 0,
      maxBiomass: 0,
      averageBiomass: 0,
      highBiomassAreas: 0
    }
  }

  /**
   * Set biomass overlay opacity
   */
  setBiomassOverlayOpacity(opacity: number): void {
    if (this.biomassOverlay) {
      this.biomassOverlay.setOpacity(opacity)
    }
  }

  /**
   * Force biomass overlay data refresh
   */
  async refreshBiomassOverlay(): Promise<void> {
    if (this.biomassOverlay) {
      await this.biomassOverlay.refreshData()
    }
  }

  /**
   * Get the UI overlay
   */
  getUIOverlay(): UIOverlay | null {
    return this.uiOverlay
  }

  /**
   * Show/hide the entire UI
   */
  setUIVisible(visible: boolean): void {
    if (this.uiOverlay) {
      if (visible) {
        this.uiOverlay.show()
      } else {
        this.uiOverlay.hide()
      }
    }
  }

  /**
   * Check if UI is visible
   */
  isUIVisible(): boolean {
    return this.uiOverlay ? this.uiOverlay.isUIVisible() : false
  }

  /**
   * Get current UI state
   */
  getUIState() {
    return this.uiOverlay ? this.uiOverlay.getUIState() : {
      isVisible: false,
      showPerformanceStats: false,
      showCameraInfo: false,
      showBiomassStats: false,
      biomassOverlayActive: false
    }
  }

  /**
   * Get the adaptive polling manager
   */
  getAdaptivePollingManager(): AdaptivePollingManager | null {
    return this.adaptivePollingManager
  }

  /**
   * Get the API client
   */
  getAPIClient(): APIClient {
    return this.apiClient
  }

  /**
   * Get the polling controller
   */
  getPollingController(): PollingController {
    return this.pollingController
  }

  /**
   * Get adaptive polling status
   */
  getAdaptivePollingStatus() {
    return this.adaptivePollingManager ? this.adaptivePollingManager.getAdaptiveStatus() : {
      enabled: false,
      currentScalingFactor: 1.0,
      performanceGrade: 'unknown',
      lastEvaluation: new Date(),
      adaptationHistory: []
    }
  }

  /**
   * Enable/disable adaptive polling
   */
  setAdaptivePollingEnabled(enabled: boolean): void {
    if (this.adaptivePollingManager) {
      this.adaptivePollingManager.setEnabled(enabled)
    }
  }
}