import { ISystem } from '../SystemRegistry'
import { Actor, Text, Font, Color } from 'excalibur'
import { Logger } from '@/utils/Logger'
import { Point2D } from '@/utils/CoordinateSystem'
import { EntityType, EntitySex, EntityAction, Entity, EntitiesResponse } from '@/data/types'
import { APIClient } from '@/data/APIClient'
import { DataCache } from '@/data/DataCache'
import { MovementSystem } from './MovementSystem'

/**
 * Entity actor with emoji representation
 */
export class EntityActor extends Actor {
  public readonly entityId: string
  public readonly entityType: EntityType
  public readonly species: string
  public readonly name: string

  private entityGraphics: Text
  private targetPosition: Point2D | null = null
  private moveSpeed: number = 50 // pixels per second

  constructor(entity: Entity) {
    super({
      width: 32,
      height: 32,
      z: 100 // Entities render above terrain and resources
    })

    this.entityId = entity.id
    this.entityType = entity.type
    this.species = entity.species
    this.name = entity.name

    // Create entity graphics (emoji)
    const emoji = EntityType.getEmoji(entity.type)
    this.entityGraphics = new Text({
      text: emoji,
      font: new Font({
        size: 24,
        family: 'Arial, sans-serif'
      }),
      color: Color.White
    })

    this.graphics.add(this.entityGraphics)

    // Set initial position
    this.updatePosition(entity.position)
  }

  /**
   * Update entity position
   */
  updatePosition(position: Point2D): void {
    this.pos.x = position.x
    this.pos.y = position.y
  }

  /**
   * Move entity towards target position
   */
  moveTo(target: Point2D): void {
    this.targetPosition = target
  }

  /**
   * Update entity movement
   */
  updateMovement(deltaTime: number): void {
    if (!this.targetPosition) return

    const dx = this.targetPosition.x - this.pos.x
    const dy = this.targetPosition.y - this.pos.y
    const distance = Math.sqrt(dx * dx + dy * dy)

    if (distance < 2) {
      // Reached target
      this.targetPosition = null
      return
    }

    // Move towards target
    const moveDistance = this.moveSpeed * deltaTime / 1000
    const ratio = moveDistance / distance

    this.pos.x += dx * ratio
    this.pos.y += dy * ratio
  }

  /**
   * Update entity visual state based on action
   */
  updateAction(action: EntityAction): void {
    // Could change emoji, color, or add visual effects based on action
    switch (action) {
      case EntityAction.MOVING:
        this.entityGraphics.color = Color.Yellow
        break
      case EntityAction.EATING:
        this.entityGraphics.color = Color.Green
        break
      case EntityAction.DRINKING:
        this.entityGraphics.color = Color.Blue
        break
      case EntityAction.SLEEPING:
        this.entityGraphics.color = Color.Gray
        break
      default:
        this.entityGraphics.color = Color.White
    }
  }

  /**
   * Get entity data for debugging
   */
  getEntityData(): {
    id: string
    type: EntityType
    species: string
    name: string
    position: { x: number; y: number }
    targetPosition: { x: number; y: number } | null
  } {
    return {
      id: this.entityId,
      type: this.entityType,
      species: this.species,
      name: this.name,
      position: { x: this.pos.x, y: this.pos.y },
      targetPosition: this.targetPosition ? { x: this.targetPosition.x, y: this.targetPosition.y } : null
    }
  }
}

/**
 * Factory for creating entity actors
 */
export class SpeciesFactory {
  /**
   * Create an entity actor from entity data
   */
  static createEntityActor(entity: Entity): EntityActor {
    return new EntityActor(entity)
  }

  /**
   * Get emoji for entity type
   */
  static getEntityEmoji(entityType: EntityType): string {
    const emojiMap: Record<EntityType, string> = {
      [EntityType.HUMAN]: '🧍‍♂️',
      [EntityType.RABBIT]: '🐇',
      [EntityType.DEER]: '🦌',
      [EntityType.WOLF]: '🐺',
      [EntityType.BEAR]: '🐻',
      [EntityType.BIRD]: '🦅',
      [EntityType.FISH]: '🐟',
      [EntityType.INSECT]: '🐛'
    }

    return emojiMap[entityType] || '❓'
  }

