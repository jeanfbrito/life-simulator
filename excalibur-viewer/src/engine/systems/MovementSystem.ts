import { ISystem } from '../SystemRegistry'
import { Actor, Circle, Color } from 'excalibur'
import { Logger } from '@/utils/Logger'
import { Point2D } from '@/utils/CoordinateSystem'
import { EntityActor } from './EntityManager'

/**
 * Movement interpolation data for entities
 */
interface MovementData {
  entity: EntityActor
  startPosition: Point2D
  targetPosition: Point2D
  startTime: number
  duration: number
  isMoving: boolean
}

/**
 * Fear overlay data for entities
 */
interface FearOverlay {
  entity: EntityActor
  actor: Actor
  radius: number
  intensity: number
  color: Color
}

/**
 * MovementSystem handles smooth entity movement interpolation
 * and visual overlays like fear states
 */
export class MovementSystem implements ISystem {
  readonly name = 'MovementSystem'

  private logger: Logger
  private movements: Map<string, MovementData> = new Map()
  private fearOverlays: Map<string, FearOverlay> = new Map()
  private interpolationSpeed: number = 200 // ms for smooth movement

  constructor() {
    this.logger = new Logger('MovementSystem')
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing movement system...')
    this.logger.info('Movement system initialized')
  }

  update(deltaTime: number): void {
    const currentTime = Date.now()

    // Update smooth movements
    this.updateMovements(currentTime)

    // Update fear overlays
    this.updateFearOverlays()

    // Clean up completed movements
    this.cleanupCompletedMovements()
  }

  destroy(): void {
    this.logger.debug('Destroying movement system...')

    // Clean up all overlays
    for (const overlay of this.fearOverlays.values()) {
      overlay.actor.kill()
    }

    this.movements.clear()
    this.fearOverlays.clear()
    this.logger.debug('Movement system destroyed')
  }

  /**
   * Start smooth movement for an entity
   */
  moveEntity(entity: EntityActor, targetPosition: Point2D, duration?: number): void {
    const movementDuration = duration || this.interpolationSpeed
    const currentPosition = { x: entity.pos.x, y: entity.pos.y }

    // Calculate distance to determine if movement is needed
    const dx = targetPosition.x - currentPosition.x
    const dy = targetPosition.y - currentPosition.y
    const distance = Math.sqrt(dx * dx + dy * dy)

    if (distance < 1) {
      // Already at target position
      return
    }

    // Create or update movement data
    this.movements.set(entity.entityId, {
      entity,
      startPosition: currentPosition,
      targetPosition,
      startTime: Date.now(),
      duration: movementDuration,
      isMoving: true
    })

    this.logger.debug(`Starting movement for entity ${entity.entityId} to (${targetPosition.x}, ${targetPosition.y})`)
  }

  /**
   * Show fear overlay for an entity
   */
  showFearOverlay(entity: EntityActor, intensity: number = 0.5, radius: number = 50): void {
    const overlay = this.fearOverlays.get(entity.entityId)

    if (overlay) {
      // Update existing overlay
      overlay.intensity = intensity
      overlay.radius = radius
      this.updateOverlayAppearance(overlay)
    } else {
      // Create new overlay
      const overlayActor = new Actor({
        x: entity.pos.x,
        y: entity.pos.y,
        z: 50 // Below entities but above terrain
      })

      const fearCircle = new Circle({
        radius: radius,
        color: Color.Red
      })

      overlayActor.graphics.add(fearCircle)

      // Make it translucent
      overlayActor.graphics.opacity = 0.3

      // Add to scene
      const engine = (globalThis as any).excaliburEngine
      if (engine && engine.currentScene) {
        engine.currentScene.add(overlayActor)
      }

      const fearOverlay: FearOverlay = {
        entity,
        actor: overlayActor,
        radius,
        intensity,
        color: Color.Red
      }

      this.fearOverlays.set(entity.entityId, fearOverlay)
      this.updateOverlayAppearance(fearOverlay)

      this.logger.debug(`Added fear overlay for entity ${entity.entityId}`)
    }
  }

  /**
   * Hide fear overlay for an entity
   */
  hideFearOverlay(entityId: string): void {
    const overlay = this.fearOverlays.get(entityId)
    if (overlay) {
      overlay.actor.kill()
      this.fearOverlays.delete(entityId)
      this.logger.debug(`Removed fear overlay for entity ${entityId}`)
    }
  }

  /**
   * Update entity states based on actions
   */
  updateEntityAction(entity: EntityActor, action: string): void {
    // Show different overlays based on action
    switch (action) {
      case 'Fleeing':
        this.showFearOverlay(entity, 0.7, 60)
        break
      case 'Hunting':
        this.showFearOverlay(entity, 0.3, 40)
        break
      case 'Eating':
        this.hideFearOverlay(entity.entityId)
        break
      case 'Drinking':
        this.hideFearOverlay(entity.entityId)
        break
      default:
        // Gradually fade out overlay
        this.fadeOutOverlay(entity.entityId)
    }
  }

