import { ISystem } from '../SystemRegistry'
import { Logger } from '@/utils/Logger'
import { LifeSimulatorEngine } from '../LifeSimulatorEngine'
import { PerformanceMonitor, PerformanceMetrics } from './PerformanceMonitor'
import { BiomassOverlay } from './BiomassOverlay'
import { IsoCameraController } from './IsoCameraController'

/**
 * UI Overlay system for HUD, controls, and stats display
 */
export class UIOverlay implements ISystem {
  readonly name = 'UIOverlay'

  private logger: Logger
  private engine: LifeSimulatorEngine
  private performanceMonitor: PerformanceMonitor
  private biomassOverlay: BiomassOverlay | null = null
  private cameraController: IsoCameraController
  private isVisible: boolean = true
  private updateInterval: number = 500 // Update every 500ms
  private lastUpdateTime: number = 0

  // UI elements
  private hudContainer: HTMLElement | null = null
  private controlsContainer: HTMLElement | null = null
  private overlayContainer: HTMLElement | null = null

  // UI state
  private showPerformanceStats: boolean = true
  private showCameraInfo: boolean = true
  private showBiomassStats: boolean = false

  constructor(
    engine: LifeSimulatorEngine,
    performanceMonitor: PerformanceMonitor,
    cameraController: IsoCameraController
  ) {
    this.logger = new Logger('UIOverlay')
    this.engine = engine
    this.performanceMonitor = performanceMonitor
    this.cameraController = cameraController

    // Get biomass overlay reference
    this.biomassOverlay = engine.getSystem<BiomassOverlay>('BiomassOverlay') || null
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing UI overlay...')

    // Create UI containers
    this.createUIContainers()

    // Create HUD elements
    this.createHUDElements()

    // Create control buttons
    this.createControlButtons()

    // Create overlay toggles
    this.createOverlayToggles()

    // Initially show all elements
    this.show()

    this.logger.info('UI overlay initialized')
  }

  update(deltaTime: number): void {
    const currentTime = Date.now()

    // Update HUD at regular intervals
    if (currentTime - this.lastUpdateTime > this.updateInterval) {
      this.updateHUD()
      this.lastUpdateTime = currentTime
    }

    // Update overlay positions based on camera
    this.updateOverlayPositions()
  }

  destroy(): void {
    this.logger.debug('Destroying UI overlay...')

    // Remove UI containers
    if (this.hudContainer) {
      this.hudContainer.remove()
      this.hudContainer = null
    }

    if (this.controlsContainer) {
      this.controlsContainer.remove()
      this.controlsContainer = null
    }

    if (this.overlayContainer) {
      this.overlayContainer.remove()
      this.overlayContainer = null
    }

    this.logger.debug('UI overlay destroyed')
  }

  /**
   * Create main UI containers
   */
  private createUIContainers(): void {
    // Get or create HUD container
    this.hudContainer = document.getElementById('hud')
    if (!this.hudContainer) {
      this.hudContainer = document.createElement('div')
      this.hudContainer.id = 'hud'
      this.hudContainer.className = 'hud-container'
      document.body.appendChild(this.hudContainer)
    }

    // Get or create controls container
    this.controlsContainer = document.querySelector('.controls') as HTMLElement
    if (!this.controlsContainer) {
      this.controlsContainer = document.createElement('div')
      this.controlsContainer.className = 'controls-container'
      document.body.appendChild(this.controlsContainer)
    }

    // Create overlay controls container
    this.overlayContainer = document.createElement('div')
    this.overlayContainer.className = 'overlay-container'
    this.overlayContainer.id = 'overlay-controls'
    document.body.appendChild(this.overlayContainer)

    // Apply base styles
    this.applyBaseStyles()
  }

