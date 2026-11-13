/**
 * 前端错误自动上报服务
 * 
 * Fire-and-forget: 错误上报绝不影响用户体验
 * - ErrorBoundary.componentDidCatch → render_crash
 * - window.onerror → unhandled_exception
 * - window.onunhandledrejection → promise_rejection
 * - client.ts request() HTTP 错误 → api_error
 */

import { api, hasAuthenticatedSession } from '@/integrations/api/client';

export type ErrorType = 'render_crash' | 'api_error' | 'unhandled_exception' | 'promise_rejection';

interface ErrorReport {
    error_type: ErrorType;
    error_message: string;
    stack_trace?: string;
    component_stack?: string;
    page_url?: string;
    user_agent?: string;
    metadata?: string; // JSON string
}

// Dedup: 同一个 error_message 在 5 秒内不重复上报
const recentErrors = new Map<string, number>();
const DEDUP_INTERVAL_MS = 5000;

const THIRD_PARTY_NOISE = [
    /cloudflareinsights/i,
    /beacon\.min\.js/i,
    /static\.cloudflareinsights\.com/i,
    /cannot assign to read only property ['"]pushstate['"]/i,
    /blob is not a constructor/i,
    /^script error\.?$/i,
];

export function isThirdPartyNoise(message: string, stack?: string): boolean {
    const blob = `${message}\n${stack ?? ""}`;
    return THIRD_PARTY_NOISE.some((pattern) => pattern.test(blob));
}

function isDuplicate(msg: string): boolean {
    const now = Date.now();
    const lastTime = recentErrors.get(msg);
    if (lastTime && now - lastTime < DEDUP_INTERVAL_MS) {
        return true;
    }
    recentErrors.set(msg, now);
    // 清理旧记录，防止内存泄漏
    if (recentErrors.size > 100) {
        const cutoff = now - DEDUP_INTERVAL_MS;
        for (const [key, time] of recentErrors) {
            if (time < cutoff) recentErrors.delete(key);
        }
    }
    return false;
}

/**
 * 上报错误到后端数据库 (fire-and-forget)
 */
export function reportError(params: {
    error_type: ErrorType;
    error_message: string;
    stack_trace?: string;
    component_stack?: string;
    metadata?: Record<string, unknown>;
}): void {
    try {
        // 未登录状态不上报
        if (!hasAuthenticatedSession()) return;

        if (isThirdPartyNoise(params.error_message, params.stack_trace)) return;

        // 去重
        if (isDuplicate(params.error_message)) return;

        const report: ErrorReport = {
            error_type: params.error_type,
            error_message: params.error_message.substring(0, 2000), // 限制长度
            stack_trace: params.stack_trace?.substring(0, 5000),
            component_stack: params.component_stack?.substring(0, 3000),
            page_url: window.location.href,
            user_agent: navigator.userAgent,
            metadata: params.metadata ? JSON.stringify(params.metadata) : undefined,
        };

        // Fire-and-forget: 不 await, 不 catch 影响用户
        api.post('/api/v1/error_logs', report).catch(() => {
            // 上报失败静默处理，避免递归错误
        });
    } catch {
        // 上报逻辑本身的错误也静默处理
    }
}

/**
 * 从 Error 对象提取信息并上报
 */
export function reportErrorFromException(
    error: unknown,
    errorType: ErrorType,
    extra?: { componentStack?: string; metadata?: Record<string, unknown> }
): void {
    const err = error instanceof Error ? error : new Error(String(error));
    reportError({
        error_type: errorType,
        error_message: err.message || String(error),
        stack_trace: err.stack,
        component_stack: extra?.componentStack,
        metadata: {
            ...extra?.metadata,
            name: err.name,
        },
    });
}
