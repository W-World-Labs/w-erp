/**
 * Date Safety Audit Tests (日期安全审计测试)
 *
 * Round 3: 验证 new Date(apiField) 在 null/undefined/Invalid 时的行为安全。
 * 覆盖 Dashboard, KPIDashboard, Rent, CashFlowForecast, BudgetManagement 页面的修复。
 */
import { describe, it, expect } from 'vitest';
import { isValid, format } from 'date-fns';

// ==================== new Date() with null/undefined API data ====================

describe('Date construction from API data', () => {
    it('new Date(null) produces 1970-01-01 (epoch), not crash', () => {
        const d = new Date(null as any);
        expect(d.getTime()).toBe(0);
        expect(isValid(d)).toBe(true); // epoch IS valid
    });

    it('new Date(undefined) produces Invalid Date', () => {
        const d = new Date(undefined as any);
        expect(isValid(d)).toBe(false);
    });

    it('new Date("") produces Invalid Date', () => {
        const d = new Date('');
        expect(isValid(d)).toBe(false);
    });

    it('new Date("bad-date") produces Invalid Date', () => {
        const d = new Date('bad-date');
        expect(isValid(d)).toBe(false);
    });

    it('Invalid Date comparison returns false (silent filter skip)', () => {
        const invalid = new Date('bad');
        const valid = new Date('2024-01-01');
        expect(invalid >= valid).toBe(false);
        expect(invalid <= valid).toBe(false);
        expect(invalid > valid).toBe(false);
        expect(invalid < valid).toBe(false);
    });
});

// ==================== format() crash risk with Invalid Date ====================

describe('format() crash with Invalid Date (RangeError)', () => {
    it('format(invalidDate) should throw RangeError', () => {
        const invalid = new Date('bad');
        expect(() => format(invalid, 'yyyy-MM')).toThrow();
    });

    it('isValid guard prevents format crash', () => {
        const invalid = new Date('bad');
        // This is the pattern we use in KPIDashboard
        const result = isValid(invalid) ? format(invalid, 'yyyy-MM') : null;
        expect(result).toBeNull();
    });

    it('null date check prevents format crash', () => {
        const dateStr: string | null = null;
        // This is the pattern: if (!dateStr) return false
        const shouldProcess = !!dateStr;
        expect(shouldProcess).toBe(false);
    });
});

// ==================== Dashboard: invoice date filter ====================

describe('Dashboard: invoice date filter with null invoice_date', () => {
    it('should skip invoices with null invoice_date', () => {
        const invoices = [
            { invoice_date: '2024-03-15', amount: 1000 },
            { invoice_date: null, amount: 500 },
            { invoice_date: undefined, amount: 200 },
            { invoice_date: '', amount: 300 },
        ];

        const monthStart = new Date('2024-03-01');
        const monthEnd = new Date('2024-03-31');

        const filtered = invoices.filter(invoice => {
            if (!invoice.invoice_date) return false;
            const invoiceDate = new Date(invoice.invoice_date);
            return invoiceDate >= monthStart && invoiceDate <= monthEnd;
        });

        expect(filtered).toHaveLength(1);
        expect(filtered[0].amount).toBe(1000);
    });
});

// ==================== KPIDashboard: format(invDate) safe ====================

describe('KPIDashboard: format with isValid guard', () => {
    it('should match current period for valid date', () => {
        const invDate = new Date('2024-03-15');
        expect(isValid(invDate)).toBe(true);
        expect(format(invDate, 'yyyy-MM')).toBe('2024-03');
    });

    it('should not match null invoice_date (early return)', () => {
        const inv = { invoice_date: null };
        // Pattern: if (!inv.invoice_date) return false
        const shouldProcess = !!inv.invoice_date;
        expect(shouldProcess).toBe(false);
    });
});

// ==================== Rent: contract_end_date null guard ====================

describe('Rent: expiring filter with null contract_end_date', () => {
    it('should skip rents without contract_end_date', () => {
        const rents = [
            { contract_end_date: '2024-06-01', status: 'active' },
            { contract_end_date: null, status: 'active' },
            { contract_end_date: undefined, status: 'active' },
        ];

        const threeMonthsLater = new Date('2024-09-01');

        const expiring = rents.filter(r => {
            if (!r.contract_end_date) return false;
            const endDate = new Date(r.contract_end_date);
            return r.status === 'active' && endDate <= threeMonthsLater;
        });

        expect(expiring).toHaveLength(1);
    });
});

