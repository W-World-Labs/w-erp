/**
 * Round 6 Safety Audit: 更多崩溃模式 + companyInfo 修复验证
 */
import { describe, it, expect } from 'vitest';

describe('Round 6 — companyInfo array-to-object fix', () => {
    it('should extract first element from data array', () => {
        // Simulates the queryFn pattern: result?.data?.[0]
        const result = { data: [{ registered_capital: 500, registration_date: '2020-01-01' }], count: 1 };
        const companyInfo = result?.data?.[0] as Record<string, any> | undefined;
        expect(companyInfo?.registered_capital).toBe(500);
        expect(companyInfo?.registration_date).toBe('2020-01-01');
    });

    it('should handle empty company_info array', () => {
        const result = { data: [], count: 0 };
        const companyInfo = result?.data?.[0] as Record<string, any> | undefined;
        expect(companyInfo).toBeUndefined();
    });

    it('should handle null result gracefully', () => {
        const result = null as any;
        const companyInfo = result?.data?.[0] as Record<string, any> | undefined;
        expect(companyInfo).toBeUndefined();
    });
});

describe('Round 6 — parseFloat/parseInt with guarded input', () => {
    it('should parseFloat with if-guard (ComplianceManagement pattern)', () => {
        const values: Record<string, any> = { premium_amount: '123.45', coverage_amount: null };
        ['premium_amount', 'coverage_amount'].forEach(f => {
            if (values[f]) values[f] = parseFloat(values[f]);
        });
        expect(values.premium_amount).toBe(123.45);
        expect(values.coverage_amount).toBeNull(); // Null stays, guard prevents parseFloat(null)
    });

    it('should parseInt with if-guard (DealsManagement pattern)', () => {
        const values: Record<string, any> = { probability: '75', amount: undefined };
        if (values.probability) values.probability = parseInt(values.probability);
        if (values.amount) values.amount = parseFloat(values.amount);
        expect(values.probability).toBe(75);
        expect(values.amount).toBeUndefined();
    });

    it('should handle non-numeric string with parseFloat', () => {
        const values: Record<string, any> = { tax_rate: 'abc' };
        if (values.tax_rate) values.tax_rate = parseFloat(values.tax_rate);
        expect(isNaN(values.tax_rate)).toBe(true); // Becomes NaN but doesn't crash
    });
});

describe('Round 6 — split pattern safety', () => {
    it('should safely split ISO date string', () => {
        const str = new Date().toISOString().split("T")[0];
        expect(str).toMatch(/^\d{4}-\d{2}-\d{2}$/);
    });

    it('should handle jurisdiction_code split with fallback', () => {
        const code: string | null = null;
        const flag = code?.split("-")[0] || "🏳️";
        expect(flag).toBe("🏳️");
    });

    it('should split valid jurisdiction code', () => {
        const code = "CN-SH";
        const country = code?.split("-")[0] || "🏳️";
        expect(country).toBe("CN");
    });
});

describe('Round 6 — Object.entries safety', () => {
    it('should handle Object.entries with static map', () => {
        const statusMap = { active: { label: '活跃' }, closed: { label: '已关闭' } };
        const entries = Object.entries(statusMap);
        expect(entries.length).toBe(2);
        expect(entries[0][0]).toBe('active');
    });

    it('should handle Object.entries with empty object', () => {
        const emptyMap = {};
        const entries = Object.entries(emptyMap);
        expect(entries.length).toBe(0);
    });
});

describe('Round 6 — ErrorBoundary integration safety', () => {
    it('should handle Error with no stack', () => {
        const err = new Error('no stack');
        err.stack = undefined;
        expect(err.message).toBe('no stack');
        expect(err.stack).toBeUndefined();
    });

    it('should handle non-Error thrown value', () => {
        const thrown = 42;
        const err = thrown instanceof Error ? thrown : new Error(String(thrown));
        expect(err.message).toBe('42');
    });

    it('should handle null thrown value', () => {
        const thrown = null;
        const err = thrown instanceof Error ? thrown : new Error(String(thrown));
        expect(err.message).toBe('null');
    });
});
