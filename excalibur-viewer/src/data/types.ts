/**
 * Type definitions for Life Simulator API responses
 * Mirrors the backend schemas for type safety and consistency
 */

// ============================================================================
// Core Types
// ============================================================================

/**
 * 2D Position vector
 */
export interface Position {
  x: number
  y: number
}

/**
 * World bounds
 */
export interface WorldBounds {
  min: Position
  max: Position
}

// ============================================================================
// World Info API
// ============================================================================

/**
 * World information response from /api/world_info
 */
export interface WorldInfoResponse {
  name: string
  seed: number
  chunk_count: number
  bounds: WorldBounds
}

// ============================================================================
// Terrain Types
// ============================================================================

/**
 * Terrain types as defined in the backend
 */
export enum TerrainType {
  DEEP_WATER = 'DeepWater',
  SHALLOW_WATER = 'ShallowWater',
  SAND = 'Sand',
  GRASS = 'Grass',
  FOREST = 'Forest',
  DESERT = 'Desert',
  DIRT = 'Dirt',
  MOUNTAINS = 'Mountains',
  SNOW = 'Snow',
  STONE = 'Stone',
  SWAMP = 'Swamp',
}

/**
 * Resource types as defined in the backend
 */
export enum ResourceType {
  TREE_PINE = 'TreePine',
  TREE_OAK = 'TreeOak',
  TREE_BIRCH = 'TreeBirch',
  TREE_PALM = 'TreePalm',
  BUSH = 'Bush',
  FLOWER = 'Flower',
  ROCK = 'Rock',
  STONE = 'Stone',
  GOLD = 'Gold',
  COPPER = 'Copper',
  IRON = 'Iron',
  FRUIT = 'Fruit',
  VEGETABLES = 'Vegetables',
  HERB = 'Herb',
  MUSHROOM = 'Mushroom',
  // Empty string represents no resource
  EMPTY = '',
}

// ============================================================================
// Chunk Data API
// ============================================================================

/**
 * Single chunk data containing terrain and resources
 */
export interface ChunkData {
  terrain: string[][]
  resources: string[][]
}

/**
 * Multi-layer chunk response from /api/chunks?layers=true
 */
export interface ChunkLayersResponse {
  chunk_data: Record<string, ChunkData>
}

/**
 * Legacy terrain-only chunk response (backward compatibility)
 */
export interface ChunkResponse {
  chunk_data: Record<string, string[][]>
}

// ============================================================================
// Entity Types
// ============================================================================

/**
 * Entity types supported in the simulation
 */
export enum EntityType {
  HUMAN = 'Human',
  RABBIT = 'Rabbit',
  DEER = 'Deer',
  WOLF = 'Wolf',
  FOX = 'Fox',
  BEAR = 'Bear',
  RACCOON = 'Raccoon',
}

/**
 * Helper methods for EntityType
 */
export namespace EntityType {
  /**
   * Get emoji representation for entity type
   */
  export function getEmoji(entityType: EntityType): string {
    const emojiMap: Record<EntityType, string> = {
      [EntityType.HUMAN]: '🧍‍♂️',
      [EntityType.RABBIT]: '🐇',
      [EntityType.DEER]: '🦌',
      [EntityType.WOLF]: '🐺',
      [EntityType.FOX]: '🦊',
      [EntityType.BEAR]: '🐻',
      [EntityType.RACCOON]: '🦝'
    }

    return emojiMap[entityType] || '❓'
  }
}

/**
 * Sex types for entities
 */
export enum EntitySex {
  MALE = 'male',
  FEMALE = 'female',
}

/**
 * Current action states for entities
 */
export enum EntityAction {
  IDLE = 'Idle',
  WANDERING = 'Wandering',
  MOVING = 'Moving',
  EATING = 'Eating',
  DRINKING = 'Drinking',
  SLEEPING = 'Sleeping',
  FLEEING = 'Fleeing',
  HUNTING = 'Hunting',
  MATING = 'Mating',
}