// ==================== CashFlowForecast: contract date guard ====================

describe('CashFlowForecast: contract income with null dates', () => {
    it('should skip contracts with null start/end dates', () => {
        const contracts = [
            { start_date: '2024-01-01', end_date: '2024-12-31', amount: 120000 },
            { start_date: null, end_date: '2024-12-31', amount: 50000 },
            { start_date: '2024-01-01', end_date: null, amount: 30000 },
        ];

        const monthlyIncome = contracts.reduce((sum, c) => {
            if (!c.start_date || !c.end_date) return sum;
            const start = new Date(c.start_date);
            const end = new Date(c.end_date);
            if (isNaN(start.getTime()) || isNaN(end.getTime())) return sum;
            const months = Math.max(1, (end.getFullYear() - start.getFullYear()) * 12 + (end.getMonth() - start.getMonth()));
            return sum + (Number(c.amount) || 0) / months;
        }, 0);

        // Only first contract: 120000 / 11 months ≈ 10909
        expect(monthlyIncome).toBeGreaterThan(10000);
        expect(monthlyIncome).toBeLessThan(12000);
    });
});

// ==================== BudgetManagement: period null guard ====================

describe('BudgetManagement: salary/invoice date filter with null period', () => {
    it('should skip salary entries with null period', () => {
        const salaries = [
            { period: '2024-03', base_salary: 10000, bonus: 2000 },
            { period: null, base_salary: 5000, bonus: 0 },
            { period: undefined, base_salary: 3000, bonus: 0 },
        ];

        const start = new Date('2024-03-01');
        const end = new Date('2024-03-31');

        const total = salaries.filter(s => {
            if (!s.period) return false;
            const salaryDate = new Date(s.period + '-01');
            return !isNaN(salaryDate.getTime()) && salaryDate >= start && salaryDate <= end;
        }).reduce((sum, s) => sum + (Number(s.base_salary) || 0) + (Number(s.bonus) || 0), 0);

        expect(total).toBe(12000); // only first entry
    });
});

// ==================== GlobalOpcHub: reduce() empty array ====================

describe('GlobalOpcHub: reduce on empty calcResults', () => {
    it('should return null when calcResults is empty', () => {
        const calcResults: any[] = [];
        const getBestHub = () => {
            if (!calcResults?.length) return null;
            return calcResults.reduce((best, r) => r.net_take_home > best.net_take_home ? r : best, calcResults[0]);
        };
        expect(getBestHub()).toBeNull();
    });

    it('should return null when calcResults is null', () => {
        const calcResults = null;
        const getBestHub = () => {
            if (!calcResults?.length) return null;
            return (calcResults as any[]).reduce((best: any, r: any) => r.net_take_home > best.net_take_home ? r : best, (calcResults as any[])[0]);
        };
        expect(getBestHub()).toBeNull();
    });

    it('should find best hub in non-empty array', () => {
        const calcResults = [
            { city: 'Dubai', net_take_home: 50000 },
            { city: 'Singapore', net_take_home: 45000 },
            { city: 'Tallinn', net_take_home: 48000 },
        ];
        const getBestHub = () => {
            if (!calcResults?.length) return null;
            return calcResults.reduce((best, r) => r.net_take_home > best.net_take_home ? r : best, calcResults[0]);
        };
        expect(getBestHub()?.city).toBe('Dubai');
    });
});

// ==================== isNaN guard pattern ====================

describe('isNaN guard pattern', () => {
    it('isNaN(new Date(null).getTime()) should be false (epoch is valid)', () => {
        expect(isNaN(new Date(null as any).getTime())).toBe(false);
    });

    it('isNaN(new Date(undefined).getTime()) should be true', () => {
        expect(isNaN(new Date(undefined as any).getTime())).toBe(true);
    });

    it('isNaN(new Date("").getTime()) should be true', () => {
        expect(isNaN(new Date('').getTime())).toBe(true);
    });

    it('combined null + isNaN check catches all bad dates', () => {
        const badDates = [null, undefined, '', 'bad-date', 'NaN'];
        for (const d of badDates) {
            const result = !d || isNaN(new Date(d).getTime());
            expect(result).toBe(true);
        }
    });
});
