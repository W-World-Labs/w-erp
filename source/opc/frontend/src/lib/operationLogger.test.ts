import { describe, it, expect, vi, beforeEach } from 'vitest';

// Mock the api + auth modules
vi.mock('@/integrations/api/client', () => ({
    api: {
        post: vi.fn().mockResolvedValue({}),
    },
    auth: {
        isAuthenticated: vi.fn().mockReturnValue(true),
        getCurrentUser: vi.fn().mockResolvedValue({ id: 'user-1', email: 'test@test-com.example.invalid' }),
    },
}));

import { logOperation, logSensitiveOperation } from './operationLogger';
import { api, auth } from '@/integrations/api/client';

describe('logOperation', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should POST to /api/logs with correct payload', async () => {
        await logOperation({
            operationType: 'CREATE',
            tableName: 'invoices',
            recordId: 'inv-001',
            operationDetail: { amount: 1000 },
        });

        expect(api.post).toHaveBeenCalledWith('/api/logs', expect.objectContaining({
            user_id: 'user-1',
            operation_type: 'CREATE',
            table_name: 'invoices',
            record_id: 'inv-001',
            operation_detail: { amount: 1000 },
        }));
    });

    it('should not POST if user is not authenticated', async () => {
        (auth.isAuthenticated as ReturnType<typeof vi.fn>).mockReturnValueOnce(false);

        await logOperation({
            operationType: 'DELETE',
            tableName: 'employees',
        });

        expect(api.post).not.toHaveBeenCalled();
    });

    it('should not POST if getCurrentUser returns null', async () => {
        (auth.getCurrentUser as ReturnType<typeof vi.fn>).mockResolvedValueOnce(null);

        await logOperation({
            operationType: 'UPDATE',
            tableName: 'contracts',
        });

        expect(api.post).not.toHaveBeenCalled();
    });

    it('should silently fail without throwing', async () => {
        (api.post as ReturnType<typeof vi.fn>).mockRejectedValueOnce(new Error('Network error'));

        // Should not throw
        await expect(logOperation({
            operationType: 'CREATE',
            tableName: 'invoices',
        })).resolves.not.toThrow();
    });
});

describe('logSensitiveOperation', () => {
    beforeEach(() => {
        vi.clearAllMocks();
    });

    it('should log operations on sensitive tables', async () => {
        await logSensitiveOperation('CREATE', 'employees', 'emp-001');
        expect(api.post).toHaveBeenCalled();
    });

    it('should log operations on invoices table', async () => {
        await logSensitiveOperation('UPDATE', 'invoices', 'inv-001', { amount: 500 });
        expect(api.post).toHaveBeenCalledWith('/api/logs', expect.objectContaining({
            table_name: 'invoices',
            operation_type: 'UPDATE',
        }));
    });

    it('should log operations on contracts table', async () => {
        await logSensitiveOperation('DELETE', 'contracts', 'c-001');
        expect(api.post).toHaveBeenCalled();
    });

    it('should NOT log operations on non-sensitive tables', async () => {
        await logSensitiveOperation('CREATE', 'reminders', 'rem-001');
        expect(api.post).not.toHaveBeenCalled();
    });

    it('should NOT log operations on assets table (not in sensitive list)', async () => {
        await logSensitiveOperation('UPDATE', 'assets', 'a-001');
        expect(api.post).not.toHaveBeenCalled();
    });

    it('should log all sensitive table types', async () => {
        const sensitiveTables = [
            'employees', 'salaries', 'contracts',
            'invoices', 'payments', 'company_info',
            'user_roles', 'tax_declarations', 'budgets'
        ];

        for (const table of sensitiveTables) {
            vi.clearAllMocks();
            await logSensitiveOperation('VIEW', table);
            expect(api.post).toHaveBeenCalled();
        }
    });
});
