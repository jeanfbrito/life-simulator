import { LifeSimulatorEngine } from '@/engine/LifeSimulatorEngine'
import { Logger } from '@/utils/Logger'

/**
 * MVP Main entry point - simplified to get something working quickly
 */
async function main(): Promise<void> {
  const logger = new Logger('Main')

  try {
    logger.info('Starting Life Simulator MVP...')

    // Check URL parameters for debug mode
    const urlParams = new URLSearchParams(window.location.search)
    const enableDebug = urlParams.has('debug') || urlParams.has('d')
    const enableHiDPI = !urlParams.has('no-hidpi')

    // Initialize and start the engine
    const engine = new LifeSimulatorEngine({
      enableDebug,
      logLevel: enableDebug ? 'debug' : 'info',
      enableHiDPI,
      targetFPS: 60,
    })

    await engine.initialize()

    // Hide loading screen and show HUD after initialization
    const loadingElement = document.getElementById('loading')
    if (loadingElement) {
      loadingElement.style.display = 'none'
    }

    const hudElement = document.getElementById('hud')
    if (hudElement) {
      hudElement.style.display = 'block'
    }

    await engine.start()

    logger.info('Life Simulator MVP started successfully!')

    // Handle window resize
    window.addEventListener('resize', () => {
      engine.handleResize()
    })

    // Handle page visibility changes
    document.addEventListener('visibilitychange', () => {
      if (document.hidden) {
        engine.pause()
        logger.debug('Application paused (page hidden)')
      } else {
        engine.resume()
        logger.debug('Application resumed (page visible)')
      }
    })

    // Graceful shutdown
    window.addEventListener('beforeunload', () => {
      logger.info('Shutting down Life Simulator...')
      engine.shutdown()
    })

    // Global error handling
    window.addEventListener('error', (event) => {
      logger.error('Global error:', event.error)
    })

    window.addEventListener('unhandledrejection', (event) => {
      logger.error('Unhandled promise rejection:', event.reason)
    })

    // Log initial performance metrics
    setTimeout(() => {
      const metrics = engine.getPerformanceMetrics()
      logger.info(`Initial performance: FPS=${metrics.fps}, Frame Time=${metrics.frameTime.toFixed(2)}ms`)

      // Log chunk controller info if available
      const chunkController = engine.getSystem('ChunkController')
      if (chunkController) {
        logger.info(`Chunks loaded: ${(chunkController as any).getLoadedChunkCount()}`)
      }
    }, 2000)

  } catch (error) {
    logger.error('Failed to start Life Simulator MVP:', error)

    // Show error message
    const loadingElement = document.getElementById('loading')
    if (loadingElement) {
      loadingElement.innerHTML = `
        <div class="loading-content">
          <div style="color: #ff6b6b; margin-bottom: 20px;">❌ Failed to start</div>
          <div style="margin-bottom: 10px;">Could not start the Life Simulator viewer</div>
          <div style="font-size: 12px; opacity: 0.7;">${error instanceof Error ? error.message : 'Unknown error'}</div>
          <div style="font-size: 12px; margin-top: 10px; opacity: 0.7;">Check console for details</div>
        </div>
      `
      loadingElement.style.display = 'flex'
    }
  }
}

// Start the application when the DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', main)
} else {
  main()
}