import { vec } from 'excalibur'
import { Logger } from '@/utils/Logger'

/**
 * 2D point with integer coordinates
 */
export interface Point2D {
  x: number
  y: number
}

/**
 * 3D point for isometric calculations
 */
export interface Point3D extends Point2D {
  z: number
}

/**
 * Coordinate conversion utilities for isometric projection
 */
export class CoordinateSystem {
  private static readonly TILE_WIDTH = 32
  private static readonly TILE_HEIGHT = 16
  private static readonly TILE_DEPTH = 32

  /**
   * Convert world coordinates to isometric screen coordinates
   * World: (x, y) represents 2D position on the map
   * Isometric: Projects to 2D screen with diamond-shaped tiles
   */
  static worldToIsometric(worldPos: Point2D): Point2D {
    const x = (worldPos.x - worldPos.y) * (this.TILE_WIDTH / 2)
    const y = (worldPos.x + worldPos.y) * (this.TILE_HEIGHT / 2)
    return { x, y }
  }

  /**
   * Convert isometric screen coordinates to world coordinates
   * Inverse of worldToIsometric
   */
  static isometricToWorld(isoPos: Point2D): Point2D {
    const x = (isoPos.x / (this.TILE_WIDTH / 2) + isoPos.y / (this.TILE_HEIGHT / 2)) / 2
    const y = (isoPos.y / (this.TILE_HEIGHT / 2) - isoPos.x / (this.TILE_WIDTH / 2)) / 2
    return { x, y }
  }

  /**
   * Convert world coordinates to 3D isometric coordinates (for height/depth)
   */
  static worldToIsometric3D(worldPos: Point2D, height: number = 0): Point3D {
    const iso = this.worldToIsometric(worldPos)
    return {
      x: iso.x,
      y: iso.y - height * (this.TILE_DEPTH / 4), // Adjust Y for height
      z: height
    }
  }

  /**
   * Convert tile coordinates to world coordinates
   * Tiles are typically 16x16 in world space
   */
  static tileToWorld(tilePos: Point2D): Point2D {
    return {
      x: tilePos.x * 16,
      y: tilePos.y * 16
    }
  }

  /**
   * Convert world coordinates to tile coordinates
   */
  static worldToTile(worldPos: Point2D): Point2D {
    return {
      x: Math.floor(worldPos.x / 16),
      y: Math.floor(worldPos.y / 16)
    }
  }

  /**
   * Convert tile coordinates to isometric screen coordinates
   */
  static tileToIsometric(tilePos: Point2D): Point2D {
    const worldPos = this.tileToWorld(tilePos)
    return this.worldToIsometric(worldPos)
  }

  /**
   * Convert isometric screen coordinates to tile coordinates
   */
  static isometricToTile(isoPos: Point2D): Point2D {
    const worldPos = this.isometricToWorld(isoPos)
    return this.worldToTile(worldPos)
  }

  /**
   * Get the screen bounds for a given world area
   */
  static getWorldBoundsInScreen(minWorld: Point2D, maxWorld: Point2D): {
    min: Point2D
    max: Point2D
  } {
    const corners = [
      { x: minWorld.x, y: minWorld.y },
      { x: maxWorld.x, y: minWorld.y },
      { x: minWorld.x, y: maxWorld.y },
      { x: maxWorld.x, y: maxWorld.y }
    ]

    const isoCorners = corners.map(corner => this.worldToIsometric(corner))

    const xs = isoCorners.map(c => c.x)
    const ys = isoCorners.map(c => c.y)

    return {
      min: { x: Math.min(...xs), y: Math.min(...ys) },
      max: { x: Math.max(...xs), y: Math.max(...ys) }
    }
  }

  /**
   * Get the world bounds for a given screen area
   */
  static getScreenBoundsInWorld(minScreen: Point2D, maxScreen: Point2D): {
    min: Point2D
    max: Point2D
  } {
    const corners = [
      { x: minScreen.x, y: minScreen.y },
      { x: maxScreen.x, y: minScreen.y },
      { x: minScreen.x, y: maxScreen.y },
      { x: maxScreen.x, y: maxScreen.y }
    ]

    const worldCorners = corners.map(corner => this.isometricToWorld(corner))

    const xs = worldCorners.map(c => c.x)
    const ys = worldCorners.map(c => c.y)

    return {
      min: { x: Math.min(...xs), y: Math.min(...ys) },
      max: { x: Math.max(...xs), y: Math.max(...ys) }
    }
  }