  /**
   * Apply base styles to UI containers
   */
  private applyBaseStyles(): void {
    if (!this.hudContainer) return

    const style = document.createElement('style')
    style.textContent = `
      .hud-container {
        position: fixed;
        top: 10px;
        left: 10px;
        background: rgba(0, 0, 0, 0.85);
        padding: 12px;
        border-radius: 8px;
        font-family: 'SF Mono', 'Monaco', 'Inconsolata', 'Roboto Mono', monospace;
        font-size: 11px;
        color: #ffffff;
        z-index: 100;
        min-width: 200px;
        border: 1px solid rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(10px);
      }

      .hud-section {
        margin: 8px 0;
        padding: 6px 0;
        border-bottom: 1px solid rgba(255, 255, 255, 0.1);
      }

      .hud-section:last-child {
        border-bottom: none;
      }

      .hud-title {
        font-weight: bold;
        color: #4CAF50;
        margin-bottom: 4px;
        text-transform: uppercase;
        font-size: 9px;
        letter-spacing: 1px;
      }

      .hud-item {
        margin: 3px 0;
        display: flex;
        justify-content: space-between;
        align-items: center;
      }

      .hud-label {
        color: rgba(255, 255, 255, 0.7);
      }

      .hud-value {
        font-weight: bold;
        color: #ffffff;
      }

      .hud-value.warning {
        color: #ff9800;
      }

      .hud-value.danger {
        color: #f44336;
      }

      .hud-value.success {
        color: #4CAF50;
      }

      .controls-container {
        position: fixed;
        bottom: 10px;
        left: 10px;
        background: rgba(0, 0, 0, 0.85);
        padding: 12px;
        border-radius: 8px;
        font-size: 12px;
        z-index: 100;
        border: 1px solid rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(10px);
      }

      .overlay-container {
        position: fixed;
        top: 10px;
        right: 10px;
        background: rgba(0, 0, 0, 0.85);
        padding: 12px;
        border-radius: 8px;
        font-size: 12px;
        z-index: 100;
        border: 1px solid rgba(255, 255, 255, 0.1);
        backdrop-filter: blur(10px);
      }

      .overlay-section {
        margin: 8px 0;
      }

      .overlay-title {
        font-weight: bold;
        color: #4CAF50;
        margin-bottom: 6px;
        text-transform: uppercase;
        font-size: 9px;
        letter-spacing: 1px;
      }

      .toggle-button {
        background: rgba(255, 255, 255, 0.1);
        border: 1px solid rgba(255, 255, 255, 0.2);
        color: white;
        padding: 6px 10px;
        margin: 3px;
        border-radius: 4px;
        cursor: pointer;
        font-size: 11px;
        transition: all 0.2s ease;
        font-family: inherit;
      }

      .toggle-button:hover {
        background: rgba(255, 255, 255, 0.2);
        border-color: rgba(255, 255, 255, 0.3);
      }

      .toggle-button.active {
        background: #4CAF50;
        border-color: #4CAF50;
        color: white;
      }

      .toggle-button:disabled {
        opacity: 0.5;
        cursor: not-allowed;
      }

      .control-hint {
        margin: 3px 0;
        opacity: 0.8;
        font-size: 11px;
      }

      .keyboard-shortcut {
        background: rgba(255, 255, 255, 0.1);
        padding: 2px 6px;
        border-radius: 3px;
        font-family: monospace;
        font-size: 10px;
        opacity: 0.8;
      }

      .stats-grid {
        display: grid;
        grid-template-columns: auto 1fr;
        gap: 4px 12px;
        align-items: center;
      }

      .performance-bar {
        width: 100%;
        height: 4px;
        background: rgba(255, 255, 255, 0.1);
        border-radius: 2px;
        margin-top: 2px;
        overflow: hidden;
      }

      .performance-fill {
        height: 100%;
        background: #4CAF50;
        transition: width 0.3s ease, background-color 0.3s ease;
      }

      .performance-fill.warning {
        background: #ff9800;
      }

      .performance-fill.danger {
        background: #f44336;
      }
    `
    document.head.appendChild(style)
  }

