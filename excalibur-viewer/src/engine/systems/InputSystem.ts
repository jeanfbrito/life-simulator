import { ISystem } from '../SystemRegistry'
import { Logger } from '@/utils/Logger'

/**
 * Input event types
 */
export interface InputEvents {
  'pointer.down': (event: PointerEvent) => void
  'pointer.up': (event: PointerEvent) => void
  'pointer.move': (event: PointerEvent) => void
  'wheel': (event: WheelEvent) => void
  'keydown': (event: KeyboardEvent) => void
  'keyup': (event: KeyboardEvent) => void
}

/**
 * Input state tracking
 */
export interface InputState {
  pointer: {
    isDown: boolean
    position: { x: number; y: number }
    lastPosition: { x: number; y: number }
    dragStart: { x: number; y: number } | null
  }
  keyboard: {
    keys: Set<string>
    modifiers: {
      shift: boolean
      ctrl: boolean
      alt: boolean
      meta: boolean
    }
  }
}

/**
 * Input handling system
 */
export class InputSystem implements ISystem {
  readonly name = 'InputSystem'

  private logger: Logger
  private eventListeners: Map<keyof InputEvents, EventListener> = new Map()
  private customHandlers: Map<keyof InputEvents, Set<Function>> = new Map()
  private state: InputState
  private element: HTMLElement

  constructor(element: HTMLElement = document.body) {
    this.logger = new Logger('InputSystem')
    this.element = element

    this.state = {
      pointer: {
        isDown: false,
        position: { x: 0, y: 0 },
        lastPosition: { x: 0, y: 0 },
        dragStart: null,
      },
      keyboard: {
        keys: new Set(),
        modifiers: {
          shift: false,
          ctrl: false,
          alt: false,
          meta: false,
        },
      },
    }
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing input system...')

    this.setupEventListeners()
    this.setupKeyboardListeners()

    this.logger.debug('Input system initialized')
  }

  update(deltaTime: number): void {
    // Update pointer last position
    this.state.pointer.lastPosition = { ...this.state.pointer.position }
  }

  destroy(): void {
    this.logger.debug('Destroying input system...')

    this.removeEventListeners()
    this.customHandlers.clear()

    this.logger.debug('Input system destroyed')
  }

  /**
   * Register a custom input handler
   */
  on<K extends keyof InputEvents>(event: K, handler: InputEvents[K]): void {
    if (!this.customHandlers.has(event)) {
      this.customHandlers.set(event, new Set())
    }
    this.customHandlers.get(event)!.add(handler)
  }

  /**
   * Unregister a custom input handler
   */
  off<K extends keyof InputEvents>(event: K, handler: InputEvents[K]): void {
    const handlers = this.customHandlers.get(event)
    if (handlers) {
      handlers.delete(handler)
      if (handlers.size === 0) {
        this.customHandlers.delete(event)
      }
    }
  }

  /**
   * Get current input state
   */
  getState(): Readonly<InputState> {
    return this.state
  }

  /**
   * Check if a key is currently pressed
   */
  isKeyPressed(key: string): boolean {
    return this.state.keyboard.keys.has(key.toLowerCase())
  }

  /**
   * Check if pointer is currently down
   */
  isPointerDown(): boolean {
    return this.state.pointer.isDown
  }

  /**
   * Get pointer drag delta
   */
  getPointerDelta(): { x: number; y: number } {
    return {
      x: this.state.pointer.position.x - this.state.pointer.lastPosition.x,
      y: this.state.pointer.position.y - this.state.pointer.lastPosition.y,
    }
  }

  /**
   * Get drag distance from start
   */
  getDragDistance(): { x: number; y: number } {
    if (!this.state.pointer.dragStart) {
      return { x: 0, y: 0 }
    }

    return {
      x: this.state.pointer.position.x - this.state.pointer.dragStart.x,
      y: this.state.pointer.position.y - this.state.pointer.dragStart.y,
    }
  }

