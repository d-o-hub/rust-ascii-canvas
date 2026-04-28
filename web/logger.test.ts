import { describe, it, expect, vi, beforeEach } from 'vitest';
import { logger, LogLevel } from './logger';

describe('Logger', () => {
    beforeEach(() => {
        vi.spyOn(console, 'debug').mockImplementation(() => {});
        vi.spyOn(console, 'info').mockImplementation(() => {});
        vi.spyOn(console, 'warn').mockImplementation(() => {});
        vi.spyOn(console, 'error').mockImplementation(() => {});
    });

    it('should respect log levels', () => {
        logger.configure({ level: LogLevel.WARN, prefix: '' });

        logger.debug('test debug');
        logger.info('test info');
        logger.warn('test warn');
        logger.error('test error');

        expect(console.debug).not.toHaveBeenCalled();
        expect(console.info).not.toHaveBeenCalled();
        expect(console.warn).toHaveBeenCalledWith('test warn');
        expect(console.error).toHaveBeenCalledWith('test error');
    });

    it('should format message with prefix', () => {
        logger.configure({ level: LogLevel.DEBUG, prefix: 'TEST' });
        logger.info('message');
        expect(console.info).toHaveBeenCalledWith('[TEST] message');
    });
});