  /**
   * Create HUD elements
   */
  private createHUDElements(): void {
    if (!this.hudContainer) return

    this.hudContainer.innerHTML = `
      <div class="hud-section">
        <div class="hud-title">Performance</div>
        <div class="stats-grid">
          <span class="hud-label">FPS:</span>
          <span class="hud-value" id="fps">0</span>
          <span class="hud-label">Frame Time:</span>
          <span class="hud-value" id="frame-time">0ms</span>
          <span class="hud-label">Entities:</span>
          <span class="hud-value" id="entity-count">0</span>
          <span class="hud-label">Chunks:</span>
          <span class="hud-value" id="chunk-count">0</span>
          <span class="hud-label">Draw Calls:</span>
          <span class="hud-value" id="draw-calls">0</span>
          <span class="hud-label">Memory:</span>
          <span class="hud-value" id="memory-usage">--</span>
        </div>
        <div class="performance-bar">
          <div class="performance-fill" id="fps-bar"></div>
        </div>
      </div>

      <div class="hud-section" id="camera-section">
        <div class="hud-title">Camera</div>
        <div class="stats-grid">
          <span class="hud-label">Position:</span>
          <span class="hud-value" id="camera-pos">0, 0</span>
          <span class="hud-label">Zoom:</span>
          <span class="hud-value" id="zoom-level">1.0x</span>
          <span class="hud-label">Visible:</span>
          <span class="hud-value" id="visible-chunks">0</span>
        </div>
      </div>

      <div class="hud-section" id="biomass-section" style="display: none;">
        <div class="hud-title">Biomass</div>
        <div class="stats-grid">
          <span class="hud-label">Points:</span>
          <span class="hud-value" id="biomass-points">0</span>
          <span class="hud-label">Max:</span>
          <span class="hud-value" id="biomass-max">0</span>
          <span class="hud-label">Average:</span>
          <span class="hud-value" id="biomass-avg">0</span>
          <span class="hud-label">High Areas:</span>
          <span class="hud-value" id="biomass-high-areas">0</span>
        </div>
      </div>

      <div class="hud-section">
        <div class="hud-title">System</div>
        <div class="stats-grid">
          <span class="hud-label">Status:</span>
          <span class="hud-value success" id="system-status">Running</span>
          <span class="hud-label">API:</span>
          <span class="hud-value" id="api-status">Connected</span>
        </div>
      </div>
    `
  }

  /**
   * Create control buttons
   */
  private createControlButtons(): void {
    if (!this.controlsContainer) return

    this.controlsContainer.innerHTML = `
      <div class="overlay-section">
        <div class="overlay-title">Controls</div>
        <div class="control-hint">🖱️ Drag to pan</div>
        <div class="control-hint">⚡ Scroll to zoom</div>
        <div class="control-hint">🔄 Right-click to reset</div>
        <div class="control-hint">
          <span class="keyboard-shortcut">B</span> Toggle Biomass
        </div>
        <div class="control-hint">
          <span class="keyboard-shortcut">P</span> Toggle Performance
        </div>
        <div class="control-hint">
          <span class="keyboard-shortcut">H</span> Toggle HUD
        </div>
        <div class="control-hint">
          <span class="keyboard-shortcut">R</span> Reset Camera
        </div>
      </div>
    `
  }

  /**
   * Create overlay toggle buttons
   */
  private createOverlayToggles(): void {
    if (!this.overlayContainer) return

    this.overlayContainer.innerHTML = `
      <div class="overlay-section">
        <div class="overlay-title">Overlays</div>
        <button class="toggle-button" id="biomass-toggle" title="Toggle biomass overlay [B]">
          🌱 Biomass
        </button>
        <button class="toggle-button" id="performance-toggle" title="Toggle performance details [P]">
          📊 Performance
        </button>
        <button class="toggle-button" id="camera-toggle" title="Toggle camera info [C]">
          📷 Camera
        </button>
        <button class="toggle-button" id="hud-toggle" title="Toggle entire HUD [H]">
          👁️ HUD
        </button>
      </div>

      <div class="overlay-section">
        <div class="overlay-title">Actions</div>
        <button class="toggle-button" id="reset-camera" title="Reset camera view [R]">
          🔄 Reset View
        </button>
        <button class="toggle-button" id="refresh-biomass" title="Refresh biomass data">
          🔄 Refresh Data
        </button>
        <button class="toggle-button" id="screenshot" title="Take screenshot">
          📸 Screenshot
        </button>
      </div>
    `

    // Add event listeners
    this.attachButtonListeners()
  }

  /**
   * Attach event listeners to buttons
   */
  private attachButtonListeners(): void {
    // Biomass toggle
    const biomassToggle = document.getElementById('biomass-toggle')
    if (biomassToggle) {
      biomassToggle.addEventListener('click', () => this.toggleBiomassOverlay())
    }

    // Performance toggle
    const performanceToggle = document.getElementById('performance-toggle')
    if (performanceToggle) {
      performanceToggle.addEventListener('click', () => this.togglePerformanceDetails())
    }

    // Camera toggle
    const cameraToggle = document.getElementById('camera-toggle')
    if (cameraToggle) {
      cameraToggle.addEventListener('click', () => this.toggleCameraInfo())
    }

    // HUD toggle
    const hudToggle = document.getElementById('hud-toggle')
    if (hudToggle) {
      hudToggle.addEventListener('click', () => this.toggleHUD())
    }

    // Reset camera
    const resetCamera = document.getElementById('reset-camera')
    if (resetCamera) {
      resetCamera.addEventListener('click', () => this.resetCamera())
    }

    // Refresh biomass
    const refreshBiomass = document.getElementById('refresh-biomass')
    if (refreshBiomass) {
      refreshBiomass.addEventListener('click', () => this.refreshBiomassData())
    }

    // Screenshot
    const screenshot = document.getElementById('screenshot')
    if (screenshot) {
      screenshot.addEventListener('click', () => this.takeScreenshot())
    }
  }

