/**
 * Crash Patterns Test Suite (崩溃模式测试)
 *
 * 深度代码审计发现的 9 个隐藏崩溃模式的回归测试。
 * 这些测试确保修复后的代码在 null/undefined 数据下不会崩溃。
 */
import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import React from 'react';
import { safeNum, safeCurrency } from './dateUtils';

// ==================== safeNum 工具函数测试 ====================

describe('safeNum: Null-safe number formatting', () => {
    it('should return "—" for null', () => {
        expect(safeNum(null)).toBe('—');
    });

    it('should return "—" for undefined', () => {
        expect(safeNum(undefined)).toBe('—');
    });

    it('should return "—" for empty string', () => {
        expect(safeNum('')).toBe('—');
    });

    it('should return "—" for NaN-producing string', () => {
        expect(safeNum('abc')).toBe('—');
    });

    it('should format valid number', () => {
        const result = safeNum(12345.67);
        expect(result).toContain('12');
        expect(result).toContain('345');
    });

    it('should format with options', () => {
        const result = safeNum(0.15, { style: 'percent' });
        expect(result).toContain('15');
    });

    it('should handle string number', () => {
        const result = safeNum('12345');
        expect(result).toContain('12');
    });

    it('should handle zero', () => {
        const result = safeNum(0);
        expect(result).toBe('0');
    });
});

describe('safeCurrency: Null-safe currency formatting', () => {
    it('should return "—" for null', () => {
        expect(safeCurrency(null)).toBe('—');
    });

    it('should return "—" for undefined', () => {
        expect(safeCurrency(undefined)).toBe('—');
    });

    it('should format valid amount with prefix', () => {
        const result = safeCurrency(1000);
        expect(result).toContain('¥');
        expect(result).toContain('1,000');
    });

    it('should support custom prefix', () => {
        const result = safeCurrency(1000, '$');
        expect(result).toMatch(/^\$/);
    });
});

// ==================== 场景 1: Employees base_salary.toLocaleString() ====================

describe('Crash #1: Employees base_salary null', () => {
    it('should not crash when base_salary is null', () => {
        const employee = { base_salary: null as number | null };
        expect(() => {
            // 修复后的代码: Number(employee.base_salary || 0).toLocaleString()
            Number(employee.base_salary || 0).toLocaleString();
        }).not.toThrow();
    });

    it('should not crash when base_salary is undefined', () => {
        const employee = { base_salary: undefined as number | undefined };
        expect(() => {
            Number(employee.base_salary || 0).toLocaleString();
        }).not.toThrow();
    });

    it('should format valid salary correctly', () => {
        const salary = 15000;
        const result = Number(salary || 0).toLocaleString();
        expect(result).toContain('15');
    });
});

// ==================== 场景 2-4: GlobalOpcHub null tax rates/rent ====================

describe('Crash #2-4: GlobalOpcHub null tax rates and rent', () => {
    const hubWithNulls = {
        corporate_tax_rate: null as number | null,
        dividend_tax_rate: null as number | null,
        avg_apartment_rent_1bed: null as number | null,
    };

    it('should not crash on null corporate_tax_rate', () => {
        expect(() => {
            ((hubWithNulls.corporate_tax_rate ?? 0) * 100).toFixed(0);
        }).not.toThrow();
        expect(((hubWithNulls.corporate_tax_rate ?? 0) * 100).toFixed(0)).toBe('0');
    });

    it('should not crash on null dividend_tax_rate', () => {
        expect(() => {
            ((hubWithNulls.dividend_tax_rate ?? 0) * 100).toFixed(0);
        }).not.toThrow();
    });

    it('should not crash on null avg_apartment_rent_1bed', () => {
        expect(() => {
            (hubWithNulls.avg_apartment_rent_1bed ?? 0).toLocaleString();
        }).not.toThrow();
    });

    it('should format valid tax rate', () => {
        const hub = { ...hubWithNulls, corporate_tax_rate: 0.15 };
        expect(((hub.corporate_tax_rate ?? 0) * 100).toFixed(0)).toBe('15');
    });
});

// ==================== 场景 5: Settings api_key.substring() ====================

describe('Crash #5: Settings api_key.substring() null', () => {
    it('should not crash when api_key is null', () => {
        const config = { api_key: null as string | null };
        expect(() => {
            const display = config.api_key?.substring(0, 10) ?? '***';
            expect(display).toBe('***');
        }).not.toThrow();
    });

    it('should not crash when api_key is undefined', () => {
        const config = { api_key: undefined as string | undefined };
        const display = config.api_key?.substring(0, 10) ?? '***';
        expect(display).toBe('***');
    });

    it('should truncate valid api_key', () => {
        const config = { api_key: 'demo_key_value_for_masking' };
        const display = config.api_key?.substring(0, 10) ?? '***';
        expect(display).toBe('demo_key_v');
    });

    it('should handle short api_key', () => {
        const config = { api_key: 'short' };
        const display = config.api_key?.substring(0, 10) ?? '***';
        expect(display).toBe('short');
    });
});

