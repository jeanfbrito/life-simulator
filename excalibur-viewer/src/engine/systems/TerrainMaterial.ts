import { vec, Color, Rectangle, Font, Text } from 'excalibur'
import { Logger } from '@/utils/Logger'
import { CoordinateSystem } from '@/utils/CoordinateSystem'
import { TerrainType, ResourceType } from '@/data/types'

/**
 * Resource rendering configuration
 */
interface ResourceConfig {
  emoji: string
  color?: string
  scale: number
  yOffset: number
  size: number
}

/**
 * Terrain rendering configuration
 */
interface TerrainConfig {
  color: string
  borderColor?: string
  borderWidth: number
  pattern?: 'solid' | 'dotted' | 'crosshatch'
}

/**
 * Terrain material system for rendering terrain and resources
 */
export class TerrainMaterial {
  private static readonly logger = new Logger('TerrainMaterial')

  // Terrain type configurations
  private static readonly TERRAIN_CONFIGS: Record<TerrainType, TerrainConfig> = {
    [TerrainType.DEEP_WATER]: {
      color: '#003366',
      borderColor: '#002244',
      borderWidth: 1,
      pattern: 'solid'
    },
    [TerrainType.SHALLOW_WATER]: {
      color: '#4a7ba7',
      borderColor: '#3a6b97',
      borderWidth: 1,
      pattern: 'solid'
    },
    [TerrainType.SAND]: {
      color: '#f4e4bc',
      borderColor: '#d4d4ac',
      borderWidth: 1,
      pattern: 'solid'
    },
    [TerrainType.GRASS]: {
      color: '#7cfc00',
      borderColor: '#6cc000',
      borderWidth: 0.5,
      pattern: 'solid'
    },
    [TerrainType.FOREST]: {
      color: '#228b22',
      borderColor: '#1a6b1a',
      borderWidth: 1,
      pattern: 'crosshatch'
    },
    [TerrainType.DIRT]: {
      color: '#8b4513',
      borderColor: '#6b3503',
      borderWidth: 1,
      pattern: 'solid'
    },
    [TerrainType.DESERT]: {
      color: '#edc9af',
      borderColor: '#ddb99f',
      borderWidth: 1,
      pattern: 'dotted'
    },
    [TerrainType.MOUNTAINS]: {
      color: '#8b7355',
      borderColor: '#7b6345',
      borderWidth: 1,
      pattern: 'solid'
    },
    [TerrainType.SNOW]: {
      color: '#ffffff',
      borderColor: '#f0f0f0',
      borderWidth: 0.5,
      pattern: 'solid'
    },
    [TerrainType.STONE]: {
      color: '#696969',
      borderColor: '#595959',
      borderWidth: 1,
      pattern: 'solid'
    },
    [TerrainType.SWAMP]: {
      color: '#556b2f',
      borderColor: '#455b1f',
      borderWidth: 1,
      pattern: 'dotted'
    }
  }

  // Resource type configurations
  private static readonly RESOURCE_CONFIGS: Record<ResourceType, ResourceConfig> = {
    [ResourceType.TREE_PINE]: {
      emoji: '🌲',
      scale: 1.2,
      yOffset: -0.3,
      size: 20
    },
    [ResourceType.TREE_OAK]: {
      emoji: '🌳',
      scale: 1.3,
      yOffset: -0.4,
      size: 22
    },
    [ResourceType.TREE_BIRCH]: {
      emoji: '🌴',
      scale: 1.1,
      yOffset: -0.3,
      size: 18
    },
    [ResourceType.TREE_PALM]: {
      emoji: '🌴',
      scale: 1.4,
      yOffset: -0.4,
      size: 24
    },
    [ResourceType.BUSH]: {
      emoji: '🌿',
      scale: 0.8,
      yOffset: -0.2,
      size: 16
    },
    [ResourceType.FLOWER]: {
      emoji: '🌸',
      scale: 0.6,
      yOffset: -0.15,
      size: 12
    },
    [ResourceType.ROCK]: {
      emoji: '🪨',
      scale: 0.9,
      yOffset: -0.1,
      size: 14
    },
    [ResourceType.STONE]: {
      emoji: '🪨',
      scale: 0.7,
      yOffset: -0.1,
      size: 12
    },
    [ResourceType.GOLD]: {
      emoji: '💰',
      scale: 0.9,
      yOffset: -0.1,
      size: 14
    },
    [ResourceType.COPPER]: {
      emoji: '🪙',
      scale: 0.9,
      yOffset: -0.1,
      size: 14
    },
    [ResourceType.IRON]: {
      emoji: '🔩',
      scale: 0.9,
      yOffset: -0.1,
      size: 14
    },
    [ResourceType.FRUIT]: {
      emoji: '🍎',
      scale: 0.8,
      yOffset: -0.2,
      size: 16
    },
    [ResourceType.VEGETABLES]: {
      emoji: '🥬',
      scale: 0.8,
      yOffset: -0.2,
      size: 16
    },
    [ResourceType.HERB]: {
      emoji: '🌿',
      scale: 0.7,
      yOffset: -0.15,
      size: 14
    },
    [ResourceType.MUSHROOM]: {
      emoji: '🍄',
      scale: 0.7,
      yOffset: -0.15,
      size: 14
    }
  }

