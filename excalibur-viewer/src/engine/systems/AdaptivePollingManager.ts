import { ISystem } from '../SystemRegistry'
import { Logger } from '@/utils/Logger'
import { PerformanceMonitor, PerformanceMetrics } from './PerformanceMonitor'
import { PollingController } from '@/data/PollingController'
import { PollingOptions } from '@/data/types'

/**
 * Performance thresholds for adaptive polling
 */
interface PerformanceThresholds {
  fpsExcellent: number      // FPS at which we increase polling rate
  fpsGood: number          // Normal FPS target
  fpsFair: number          // FPS at which we start reducing polling
  fpsPoor: number          // FPS at which we significantly reduce polling
  frameTimeTarget: number  // Target frame time in ms (16.67ms for 60fps)
  memoryLimit: number      // Memory limit in MB for scaling
}

/**
 * Adaptive polling configuration
 */
interface AdaptiveConfig {
  enabled: boolean
  thresholds: PerformanceThresholds
  scalingFactors: {
    excellent: number    // Multiplier when performance is excellent
    good: number         // Multiplier when performance is good
    fair: number         // Multiplier when performance is fair
    poor: number         // Multiplier when performance is poor
  }
  minIntervals: PollingOptions  // Minimum polling intervals
  maxIntervals: PollingOptions  // Maximum polling intervals
  adaptationRate: number        // How quickly to adapt (0-1, higher = faster)
  evaluationInterval: number    // How often to evaluate performance (ms)
}

/**
 * Performance-based adaptive polling manager
 * Adjusts polling rates based on system performance metrics
 */
export class AdaptivePollingManager implements ISystem {
  readonly name = 'AdaptivePollingManager'

  private logger: Logger
  private performanceMonitor: PerformanceMonitor
  private pollingController: PollingController
  private config: AdaptiveConfig
  private lastEvaluationTime: number = 0
  private currentScalingFactor: number = 1.0
  private isAdapting: boolean = false

  // Performance history for smoothing
  private performanceHistory: PerformanceMetrics[] = []
  private readonly maxHistorySize = 10

  // Default configuration
  private readonly defaultConfig: AdaptiveConfig = {
    enabled: true,
    thresholds: {
      fpsExcellent: 55,
      fpsGood: 45,
      fpsFair: 30,
      fpsPoor: 20,
      frameTimeTarget: 16.67, // 60 FPS
      memoryLimit: 200, // 200MB
    },
    scalingFactors: {
      excellent: 0.7,   // Increase polling rate (30% faster)
      good: 1.0,        // Normal polling rate
      fair: 1.5,        // Reduce polling rate (50% slower)
      poor: 2.5,        // Significantly reduce polling rate (150% slower)
    },
    minIntervals: {
      entities: 100,    // Minimum 100ms (10 times per second)
      vegetation: 2000, // Minimum 2 seconds
      chunks: 500,      // Minimum 500ms
    },
    maxIntervals: {
      entities: 1000,   // Maximum 1 second
      vegetation: 10000, // Maximum 10 seconds
      chunks: 2000,     // Maximum 2 seconds
    },
    adaptationRate: 0.3,  // Smooth adaptation
    evaluationInterval: 2000, // Evaluate every 2 seconds
  }

  constructor(
    performanceMonitor: PerformanceMonitor,
    pollingController: PollingController,
    config: Partial<AdaptiveConfig> = {}
  ) {
    this.logger = new Logger('AdaptivePollingManager')
    this.performanceMonitor = performanceMonitor
    this.pollingController = pollingController
    this.config = { ...this.defaultConfig, ...config }
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing adaptive polling manager...')

    // Initialize with current scaling factor
    this.currentScalingFactor = 1.0
    this.lastEvaluationTime = Date.now()

    // Store initial polling options
    const initialOptions = this.pollingController.getOptions()
    this.logger.info(`Initial polling intervals: entities=${initialOptions.entities}ms, vegetation=${initialOptions.vegetation}ms`)

    this.logger.info(`Adaptive polling manager initialized (enabled: ${this.config.enabled})`)
  }

  update(deltaTime: number): void {
    if (!this.config.enabled) {
      return
    }

    const currentTime = Date.now()

    // Check if it's time to evaluate performance
    if (currentTime - this.lastEvaluationTime >= this.config.evaluationInterval) {
      this.evaluateAndAdapt()
      this.lastEvaluationTime = currentTime
    }
  }

  destroy(): void {
    this.logger.debug('Destroying adaptive polling manager...')
    this.performanceHistory = []
    this.logger.debug('Adaptive polling manager destroyed')
  }

