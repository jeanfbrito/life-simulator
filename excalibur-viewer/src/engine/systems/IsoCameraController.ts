import { ISystem } from '../SystemRegistry'
import { vec } from 'excalibur'
import { Logger } from '@/utils/Logger'
import { InputSystem } from './InputSystem'

/**
 * Isometric camera controller with pan inertia and zoom bounds
 */
export class IsoCameraController implements ISystem {
  readonly name = 'IsoCameraController'

  private logger: Logger
  private inputSystem: InputSystem

  // Camera state
  private position: vec
  private velocity: vec
  private targetPosition: vec
  private zoom: number
  private targetZoom: number

  // Camera constraints
  private minZoom: number
  private maxZoom: number
  private bounds: { min: vec; max: vec } | null

  // Viewport settings
  private viewportSize: { width: number; height: number }
  private tileSize: number

  // Inertia settings
  private dragInertia: number
  private zoomSmoothing: number
  private positionSmoothing: number

  constructor(inputSystem: InputSystem) {
    this.logger = new Logger('IsoCameraController')
    this.inputSystem = inputSystem

    // Initialize camera state
    this.position = vec(0, 0)
    this.velocity = vec(0, 0)
    this.targetPosition = vec(0, 0)
    this.zoom = 1.0
    this.targetZoom = 1.0

    // Set constraints
    this.minZoom = 0.2
    this.maxZoom = 3.0
    this.bounds = null // No bounds initially

    // Viewport settings
    this.viewportSize = { width: 1200, height: 800 }
    this.tileSize = 32 // Standard tile size for isometric

    // Physics settings
    this.dragInertia = 0.85
    this.zoomSmoothing = 0.2
    this.positionSmoothing = 0.3
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing isometric camera controller...')

    this.updateViewportSize()
    this.setupInputHandlers()

    this.logger.debug('Isometric camera controller initialized')
  }

  update(deltaTime: number): void {
    // Update camera position with smoothing
    const positionDelta = this.targetPosition.sub(this.position)
    this.position = this.position.add(positionDelta.scale(this.positionSmoothing))

    // Apply velocity with drag
    if (this.velocity.magnitude() > 0.01) {
      this.targetPosition = this.targetPosition.add(this.velocity)
      this.velocity = this.velocity.scale(this.dragInertia)
    } else {
      this.velocity = vec(0, 0)
    }

    // Update zoom with smoothing
    const zoomDelta = this.targetZoom - this.zoom
    this.zoom += zoomDelta * this.zoomSmoothing

    // Apply bounds if set
    this.applyBounds()

    // Update HUD
    this.updateHUD()
  }

  destroy(): void {
    this.logger.debug('Isometric camera controller destroyed')
  }

  /**
   * Set world bounds for camera movement
   */
  setBounds(min: vec, max: vec): void {
    this.bounds = { min: min.clone(), max: max.clone() }
    this.applyBounds()
  }

  /**
   * Clear world bounds
   */
  clearBounds(): void {
    this.bounds = null
  }

  /**
   * Move camera to position with smoothing
   */
  moveTo(position: vec): void {
    this.targetPosition = this.isometricToScreen(position.clone())
    this.velocity = vec(0, 0) // Clear velocity when moving to specific position
  }

  /**
   * Pan camera by amount (in screen coordinates)
   */
  pan(amount: vec): void {
    this.targetPosition = this.targetPosition.add(amount)
    this.velocity = this.velocity.add(amount.scale(0.5)) // Add some velocity for inertia
  }

  /**
   * Zoom to specific level
   */
  zoomTo(zoom: number, focusPoint?: vec): void {
    const clampedZoom = Math.max(this.minZoom, Math.min(this.maxZoom, zoom))

    if (focusPoint && this.zoom !== clampedZoom) {
      // Zoom towards focus point
      const zoomFactor = clampedZoom / this.zoom
      const viewportCenter = vec(this.viewportSize.width / 2, this.viewportSize.height / 2)
      const offset = focusPoint.sub(viewportCenter)
      const newPosition = this.targetPosition.add(offset.scale(1 - zoomFactor))
      this.targetPosition = newPosition
    }

    this.targetZoom = clampedZoom
  }

  /**
   * Get camera position in world coordinates
   */
  getWorldPosition(): vec {
    return this.screenToIsometric(this.position)
  }

  /**
   * Get camera zoom level
   */
  getZoom(): number {
    return this.zoom
  }

  /**
   * Convert world coordinates to screen coordinates
   */
  worldToScreen(worldPos: vec): vec {
    const iso = this.isometricToScreen(worldPos)
    return iso.sub(this.position).scale(this.zoom).add(vec(this.viewportSize.width / 2, this.viewportSize.height / 2))
  }

  /**
   * Convert screen coordinates to world coordinates
   */
  screenToWorld(screenPos: vec): vec {
    const relative = screenPos.sub(vec(this.viewportSize.width / 2, this.viewportSize.height / 2)).scale(1 / this.zoom)
    const iso = relative.add(this.position)
    return this.screenToIsometric(iso)
  }

  /**
   * Convert isometric coordinates to screen coordinates
   */
  private isometricToScreen(isoPos: vec): vec {
    const x = (isoPos.x - isoPos.y) * this.tileSize
    const y = (isoPos.x + isoPos.y) * this.tileSize * 0.5
    return vec(x, y)
  }

