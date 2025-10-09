import { Logger } from '@/utils/Logger'

/**
 * System interface for all game systems
 */
export interface ISystem {
  readonly name: string
  initialize(): Promise<void>
  update(deltaTime: number): void
  destroy(): void
}

/**
 * System registry for managing all game systems
 */
export class SystemRegistry {
  private systems: Map<string, ISystem> = new Map()
  private logger: Logger
  private isInitialized = false

  constructor() {
    this.logger = new Logger('SystemRegistry')
  }

  /**
   * Register a system
   */
  register(system: ISystem): void {
    if (this.systems.has(system.name)) {
      this.logger.warn(`System '${system.name}' already registered, skipping`)
      return
    }

    this.systems.set(system.name, system)
    this.logger.debug(`Registered system: ${system.name}`)
  }

  /**
   * Initialize all registered systems
   */
  async initializeAll(): Promise<void> {
    if (this.isInitialized) {
      this.logger.warn('Systems already initialized')
      return
    }

    this.logger.info(`Initializing ${this.systems.size} systems...`)

    const initPromises = Array.from(this.systems.values()).map(async (system) => {
      try {
        this.logger.debug(`Initializing system: ${system.name}`)
        await system.initialize()
        this.logger.debug(`System initialized: ${system.name}`)
      } catch (error) {
        this.logger.error(`Failed to initialize system '${system.name}':`, error)
        throw error
      }
    })

    await Promise.all(initPromises)
    this.isInitialized = true
    this.logger.info('All systems initialized successfully')
  }

  /**
   * Update all systems (called each frame)
   */
  updateAll(deltaTime: number): void {
    if (!this.isInitialized) {
      this.logger.warn('Systems not initialized, skipping update')
      return
    }

    for (const system of this.systems.values()) {
      try {
        system.update(deltaTime)
      } catch (error) {
        this.logger.error(`Error updating system '${system.name}':`, error)
      }
    }
  }

  /**
   * Get a registered system by name
   */
  get<T extends ISystem>(name: string): T | undefined {
    return this.systems.get(name) as T
  }

  /**
   * Check if a system is registered
   */
  has(name: string): boolean {
    return this.systems.has(name)
  }

  /**
   * Get all registered system names
   */
  getSystemNames(): string[] {
    return Array.from(this.systems.keys())
  }

  /**
   * Destroy all systems
   */
  destroyAll(): void {
    this.logger.info('Destroying all systems...')

    for (const [name, system] of this.systems.entries()) {
      try {
        this.logger.debug(`Destroying system: ${name}`)
        system.destroy()
        this.logger.debug(`System destroyed: ${name}`)
      } catch (error) {
        this.logger.error(`Error destroying system '${name}':`, error)
      }
    }

    this.systems.clear()
    this.isInitialized = false
    this.logger.info('All systems destroyed')
  }

  /**
   * Get system count
   */
  get count(): number {
    return this.systems.size
  }

  /**
   * Check if systems are initialized
   */
  get initialized(): boolean {
    return this.isInitialized
  }
}