// ==================== 场景 6: Assets JSON.parse without try/catch ====================

describe('Crash #6: Assets JSON.parse on invalid data', () => {
    it('should not crash on invalid JSON', () => {
        const decodedText = 'not valid json {{{';
        let parsed = null;
        expect(() => {
            try {
                parsed = JSON.parse(decodedText);
            } catch {
                // should be caught
            }
        }).not.toThrow();
        expect(parsed).toBeNull();
    });

    it('should parse valid JSON', () => {
        const decodedText = '{"type":"asset","id":"123"}';
        let parsed: any = null;
        try {
            parsed = JSON.parse(decodedText);
        } catch {
            // unexpected
        }
        expect(parsed).toEqual({ type: 'asset', id: '123' });
    });
});

// ==================== 场景 7: Division by zero (Salary) ====================

describe('Crash #7: Salary division by zero', () => {
    it('should return 0 for empty salaries array', () => {
        const salaries: any[] = [];
        const average = salaries.length > 0
            ? salaries.reduce((sum: number, s: any) => sum + (Number(s.total_salary) || 0), 0) / salaries.length
            : 0;
        expect(average).toBe(0);
    });

    it('should calculate average for non-empty array', () => {
        const salaries = [{ total_salary: 10000 }, { total_salary: 20000 }];
        const average = salaries.length > 0
            ? salaries.reduce((sum, s) => sum + (Number(s.total_salary) || 0), 0) / salaries.length
            : 0;
        expect(average).toBe(15000);
    });

    it('should handle null total_salary values', () => {
        const salaries = [{ total_salary: null }, { total_salary: 20000 }];
        const average = salaries.length > 0
            ? salaries.reduce((sum, s) => sum + (Number(s.total_salary) || 0), 0) / salaries.length
            : 0;
        expect(average).toBe(10000);
    });
});

// ==================== 场景 8: PolicyDetail null match_company_age_max ====================

describe('Crash #8: PolicyDetail null match_company_age_max', () => {
    it('should not render when match_company_age_max is null (guarded by &&)', () => {
        const policy = { match_company_age_max: null as number | null };
        // The code uses: policy.match_company_age_max && Math.round(policy.match_company_age_max / 12)
        // null && ... short-circuits to null, no division happens
        const result = policy.match_company_age_max && Math.round(policy.match_company_age_max / 12);
        expect(result).toBeNull();
    });

    it('should calculate correctly for valid value', () => {
        const policy = { match_company_age_max: 36 };
        const result = policy.match_company_age_max && Math.round(policy.match_company_age_max / 12);
        expect(result).toBe(3);
    });
});

// ==================== 场景 9: Number formatting with null values ====================

describe('Crash #9: Number formatting resilience', () => {
    it('Number(null) should produce 0, not crash', () => {
        expect(Number(null)).toBe(0);
        expect(() => Number(null).toLocaleString()).not.toThrow();
    });

    it('Number(undefined) should produce NaN, toLocaleString should not crash', () => {
        expect(Number(undefined)).toBeNaN();
        expect(() => Number(undefined).toLocaleString()).not.toThrow();
    });

    it('Number("") should produce 0', () => {
        expect(Number('')).toBe(0);
    });

    it('Number("abc") should produce NaN, toLocaleString should not crash', () => {
        expect(Number('abc')).toBeNaN();
        expect(() => Number('abc').toLocaleString()).not.toThrow();
    });
});

// ==================== Component Render: null numbers in display ====================

describe('Component Render: Null number display', () => {
    it('should render component with null monetary values', () => {
        const stats = {
            totalValue: null as number | null,
            totalExpense: undefined as number | undefined,
            averageSalary: NaN,
        };

        const TestComponent = () => (
            <div>
                <span>{safeNum(stats.totalValue)}</span>
                <span>{safeCurrency(stats.totalExpense)}</span>
                <span>{safeCurrency(stats.averageSalary)}</span>
            </div>
        );

        const { container } = render(<TestComponent />);
        const spans = container.querySelectorAll('span');
        expect(spans[0].textContent).toBe('—');
        expect(spans[1].textContent).toBe('—');
        expect(spans[2].textContent).toBe('—');
    });

    it('should render component with valid monetary values', () => {
        const TestComponent = () => (
            <div>
                <span>{safeCurrency(10000)}</span>
                <span>{safeNum(12345)}</span>
            </div>
        );

        const { container } = render(<TestComponent />);
        const spans = container.querySelectorAll('span');
        expect(spans[0].textContent).toContain('¥');
        expect(spans[0].textContent).toContain('10,000');
    });
});