  /**
   * Convert screen coordinates to isometric coordinates
   */
  private screenToIsometric(screenPos: vec): vec {
    const x = screenPos.x / this.tileSize
    const y = screenPos.y / (this.tileSize * 0.5)

    const isoX = (x + y) * 0.5
    const isoY = (y - x) * 0.5

    return vec(isoX, isoY)
  }

  /**
   * Get visible area in world coordinates
   */
  getVisibleBounds(): { min: vec; max: vec } {
    const corners = [
      vec(0, 0),
      vec(this.viewportSize.width, 0),
      vec(0, this.viewportSize.height),
      vec(this.viewportSize.width, this.viewportSize.height)
    ]

    const worldCorners = corners.map(corner => this.screenToWorld(corner))

    const xs = worldCorners.map(c => c.x)
    const ys = worldCorners.map(c => c.y)

    return {
      min: vec(Math.min(...xs), Math.min(...ys)),
      max: vec(Math.max(...xs), Math.max(...ys))
    }
  }

  /**
   * Update viewport size on resize
   */
  updateViewportSize(): void {
    this.viewportSize = {
      width: window.innerWidth,
      height: window.innerHeight
    }
  }

  /**
   * Setup input handlers for camera controls
   */
  private setupInputHandlers(): void {
    // Mouse drag for panning
    let dragStart: vec | null = null
    let lastPos: vec | null = null

    this.inputSystem.on('pointer.down', (event: PointerEvent) => {
      dragStart = vec(event.clientX, event.clientY)
      lastPos = dragStart.clone()
      this.velocity = vec(0, 0) // Clear velocity on new drag
    })

    this.inputSystem.on('pointer.move', (event: PointerEvent) => {
      const currentPos = vec(event.clientX, event.clientY)

      if (dragStart && lastPos) {
        const delta = currentPos.sub(lastPos)
        this.pan(delta)
        lastPos = currentPos
      }
    })

    this.inputSystem.on('pointer.up', () => {
      if (dragStart && lastPos) {
        // Add final velocity based on last movement
        const finalVelocity = lastPos.sub(dragStart).scale(0.1)
        this.velocity = this.velocity.add(finalVelocity)
      }
      dragStart = null
      lastPos = null
    })

    // Wheel for zooming
    this.inputSystem.on('wheel', (event: WheelEvent) => {
      const delta = -event.deltaY * 0.001
      const newZoom = this.targetZoom * (1 + delta)

      // Zoom towards mouse position
      const mousePos = vec(event.clientX, event.clientY)
      this.zoomTo(newZoom, mousePos)
    })

    // Keyboard controls
    this.inputSystem.on('keydown', (event: KeyboardEvent) => {
      const panSpeed = 20
      const zoomSpeed = 0.1

      switch (event.key) {
        case 'ArrowUp':
        case 'w':
        case 'W':
          this.pan(vec(0, -panSpeed))
          break
        case 'ArrowDown':
        case 's':
        case 'S':
          this.pan(vec(0, panSpeed))
          break
        case 'ArrowLeft':
        case 'a':
        case 'A':
          this.pan(vec(-panSpeed, 0))
          break
        case 'ArrowRight':
        case 'd':
        case 'D':
          this.pan(vec(panSpeed, 0))
          break
        case '=':
        case '+':
          this.zoomTo(this.targetZoom + zoomSpeed)
          break
        case '-':
        case '_':
          this.zoomTo(this.targetZoom - zoomSpeed)
          break
        case 'r':
        case 'R':
          if (event.ctrlKey || event.metaKey) {
            // Reset camera
            this.moveTo(vec(0, 0))
            this.zoomTo(1.0)
          }
          break
      }
    })
  }

  /**
   * Apply world bounds to camera position
   */
  private applyBounds(): void {
    if (!this.bounds) return

    // Convert bounds to screen coordinates
    const screenMin = this.isometricToScreen(this.bounds.min)
    const screenMax = this.isometricToScreen(this.bounds.max)

    // Calculate maximum allowed screen position
    const viewSize = vec(this.viewportSize.width, this.viewportSize.height).scale(1 / this.zoom)
    const maxPos = screenMax.sub(viewSize.scale(0.5))
    const minPos = screenMin.add(viewSize.scale(0.5))

    // Clamp target position
    this.targetPosition.x = Math.max(minPos.x, Math.min(maxPos.x, this.targetPosition.x))
    this.targetPosition.y = Math.max(minPos.y, Math.min(maxPos.y, this.targetPosition.y))
  }

  /**
   * Update HUD with camera information
   */
  private updateHUD(): void {
    const worldPos = this.getWorldPosition()

    const cameraPosElement = document.getElementById('camera-pos')
    if (cameraPosElement) {
      cameraPosElement.textContent = `${Math.round(worldPos.x)}, ${Math.round(worldPos.y)}`
    }

    const zoomLevelElement = document.getElementById('zoom-level')
    if (zoomLevelElement) {
      zoomLevelElement.textContent = `${this.zoom.toFixed(2)}x`
    }
  }

  /**
   * Get camera configuration
   */
  getConfig(): Readonly<{
    position: vec
    zoom: number
    minZoom: number
    maxZoom: number
    tileSize: number
    bounds: { min: vec; max: vec } | null
  }> {
    return {
      position: this.getWorldPosition(),
      zoom: this.zoom,
      minZoom: this.minZoom,
      maxZoom: this.maxZoom,
      tileSize: this.tileSize,
      bounds: this.bounds ? { ...this.bounds } : null
    }
  }
}