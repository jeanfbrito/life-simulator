import { ISystem } from '../SystemRegistry'
import { Logger } from '@/utils/Logger'
import { InputSystem } from './InputSystem'
import { IsoCameraController } from './IsoCameraController'

/**
 * Input binding configurations
 */
interface InputBinding {
  key: string
  modifiers?: {
    ctrl?: boolean
    shift?: boolean
    alt?: boolean
  }
  action: string
  description: string
}

/**
 * Input bindings system for mapping keyboard/mouse inputs to actions
 */
export class InputBindings implements ISystem {
  readonly name = 'InputBindings'

  private logger: Logger
  private inputSystem: InputSystem
  private cameraController: IsoCameraController

  // Action handlers
  private actionHandlers: Map<string, () => void> = new Map()

  // Key bindings
  private bindings: InputBinding[] = [
    // Camera movement
    { key: 'w', action: 'camera.pan.up', description: 'Pan camera up' },
    { key: 's', action: 'camera.pan.down', description: 'Pan camera down' },
    { key: 'a', action: 'camera.pan.left', description: 'Pan camera left' },
    { key: 'd', action: 'camera.pan.right', description: 'Pan camera right' },
    { key: 'ArrowUp', action: 'camera.pan.up', description: 'Pan camera up' },
    { key: 'ArrowDown', action: 'camera.pan.down', description: 'Pan camera down' },
    { key: 'ArrowLeft', action: 'camera.pan.left', description: 'Pan camera left' },
    { key: 'ArrowRight', action: 'camera.pan.right', description: 'Pan camera right' },

    // Zoom controls
    { key: '=', action: 'camera.zoom.in', description: 'Zoom in' },
    { key: '+', action: 'camera.zoom.in', description: 'Zoom in' },
    { key: '-', action: 'camera.zoom.out', description: 'Zoom out' },
    { key: '_', action: 'camera.zoom.out', description: 'Zoom out' },

    // Debug/Utility actions
    { key: 'r', modifiers: { ctrl: true }, action: 'camera.reset', description: 'Reset camera' },
    { key: 'r', modifiers: { meta: true }, action: 'camera.reset', description: 'Reset camera' },

    // Toggle actions (for future features)
    { key: '1', action: 'toggle.entities', description: 'Toggle entities overlay' },
    { key: '2', action: 'toggle.terrain', description: 'Toggle terrain overlay' },
    { key: '3', action: 'toggle.resources', description: 'Toggle resources overlay' },
    { key: '4', action: 'toggle.grid', description: 'Toggle grid overlay' },
    { key: 'b', action: 'toggle.biomass', description: 'Toggle biomass heatmap' },
    { key: 'f', action: 'toggle.fear', description: 'Toggle fear overlay' },
    { key: 'i', action: 'toggle.info', description: 'Toggle info panel' },

    // Performance actions
    { key: 'p', modifiers: { ctrl: true }, action: 'debug.performance', description: 'Show performance info' },
    { key: 'p', modifiers: { meta: true }, action: 'debug.performance', description: 'Show performance info' },

    // Screenshot
    { key: 'PrintScreen', action: 'screenshot', description: 'Take screenshot' },
    { key: 'F12', action: 'screenshot', description: 'Take screenshot' },
  ]

  constructor(inputSystem: InputSystem, cameraController: IsoCameraController) {
    this.logger = new Logger('InputBindings')
    this.inputSystem = inputSystem
    this.cameraController = cameraController

    this.setupActionHandlers()
  }

  async initialize(): Promise<void> {
    this.logger.debug('Initializing input bindings...')

    this.setupKeyboardBindings()
    this.updateControlsHelp()

    this.logger.debug('Input bindings initialized')
  }

  update(deltaTime: number): void {
    // Continuous actions can be handled here if needed
  }

  destroy(): void {
    this.logger.debug('Input bindings destroyed')
  }

  /**
   * Register a custom action handler
   */
  registerAction(action: string, handler: () => void): void {
    this.actionHandlers.set(action, handler)
  }

  /**
   * Execute an action by name
   */
  executeAction(action: string): boolean {
    const handler = this.actionHandlers.get(action)
    if (handler) {
      try {
        handler()
        return true
      } catch (error) {
        this.logger.error(`Error executing action '${action}':`, error)
        return false
      }
    }
    return false
  }

  /**
   * Get all available bindings
   */
  getBindings(): InputBinding[] {
    return [...this.bindings]
  }

  /**
   * Get bindings for a specific action
   */
  getBindingsForAction(action: string): InputBinding[] {
    return this.bindings.filter(binding => binding.action === action)
  }

  /**
   * Add a custom binding
   */
  addBinding(binding: InputBinding): void {
    this.bindings.push(binding)
    this.updateControlsHelp()
  }

  /**
   * Remove a binding
   */
  removeBinding(action: string, key: string): boolean {
    const index = this.bindings.findIndex(
      binding => binding.action === action && binding.key === key
    )

    if (index >= 0) {
      this.bindings.splice(index, 1)
      this.updateControlsHelp()
      return true
    }
    return false
  }