  /**
   * Get movement speed for entity type
   */
  static getMovementSpeed(entityType: EntityType): number {
    const speedMap: Record<EntityType, number> = {
      [EntityType.HUMAN]: 40,
      [EntityType.RABBIT]: 60,
      [EntityType.DEER]: 50,
      [EntityType.WOLF]: 55,
      [EntityType.BEAR]: 30,
      [EntityType.BIRD]: 80,
      [EntityType.FISH]: 45,
      [EntityType.INSECT]: 70
    }

    return speedMap[entityType] || 40
  }

  /**
   * Get display size for entity type
   */
  static getDisplaySize(entityType: EntityType): number {
    const sizeMap: Record<EntityType, number> = {
      [EntityType.HUMAN]: 24,
      [EntityType.RABBIT]: 20,
      [EntityType.DEER]: 28,
      [EntityType.WOLF]: 26,
      [EntityType.BEAR]: 32,
      [EntityType.BIRD]: 18,
      [EntityType.FISH]: 22,
      [EntityType.INSECT]: 16
    }

    return sizeMap[entityType] || 24
  }
}

/**
 * Entity manager for handling entity lifecycle
 */
export class EntityManager implements ISystem {
  readonly name = 'EntityManager'

  private logger: Logger
  private entities: Map<string, EntityActor> = new Map()
  private lastUpdateTime: number = 0
  private pollingInterval: number = 200 // 5 times per second

  // Data sources
  private apiClient: APIClient
  private dataCache: DataCache
  private movementSystem: MovementSystem | null = null

  constructor() {
    this.logger = new Logger('EntityManager')
    this.apiClient = new APIClient('http://localhost:54321')
    this.dataCache = new DataCache({
      maxChunks: 100,
      maxEntities: 500,
      ttl: 60000 // 1 minute for entities
    })
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing entity manager...')

    // Try to load initial entities
    try {
      await this.loadInitialEntities()
      this.logger.info('Entity manager initialized successfully')
    } catch (error) {
      this.logger.warn('Failed to load initial entities, using mock data:', error)
      this.loadMockEntities()
    }

    this.lastUpdateTime = Date.now()
  }

  update(deltaTime: number): void {
    const now = Date.now()

    // Poll for entity updates
    if (now - this.lastUpdateTime > this.pollingInterval) {
      this.pollEntityUpdates()
      this.lastUpdateTime = now
    }

    // Update all entity movements
    for (const entity of this.entities.values()) {
      entity.updateMovement(deltaTime)
    }
  }

  destroy(): void {
    this.logger.debug('Destroying entity manager...')

    // Clean up all entity actors
    for (const entity of this.entities.values()) {
      entity.kill()
    }

    this.entities.clear()
    this.logger.debug('Entity manager destroyed')
  }

  /**
   * Load initial entities
   */
  private async loadInitialEntities(): Promise<void> {
    const response = await this.apiClient.getEntities()
    const entities = response.entities || []

    for (const entity of entities) {
      this.createEntity(entity)
    }

    this.logger.info(`Loaded ${entities.length} initial entities`)
  }

  /**
   * Load mock entities for testing
   */
  private loadMockEntities(): void {
    const mockEntities: Entity[] = [
      {
        id: 'mock-human-1',
        type: EntityType.HUMAN,
        species: 'Human',
        name: 'Alice',
        position: { x: 100, y: 100 },
        sex: EntitySex.FEMALE,
        action: EntityAction.IDLE,
        stats: {
          health: 100,
          thirst: 75,
          hunger: 60,
          energy: 90
        }
      },
      {
        id: 'mock-rabbit-1',
        type: EntityType.RABBIT,
        species: 'Rabbit',
        name: 'Bunny',
        position: { x: 200, y: 150 },
        sex: EntitySex.MALE,
        action: EntityAction.FLEEING,
        stats: {
          health: 80,
          thirst: 50,
          hunger: 30,
          energy: 85
        }
      },
      {
        id: 'mock-deer-1',
        type: EntityType.DEER,
        species: 'Deer',
        name: 'Bambi',
        position: { x: 300, y: 200 },
        sex: EntitySex.MALE,
        action: EntityAction.MOVING,
        stats: {
          health: 90,
          thirst: 70,
          hunger: 45,
          energy: 75
        }
      }
    ]

    for (const entity of mockEntities) {
      this.createEntity(entity)
    }

    // Schedule some test movements
    this.scheduleTestMovements()

    this.logger.info(`Loaded ${mockEntities.length} mock entities`)
  }

