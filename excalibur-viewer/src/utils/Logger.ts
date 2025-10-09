/**
 * Simple logging utility for the Life Simulator viewer
 */
export const LOG_LEVEL = Object.freeze({
  DEBUG: 0,
  INFO: 1,
  WARN: 2,
  ERROR: 3,
} as const)

export type LogLevel = typeof LOG_LEVEL[keyof typeof LOG_LEVEL]

export class Logger {
  private context: string
  private static globalLevel: LogLevel = LOG_LEVEL.INFO

  constructor(context: string) {
    this.context = context
  }

  private shouldLog(level: LogLevel): boolean {
    return level >= Logger.globalLevel
  }

  private formatMessage(level: string, message: string): string {
    const timestamp = new Date().toISOString()
    return `[${timestamp}] [${level}] [${this.context}] ${message}`
  }

  debug(message: string, ...args: unknown[]): void {
    if (this.shouldLog(LOG_LEVEL.DEBUG)) {
      console.debug(this.formatMessage('DEBUG', message), ...args)
    }
  }

  info(message: string, ...args: unknown[]): void {
    if (this.shouldLog(LOG_LEVEL.INFO)) {
      console.info(this.formatMessage('INFO', message), ...args)
    }
  }

  warn(message: string, ...args: unknown[]): void {
    if (this.shouldLog(LOG_LEVEL.WARN)) {
      console.warn(this.formatMessage('WARN', message), ...args)
    }
  }

  error(message: string, ...args: unknown[]): void {
    if (this.shouldLog(LOG_LEVEL.ERROR)) {
      console.error(this.formatMessage('ERROR', message), ...args)
    }
  }

  static setGlobalLevel(level: LogLevel): void {
    Logger.globalLevel = level
  }
}