  /**
   * Setup default action handlers
   */
  private setupActionHandlers(): void {
    // Camera pan actions
    this.registerAction('camera.pan.up', () => {
      this.cameraController.pan(vec(0, -20))
    })

    this.registerAction('camera.pan.down', () => {
      this.cameraController.pan(vec(0, 20))
    })

    this.registerAction('camera.pan.left', () => {
      this.cameraController.pan(vec(-20, 0))
    })

    this.registerAction('camera.pan.right', () => {
      this.cameraController.pan(vec(20, 0))
    })

    // Camera zoom actions
    this.registerAction('camera.zoom.in', () => {
      const currentZoom = this.cameraController.getZoom()
      this.cameraController.zoomTo(currentZoom + 0.1)
    })

    this.registerAction('camera.zoom.out', () => {
      const currentZoom = this.cameraController.getZoom()
      this.cameraController.zoomTo(currentZoom - 0.1)
    })

    // Camera reset
    this.registerAction('camera.reset', () => {
      this.cameraController.moveTo(vec(0, 0))
      this.cameraController.zoomTo(1.0)
    })

    // Screenshot
    this.registerAction('screenshot', () => {
      this.takeScreenshot()
    })

    // Debug performance
    this.registerAction('debug.performance', () => {
      const metrics = this.cameraController.getConfig()
      console.log('Camera Configuration:', metrics)
    })

    // Placeholder actions for future features
    const placeholderActions = [
      'toggle.entities', 'toggle.terrain', 'toggle.resources', 'toggle.grid',
      'toggle.biomass', 'toggle.fear', 'toggle.info'
    ]

    placeholderActions.forEach(action => {
      this.registerAction(action, () => {
        this.logger.info(`Action '${action}' not implemented yet`)
      })
    })
  }

  /**
   * Setup keyboard event handlers
   */
  private setupKeyboardBindings(): void {
    this.inputSystem.on('keydown', (event: KeyboardEvent) => {
      for (const binding of this.bindings) {
        if (this.matchesBinding(binding, event)) {
          event.preventDefault()
          this.executeAction(binding.action)
          break
        }
      }
    })
  }

  /**
   * Check if an event matches a binding
   */
  private matchesBinding(binding: InputBinding, event: KeyboardEvent): boolean {
    // Check key
    if (event.key.toLowerCase() !== binding.key.toLowerCase()) {
      return false
    }

    // Check modifiers
    if (binding.modifiers) {
      if (binding.modifiers.ctrl && !event.ctrlKey && !event.metaKey) {
        return false
      }
      if (binding.modifiers.shift && !event.shiftKey) {
        return false
      }
      if (binding.modifiers.alt && !event.altKey) {
        return false
      }
    }

    return true
  }

  /**
   * Take a screenshot
   */
  private takeScreenshot(): void {
    const canvas = document.querySelector('canvas') as HTMLCanvasElement
    if (!canvas) {
      this.logger.warn('No canvas found for screenshot')
      return
    }

    // Convert canvas to blob and download
    canvas.toBlob((blob) => {
      if (!blob) return

      const url = URL.createObjectURL(blob)
      const a = document.createElement('a')
      a.href = url
      a.download = `life-simulator-${new Date().toISOString().slice(0, 19)}.png`
      document.body.appendChild(a)
      a.click()
      document.body.removeChild(a)
      URL.revokeObjectURL(url)

      this.logger.info('Screenshot saved')
    })
  }

  /**
   * Update the controls help display
   */
  private updateControlsHelp(): void {
    const controlsElement = document.querySelector('.controls')
    if (!controlsElement) return

    // Group bindings by action category
    const cameraBindings = this.bindings.filter(b => b.action.startsWith('camera.'))
    const toggleBindings = this.bindings.filter(b => b.action.startsWith('toggle.'))
    const debugBindings = this.bindings.filter(b => b.action.startsWith('debug.') || b.action === 'screenshot')

    let html = '<h4>Controls</h4>'

    if (cameraBindings.length > 0) {
      html += '<div><strong>Camera:</strong><br>'
      cameraBindings.forEach(binding => {
        const key = binding.modifiers?.ctrl ? 'Ctrl+' + binding.key.toUpperCase() : binding.key.toUpperCase()
        html += `<span class="control-hint">${key} - ${binding.description}</span><br>`
      })
      html += '</div>'
    }

    if (toggleBindings.length > 0) {
      html += '<div><strong>Toggles:</strong><br>'
      toggleBindings.forEach(binding => {
        const key = binding.key.toUpperCase()
        html += `<span class="control-hint">${key} - ${binding.description}</span><br>`
      })
      html += '</div>'
    }

    if (debugBindings.length > 0) {
      html += '<div><strong>Debug:</strong><br>'
      debugBindings.forEach(binding => {
        const key = binding.modifiers?.ctrl ? 'Ctrl+' + binding.key.toUpperCase() : binding.key.toUpperCase()
        html += `<span class="control-hint">${key} - ${binding.description}</span><br>`
      })
      html += '</div>'
    }

    html += '<div style="margin-top: 10px; font-size: 11px; opacity: 0.7;">🖱️ Drag to pan • Scroll to zoom</div>'

    controlsElement.innerHTML = html
  }
}