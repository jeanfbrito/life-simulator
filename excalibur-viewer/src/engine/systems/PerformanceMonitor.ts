import { ISystem } from '../SystemRegistry'
import { Logger } from '@/utils/Logger'

/**
 * Performance metrics interface
 */
export interface PerformanceMetrics {
  fps: number
  frameTime: number
  memoryUsage?: number
  entities: number
  chunks: number
  drawCalls: number
}

/**
 * Performance monitoring system
 */
export class PerformanceMonitor implements ISystem {
  readonly name = 'PerformanceMonitor'

  private logger: Logger
  private frameCount = 0
  private lastFpsUpdate = 0
  private fps = 0
  private frameTime = 0
  private lastFrameTime = 0
  private drawCalls = 0

  // External metrics that will be set by other systems
  private entityCount = 0
  private chunkCount = 0

  constructor() {
    this.logger = new Logger('PerformanceMonitor')
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing performance monitor...')
    this.lastFpsUpdate = performance.now()
    this.lastFrameTime = this.lastFpsUpdate
    this.logger.debug('Performance monitor initialized')
  }

  update(deltaTime: number): void {
    const now = performance.now()

    // Calculate frame time
    this.frameTime = now - this.lastFrameTime
    this.lastFrameTime = now

    // Update FPS counter
    this.frameCount++
    if (now - this.lastFpsUpdate >= 1000) {
      this.fps = Math.round(this.frameCount * 1000 / (now - this.lastFpsUpdate))
      this.frameCount = 0
      this.lastFpsUpdate = now

      // Update HUD with current metrics
      this.updateHUD()
    }

    // Reset draw calls counter (will be incremented by rendering systems)
    this.drawCalls = 0
  }

  destroy(): void {
    this.logger.debug('Performance monitor destroyed')
  }

  /**
   * Get current performance metrics
   */
  getMetrics(): PerformanceMetrics {
    return {
      fps: this.fps,
      frameTime: this.frameTime,
      entities: this.entityCount,
      chunks: this.chunkCount,
      drawCalls: this.drawCalls,
      memoryUsage: this.getMemoryUsage()
    }
  }

  /**
   * Update entity count (called by EntityManager)
   */
  setEntityCount(count: number): void {
    this.entityCount = count
  }

  /**
   * Update chunk count (called by ChunkController)
   */
  setChunkCount(count: number): void {
    this.chunkCount = count
  }

  /**
   * Increment draw calls counter (called by rendering systems)
   */
  incrementDrawCalls(): void {
    this.drawCalls++
  }

  /**
   * Get memory usage (if available)
   */
  private getMemoryUsage(): number | undefined {
    if ('memory' in performance) {
      const memory = (performance as any).memory
      return Math.round(memory.usedJSHeapSize / 1024 / 1024) // MB
    }
    return undefined
  }

  /**
   * Update HUD elements with performance data
   */
  private updateHUD(): void {
    const fpsElement = document.getElementById('fps')
    if (fpsElement) {
      fpsElement.textContent = this.fps.toString()
    }

    const entityCountElement = document.getElementById('entity-count')
    if (entityCountElement) {
      entityCountElement.textContent = this.entityCount.toString()
    }

    const chunkCountElement = document.getElementById('chunk-count')
    if (chunkCountElement) {
      chunkCountElement.textContent = this.chunkCount.toString()
    }

    // Log performance warnings
    if (this.fps < 30 && this.fps > 0) {
      this.logger.warn(`Low FPS detected: ${this.fps}`)
    }

    if (this.frameTime > 33) { // > 33ms = < 30 FPS
      this.logger.warn(`High frame time detected: ${this.frameTime.toFixed(2)}ms`)
    }
  }

  /**
   * Get performance grade
   */
  getPerformanceGrade(): 'excellent' | 'good' | 'fair' | 'poor' {
    if (this.fps >= 55) return 'excellent'
    if (this.fps >= 40) return 'good'
    if (this.fps >= 25) return 'fair'
    return 'poor'
  }

  /**
   * Get detailed performance report
   */
  getPerformanceReport(): string {
    const metrics = this.getMetrics()
    const grade = this.getPerformanceGrade()
    const memory = metrics.memoryUsage ? ` | Memory: ${metrics.memoryUsage}MB` : ''

    return [
      `Performance Grade: ${grade.toUpperCase()}`,
      `FPS: ${metrics.fps} | Frame Time: ${metrics.frameTime.toFixed(2)}ms`,
      `Entities: ${metrics.entities} | Chunks: ${metrics.chunks} | Draw Calls: ${metrics.drawCalls}${memory}`
    ].join('\n')
  }
}