  /**
   * Update HUD with current metrics
   */
  private updateHUD(): void {
    const metrics = this.performanceMonitor.getMetrics()

    // Update performance metrics
    this.updateValue('fps', metrics.fps.toString(), this.getFPSColor(metrics.fps))
    this.updateValue('frame-time', `${metrics.frameTime.toFixed(1)}ms`, this.getFrameTimeColor(metrics.frameTime))
    this.updateValue('entity-count', metrics.entities.toString())
    this.updateValue('chunk-count', metrics.chunks.toString())
    this.updateValue('draw-calls', metrics.drawCalls.toString())
    this.updateValue('memory-usage', metrics.memoryUsage ? `${metrics.memoryUsage}MB` : '--')

    // Update FPS bar
    this.updateFPSBar(metrics.fps)

    // Update camera info
    const cameraPos = this.cameraController.getPosition()
    const zoom = this.cameraController.getZoom()
    this.updateValue('camera-pos', `${Math.round(cameraPos.x)}, ${Math.round(cameraPos.y)}`)
    this.updateValue('zoom-level', `${zoom.toFixed(2)}x`)

    // Update biomass stats if visible
    if (this.showBiomassStats && this.biomassOverlay) {
      const biomassStats = this.biomassOverlay.getOverlayStats()
      this.updateValue('biomass-points', biomassStats.pointCount.toString())
      this.updateValue('biomass-max', Math.round(biomassStats.maxBiomass).toString())
      this.updateValue('biomass-avg', Math.round(biomassStats.averageBiomass).toString())
      this.updateValue('biomass-high-areas', biomassStats.highBiomassAreas.toString())
    }

    // Update system status
    this.updateValue('system-status', 'Running', 'success')
    this.updateValue('api-status', 'Connected', 'success')
  }

  /**
   * Update a specific HUD value
   */
  private updateValue(id: string, value: string, className?: string): void {
    const element = document.getElementById(id)
    if (element) {
      element.textContent = value
      if (className) {
        element.className = `hud-value ${className}`
      }
    }
  }

  /**
   * Update FPS performance bar
   */
  private updateFPSBar(fps: number): void {
    const fpsBar = document.getElementById('fps-bar')
    if (fpsBar) {
      const percentage = Math.min((fps / 60) * 100, 100)
      fpsBar.style.width = `${percentage}%`

      // Update color based on performance
      fpsBar.className = 'performance-fill'
      if (fps < 30) {
        fpsBar.classList.add('danger')
      } else if (fps < 45) {
        fpsBar.classList.add('warning')
      }
    }
  }

  /**
   * Get color class for FPS value
   */
  private getFPSColor(fps: number): string {
    if (fps < 30) return 'danger'
    if (fps < 45) return 'warning'
    return 'success'
  }

  /**
   * Get color class for frame time
   */
  private getFrameTimeColor(frameTime: number): string {
    if (frameTime > 33) return 'danger'  // < 30 FPS
    if (frameTime > 22) return 'warning' // < 45 FPS
    return 'success'
  }

  /**
   * Update overlay positions based on camera
   */
  private updateOverlayPositions(): void {
    // Position overlay elements relative to camera if needed
    // This can be used for floating UI elements
  }

  /**
   * Toggle biomass overlay
   */
  private toggleBiomassOverlay(): void {
    if (this.biomassOverlay) {
      this.biomassOverlay.toggle()
      this.updateButtonState('biomass-toggle', this.biomassOverlay.isOverlayVisible())
      this.showBiomassStats = this.biomassOverlay.isOverlayVisible()

      // Show/hide biomass section
      const biomassSection = document.getElementById('biomass-section')
      if (biomassSection) {
        biomassSection.style.display = this.showBiomassStats ? 'block' : 'none'
      }

      this.logger.info(`Biomass overlay ${this.biomassOverlay.isOverlayVisible() ? 'enabled' : 'disabled'}`)
    }
  }

  /**
   * Toggle performance details
   */
  private togglePerformanceDetails(): void {
    this.showPerformanceStats = !this.showPerformanceStats
    this.updateButtonState('performance-toggle', this.showPerformanceStats)

    // Show/hide performance section
    const performanceSection = this.hudContainer?.querySelector('.hud-section:first-child')
    if (performanceSection) {
      performanceSection.style.display = this.showPerformanceStats ? 'block' : 'none'
    }

    this.logger.info(`Performance details ${this.showPerformanceStats ? 'enabled' : 'disabled'}`)
  }

