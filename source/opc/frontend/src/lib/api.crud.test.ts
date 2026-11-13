import { beforeEach, describe, expect, it, vi } from 'vitest';

const { apiMock, getCurrentUserMock } = vi.hoisted(() => ({
    apiMock: {
        get: vi.fn(),
        post: vi.fn(),
        put: vi.fn(),
        delete: vi.fn(),
    },
    getCurrentUserMock: vi.fn(),
}));

vi.mock('@/integrations/api/client', () => ({
    api: apiMock,
    auth: {
        getCurrentUser: getCurrentUserMock,
    },
}));

import {
    deleteData,
    fetchData,
    getCurrentUser,
    insertData,
    isAuthenticated,
    updateData,
} from './api';

describe('api crud helpers', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('wraps successful fetchData responses', async () => {
        apiMock.get.mockResolvedValueOnce({ data: [{ id: 'inv-1' }], count: 1 });

        const result = await fetchData<{ id: string }[]>('invoices', {
            filters: { status: 'open' },
            limit: 10,
        });

        expect(apiMock.get).toHaveBeenCalledWith('/api/invoices?status=open&limit=10');
        expect(result.success).toBe(true);
        expect(result.data).toEqual([{ id: 'inv-1' }]);
    });

    it('wraps fetchData transport failures in ApiResponse', async () => {
        apiMock.get.mockRejectedValueOnce(new Error('boom'));

        const result = await fetchData('invoices');

        expect(result.success).toBe(false);
        expect(result.error?.code).toBe('FETCH_ERROR');
        expect(result.error?.message).toBe('boom');
    });

    it('uses the first item for batch insert payloads', async () => {
        apiMock.post.mockResolvedValueOnce({ id: 'created-1' });

        const result = await insertData('invoices', [
            { invoice_number: 'INV-001' },
            { invoice_number: 'INV-002' },
        ]);

        expect(apiMock.post).toHaveBeenCalledWith('/api/invoices', { invoice_number: 'INV-001' });
        expect(result.success).toBe(true);
        expect(result.data).toEqual({ id: 'created-1' });
    });

    it('builds update and delete URLs from contract truth', async () => {
        apiMock.put.mockResolvedValueOnce({ id: 'updated-1' });
        apiMock.delete.mockResolvedValueOnce(undefined);

        const updateResult = await updateData('applications', 'app-1', { status: 'approved' });
        const deleteResult = await deleteData('applications', 'app-1');

        expect(apiMock.put).toHaveBeenCalledWith('/api/v1/applications/app-1', { status: 'approved' });
        expect(apiMock.delete).toHaveBeenCalledWith('/api/v1/applications/app-1');
        expect(updateResult.success).toBe(true);
        expect(deleteResult.success).toBe(true);
    });
});

describe('api auth helpers in crud test', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('returns null current user when session is missing', async () => {
        getCurrentUserMock.mockRejectedValueOnce(new Error('Unauthorized'));

        await expect(getCurrentUser()).resolves.toBeNull();
        expect(getCurrentUserMock).toHaveBeenCalledOnce();
    });

    it('uses session presence for authentication status', async () => {
        getCurrentUserMock.mockResolvedValueOnce({ id: 'user-1', email: 'a@example.com' });
        await expect(isAuthenticated()).resolves.toBe(true);

        getCurrentUserMock.mockRejectedValueOnce(new Error('Unauthorized'));
        await expect(isAuthenticated()).resolves.toBe(false);
    });
});
