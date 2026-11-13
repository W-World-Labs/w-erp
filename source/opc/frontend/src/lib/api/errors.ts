import { ApiError as WApiError } from '@w/http-web';

/**
 * OPC's data-layer error wrapper. Now a thin subclass of the shared
 * `@w/http-web` `ApiError` superset (which already carries `code`/`details`/
 * `hint`), preserving OPC's positional `(message, code, details, hint)`
 * constructor so every existing call site (`crud.ts`, tests) is byte-identical.
 */
export class ApiError extends WApiError {
    constructor(message: string, code: string = 'UNKNOWN_ERROR', details?: string, hint?: string) {
        super({ message, code, details, hint });
        this.name = 'ApiError';
    }
}

export interface ApiResponse<T> {
    data: T | null;
    error: ApiError | null;
    success: boolean;
}

const errorMessages: Record<string, string> = {
    UNAUTHORIZED: '未授权访问，请重新登录',
    NOT_FOUND: '数据不存在',
    NETWORK_ERROR: '网络连接失败，请检查网络',
    TIMEOUT: '请求超时，请稍后重试',
    CONFLICT: '记录已存在，请检查是否重复',
};

export function getErrorMessage(error: ApiError | Error | unknown): string {
    if (error instanceof ApiError) {
        return errorMessages[error.code] || error.message;
    }
    if (error instanceof Error) {
        return error.message;
    }
    return '发生未知错误，请稍后重试';
}
