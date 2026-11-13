/**
 * Round 5: errorReporter 单元测试
 * 测试: 去重、长度限制、未登录跳过、异常对象提取
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Mock api client
const mockPost = vi.fn().mockResolvedValue({});
const mockHasAuthenticatedSession = vi.fn().mockReturnValue(true);

vi.mock('@/integrations/api/client', () => ({
    api: {
        post: (...args: any[]) => mockPost(...args),
    },
    hasAuthenticatedSession: () => mockHasAuthenticatedSession(),
}));

// Import after mocking
import { reportError, reportErrorFromException } from './errorReporter';

describe('errorReporter', () => {
    beforeEach(() => {
        vi.clearAllMocks();
        mockHasAuthenticatedSession.mockReturnValue(true);
        // Reset dedup map — we need to clear module state between tests
        // The dedup window is 5000ms, so we fake time
        vi.useFakeTimers();
    });

    afterEach(() => {
        vi.useRealTimers();
    });

    describe('reportError', () => {
        it('should call api.post with correct error data', () => {
            reportError({
                error_type: 'render_crash',
                error_message: 'test error',
                stack_trace: 'at Test.tsx:1',
            });

            expect(mockPost).toHaveBeenCalledWith('/api/v1/error_logs', expect.objectContaining({
                error_type: 'render_crash',
                error_message: 'test error',
                stack_trace: 'at Test.tsx:1',
            }));
        });

        it('should skip Cloudflare beacon and SPA history noise', () => {
            reportError({
                error_type: 'unhandled_exception',
                error_message: "Cannot assign to read only property 'pushState'",
                stack_trace: 'at cloudflareinsights.com/beacon.min.js',
            });
            reportError({
                error_type: 'unhandled_exception',
                error_message: 'Blob is not a constructor',
            });
            expect(mockPost).not.toHaveBeenCalled();
        });

        it('should skip when no session (user not logged in)', () => {
            mockHasAuthenticatedSession.mockReturnValue(false);

            reportError({
                error_type: 'api_error',
                error_message: 'test error',
            });

            expect(mockPost).not.toHaveBeenCalled();
        });

        it('should deduplicate same error within 5 seconds', () => {
            reportError({ error_type: 'api_error', error_message: 'duplicate error' });
            reportError({ error_type: 'api_error', error_message: 'duplicate error' });
            reportError({ error_type: 'api_error', error_message: 'duplicate error' });

            // Only first call should go through
            expect(mockPost).toHaveBeenCalledTimes(1);
        });

        it('should allow same error after 5 seconds', () => {
            reportError({ error_type: 'api_error', error_message: 'timed error' });
            expect(mockPost).toHaveBeenCalledTimes(1);

            // Advance past dedup window
            vi.advanceTimersByTime(6000);

            reportError({ error_type: 'api_error', error_message: 'timed error' });
            expect(mockPost).toHaveBeenCalledTimes(2);
        });

        it('should truncate error_message to 2000 characters', () => {
            const longMessage = 'X'.repeat(3000);
            reportError({ error_type: 'render_crash', error_message: longMessage });

            const calledBody = mockPost.mock.calls[0][1];
            expect(calledBody.error_message.length).toBe(2000);
        });

        it('should truncate stack_trace to 5000 characters', () => {
            const longStack = 'Y'.repeat(8000);
            reportError({
                error_type: 'render_crash',
                error_message: 'truncation test',
                stack_trace: longStack,
            });

            // Advance time to avoid dedup
            vi.advanceTimersByTime(6000);

            const calledBody = mockPost.mock.calls[0][1];
            expect(calledBody.stack_trace.length).toBe(5000);
        });

        it('should include page_url and user_agent from window/navigator', () => {
            reportError({
                error_type: 'unhandled_exception',
                error_message: 'context test',
            });

            // Advance to avoid dedup
            vi.advanceTimersByTime(6000);

            const calledBody = mockPost.mock.calls[0][1];
            expect(calledBody.page_url).toBeDefined();
            expect(calledBody.user_agent).toBeDefined();
        });

        it('should serialize metadata to JSON string', () => {
            reportError({
                error_type: 'api_error',
                error_message: 'metadata test',
                metadata: { endpoint: '/api/v1/test', status: 500 },
            });

            // Advance to avoid dedup
            vi.advanceTimersByTime(6000);

            const calledBody = mockPost.mock.calls[0][1];
            expect(calledBody.metadata).toContain('endpoint');
            expect(calledBody.metadata).toContain('500');
        });

        it('should silently handle api.post failure (no throw)', () => {
            mockPost.mockRejectedValueOnce(new Error('network error'));

            // Should not throw
            expect(() => {
                reportError({ error_type: 'render_crash', error_message: 'fail gracefully' });
            }).not.toThrow();
        });
    });

    describe('reportErrorFromException', () => {
        it('should extract Error message and stack', () => {
            const err = new Error('Something went wrong');
            err.stack = 'Error: Something went wrong\n    at test.ts:1';

            reportErrorFromException(err, 'render_crash');

            expect(mockPost).toHaveBeenCalledWith('/api/v1/error_logs', expect.objectContaining({
                error_type: 'render_crash',
                error_message: 'Something went wrong',
                stack_trace: expect.stringContaining('at test.ts:1'),
            }));
        });

        it('should handle non-Error objects (string)', () => {
            // Advance to avoid dedup
            vi.advanceTimersByTime(6000);

            reportErrorFromException('plain string error', 'unhandled_exception');

            expect(mockPost).toHaveBeenCalledWith('/api/v1/error_logs', expect.objectContaining({
                error_message: 'plain string error',
            }));
        });

        it('should include componentStack from extra params', () => {
            // Advance to avoid dedup
            vi.advanceTimersByTime(6000);

            const err = new Error('render fail');
            reportErrorFromException(err, 'render_crash', {
                componentStack: '\n    at MyComponent\n    at Page',
            });

            const calledBody = mockPost.mock.calls[0][1];
            expect(calledBody.component_stack).toContain('MyComponent');
        });
    });
});