  /**
   * Update all smooth movements
   */
  private updateMovements(currentTime: number): void {
    for (const [entityId, movement] of this.movements.entries()) {
      if (!movement.isMoving) continue

      const elapsed = currentTime - movement.startTime
      const progress = Math.min(elapsed / movement.duration, 1)

      if (progress >= 1) {
        // Movement complete
        movement.entity.updatePosition(movement.targetPosition)
        movement.isMoving = false
        this.logger.debug(`Movement completed for entity ${entityId}`)
      } else {
        // Interpolate position
        const interpolatedPosition = this.interpolatePosition(
          movement.startPosition,
          movement.targetPosition,
          progress
        )

        movement.entity.updatePosition(interpolatedPosition)
      }
    }
  }

  /**
   * Update fear overlay positions
   */
  private updateFearOverlays(): void {
    for (const overlay of this.fearOverlays.values()) {
      overlay.actor.pos.x = overlay.entity.pos.x
      overlay.actor.pos.y = overlay.entity.pos.y
    }
  }

  /**
   * Clean up completed movements
   */
  private cleanupCompletedMovements(): void {
    const toRemove: string[] = []

    for (const [entityId, movement] of this.movements.entries()) {
      if (!movement.isMoving) {
        toRemove.push(entityId)
      }
    }

    for (const entityId of toRemove) {
      this.movements.delete(entityId)
    }

    if (toRemove.length > 0) {
      this.logger.debug(`Cleaned up ${toRemove.length} completed movements`)
    }
  }

  /**
   * Smooth interpolation between positions
   */
  private interpolatePosition(start: Point2D, end: Point2D, progress: number): Point2D {
    // Use ease-in-out function for smoother movement
    const easeProgress = this.easeInOutCubic(progress)

    return {
      x: start.x + (end.x - start.x) * easeProgress,
      y: start.y + (end.y - start.y) * easeProgress
    }
  }

  /**
   * Ease-in-out cubic function
   */
  private easeInOutCubic(t: number): number {
    return t < 0.5
      ? 4 * t * t * t
      : 1 - Math.pow(-2 * t + 2, 3) / 2
  }

  /**
   * Update overlay appearance based on intensity
   */
  private updateOverlayAppearance(overlay: FearOverlay): void {
    const opacity = overlay.intensity * 0.4 // Max 40% opacity
    overlay.actor.graphics.opacity = opacity

    // Update color based on intensity
    const hue = 0 // Red
    const saturation = 80 + overlay.intensity * 20 // 80-100% saturation
    const lightness = 50 - overlay.intensity * 20 // 30-50% lightness

    // Simple color adjustment (you could use HSL conversion here)
    const color = overlay.intensity > 0.5 ? Color.Red : Color.Orange
    overlay.color = color

    // Update circle radius
    const circle = overlay.actor.graphics.members[0] as Circle
    if (circle) {
      circle.radius = overlay.radius
      circle.color = color
    }
  }

  /**
   * Gradually fade out overlay
   */
  private fadeOutOverlay(entityId: string): void {
    const overlay = this.fearOverlays.get(entityId)
    if (overlay && overlay.intensity > 0) {
      overlay.intensity = Math.max(0, overlay.intensity - 0.02)
      this.updateOverlayAppearance(overlay)

      if (overlay.intensity <= 0) {
        this.hideFearOverlay(entityId)
      }
    }
  }

  /**
   * Get movement count
   */
  getActiveMovementCount(): number {
    return Array.from(this.movements.values()).filter(m => m.isMoving).length
  }

  /**
   * Get overlay count
   */
  getActiveOverlayCount(): number {
    return this.fearOverlays.size
  }

  /**
   * Check if entity is currently moving
   */
  isEntityMoving(entityId: string): boolean {
    const movement = this.movements.get(entityId)
    return movement ? movement.isMoving : false
  }

  /**
   * Get movement data for debugging
   */
  getMovementData(): {
    activeMovements: number
    activeOverlays: number
    movements: Array<{ entityId: string; progress: number; target: Point2D }>
  } {
    const currentTime = Date.now()
    const movementData = Array.from(this.movements.values())
      .filter(m => m.isMoving)
      .map(m => ({
        entityId: m.entity.entityId,
        progress: Math.min((currentTime - m.startTime) / m.duration, 1),
        target: m.targetPosition
      }))

    return {
      activeMovements: this.getActiveMovementCount(),
      activeOverlays: this.getActiveOverlayCount(),
      movements: movementData
    }
  }
}