  /**
   * Evaluate current performance and adapt polling rates
   */
  private evaluateAndAdapt(): void {
    const metrics = this.performanceMonitor.getMetrics()

    // Add to performance history
    this.addToHistory(metrics)

    // Calculate smoothed performance metrics
    const smoothedMetrics = this.calculateSmoothedMetrics()

    // Determine target scaling factor based on performance
    const targetScalingFactor = this.calculateTargetScalingFactor(smoothedMetrics)

    // Apply smoothing to scaling factor changes
    this.currentScalingFactor = this.applyAdaptationSmoothing(
      this.currentScalingFactor,
      targetScalingFactor
    )

    // Calculate new polling intervals
    const newIntervals = this.calculateAdaptiveIntervals(this.currentScalingFactor)

    // Apply new intervals if they've changed significantly
    this.applyAdaptiveIntervals(newIntervals)

    // Log adaptation details
    this.logAdaptation(smoothedMetrics, targetScalingFactor, newIntervals)
  }

  /**
   * Add metrics to performance history
   */
  private addToHistory(metrics: PerformanceMetrics): void {
    this.performanceHistory.push(metrics)

    // Limit history size
    if (this.performanceHistory.length > this.maxHistorySize) {
      this.performanceHistory.shift()
    }
  }

  /**
   * Calculate smoothed performance metrics from history
   */
  private calculateSmoothedMetrics(): PerformanceMetrics {
    if (this.performanceHistory.length === 0) {
      return this.performanceMonitor.getMetrics()
    }

    const sum = this.performanceHistory.reduce((acc, metrics) => ({
      fps: acc.fps + metrics.fps,
      frameTime: acc.frameTime + metrics.frameTime,
      entities: acc.entities + metrics.entities,
      chunks: acc.chunks + metrics.chunks,
      drawCalls: acc.drawCalls + metrics.drawCalls,
      memoryUsage: (acc.memoryUsage || 0) + (metrics.memoryUsage || 0),
    }), {
      fps: 0,
      frameTime: 0,
      entities: 0,
      chunks: 0,
      drawCalls: 0,
      memoryUsage: 0,
    })

    const count = this.performanceHistory.length
    return {
      fps: Math.round(sum.fps / count),
      frameTime: sum.frameTime / count,
      entities: Math.round(sum.entities / count),
      chunks: Math.round(sum.chunks / count),
      drawCalls: Math.round(sum.drawCalls / count),
      memoryUsage: sum.memoryUsage ? Math.round(sum.memoryUsage / count) : undefined,
    }
  }

  /**
   * Calculate target scaling factor based on performance metrics
   */
  private calculateTargetScalingFactor(metrics: PerformanceMetrics): number {
    const { thresholds } = this.config

    // Primary factor: FPS
    let scalingFactor = 1.0
    if (metrics.fps >= thresholds.fpsExcellent) {
      scalingFactor = this.config.scalingFactors.excellent
    } else if (metrics.fps >= thresholds.fpsGood) {
      scalingFactor = this.config.scalingFactors.good
    } else if (metrics.fps >= thresholds.fpsFair) {
      scalingFactor = this.config.scalingFactors.fair
    } else {
      scalingFactor = this.config.scalingFactors.poor
    }

    // Secondary factor: Frame time consistency
    const frameTimeVariance = Math.abs(metrics.frameTime - thresholds.frameTimeTarget)
    if (frameTimeVariance > thresholds.frameTimeTarget * 0.5) {
      scalingFactor *= 1.2 // Further reduce polling if frame times are inconsistent
    }

    // Tertiary factor: Memory usage (if available)
    if (metrics.memoryUsage && metrics.memoryUsage > thresholds.memoryLimit) {
      const memoryOverage = (metrics.memoryUsage - thresholds.memoryLimit) / thresholds.memoryLimit
      scalingFactor *= (1 + memoryOverage) // Reduce polling based on memory overage
    }

    return Math.max(0.1, Math.min(5.0, scalingFactor)) // Clamp to reasonable range
  }

  /**
   * Apply smoothing to scaling factor changes
   */
  private applyAdaptationSmoothing(current: number, target: number): number {
    const difference = target - current
    const smoothedDifference = difference * this.config.adaptationRate
    return current + smoothedDifference
  }

