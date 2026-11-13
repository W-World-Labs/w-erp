import { describe, it, expect } from 'vitest';
import { cn } from './utils';

describe('cn utility function', () => {
    it('should merge class names correctly', () => {
        const result = cn('foo', 'bar');
        expect(result).toBe('foo bar');
    });

    it('should handle conditional classes', () => {
        const isActive = true;
        const result = cn('base', isActive && 'active');
        expect(result).toBe('base active');
    });

    it('should handle falsy values', () => {
        const result = cn('base', false, null, undefined, 'valid');
        expect(result).toBe('base valid');
    });

    it('should merge Tailwind classes correctly', () => {
        // tailwind-merge should handle conflicting classes
        const result = cn('px-2 py-1', 'px-4');
        expect(result).toBe('py-1 px-4');
    });

    it('should handle empty input', () => {
        const result = cn();
        expect(result).toBe('');
    });

    it('should handle object syntax', () => {
        const result = cn('base', { active: true, disabled: false });
        expect(result).toBe('base active');
    });
});
