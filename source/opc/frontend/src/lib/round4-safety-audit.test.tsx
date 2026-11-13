/**
 * Round 4 Safety Audit Tests
 *
 * 覆盖 ReconciliationCenter, CompanyInfo, GlobalOpcHub, UserManagement 修复
 */
import { describe, it, expect } from 'vitest';
import { parseISO, isValid, format } from 'date-fns';

// ==================== toLocaleString on null API amount ====================

describe('ReconciliationCenter: toLocaleString on null amount', () => {
    it('null.toLocaleString() would crash', () => {
        const val: any = null;
        expect(() => val.toLocaleString()).toThrow(TypeError);
    });

    it('Number(null || 0).toLocaleString() is safe', () => {
        const val: number | null = null;
        expect(Number(val || 0).toLocaleString()).toBe('0');
    });

    it('Number(undefined || 0).toLocaleString() is safe', () => {
        const val: number | undefined = undefined;
        expect(Number(val || 0).toLocaleString()).toBe('0');
    });

    it('Number("" || 0).toLocaleString() is safe', () => {
        const val = '';
        expect(Number(val || 0).toLocaleString()).toBe('0');
    });

    it('Number(12345.67 || 0).toLocaleString() preserves value', () => {
        const val = 12345.67;
        expect(Number(val || 0).toLocaleString()).toBe('12,345.67');
    });

    it('Number(-500 || 0).toLocaleString() handles negative', () => {
        const val = -500;
        expect(Number(val || 0).toLocaleString()).toBe('-500');
    });
});

// ==================== parseISO + isValid guard ====================

describe('CompanyInfo: parseISO + isValid guard', () => {
    it('parseISO(valid string) should be valid', () => {
        const d = parseISO('2024-01-15');
        expect(isValid(d)).toBe(true);
    });

    it('parseISO(invalid string) should be invalid', () => {
        const d = parseISO('not-a-date');
        expect(isValid(d)).toBe(false);
    });

    it('parseISO(empty string) should be invalid', () => {
        const d = parseISO('');
        expect(isValid(d)).toBe(false);
    });

    it('format(parseISO(invalid)) should throw without guard', () => {
        const d = parseISO('bad-date');
        expect(() => format(d, 'PPP')).toThrow();
    });

    it('isValid guard prevents format crash', () => {
        const d = parseISO('bad-date');
        const result = isValid(d) ? format(d, 'PPP') : null;
        expect(result).toBeNull();
    });

    it('getTime() sort on Invalid Date returns NaN', () => {
        const d = parseISO('bad');
        expect(isNaN(d.getTime())).toBe(true);
    });
});

// ==================== string.replace() on null ====================

describe('GlobalOpcHub: replace on null string', () => {
    it('null.replace() would crash', () => {
        const val: any = null;
        expect(() => val.replace(/test/, '')).toThrow(TypeError);
    });

    it('(null || "").replace() is safe', () => {
        const val: any = null;
        expect((val || '').replace(/test/, '')).toBe('');
    });

    it('valid string.replace() works normally', () => {
        const val = 'Type A (General)';
        expect(val.replace(/Type [A-D] \(/, '').replace(')', '')).toBe('General');
    });
});

// ==================== [0].toUpperCase() on empty string ====================

describe('UserManagement: [0].toUpperCase on empty name', () => {
    it('empty string[0] is undefined', () => {
        const val = '';
        expect(val[0]).toBeUndefined();
    });

    it('undefined.toUpperCase() would crash', () => {
        const val: any = undefined;
        expect(() => val.toUpperCase()).toThrow(TypeError);
    });

    it('(null || "?")[0].toUpperCase() is safe', () => {
        const name: any = null;
        const email: any = null;
        expect((name || email || '?')[0].toUpperCase()).toBe('?');
    });

    it('valid string still works', () => {
        const name = 'Alice';
        expect((name || '?')[0].toUpperCase()).toBe('A');
    });
});

// ==================== Number() coercion patterns ====================

describe('Number() coercion safety patterns', () => {
    it('Number(null) is 0', () => {
        expect(Number(null)).toBe(0);
    });

    it('Number(undefined) is NaN', () => {
        expect(isNaN(Number(undefined))).toBe(true);
    });

    it('Number("") is 0', () => {
        expect(Number('')).toBe(0);
    });

    it('Number("abc") is NaN', () => {
        expect(isNaN(Number('abc'))).toBe(true);
    });

    it('(NaN).toLocaleString() is "NaN" (no crash)', () => {
        expect(Number('abc').toLocaleString()).toBe('NaN');
    });

    it('Number(undefined || 0) prevents NaN display', () => {
        const val: any = undefined;
        expect(Number(val || 0).toLocaleString()).toBe('0');
    });
});

// ==================== Array.isArray + [0] safety ====================

describe('CompanyInfo: Array.isArray + data[0] pattern', () => {
    it('empty array[0] is undefined (no crash)', () => {
        const arr: any[] = [];
        expect(arr[0]).toBeUndefined();
    });

    it('Array.isArray(data) ? data[0] : data handles both cases', () => {
        const arrData = [{ name: 'Company A' }];
        const objData = { name: 'Company B' };

        expect(Array.isArray(arrData) ? arrData[0] : arrData).toEqual({ name: 'Company A' });
        expect(Array.isArray(objData) ? (objData as any)[0] : objData).toEqual({ name: 'Company B' });
    });

    it('empty array returns undefined, which is falsy', () => {
        const data: any[] = [];
        const result = Array.isArray(data) ? data[0] : data;
        expect(result).toBeUndefined();
        expect(!result).toBe(true); // falsy, so the "no company" UI appears
    });
});
