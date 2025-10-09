import { ISystem } from '../SystemRegistry'
import { vec } from 'excalibur'
import { Logger } from '@/utils/Logger'

/**
 * Simple 2D camera controller for MVP demo
 */
export class CameraController implements ISystem {
  readonly name = 'CameraController'

  private logger: Logger
  private position: vec
  private zoom: number
  private minZoom: number
  private maxZoom: number
  private viewportSize: { width: number; height: number }

  // For smooth camera movement
  private targetPosition: vec
  private targetZoom: number
  private smoothing: number

  // Input state
  private isDragging = false
  private dragStart: vec
  private dragStartCamera: vec

  constructor() {
    this.logger = new Logger('CameraController')

    // Initialize camera at origin
    this.position = vec(0, 0)
    this.targetPosition = vec(0, 0)

    // Initialize zoom
    this.zoom = 1.0
    this.targetZoom = 1.0
    this.minZoom = 0.1
    this.maxZoom = 5.0

    // Viewport size (will be updated on resize)
    this.viewportSize = { width: 1200, height: 800 }

    // Camera smoothing
    this.smoothing = 0.15

    // Drag state
    this.dragStart = vec(0, 0)
    this.dragStartCamera = vec(0, 0)
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing camera controller...')

    // Update viewport size
    this.updateViewportSize()

    this.logger.debug('Camera controller initialized')
  }

  update(deltaTime: number): void {
    // Smooth camera movement
    const positionDelta = this.targetPosition.sub(this.position)
    this.position = this.position.add(positionDelta.scale(this.smoothing))

    // Smooth zoom
    const zoomDelta = this.targetZoom - this.zoom
    this.zoom += zoomDelta * this.smoothing

    // Update HUD with camera info
    this.updateHUD()
  }

  destroy(): void {
    this.logger.debug('Camera controller destroyed')
  }

  /**
   * Set camera position (instant)
   */
  setPosition(position: vec): void {
    this.position = position.clone()
    this.targetPosition = position.clone()
  }

  /**
   * Set camera position with smooth movement
   */
  moveTo(position: vec): void {
    this.targetPosition = position.clone()
  }

  /**
   * Get current camera position
   */
  getPosition(): vec {
    return this.position.clone()
  }

  /**
   * Set zoom level (instant)
   */
  setZoom(zoom: number): void {
    this.zoom = Math.max(this.minZoom, Math.min(this.maxZoom, zoom))
    this.targetZoom = this.zoom
  }

  /**
   * Set zoom level with smooth transition
   */
  zoomTo(zoom: number): void {
    this.targetZoom = Math.max(this.minZoom, Math.min(this.maxZoom, zoom))
  }

  /**
   * Get current zoom level
   */
  getZoom(): number {
    return this.zoom
  }

  /**
   * Handle pointer down
   */
  handlePointerDown(worldPos: vec): void {
    this.isDragging = true
    this.dragStart = worldPos.clone()
    this.dragStartCamera = this.position.clone()
  }

  /**
   * Handle pointer move
   */
  handlePointerMove(worldPos: vec): void {
    if (!this.isDragging) return

    const delta = this.dragStart.sub(worldPos)
    this.targetPosition = this.dragStartCamera.add(delta)
  }

  /**
   * Handle pointer up
   */
  handlePointerUp(): void {
    this.isDragging = false
  }

  /**
   * Handle wheel for zooming
   */
  handleWheel(delta: number, worldPos: vec): void {
    // Calculate new zoom
    const zoomDelta = delta > 0 ? 0.9 : 1.1
    const newZoom = this.targetZoom * zoomDelta
    this.zoomTo(newZoom)

    // Zoom towards pointer position (optional enhancement)
    // For now, just zoom from center
  }

  /**
   * Reset camera to origin
   */
  reset(): void {
    this.moveTo(vec(0, 0))
    this.zoomTo(1.0)
  }

  /**
   * Pan camera by amount
   */
  pan(amount: vec): void {
    this.targetPosition = this.targetPosition.add(amount)
  }

  /**
   * Convert screen coordinates to world coordinates
   */
  screenToWorld(screenPos: vec): vec {
    const center = vec(this.viewportSize.width / 2, this.viewportSize.height / 2)
    const scaledOffset = screenPos.sub(center).scale(1 / this.zoom)
    return this.position.add(scaledOffset)
  }

  /**
   * Convert world coordinates to screen coordinates
   */
  worldToScreen(worldPos: vec): vec {
    const center = vec(this.viewportSize.width / 2, this.viewportSize.height / 2)
    const worldOffset = worldPos.sub(this.position).scale(this.zoom)
    return center.add(worldOffset)
  }

  /**
   * Get visible world bounds
   */
  getVisibleBounds(): {
    min: vec
    max: vec
  } {
    const topLeft = this.screenToWorld(vec(0, 0))
    const bottomRight = this.screenToWorld(vec(this.viewportSize.width, this.viewportSize.height))

    return {
      min: vec(Math.min(topLeft.x, bottomRight.x), Math.min(topLeft.y, bottomRight.y)),
      max: vec(Math.max(topLeft.x, bottomRight.x), Math.max(topLeft.y, bottomRight.y))
    }
  }

  /**
   * Update viewport size
   */
  updateViewportSize(): void {
    this.viewportSize = {
      width: window.innerWidth,
      height: window.innerHeight
    }
  }

  /**
   * Update HUD with camera information
   */
  private updateHUD(): void {
    const cameraPosElement = document.getElementById('camera-pos')
    if (cameraPosElement) {
      cameraPosElement.textContent = `${Math.round(this.position.x)}, ${Math.round(this.position.y)}`
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
    viewportSize: { width: number; height: number }
  }> {
    return {
      position: this.position.clone(),
      zoom: this.zoom,
      minZoom: this.minZoom,
      maxZoom: this.maxZoom,
      viewportSize: { ...this.viewportSize }
    }
  }
}