import { invoke } from '@tauri-apps/api/tauri';

type LogLevel = 'error' | 'warn' | 'info' | 'debug' | 'trace';

class Log {
    public log(level: LogLevel, message: string, data: unknown = null): void {
        try {
            await invoke('log', {
                level,
                message,
                context: data !== null ? JSON.stringify(data) : null,
            });
        } catch (error) {
            console.error('Logging failed:', error);
        }
    }

    public error(msg: string, data?: unknown): void {
        this.log('error', msg, data);
    }

    public warn(msg: Loggable, data?: unknown): void {
        this.log('warn', msg, data);
    }

    public info(msg: Loggable, data?: unknown): void {
        this.log('info', msg, data);
    }

    public debug(msg: Loggable, data?: unknown): void {
        this.log('debug', msg, data);
    }

    public trace(msg: Loggable, data?: unknown): void {
        this.log('trace', msg, data);
    }
}

export const logger = new Log();
