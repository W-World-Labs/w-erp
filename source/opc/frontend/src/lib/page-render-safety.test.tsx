/**
 * Page Render Safety Tests (页面渲染安全测试)
 *
 * 这些测试模拟用户在浏览器中实际遇到的崩溃场景：
 * - RangeError: Invalid time value (null date → format(new Date(null)))
 * - TypeError: Cannot read properties of undefined (reading 'country')
 * - 各种 null/undefined 字段渲染导致的崩溃
 *
 * 测试策略: 不需要真正渲染组件（避免依赖太多 mock），
 * 而是直接测试组件中使用的数据处理逻辑 + safeFormat 在真实数据场景下的行为。
 * 以及用 @testing-library/react 渲染轻量组件验证不崩溃。
 */
import { describe, it, expect, vi } from 'vitest';
import { render } from '@testing-library/react';
import React from 'react';
import { safeFormat, safeDiffDays } from './dateUtils';

// ==================== 场景 1: Payments 页面崩溃复现 ====================
// 原始崩溃: RangeError: Invalid time value
// 原因: format(new Date(payment.payment_date), "yyyy-MM-dd")
// payment_date 为 null/undefined

describe('Payments Page: Null Date Safety', () => {
    // 模拟后端返回的支付数据 (各种字段可能为 null)
    const paymentWithNullDates = {
        id: 'test-1',
        payment_no: 'PAY-001',
        payment_date: null as string | null,
        amount: 1000,
        payment_method: 'bank_transfer_b2b',
        payment_type: '支出',
        payer: 'Company A',
        payee: 'Company B',
        description: 'Test payment',
        category: '办公费用',
        invoice_required: true,
        invoice_received: false,
        invoice_deadline: null as string | null,
        status: 'completed',
        notes: '',
    };

    it('should not crash when payment_date is null', () => {
        expect(() => safeFormat(paymentWithNullDates.payment_date)).not.toThrow();
        expect(safeFormat(paymentWithNullDates.payment_date)).toBe('—');
    });

    it('should not crash when invoice_deadline is null', () => {
        expect(() => safeFormat(paymentWithNullDates.invoice_deadline)).not.toThrow();
    });

    it('should not crash when payment_date is undefined', () => {
        const p = { ...paymentWithNullDates, payment_date: undefined };
        expect(() => safeFormat(p.payment_date)).not.toThrow();
    });

    it('should not crash when all dates are empty strings', () => {
        const p = { ...paymentWithNullDates, payment_date: '', invoice_deadline: '' };
        expect(() => {
            safeFormat(p.payment_date);
            safeFormat(p.invoice_deadline);
        }).not.toThrow();
    });

    it('should handle urgency badge with null deadline', () => {
        const daysLeft = safeDiffDays(paymentWithNullDates.invoice_deadline);
        expect(daysLeft).toBe(999); // 表示"未设定"
    });

    it('should calculate stats correctly with empty payments array', () => {
        const payments: typeof paymentWithNullDates[] = [];
        const stats = {
            total: payments.length,
            expense: payments
                .filter((p) => p.payment_type === '支出')
                .reduce((sum, p) => sum + Number(p.amount), 0),
            income: payments
                .filter((p) => p.payment_type === '收入')
                .reduce((sum, p) => sum + Number(p.amount), 0),
            pendingInvoice: payments.filter((p) => p.invoice_required && !p.invoice_received).length,
        };
        expect(stats.total).toBe(0);
        expect(stats.expense).toBe(0);
        expect(stats.income).toBe(0);
        expect(stats.pendingInvoice).toBe(0);
    });

    it('should handle payments with mix of valid and null dates', () => {
        const payments = [
            { ...paymentWithNullDates, payment_date: '2024-03-15' },
            { ...paymentWithNullDates, payment_date: null },
            { ...paymentWithNullDates, payment_date: undefined },
            { ...paymentWithNullDates, payment_date: '' },
            { ...paymentWithNullDates, payment_date: 'invalid' },
        ];

        expect(() => {
            payments.forEach(p => safeFormat(p.payment_date));
        }).not.toThrow();

        const results = payments.map(p => safeFormat(p.payment_date));
        expect(results[0]).toBe('2024-03-15');
        expect(results[1]).toBe('—');
        expect(results[2]).toBe('—');
        expect(results[3]).toBe('—');
        expect(results[4]).toBe('—');
    });
});

