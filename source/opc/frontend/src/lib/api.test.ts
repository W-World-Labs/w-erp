import { beforeEach, describe, expect, it, vi } from 'vitest';

const { mockApiGet, mockApiPost, mockApiPut, mockApiDelete, mockGetCurrentUser } = vi.hoisted(() => ({
    mockApiGet: vi.fn(),
    mockApiPost: vi.fn(),
    mockApiPut: vi.fn(),
    mockApiDelete: vi.fn(),
    mockGetCurrentUser: vi.fn(),
}));

vi.mock('@/integrations/api/client', () => ({
    api: {
        get: mockApiGet,
        post: mockApiPost,
        put: mockApiPut,
        delete: mockApiDelete,
    },
    auth: {
        getCurrentUser: mockGetCurrentUser,
    },
}));

import {
    ApiError,
    API_ENDPOINTS,
    PAGE_ENDPOINT_CONTRACTS,
    getEndpoint,
    getErrorMessage,
    getCurrentUser,
    isAuthenticated,
} from './api';

beforeEach(() => {
    mockApiGet.mockReset();
    mockApiPost.mockReset();
    mockApiPut.mockReset();
    mockApiDelete.mockReset();
    mockGetCurrentUser.mockReset();
});

describe('ApiError', () => {
    it('should create error with message and code', () => {
        const error = new ApiError('Test error', 'TEST_CODE');

        expect(error.message).toBe('Test error');
        expect(error.code).toBe('TEST_CODE');
        expect(error.name).toBe('ApiError');
    });

    it('should create error with details and hint', () => {
        const error = new ApiError('Error', 'CODE', 'Details here', 'Hint here');

        expect(error.details).toBe('Details here');
        expect(error.hint).toBe('Hint here');
    });

    it('should create error with default code', () => {
        const error = new ApiError('Test error');

        expect(error.code).toBe('UNKNOWN_ERROR');
    });

    it('should be instance of Error', () => {
        const error = new ApiError('Test', 'CODE');

        expect(error).toBeInstanceOf(Error);
        expect(error).toBeInstanceOf(ApiError);
    });
});

describe('getErrorMessage', () => {
    it('should return friendly message for UNAUTHORIZED', () => {
        const error = new ApiError('Auth error', 'UNAUTHORIZED');
        const message = getErrorMessage(error);

        expect(message).toBe('未授权访问，请重新登录');
    });

    it('should return friendly message for NOT_FOUND', () => {
        const error = new ApiError('Not found', 'NOT_FOUND');
        const message = getErrorMessage(error);

        expect(message).toBe('数据不存在');
    });

    it('should return friendly message for NETWORK_ERROR', () => {
        const error = new ApiError('Network fail', 'NETWORK_ERROR');
        const message = getErrorMessage(error);

        expect(message).toBe('网络连接失败，请检查网络');
    });

    it('should return friendly message for CONFLICT', () => {
        const error = new ApiError('Conflict', 'CONFLICT');
        const message = getErrorMessage(error);

        expect(message).toBe('记录已存在，请检查是否重复');
    });

    it('should return original message for unknown error codes', () => {
        const error = new ApiError('Custom error', 'CUSTOM_CODE');
        const message = getErrorMessage(error);

        expect(message).toBe('Custom error');
    });

    it('should handle regular Error', () => {
        const error = new Error('Regular error');
        const message = getErrorMessage(error);

        expect(message).toBe('Regular error');
    });

    it('should handle unknown error types', () => {
        const message = getErrorMessage('string error');
        expect(message).toBe('发生未知错误，请稍后重试');
    });

    it('should handle null/undefined', () => {
        expect(getErrorMessage(null)).toBe('发生未知错误，请稍后重试');
        expect(getErrorMessage(undefined)).toBe('发生未知错误，请稍后重试');
    });
});

describe('auth helpers', () => {
    it('returns null when there is no active session', async () => {
        mockGetCurrentUser.mockRejectedValue(new Error('Unauthorized'));

        await expect(getCurrentUser()).resolves.toBeNull();
        expect(mockGetCurrentUser).toHaveBeenCalledOnce();
    });

    it('maps the authenticated user shape from the auth client', async () => {
        mockGetCurrentUser.mockResolvedValue({
            id: 'user-1',
            email: 'person@example.com',
            name: 'One Man',
            avatar_url: 'https://example.com/avatar.png',
        });

        await expect(getCurrentUser()).resolves.toEqual({
            id: 'user-1',
            email: 'person@example.com',
            user_metadata: {
                name: 'One Man',
                avatar_url: 'https://example.com/avatar.png',
            },
        });
    });

    it('reports authentication state from the current session', async () => {
        mockGetCurrentUser.mockResolvedValueOnce({ id: 'user-1', email: 'a@example.com' });
        mockGetCurrentUser.mockRejectedValueOnce(new Error('Unauthorized'));

        await expect(isAuthenticated()).resolves.toBe(true);
        await expect(isAuthenticated()).resolves.toBe(false);
    });
});

describe('API contracts', () => {
    it('maps known tables to explicit endpoints', () => {
        expect(getEndpoint('invoices')).toBe(API_ENDPOINTS.invoices);
        expect(getEndpoint('policies')).toBe(API_ENDPOINTS.policies);
    });

    it('falls back to /api/v1 for unknown tables', () => {
        expect(getEndpoint('custom_table')).toBe('/api/v1/custom_table');
    });

    it('keeps page endpoint contracts aligned with endpoint constants', () => {
        expect(PAGE_ENDPOINT_CONTRACTS.CompanyInfo).toContain(API_ENDPOINTS.companyInfo);
        expect(PAGE_ENDPOINT_CONTRACTS.ReconciliationCenter).toContain(API_ENDPOINTS.bankTransactionsLegacy);
    });
});