  /**
   * Check if a world position is within view bounds
   */
  static isWorldPositionInView(worldPos: Point2D, cameraPos: Point2D, screenSize: Point2D, zoom: number): boolean {
    // Convert world position to screen coordinates
    const isoPos = this.worldToIsometric(worldPos)
    const cameraIso = this.worldToIsometric(cameraPos)

    // Apply zoom and centering
    const screenX = (isoPos.x - cameraIso.x) * zoom + screenSize.x / 2
    const screenY = (isoPos.y - cameraIso.y) * zoom + screenSize.y / 2

    // Check if within screen bounds (with some margin)
    const margin = 64 // pixels
    return screenX >= -margin && screenX <= screenSize.x + margin &&
           screenY >= -margin && screenY <= screenSize.y + margin
  }

  /**
   * Get tiles visible within camera view
   */
  static getVisibleTiles(cameraPos: Point2D, screenSize: Point2D, zoom: number, padding: number = 2): Point2D[] {
    // Get screen bounds in world coordinates
    const screenMin = { x: 0, y: 0 }
    const screenMax = screenSize
    const worldBounds = this.getScreenBoundsInWorld(screenMin, screenMax)

    // Convert to tile coordinates and add padding
    const minTile = this.worldToTile({
      x: worldBounds.min.x - padding * 16,
      y: worldBounds.min.y - padding * 16
    })

    const maxTile = this.worldToTile({
      x: worldBounds.max.x + padding * 16,
      y: worldBounds.max.y + padding * 16
    })

    // Generate all tile positions in range
    const tiles: Point2D[] = []
    for (let x = minTile.x; x <= maxTile.x; x++) {
      for (let y = minTile.y; y <= maxTile.y; y++) {
        tiles.push({ x, y })
      }
    }

    return tiles
  }

  /**
   * Get tile center position in screen coordinates
   */
  static getTileScreenCenter(tilePos: Point2D, cameraPos: Point2D, screenSize: Point2D, zoom: number): Point2D {
    const worldPos = this.tileToWorld(tilePos)
    const isoPos = this.worldToIsometric(worldPos)
    const cameraIso = this.worldToIsometric(cameraPos)

    return {
      x: (isoPos.x - cameraIso.x) * zoom + screenSize.x / 2,
      y: (isoPos.y - cameraIso.y) * zoom + screenSize.y / 2
    }
  }

  /**
   * Calculate distance between two world positions
   */
  static worldDistance(pos1: Point2D, pos2: Point2D): number {
    const dx = pos2.x - pos1.x
    const dy = pos2.y - pos1.y
    return Math.sqrt(dx * dx + dy * dy)
  }

  /**
   * Calculate distance between two tile positions
   */
  static tileDistance(tile1: Point2D, tile2: Point2D): number {
    const dx = tile2.x - tile1.x
    const dy = tile2.y - tile1.y
    return Math.sqrt(dx * dx + dy * dy)
  }

  /**
   * Check if two world positions are the same (with tolerance)
   */
  static worldPositionsEqual(pos1: Point2D, pos2: Point2D, tolerance: number = 0.1): boolean {
    return Math.abs(pos1.x - pos2.x) < tolerance && Math.abs(pos1.y - pos2.y) < tolerance
  }

  /**
   * Check if two tile positions are the same
   */
  static tilePositionsEqual(tile1: Point2D, tile2: Point2D): boolean {
    return tile1.x === tile2.x && tile1.y === tile2.y
  }
}

/**
 * Interpolation utilities for smooth movement
 */
export class Interpolation {
  /**
   * Linear interpolation between two values
   */
  static lerp(a: number, b: number, t: number): number {
    return a + (b - a) * t
  }

  /**
   * Linear interpolation between two points
   */
  static lerpPoint(a: Point2D, b: Point2D, t: number): Point2D {
    return {
      x: this.lerp(a.x, b.x, t),
      y: this.lerp(a.y, b.y, t)
    }
  }

