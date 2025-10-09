import { Rectangle, Text, Actor, Color } from 'excalibur'
import { CoordinateSystem, Point2D } from '@/utils/CoordinateSystem'
import { TerrainType, ResourceType } from '@/data/types'
import { TerrainMaterial } from './TerrainMaterial'

/**
 * Enhanced tile actor with resource rendering
 */
export class TileActor extends Actor {
  public readonly tilePos: Point2D
  public readonly terrainType: TerrainType
  public readonly resourceType: ResourceType | null

  constructor(tilePos: Point2D, terrainType: TerrainType, resourceType: ResourceType | null = null) {
    super({
      width: 32,
      height: 32
    })

    this.tilePos = tilePos
    this.terrainType = terrainType
    this.resourceType = resourceType

    // Create terrain graphics
    const terrainGraphics = TerrainMaterial.createTerrainGraphics(terrainType)
    this.graphics.add(terrainGraphics)

    // Add resource graphics if present
    if (resourceType) {
      const resourceGraphics = TerrainMaterial.createResourceGraphics(resourceType)
      resourceGraphics.pos.y = -10 // Offset resource above terrain
      this.graphics.add(resourceGraphics)
    }
  }

  /**
   * Update tile rendering based on camera position
   */
  updateRender(cameraPos: Point2D, screenSize: Point2D, zoom: number): void {
    // Calculate screen position using coordinate system
    const worldPos = { x: this.tilePos.x * 16, y: this.tilePos.y * 16 }
    const isoPos = CoordinateSystem.worldToIsometric(worldPos)
    const cameraIso = CoordinateSystem.worldToIsometric(cameraPos)

    const screenX = (isoPos.x - cameraIso.x) * zoom + screenSize.x / 2
    const screenY = (isoPos.y - cameraIso.y) * zoom + screenSize.y / 2

    // Update actor position
    this.pos.x = screenX
    this.pos.y = screenY

    // Update z-order based on terrain type (for layering)
    this.z = this.calculateZOrder()
  }

  /**
   * Check if tile is visible on screen
   */
  isVisible(cameraPos: Point2D, screenSize: Point2D, zoom: number, padding: number = 32): boolean {
    const screenPos = this.pos
    return (
      screenPos.x >= -padding &&
      screenPos.x <= screenSize.x + padding &&
      screenPos.y >= -padding &&
      screenPos.y <= screenSize.y + padding
    )
  }

  /**
   * Check if tile contains a resource
   */
  hasResource(): boolean {
    return this.resourceType !== null
  }

  /**
   * Get terrain color
   */
  getTerrainColor(): string {
    return TerrainMaterial.getTerrainColor(this.terrainType)
  }

  /**
   * Get resource emoji
   */
  getResourceEmoji(): string | null {
    return this.resourceType ? TerrainMaterial.getResourceEmoji(this.resourceType) : null
  }

  /**
   * Get tile data for debugging
   */
  getTileData(): {
    position: Point2D
    terrain: TerrainType
    resource: ResourceType | null
    screenPos: { x: number; y: number }
    visible: boolean
  } {
    return {
      position: this.tilePos,
      terrain: this.terrainType,
      resource: this.resourceType,
      screenPos: { x: this.pos.x, y: this.pos.y },
      visible: false // Will be updated by controller
    }
  }

  /**
   * Calculate z-order for proper layering
   */
  private calculateZOrder(): number {
    let zOrder = 0

    // Base z-order based on terrain type
    switch (this.terrainType) {
      case TerrainType.DEEP_WATER:
      case TerrainType.SHALLOW_WATER:
        zOrder = -10
        break
      case TerrainType.SAND:
        zOrder = 0
        break
      case TerrainType.GRASS:
        zOrder = 1
        break
      case TerrainType.FOREST:
        zOrder = 5
        break
      case TerrainType.MOUNTAINS:
        zOrder = 8
        break
      case TerrainType.SNOW:
        zOrder = 10
        break
      default:
        zOrder = 2
    }

    // Adjust for resources
    if (this.resourceType) {
      const shouldRenderAbove = TerrainMaterial.shouldRenderAboveEntities(this.resourceType)
      if (shouldRenderAbove) {
        zOrder += 20
      } else {
        zOrder += 0.5
      }
    }

    return zOrder
  }
}