// ==================== 场景 2: PolicyDashboard 页面崩溃复现 ====================
// 原始崩溃: TypeError: Cannot read properties of undefined (reading 'country')
// 原因: match.policy 为 undefined 时访问 match.policy.country

describe('PolicyDashboard: Undefined Policy Safety', () => {
    // 模拟 match_policies API 返回的数据
    type PolicyMatch = {
        policy: {
            id: string;
            name: string;
            country: string;
            provider: string;
        } | undefined;
        score: number;
        reasons: string[];
    };

    it('should safely access country on undefined policy', () => {
        const match: PolicyMatch = {
            policy: undefined,
            score: 0.8,
            reasons: ['Test'],
        };

        // 安全访问模式 (optional chaining)
        const country = match.policy?.country;
        expect(country).toBeUndefined();
        expect(() => {
            const _ = match.policy?.country ?? '未知';
        }).not.toThrow();
    });

    it('should handle array of matches with some undefined policies', () => {
        const matches: PolicyMatch[] = [
            {
                policy: { id: '1', name: 'YC', country: 'US', provider: 'YC' },
                score: 0.9,
                reasons: ['Good match'],
            },
            {
                policy: undefined,
                score: 0.5,
                reasons: ['Partial'],
            },
        ];

        expect(() => {
            matches.forEach((m) => {
                const country = m.policy?.country ?? '未知';
                const name = m.policy?.name ?? '未知政策';
                expect(typeof country).toBe('string');
                expect(typeof name).toBe('string');
            });
        }).not.toThrow();
    });

    it('should handle policy with null country', () => {
        const policy = { id: '1', name: 'Test', country: null as string | null, provider: '' };
        const country = policy.country ?? '未知';
        expect(country).toBe('未知');
    });
});

// ==================== 场景 3: ReconciliationCenter 多表数据 ====================

describe('ReconciliationCenter: Mixed Null Date Fields', () => {
    it('should safely format transaction dates', () => {
        const transactions = [
            { id: '1', transaction_date: '2024-03-15T10:30:00Z' },
            { id: '2', transaction_date: null },
            { id: '3', transaction_date: undefined },
        ];

        const results = transactions.map(t => safeFormat(t.transaction_date as string | null));
        expect(results[0]).toBe('2024-03-15');
        expect(results[1]).toBe('—');
        expect(results[2]).toBe('—');
    });

    it('should safely format payment dates with locale', () => {
        // This tests the third parameter (options) path
        const result = safeFormat('2024-03-15', 'yyyy-MM-dd');
        expect(result).toBe('2024-03-15');

        const nullResult = safeFormat(null, 'yyyy-MM-dd');
        expect(nullResult).toBe('—');
    });

    it('should safely format salary payment dates', () => {
        const salaryRecord = { payment_date: null as string | null };
        expect(() => {
            const formatted = salaryRecord.payment_date
                ? safeFormat(salaryRecord.payment_date, 'yyyy-MM-dd')
                : '-';
            expect(formatted).toBe('-');
        }).not.toThrow();
    });
});

// ==================== 场景 4: Invoices 页面 ====================

describe('Invoices Page: Null Invoice Date Safety', () => {
    it('should safely format null invoice_date', () => {
        const invoice = {
            id: '1',
            invoice_number: 'INV-001',
            invoice_date: null as string | null,
            due_date: null as string | null,
            amount: 5000,
        };

        expect(() => {
            safeFormat(invoice.invoice_date);
            safeFormat(invoice.due_date);
        }).not.toThrow();
    });

    it('should handle invoice with valid date', () => {
        const result = safeFormat('2024-06-15', 'yyyy-MM-dd');
        expect(result).toBe('2024-06-15');
    });
});

// ==================== 场景 5: Component Render with Null Data ====================