  /**
   * Smooth step interpolation (ease-in-out)
   */
  static smoothStep(t: number): number {
    return t * t * (3 - 2 * t)
  }

  /**
   * Smooth interpolation between two values
   */
  static smoothLerp(a: number, b: number, t: number): number {
    return this.lerp(a, b, this.smoothStep(t))
  }

  /**
   * Smooth interpolation between two points
   */
  static smoothLerpPoint(a: Point2D, b: Point2D, t: number): Point2D {
    return {
      x: this.smoothLerp(a.x, b.x, t),
      y: this.smoothLerp(a.y, b.y, t)
    }
  }

  /**
   * Ease-out cubic interpolation
   */
  static easeOutCubic(t: number): number {
    return 1 - Math.pow(1 - t, 3)
  }

  /**
   * Ease-in cubic interpolation
   */
  static easeInCubic(t: number): number {
    return t * t * t
  }

  /**
   * Ease-in-out cubic interpolation
   */
  static easeInOutCubic(t: number): number {
    return t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2
  }

  /**
   * Interpolate position with specified easing function
   */
  static lerpPointWithEasing(a: Point2D, b: Point2D, t: number, easing: (t: number) => number = this.lerp): Point2D {
    return {
      x: this.lerp(a.x, b.x, easing(t)),
      y: this.lerp(a.y, b.y, easing(t))
    }
  }

  /**
   * Interpolate a path of points
   */
  static lerpPath(path: Point2D[], t: number): Point2D {
    if (path.length === 0) return { x: 0, y: 0 }
    if (path.length === 1) return path[0]

    const totalLength = path.length - 1
    const scaledT = t * totalLength
    const index = Math.floor(scaledT)
    const localT = scaledT - index

    if (index >= path.length - 1) {
      return path[path.length - 1]
    }

    return this.lerpPoint(path[index], path[index + 1], localT)
  }

  /**
   * Get interpolation progress based on time and duration
   */
  static getProgress(startTime: number, currentTime: number, duration: number): number {
    const elapsed = currentTime - startTime
    return Math.min(Math.max(elapsed / duration, 0), 1)
  }

  /**
   * Check if interpolation is complete
   */
  static isComplete(startTime: number, currentTime: number, duration: number): boolean {
    return this.getProgress(startTime, currentTime, duration) >= 1
  }
}

/**
 * Animation utilities for timed movements
 */
export class Animation {
  private static animations: Map<string, {
    startTime: number
    duration: number
    from: Point2D
    to: Point2D
    easing: (t: number) => number
    onComplete?: () => void
  }> = new Map()

  /**
   * Start a new animation
   */
  static start(id: string, from: Point2D, to: Point2D, duration: number, easing: (t: number) => number = Interpolation.lerp, onComplete?: () => void): void {
    this.animations.set(id, {
      startTime: performance.now(),
      duration,
      from,
      to,
      easing,
      onComplete
    })
  }

  /**
   * Get current position of an animation
   */
  static getPosition(id: string): Point2D | null {
    const animation = this.animations.get(id)
    if (!animation) return null

    const progress = Interpolation.getProgress(animation.startTime, performance.now(), animation.duration)

    if (progress >= 1) {
      // Animation complete
      if (animation.onComplete) {
        animation.onComplete()
      }
      this.animations.delete(id)
      return animation.to
    }

    return Interpolation.lerpPointWithEasing(animation.from, animation.to, progress, animation.easing)
  }

  /**
   * Check if animation is running
   */
  static isRunning(id: string): boolean {
    return this.animations.has(id)
  }

  /**
   * Stop an animation
   */
  static stop(id: string): Point2D | null {
    const animation = this.animations.get(id)
    if (!animation) return null

    const currentPos = this.getPosition(id)
    this.animations.delete(id)
    return currentPos
  }

  /**
   * Clean up completed animations
   */
  static cleanup(): void {
    const now = performance.now()
    for (const [id, animation] of this.animations.entries()) {
      if (Interpolation.isComplete(animation.startTime, now, animation.duration)) {
        if (animation.onComplete) {
          animation.onComplete()
        }
        this.animations.delete(id)
      }
    }
  }

  /**
   * Get count of active animations
   */
  static getActiveCount(): number {
    return this.animations.size
  }
}