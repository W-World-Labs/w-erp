import { describe, it, expect } from 'vitest';
import { normalizeDate, safeFormat, safeDiffDays } from './dateUtils';

describe('normalizeDate', () => {
    // Null/undefined/empty handling
    it('should return null for null input', () => {
        expect(normalizeDate(null)).toBeNull();
    });

    it('should return null for undefined input', () => {
        expect(normalizeDate(undefined)).toBeNull();
    });

    it('should return null for empty string', () => {
        expect(normalizeDate('')).toBeNull();
    });

    it('should return null for "null" string', () => {
        expect(normalizeDate('null')).toBeNull();
    });

    // Chinese format: 2024年3月15日
    it('should parse Chinese date format with single-digit month/day', () => {
        expect(normalizeDate('2024年3月5日')).toBe('2024-03-05');
    });

    it('should parse Chinese date format with zero-padded month/day', () => {
        expect(normalizeDate('2024年03月15日')).toBe('2024-03-15');
    });

    it('should parse Chinese date format without trailing 日', () => {
        expect(normalizeDate('2024年12月1')).toBe('2024-12-01');
    });

    it('should parse Chinese date format with spaces', () => {
        expect(normalizeDate('2024 年 3 月 15 日')).toBe('2024-03-15');
    });

    // Slash format: 2024/3/15
    it('should parse slash format with single-digit month/day', () => {
        expect(normalizeDate('2024/3/5')).toBe('2024-03-05');
    });

    it('should parse slash format with zero-padded month/day', () => {
        expect(normalizeDate('2024/03/15')).toBe('2024-03-15');
    });

    // Dot format: 2024.3.15
    it('should parse dot format', () => {
        expect(normalizeDate('2024.3.15')).toBe('2024-03-15');
    });

    it('should parse dot format with zero-padding', () => {
        expect(normalizeDate('2024.03.15')).toBe('2024-03-15');
    });

    // Hyphen format: 2024-3-15
    it('should parse hyphen format with single-digit month/day', () => {
        expect(normalizeDate('2024-3-5')).toBe('2024-03-05');
    });

    // Already correct format
    it('should pass through already correct YYYY-MM-DD', () => {
        expect(normalizeDate('2024-03-15')).toBe('2024-03-15');
    });

    it('should pass through another correct date', () => {
        expect(normalizeDate('2026-01-01')).toBe('2026-01-01');
    });

    // Invalid formats
    it('should return null for invalid date string', () => {
        expect(normalizeDate('not a date')).toBeNull();
    });

    it('should return null for partial date', () => {
        expect(normalizeDate('2024-03')).toBeNull();
    });

    // Whitespace handling
    it('should trim whitespace', () => {
        expect(normalizeDate('  2024-03-15  ')).toBe('2024-03-15');
    });
});

// ==================== safeFormat Tests ====================

describe('safeFormat', () => {
    // === Null safety (这些就是导致 RangeError: Invalid time value 崩溃的场景) ===

    it('should return "—" for null input', () => {
        expect(safeFormat(null)).toBe('—');
    });

    it('should return "—" for undefined input', () => {
        expect(safeFormat(undefined)).toBe('—');
    });

    it('should return "—" for empty string', () => {
        expect(safeFormat('')).toBe('—');
    });

    it('should return "—" for invalid date string', () => {
        expect(safeFormat('not-a-date')).toBe('—');
    });

    it('should return "—" for garbage data', () => {
        expect(safeFormat('abc123xyz')).toBe('—');
    });

    // === Valid dates ===

    it('should format valid ISO date', () => {
        expect(safeFormat('2024-03-15')).toBe('2024-03-15');
    });

    it('should format valid ISO datetime', () => {
        expect(safeFormat('2024-03-15T10:30:00Z')).toBe('2024-03-15');
    });

    it('should use custom format string', () => {
        expect(safeFormat('2024-03-15', 'yyyy/MM/dd')).toBe('2024/03/15');
    });

    it('should handle datetime format', () => {
        const result = safeFormat('2024-03-15T10:30:00Z', 'yyyy-MM-dd HH:mm');
        expect(result).toContain('2024-03-15');
    });

    it('should handle date with time format', () => {
        const result = safeFormat('2024-12-25', 'MM月dd日');
        expect(result).toBe('12月25日');
    });

    // === Edge cases ===

    it('should not crash on "null" string literal', () => {
        // Some APIs return the string "null" instead of actual null
        const result = safeFormat('null');
        expect(typeof result).toBe('string'); // should not throw
    });

    it('should not crash on numeric-like string', () => {
        const result = safeFormat('0');
        expect(typeof result).toBe('string');
    });

    it('should handle partial date format gracefully', () => {
        const result = safeFormat('2024-03');
        expect(typeof result).toBe('string'); // should not throw
    });
});

// ==================== safeDiffDays Tests ====================

describe('safeDiffDays', () => {
    it('should return 999 for null', () => {
        expect(safeDiffDays(null)).toBe(999);
    });

    it('should return 999 for undefined', () => {
        expect(safeDiffDays(undefined)).toBe(999);
    });

    it('should return 999 for empty string', () => {
        expect(safeDiffDays('')).toBe(999);
    });

    it('should return 999 for invalid date', () => {
        expect(safeDiffDays('not-a-date')).toBe(999);
    });

    it('should return a negative number for past dates', () => {
        expect(safeDiffDays('2020-01-01')).toBeLessThan(0);
    });

    it('should return a positive number for far future dates', () => {
        expect(safeDiffDays('2099-12-31')).toBeGreaterThan(0);
    });

    it('should return 0 or close to 0 for today', () => {
        const today = new Date().toISOString().split('T')[0];
        expect(Math.abs(safeDiffDays(today))).toBeLessThanOrEqual(1);
    });
});