  /**
   * Schedule test movements for demo purposes
   */
  private scheduleTestMovements(): void {
    setTimeout(() => {
      const rabbit = this.entities.get('mock-rabbit-1')
      if (rabbit && this.movementSystem) {
        this.movementSystem.moveEntity(rabbit, { x: 150, y: 200 }, 1500)
      }
    }, 2000)

    setTimeout(() => {
      const deer = this.entities.get('mock-deer-1')
      if (deer && this.movementSystem) {
        this.movementSystem.moveEntity(deer, { x: 250, y: 250 }, 2000)
      }
    }, 3000)

    setTimeout(() => {
      const human = this.entities.get('mock-human-1')
      if (human && this.movementSystem) {
        this.movementSystem.moveEntity(human, { x: 150, y: 150 }, 1800)
      }
    }, 4000)
  }

  /**
   * Poll for entity updates
   */
  private async pollEntityUpdates(): Promise<void> {
    try {
      const response = await this.apiClient.getEntities()
      const entities = response.entities || []

      // Update existing entities and add new ones
      const seenIds = new Set<string>()

      for (const entityData of entities) {
        seenIds.add(entityData.id)

        const existingEntity = this.entities.get(entityData.id)
        if (existingEntity) {
          // Update existing entity position through movement system
          if (this.movementSystem) {
            this.movementSystem.moveEntity(existingEntity, entityData.position, 200)
          }

          // Update action and show corresponding overlay
          if (this.movementSystem) {
            this.movementSystem.updateEntityAction(existingEntity, entityData.action)
          }

          existingEntity.updateAction(entityData.action)
        } else {
          // Create new entity
          this.createEntity(entityData)
        }
      }

      // Remove entities that no longer exist
      for (const [id, entity] of this.entities.entries()) {
        if (!seenIds.has(id)) {
          this.removeEntity(id)
        }
      }

    } catch (error) {
      this.logger.warn('Failed to poll entity updates:', error)
    }
  }

  /**
   * Create a new entity
   */
  private createEntity(entityData: Entity): void {
    const actor = SpeciesFactory.createEntityActor(entityData)

    // Add to current scene
    const engine = (globalThis as any).excaliburEngine
    if (engine && engine.currentScene) {
      engine.currentScene.add(actor)
    }

    this.entities.set(entityData.id, actor)
    this.logger.debug(`Created entity: ${entityData.name} (${entityData.id})`)
  }

  /**
   * Remove an entity
   */
  private removeEntity(entityId: string): void {
    const entity = this.entities.get(entityId)
    if (entity) {
      entity.kill()

      // Clean up fear overlay if movement system exists
      if (this.movementSystem) {
        this.movementSystem.hideFearOverlay(entityId)
      }

      this.entities.delete(entityId)
      this.logger.debug(`Removed entity: ${entityId}`)
    }
  }

  /**
   * Set movement system reference
   */
  setMovementSystem(movementSystem: MovementSystem): void {
    this.movementSystem = movementSystem
    this.logger.debug('Movement system reference set')
  }

  /**
   * Get entity count
   */
  getEntityCount(): number {
    return this.entities.size
  }

  /**
   * Get all entities
   */
  getAllEntities(): EntityActor[] {
    return Array.from(this.entities.values())
  }

  /**
   * Get entity by ID
   */
  getEntity(entityId: string): EntityActor | undefined {
    return this.entities.get(entityId)
  }

  /**
   * Get API client for external access
   */
  getAPIClient(): APIClient {
    return this.apiClient
  }

  /**
   * Get data cache for external access
   */
  getDataCache(): DataCache {
    return this.dataCache
  }
}