describe('Component Render Safety', () => {
    it('should render a component that displays formatted dates without crashing', () => {
        const TestComponent = () => {
            const dates = [null, undefined, '', 'invalid', '2024-03-15'];
            return (
                <div>
                    {dates.map((d, i) => (
                        <span key={i}>{safeFormat(d)}</span>
                    ))}
                </div>
            );
        };

        expect(() => {
            const { container } = render(<TestComponent />);
            expect(container.querySelectorAll('span')).toHaveLength(5);
        }).not.toThrow();
    });

    it('should render a table row with null payment data without crashing', () => {
        const payment = {
            payment_no: 'PAY-001',
            payment_date: null as string | null,
            amount: null as number | null,
            payer: null as string | null,
            payee: null as string | null,
            payment_method: null as string | null,
            status: null as string | null,
        };

        const TestRow = () => (
            <table>
                <tbody>
                    <tr>
                        <td>{payment.payment_no}</td>
                        <td>{safeFormat(payment.payment_date)}</td>
                        <td>{payment.amount ?? 0}</td>
                        <td>{payment.payer || '-'}</td>
                        <td>{payment.payee || '-'}</td>
                    </tr>
                </tbody>
            </table>
        );

        expect(() => {
            const { getByText, getAllByText } = render(<TestRow />);
            expect(getByText('PAY-001')).toBeTruthy();
            expect(getAllByText('—').length).toBeGreaterThanOrEqual(1); // null date → —
            expect(getAllByText('-').length).toBeGreaterThanOrEqual(1); // null payer → -
        }).not.toThrow();
    });

    it('should render policy card with missing fields without crashing', () => {
        const policy = {
            id: '1',
            name: 'Test Policy',
            country: undefined as string | undefined,
            provider: null as string | null,
            max_funding: null as number | null,
            equity_required: undefined as boolean | undefined,
            deadline: null as string | null,
        };

        const TestPolicyCard = () => (
            <div>
                <h3>{policy.name}</h3>
                <span>{policy.country ?? '未知'}</span>
                <span>{policy.provider || '未知机构'}</span>
                <span>{policy.max_funding ? `$${policy.max_funding}` : '未知'}</span>
                <span>{safeFormat(policy.deadline)}</span>
            </div>
        );

        expect(() => {
            const { getByText, getAllByText } = render(<TestPolicyCard />);
            expect(getByText('Test Policy')).toBeTruthy();
            expect(getAllByText('未知').length).toBeGreaterThanOrEqual(1);
            expect(getByText('未知机构')).toBeTruthy();
        }).not.toThrow();
    });
});

// ==================== 场景 6: Employee/License Date Fields ====================

describe('Employee & License: Date Boundary Cases', () => {
    it('should handle employee with null hire_date', () => {
        expect(() => safeFormat(null, 'yyyy-MM-dd')).not.toThrow();
        expect(safeFormat(null)).toBe('—');
    });

    it('should handle license with far-future expiry', () => {
        const result = safeFormat('2099-12-31', 'yyyy-MM-dd');
        expect(result).toBe('2099-12-31');
    });

    it('should handle epoch timestamp string', () => {
        const result = safeFormat('1970-01-01');
        expect(result).toBe('1970-01-01');
    });
});

// ==================== 场景 7: Array.map on null/undefined data ====================

describe('Array Operations: Empty/Null Data Safety', () => {
    it('should handle .map on empty array (common pattern in all list pages)', () => {
        const items: any[] = [];
        expect(() => {
            const rendered = items.map(item => ({
                date: safeFormat(item.date),
                name: item.name ?? '—',
            }));
            expect(rendered).toEqual([]);
        }).not.toThrow();
    });

    it('should handle .filter and .reduce on empty payments (stats calculation)', () => {
        const payments: any[] = [];
        const total = payments.reduce((sum: number, p: any) => sum + Number(p.amount || 0), 0);
        expect(total).toBe(0);
    });

    it('should handle .filter with null field comparisons', () => {
        const items = [
            { status: null },
            { status: 'active' },
            { status: undefined },
            { status: '' },
        ];

        const active = items.filter(i => i.status === 'active');
        expect(active).toHaveLength(1);

        const nullish = items.filter(i => !i.status);
        expect(nullish).toHaveLength(3);
    });
});