  /**
   * Calculate adaptive polling intervals based on scaling factor
   */
  private calculateAdaptiveIntervals(scalingFactor: number): Partial<PollingOptions> {
    const baseOptions = this.pollingController.getOptions()

    return {
      entities: this.clampInterval(
        Math.round(baseOptions.entities * scalingFactor),
        this.config.minIntervals.entities,
        this.config.maxIntervals.entities
      ),
      vegetation: this.clampInterval(
        Math.round(baseOptions.vegetation * scalingFactor),
        this.config.minIntervals.vegetation,
        this.config.maxIntervals.vegetation
      ),
      chunks: this.clampInterval(
        Math.round(baseOptions.chunks * scalingFactor),
        this.config.minIntervals.chunks,
        this.config.maxIntervals.chunks
      ),
    }
  }

  /**
   * Clamp interval to min/max bounds
   */
  private clampInterval(value: number, min: number, max: number): number {
    return Math.max(min, Math.min(max, value))
  }

  /**
   * Apply new polling intervals
   */
  private applyAdaptiveIntervals(newIntervals: Partial<PollingOptions>): void {
    const currentOptions = this.pollingController.getOptions()

    // Only apply if changes are significant (more than 10% difference)
    const significantChange =
      Math.abs(newIntervals.entities! - currentOptions.entities) > currentOptions.entities * 0.1 ||
      Math.abs(newIntervals.vegetation! - currentOptions.vegetation) > currentOptions.vegetation * 0.1 ||
      Math.abs(newIntervals.chunks! - currentOptions.chunks) > currentOptions.chunks * 0.1

    if (significantChange) {
      this.pollingController.updateOptions(newIntervals)
      this.logger.debug(`Applied adaptive polling intervals: entities=${newIntervals.entities}ms, vegetation=${newIntervals.vegetation}ms, chunks=${newIntervals.chunks}ms`)
    }
  }

  /**
   * Log adaptation details for debugging
   */
  private logAdaptation(
    metrics: PerformanceMetrics,
    targetScalingFactor: number,
    newIntervals: Partial<PollingOptions>
  ): void {
    if (this.config.adaptationRate > 0.2) { // Only log detailed info for significant adaptations
      this.logger.debug(`Performance: FPS=${metrics.fps}, FrameTime=${metrics.frameTime.toFixed(2)}ms, Memory=${metrics.memoryUsage || '--'}MB`)
      this.logger.debug(`Adaptation: scaling=${this.currentScalingFactor.toFixed(2)}→${targetScalingFactor.toFixed(2)}`)
      this.logger.debug(`New intervals: entities=${newIntervals.entities}ms, vegetation=${newIntervals.vegetation}ms`)
    }
  }

  /**
   * Enable or disable adaptive polling
   */
  setEnabled(enabled: boolean): void {
    this.config.enabled = enabled
    this.logger.info(`Adaptive polling ${enabled ? 'enabled' : 'disabled'}`)

    if (!enabled) {
      // Reset to base intervals when disabled
      this.resetToBaseIntervals()
    }
  }

  /**
   * Reset polling to base intervals
   */
  resetToBaseIntervals(): void {
    const currentOptions = this.pollingController.getOptions()

    // Reset to more conservative base intervals
    const baseIntervals: Partial<PollingOptions> = {
      entities: 200,  // 5 times per second
      vegetation: 5000, // Once every 5 seconds
      chunks: 1000,
    }

    this.pollingController.updateOptions(baseIntervals)
    this.currentScalingFactor = 1.0
    this.logger.info('Reset to base polling intervals')
  }

  /**
   * Get current adaptive status
   */
  getAdaptiveStatus(): {
    enabled: boolean
    currentScalingFactor: number
    performanceGrade: string
    lastEvaluation: Date
    adaptationHistory: number[]
  } {
    const metrics = this.performanceMonitor.getMetrics()
    const performanceGrade = this.performanceMonitor.getPerformanceGrade()

    return {
      enabled: this.config.enabled,
      currentScalingFactor: this.currentScalingFactor,
      performanceGrade,
      lastEvaluation: new Date(this.lastEvaluationTime),
      adaptationHistory: this.performanceHistory.map(m => m.fps),
    }
  }

  /**
   * Get adaptive configuration
   */
  getConfig(): Readonly<AdaptiveConfig> {
    return { ...this.config }
  }

  /**
   * Update adaptive configuration
   */
  updateConfig(config: Partial<AdaptiveConfig>): void {
    this.config = { ...this.config, ...config }
    this.logger.info('Updated adaptive polling configuration')
  }

  /**
   * Force immediate adaptation evaluation
   */
  forceEvaluation(): void {
    this.logger.debug('Forcing immediate adaptation evaluation')
    this.lastEvaluationTime = 0 // Force evaluation on next update
  }
}