  /**
   * Toggle camera info
   */
  private toggleCameraInfo(): void {
    this.showCameraInfo = !this.showCameraInfo
    this.updateButtonState('camera-toggle', this.showCameraInfo)

    // Show/hide camera section
    const cameraSection = document.getElementById('camera-section')
    if (cameraSection) {
      cameraSection.style.display = this.showCameraInfo ? 'block' : 'none'
    }

    this.logger.info(`Camera info ${this.showCameraInfo ? 'enabled' : 'disabled'}`)
  }

  /**
   * Toggle entire HUD
   */
  private toggleHUD(): void {
    this.isVisible = !this.isVisible
    this.updateButtonState('hud-toggle', this.isVisible)

    if (this.hudContainer) {
      this.hudContainer.style.display = this.isVisible ? 'block' : 'none'
    }

    this.logger.info(`HUD ${this.isVisible ? 'shown' : 'hidden'}`)
  }

  /**
   * Reset camera to default position
   */
  private resetCamera(): void {
    this.cameraController.reset()
    this.logger.info('Camera reset to default position')
  }

  /**
   * Refresh biomass data
   */
  private async refreshBiomassData(): Promise<void> {
    if (this.biomassOverlay) {
      const button = document.getElementById('refresh-biomass')
      if (button) {
        button.disabled = true
        button.textContent = '🔄 Refreshing...'
      }

      try {
        await this.biomassOverlay.refreshData()
        this.logger.info('Biomass data refreshed successfully')
      } catch (error) {
        this.logger.error('Failed to refresh biomass data:', error)
      } finally {
        if (button) {
          button.disabled = false
          button.textContent = '🔄 Refresh Data'
        }
      }
    }
  }

  /**
   * Take a screenshot
   */
  private takeScreenshot(): void {
    const canvas = this.engine.getExcaliburEngine().canvas
    if (canvas) {
      canvas.toBlob((blob) => {
        if (blob) {
          const url = URL.createObjectURL(blob)
          const a = document.createElement('a')
          a.href = url
          a.download = `life-simulator-${new Date().toISOString().slice(0, 19)}.png`
          a.click()
          URL.revokeObjectURL(url)
          this.logger.info('Screenshot saved')
        }
      })
    }
  }

  /**
   * Update button state
   */
  private updateButtonState(id: string, active: boolean): void {
    const button = document.getElementById(id)
    if (button) {
      if (active) {
        button.classList.add('active')
      } else {
        button.classList.remove('active')
      }
    }
  }

  /**
   * Show the UI overlay
   */
  show(): void {
    if (this.hudContainer) this.hudContainer.style.display = 'block'
    if (this.controlsContainer) this.controlsContainer.style.display = 'block'
    if (this.overlayContainer) this.overlayContainer.style.display = 'block'
    this.isVisible = true
  }

  /**
   * Hide the UI overlay
   */
  hide(): void {
    if (this.hudContainer) this.hudContainer.style.display = 'none'
    if (this.controlsContainer) this.controlsContainer.style.display = 'none'
    if (this.overlayContainer) this.overlayContainer.style.display = 'none'
    this.isVisible = false
  }

  /**
   * Check if UI overlay is visible
   */
  isUIVisible(): boolean {
    return this.isVisible
  }

  /**
   * Get current UI state
   */
  getUIState(): {
    isVisible: boolean
    showPerformanceStats: boolean
    showCameraInfo: boolean
    showBiomassStats: boolean
    biomassOverlayActive: boolean
  } {
    return {
      isVisible: this.isVisible,
      showPerformanceStats: this.showPerformanceStats,
      showCameraInfo: this.showCameraInfo,
      showBiomassStats: this.showBiomassStats,
      biomassOverlayActive: this.biomassOverlay?.isOverlayVisible() ?? false
    }
  }

  /**
   * Handle keyboard shortcuts
   */
  handleKeyPress(key: string): void {
    switch (key.toLowerCase()) {
      case 'b':
        this.toggleBiomassOverlay()
        break
      case 'p':
        this.togglePerformanceDetails()
        break
      case 'c':
        this.toggleCameraInfo()
        break
      case 'h':
        this.toggleHUD()
        break
      case 'r':
        this.resetCamera()
        break
    }
  }
}