  private setupEventListeners(): void {
    // Pointer events
    const pointerDownHandler = (event: PointerEvent) => {
      this.handlePointerDown(event)
      this.emit('pointer.down', event)
    }

    const pointerUpHandler = (event: PointerEvent) => {
      this.handlePointerUp(event)
      this.emit('pointer.up', event)
    }

    const pointerMoveHandler = (event: PointerEvent) => {
      this.handlePointerMove(event)
      this.emit('pointer.move', event)
    }

    const wheelHandler = (event: WheelEvent) => {
      event.preventDefault()
      this.emit('wheel', event)
    }

    // Add listeners
    this.element.addEventListener('pointerdown', pointerDownHandler)
    this.element.addEventListener('pointerup', pointerUpHandler)
    this.element.addEventListener('pointermove', pointerMoveHandler)
    this.element.addEventListener('wheel', wheelHandler, { passive: false })

    // Store for cleanup
    this.eventListeners.set('pointer.down', pointerDownHandler)
    this.eventListeners.set('pointer.up', pointerUpHandler)
    this.eventListeners.set('pointer.move', pointerMoveHandler)
    this.eventListeners.set('wheel', wheelHandler)
  }

  private setupKeyboardListeners(): void {
    const keyDownHandler = (event: KeyboardEvent) => {
      this.handleKeyDown(event)
      this.emit('keydown', event)
    }

    const keyUpHandler = (event: KeyboardEvent) => {
      this.handleKeyUp(event)
      this.emit('keyup', event)
    }

    document.addEventListener('keydown', keyDownHandler)
    document.addEventListener('keyup', keyUpHandler)

    this.eventListeners.set('keydown', keyDownHandler)
    this.eventListeners.set('keyup', keyUpHandler)
  }

  private handlePointerDown(event: PointerEvent): void {
    this.state.pointer.isDown = true
    this.state.pointer.position = { x: event.clientX, y: event.clientY }
    this.state.pointer.dragStart = { x: event.clientX, y: event.clientY }
  }

  private handlePointerUp(event: PointerEvent): void {
    this.state.pointer.isDown = false
    this.state.pointer.dragStart = null
  }

  private handlePointerMove(event: PointerEvent): void {
    this.state.pointer.lastPosition = { ...this.state.pointer.position }
    this.state.pointer.position = { x: event.clientX, y: event.clientY }
  }

  private handleKeyDown(event: KeyboardEvent): void {
    const key = event.key.toLowerCase()
    this.state.keyboard.keys.add(key)

    // Update modifiers
    this.state.keyboard.modifiers.shift = event.shiftKey
    this.state.keyboard.modifiers.ctrl = event.ctrlKey
    this.state.keyboard.modifiers.alt = event.altKey
    this.state.keyboard.modifiers.meta = event.metaKey
  }

  private handleKeyUp(event: KeyboardEvent): void {
    const key = event.key.toLowerCase()
    this.state.keyboard.keys.delete(key)

    // Update modifiers
    this.state.keyboard.modifiers.shift = event.shiftKey
    this.state.keyboard.modifiers.ctrl = event.ctrlKey
    this.state.keyboard.modifiers.alt = event.altKey
    this.state.keyboard.modifiers.meta = event.metaKey
  }

  private emit<K extends keyof InputEvents>(event: K, eventData: any): void {
    const handlers = this.customHandlers.get(event)
    if (handlers) {
      for (const handler of handlers) {
        try {
          handler(eventData)
        } catch (error) {
          this.logger.error(`Error in input handler for event '${event}':`, error)
        }
      }
    }
  }

  private removeEventListeners(): void {
    for (const [event, listener] of this.eventListeners.entries()) {
      if (event === 'keydown' || event === 'keyup') {
        document.removeEventListener(event, listener)
      } else {
        this.element.removeEventListener(event, listener)
      }
    }
    this.eventListeners.clear()
  }
}