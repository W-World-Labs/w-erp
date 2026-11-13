/**
 * Normalize a date string from various AI-returned formats to YYYY-MM-DD.
 * Handles:
 *   - "2024年3月15日" or "2024年03月15日"
 *   - "2024/3/15" or "2024/03/15"
 *   - "2024-3-15" or "2024-03-15"
 *   - "2024.3.15" or "2024.03.15"
 *   - Already correct "2024-03-15"
 * Returns null if the date cannot be parsed.
 */
export function normalizeDate(raw: string | null | undefined): string | null {
    if (!raw || raw === 'null') return null;

    const s = raw.trim();

    // Pattern: 2024年3月15日 or 2024年03月15日
    const cnMatch = s.match(/(\d{4})\s*年\s*(\d{1,2})\s*月\s*(\d{1,2})\s*日?/);
    if (cnMatch) {
        const [, y, m, d] = cnMatch;
        return `${y}-${m.padStart(2, '0')}-${d.padStart(2, '0')}`;
    }

    // Pattern: 2024/3/15, 2024-3-15, 2024.3.15
    const sepMatch = s.match(/(\d{4})[/.-](\d{1,2})[/.-](\d{1,2})/);
    if (sepMatch) {
        const [, y, m, d] = sepMatch;
        return `${y}-${m.padStart(2, '0')}-${d.padStart(2, '0')}`;
    }

    // Already YYYY-MM-DD
    if (/^\d{4}-\d{2}-\d{2}$/.test(s)) {
        return s;
    }

    return null;
}

/**
 * Safely format a date string. Returns fallback "—" for null/undefined/invalid values.
 * Prevents RangeError: Invalid time value from crashing pages.
 */
import { format, isValid, differenceInDays } from "date-fns";

export function safeFormat(
    dateStr: string | null | undefined,
    fmt: string = "yyyy-MM-dd",
    options?: Parameters<typeof format>[2]
): string {
    if (!dateStr) return "—";
    const d = new Date(dateStr);
    return isValid(d) ? format(d, fmt, options) : "—";
}

/**
 * Safely calculate difference in days. Returns 999 for invalid/null dates.
 */
export function safeDiffDays(dateStr: string | null | undefined): number {
    if (!dateStr) return 999;
    const d = new Date(dateStr);
    return isValid(d) ? differenceInDays(d, new Date()) : 999;
}

/**
 * Safely format a number. Returns fallback for null/undefined/NaN.
 * Prevents TypeError: Cannot read properties of null (reading 'toLocaleString')
 */
export function safeNum(
    val: number | string | null | undefined,
    options?: Intl.NumberFormatOptions,
    locale: string = 'zh-CN'
): string {
    if (val === null || val === undefined || val === '') return '—';
    const n = Number(val);
    if (isNaN(n)) return '—';
    return n.toLocaleString(locale, options);
}

/**
 * Safely format a currency value. Returns "—" for null/undefined.
 */
export function safeCurrency(
    val: number | string | null | undefined,
    prefix: string = '¥',
    options?: Intl.NumberFormatOptions
): string {
    if (val === null || val === undefined || val === '') return '—';
    const n = Number(val);
    if (isNaN(n)) return '—';
    return `${prefix}${n.toLocaleString('zh-CN', { minimumFractionDigits: 2, maximumFractionDigits: 2, ...options })}`;
}