/**
 * Entity data from /api/entities
 */
export interface Entity {
  id: string
  type: EntityType
  species: string
  name: string
  position: Point2D
  sex: EntitySex
  action: EntityAction
  stats: {
    health: number
    hunger: number // 0-100
    thirst: number // 0-100
    energy: number // 0-100
  }
}

/**
 * Entity API response
 */
export interface EntityResponse {
  entities: Entity[]
}

/**
 * Entity list response from /api/entities
 */
export interface EntitiesResponse {
  entities: Entity[]
}

// ============================================================================
// Species Configuration API
// ============================================================================

/**
 * Species appearance and behavior configuration
 */
export interface SpeciesConfig {
  emoji: string
  is_juvenile: boolean
  juvenile_name_prefix?: string
  movement_speed: number
  name: string
  name_plural?: string
}

/**
 * Default entity configuration
 */
export interface DefaultEntityConfig {
  emoji: string
  offsetX: number
  offsetY: number
  sizeMultiplier: number
}

/**
 * Juvenile scale multipliers by species
 */
export interface JuvenileScales {
  [speciesName: string]: number
}

/**
 * Complete species configuration response from /api/species
 */
export interface SpeciesResponse {
  default_entity: DefaultEntityConfig
  juvenile_scales: JuvenileScales
  species: Record<EntityType, SpeciesConfig>
}

// ============================================================================
// Vegetation Metrics API
// ============================================================================

/**
 * Chunk LOD (Level of Detail) metrics
 */
export interface ChunkLODMetrics {
  active_chunks: number
  cold_chunks: number
  hot_chunks: number
  total_chunks: number
  warm_chunks: number
}

/**
 * Performance metrics for vegetation system
 */
export interface VegetationPerformanceMetrics {
  data_source: string
  generation_time_ms: number
}

/**
 * Resource grid metrics
 */
export interface ResourceGridMetrics {
  active_cells: number
  events_processed: number
  pending_events: number
  processing_time_ms: number
}

/**
 * Complete vegetation metrics response from /api/vegetation/metrics
 */
export interface VegetationMetricsResponse {
  chunk_lod: ChunkLODMetrics
  performance: VegetationPerformanceMetrics
  phase: string
  resource_grid: ResourceGridMetrics
}

// ============================================================================
// API Client Types
// ============================================================================

/**
 * API error response format
 */
export interface APIError {
  error: string
  message?: string
  details?: Record<string, unknown>
}

/**
 * Request parameters for chunk endpoints
 */
export interface ChunkRequestParams {
  center_x?: number
  center_y?: number
  radius?: number
  coords?: string[]
  layers?: boolean
}

/**
 * Polling configuration options
 */
export interface PollingOptions {
  entities: number // ms
  vegetation: number // ms
  chunks?: number // ms (on-demand)
}

/**
 * Cache configuration options
 */
export interface CacheOptions {
  maxChunks: number
  maxEntities: number
  ttl: number // ms
}

// ============================================================================
// Utility Types
// ============================================================================

/**
 * Type guard to check if a value is a valid terrain type
 */
export function isTerrainType(value: string): value is TerrainType {
  return Object.values(TerrainType).includes(value as TerrainType)
}

/**
 * Type guard to check if a value is a valid resource type
 */
export function isResourceType(value: string): value is ResourceType {
  return Object.values(ResourceType).includes(value as ResourceType) || value === ''
}

/**
 * Type guard to check if a value is a valid entity type
 */
export function isEntityType(value: string): value is EntityType {
  return Object.values(EntityType).includes(value as EntityType)
}

/**
 * Type guard to check if a response is an API error
 */
export function isAPIError(response: unknown): response is APIError {
  return typeof response === 'object' &&
         response !== null &&
         'error' in response
}

/**
 * Polling event data
 */
export interface PollingEvent {
  type: string // PollingEventType defined in PollingController
  data: unknown
  timestamp: number
  error?: Error
}