  /**
   * Get terrain configuration for a terrain type
   */
  static getTerrainConfig(terrainType: TerrainType): TerrainConfig {
    return this.TERRAIN_CONFIGS[terrainType] || this.TERRAIN_CONFIGS[TerrainType.GRASS]
  }

  /**
   * Get resource configuration for a resource type
   */
  static getResourceConfig(resourceType: ResourceType): ResourceConfig {
    return this.RESOURCE_CONFIGS[resourceType] || {
      emoji: '❓',
      scale: 1.0,
      yOffset: -0.2,
      size: 16
    }
  }

  /**
   * Create terrain graphics for a tile
   */
  static createTerrainGraphics(terrainType: TerrainType): Rectangle {
    const config = this.getTerrainConfig(terrainType)

    // Create a simple rectangle for now with terrain color
    const rect = new Rectangle({
      width: 32,
      height: 32,
      color: Color.fromHex(config.color)
    })

    return rect
  }

  /**
   * Create resource graphics for a resource
   */
  static createResourceGraphics(resourceType: ResourceType): Text {
    const config = this.getResourceConfig(resourceType)

    // Create text emoji
    const text = new Text({
      text: config.emoji,
      font: new Font({
        size: config.size,
        family: 'Arial, sans-serif'
      }),
      color: Color.White
    })

    return text
  }

  /**
   * Check if a terrain type has a pattern overlay
   */
  static hasPatternOverlay(terrainType: TerrainType): boolean {
    const config = this.getTerrainConfig(terrainType)
    return config.pattern !== 'solid'
  }

  /**
   * Get terrain color for rendering
   */
  static getTerrainColor(terrainType: TerrainType): string {
    const config = this.getTerrainConfig(terrainType)
    return config.color
  }

  /**
   * Get terrain border color
   */
  static getTerrainBorderColor(terrainType: TerrainType): string {
    const config = this.getTerrainConfig(terrainType)
    return config.borderColor || config.color
  }

  /**
   * Check if a resource type should be rendered above entities
   */
  static shouldRenderAboveEntities(resourceType: ResourceType): boolean {
    // Trees and large objects should render below entities
    const belowEntities = [
      ResourceType.TREE_PINE,
      ResourceType.TREE_OAK,
      ResourceType.TREE_BIRCH,
      ResourceType.TREE_PALM,
      ResourceType.ROCK,
      ResourceType.STONE
    ]

    return !belowEntities.includes(resourceType)
  }

  /**
   * Get resource render order
   */
  static getResourceRenderOrder(): ResourceType[] {
    return [
      // Large trees (render below entities)
      ResourceType.TREE_OAK,
      ResourceType.TREE_PINE,
      ResourceType.TREE_BIRCH,
      ResourceType.TREE_PALM,

      // Medium objects
      ResourceType.ROCK,
      ResourceType.STONE,

      // Small objects (render above entities)
      ResourceType.BUSH,
      ResourceType.FLOWER,
      ResourceType.FRUIT,
      ResourceType.VEGETABLES,
      ResourceType.HERB,
      ResourceType.MUSHROOM,

      // Resources (special case)
      ResourceType.GOLD,
      ResourceType.COPPER,
      ResourceType.IRON
    ]
  }

  /**
   * Create height-enhanced terrain graphics
   */
  static createHeightEnhancedTerrainGraphics(terrainType: TerrainType, height: number = 0): Rectangle {
    const config = this.getTerrainConfig(terrainType)

    // Create a simple rectangle for now with adjusted color based on height
    let baseColor = Color.fromHex(config.color)

    // Darken color based on height
    if (height > 0) {
      baseColor = baseColor.darken(height * 0.1)
    }

    const rect = new Rectangle({
      width: 32,
      height: 32,
      color: baseColor
    })

    return rect
  }

  /**
   * Create tile shadow graphics
   */
  static createShadowGraphics(): Rectangle {
    // Create simple shadow rectangle
    const shadow = new Rectangle({
      width: 32,
      height: 32,
      color: Color.fromHex('#00000033') // Semi-transparent black
    })

    return shadow
  }

  /**
   * Get resource emoji for quick rendering
   */
  static getResourceEmoji(resourceType: ResourceType): string {
    const config = this.getResourceConfig(resourceType)
    return config.emoji
  }

  /**
   * Get resource scale factor
   */
  static getResourceScale(resourceType: ResourceType): number {
    const config = this.getResourceConfig(resourceType)
    return config.scale
  }

  /**
   * Get resource Y offset for positioning
   */
  static getResourceYOffset(resourceType: ResourceType): number {
    const config = this.getResourceConfig(resourceType)
    return config.yOffset
  }
}