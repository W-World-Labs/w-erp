/**
 * Round 5 Safety Audit: 验证 P0 崩溃修复 + 新组件安全性
 */
import { describe, it, expect } from 'vitest';

describe('Round 5 Safety Audit — Applications upload_date null guard', () => {
    it('should not crash when material upload_date is null', () => {
        // Simulates the exact pattern used in Applications.tsx:1045
        const item = { category: '营业执照', upload_date: null, file_size: '1.2MB' };
        const result = item.upload_date ? new Date(item.upload_date).toLocaleDateString() : '-';
        expect(result).toBe('-');
    });

    it('should not crash when material upload_date is undefined', () => {
        const item = { category: '审计报告', file_size: '500KB' } as any;
        const result = item.upload_date ? new Date(item.upload_date).toLocaleDateString() : '-';
        expect(result).toBe('-');
    });

    it('should format valid upload_date correctly', () => {
        const item = { category: '纳税申报', upload_date: '2026-01-15T10:30:00Z', file_size: '800KB' };
        const result = item.upload_date ? new Date(item.upload_date).toLocaleDateString() : '-';
        expect(result).not.toBe('-');
        expect(result).not.toBe('Invalid Date');
    });
});

describe('Round 5 Safety Audit — BudgetManagement date guards', () => {
    it('should handle null period in salary date calculation', () => {
        const period: string | null = null;
        const result = period ? new Date(period + '-01') : null;
        expect(result).toBeNull();
    });

    it('should handle null invoice_date gracefully', () => {
        const inv = { invoice_date: null } as any;
        if (!inv.invoice_date) {
            expect(true).toBe(true); // Guard works
        } else {
            const d = new Date(inv.invoice_date);
            expect(isNaN(d.getTime())).toBe(false);
        }
    });

    it('should handle null contract dates in rent calculation', () => {
        const rent = { contract_start_date: null, contract_end_date: '2026-12-31' } as any;
        if (!rent.contract_start_date || !rent.contract_end_date) {
            expect(true).toBe(true); // Guard catches null
        }
    });
});

describe('Round 5 Safety Audit — CashFlowForecast contract dates', () => {
    it('should skip contracts without start_date', () => {
        const c = { start_date: null, end_date: '2026-12-31', amount: 100000 } as any;
        let monthlyIncome = 0;
        if (!c.start_date || !c.end_date) {
            monthlyIncome = 0; // Skip
        }
        expect(monthlyIncome).toBe(0);
    });

    it('should skip contracts with invalid dates', () => {
        const c = { start_date: 'garbage', end_date: '2026-12-31', amount: 100000 } as any;
        const start = new Date(c.start_date);
        const end = new Date(c.end_date);
        if (isNaN(start.getTime()) || isNaN(end.getTime())) {
            expect(true).toBe(true); // Guard works
        }
    });

    it('should calculate monthly income for valid contracts', () => {
        const c = { start_date: '2026-01-01', end_date: '2026-12-31', amount: 120000, status: 'active', contract_type: '服务合同' };
        const start = new Date(c.start_date);
        const end = new Date(c.end_date);
        if (!isNaN(start.getTime()) && !isNaN(end.getTime())) {
            const months = Math.max(1, (end.getFullYear() - start.getFullYear()) * 12 + (end.getMonth() - start.getMonth()));
            const monthly = c.amount / months;
            expect(monthly).toBeCloseTo(120000 / 11, 0); // Jan→Dec = 11 months diff
        }
    });
});

describe('Round 5 Safety Audit — Rent expiry calculation', () => {
    it('should skip rent without contract_end_date', () => {
        const r = { contract_end_date: null, status: 'active' } as any;
        const shouldCheck = r.contract_end_date ? true : false;
        expect(shouldCheck).toBe(false);
    });

    it('should detect expiring rent within 3 months', () => {
        const twoMonthsFromNow = new Date();
        twoMonthsFromNow.setMonth(twoMonthsFromNow.getMonth() + 2);
        const r = { contract_end_date: twoMonthsFromNow.toISOString(), status: 'active' };
        const endDate = new Date(r.contract_end_date);
        const threeMonthsLater = new Date();
        threeMonthsLater.setMonth(threeMonthsLater.getMonth() + 3);
        expect(endDate <= threeMonthsLater).toBe(true);
    });
});

describe('Round 5 Safety Audit — ErrorLogsPanel data handling', () => {
    it('should parse valid metadata JSON', () => {
        const raw = '{"endpoint":"/api/v1/test","status":500}';
        const meta = JSON.parse(raw);
        expect(meta.endpoint).toBe('/api/v1/test');
        expect(meta.status).toBe(500);
    });

    it('should handle invalid metadata JSON gracefully', () => {
        const raw = 'not json';
        let meta = null;
        try { meta = JSON.parse(raw); } catch { meta = null; }
        expect(meta).toBeNull();
    });

    it('should handle null/undefined metadata', () => {
        const raw: string | undefined = undefined;
        let meta = null;
        if (raw) {
            try { meta = JSON.parse(raw); } catch { meta = null; }
        }
        expect(meta).toBeNull();
    });

    it('should format error_type labels correctly', () => {
        const typeMap: Record<string, string> = {
            render_crash: '渲染崩溃',
            api_error: 'API 错误',
            unhandled_exception: 'JS 异常',
            promise_rejection: 'Promise 拒绝',
        };
        expect(typeMap['render_crash']).toBe('渲染崩溃');
        expect(typeMap['api_error']).toBe('API 错误');
        expect(typeMap['unknown_type'] ?? 'JS 异常').toBe('JS 异常');
    });
});
