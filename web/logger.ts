export enum LogLevel {
    DEBUG = 0,
    INFO = 1,
    WARN = 2,
    ERROR = 3,
    NONE = 4,
}

export interface LoggerOptions {
    level: LogLevel;
    prefix?: string;
}

class Logger {
    private level: LogLevel = LogLevel.INFO;
    private prefix: string = '';

    constructor(options?: LoggerOptions) {
        if (options) {
            this.configure(options);
        }
    }

    public configure(options: LoggerOptions) {
        if (options.level !== undefined) {
            this.level = options.level;
        }
        if (options.prefix !== undefined) {
            this.prefix = options.prefix;
        }
    }

    private formatMessage(message: string): string {
        return this.prefix ? `[${this.prefix}] ${message}` : message;
    }

    public debug(message: string, ...args: unknown[]) {
        if (this.level <= LogLevel.DEBUG) {
            console.debug(this.formatMessage(message), ...args);
        }
    }

    public info(message: string, ...args: unknown[]) {
        if (this.level <= LogLevel.INFO) {
            console.info(this.formatMessage(message), ...args);
        }
    }

    public warn(message: string, ...args: unknown[]) {
        if (this.level <= LogLevel.WARN) {
            console.warn(this.formatMessage(message), ...args);
        }
    }

    public error(message: string, ...args: unknown[]) {
        if (this.level <= LogLevel.ERROR) {
            console.error(this.formatMessage(message), ...args);
        }
    }
}

// Global default logger instance
export const logger = new Logger({
    // In a real app this might be driven by an env var (e.g. process.env.NODE_ENV)
    // For now, we default to WARN to suppress debug/info logs in normal usage
    level: LogLevel.WARN,
    prefix: 'ASCII-Editor'
});
