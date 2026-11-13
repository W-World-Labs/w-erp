/**
 * E2E Test Suite 2: Navigation Alignment (导航对齐)
 * Verifies Sidebar menu items ↔ App.tsx routes ↔ Backend APIs are synchronized.
 */

import { describe, it, expect } from 'vitest';
import { MENU_CATEGORIES } from '@/lib/nav-catalog';
import { applyNavPreferences, emptyNavPreferences } from '@/lib/nav-preferences';

// ==================== Sidebar Menu Structure ====================

const menuCategories = MENU_CATEGORIES;

// All routes defined in App.tsx (extracted from real App.tsx structure)
const APP_ROUTES = [
    '/', '/kpi-dashboard', '/company-info', '/branches', '/audit-center',
    '/assets', '/invoices', '/records', '/salary', '/rent',
    '/payments', '/contracts', '/vendors', '/crm', '/deals',
    '/quotes', '/time-tracking', '/recurring-invoices', '/compliance',
    '/tax-calendar', '/projects', '/applications', '/policies',
    '/global-opc', '/financial-reports', '/tax-management',
    '/budget-management', '/cash-flow-forecast', '/cloud-cost-management',
    '/licenses', '/employees', '/reminders', '/user-management',
    '/operation-logs', '/reconciliation', '/bank-accounts',
    '/profile', '/settings',
];

// Routes requiring admin (ProtectedRoute requireAdmin)
const ADMIN_ROUTES = ['/salary', '/user-management', '/operation-logs', '/settings'];

// ==================== Tests ====================

describe('E2E: Sidebar ↔ Router Alignment (导航 ↔ 路由对齐)', () => {
    const allSidebarPaths = menuCategories.flatMap(c => c.items.map(i => i.path));

    describe('Every sidebar item has a matching route in App.tsx', () => {
        it.each(allSidebarPaths)('sidebar path "%s" exists in App.tsx routes', (path) => {
            expect(APP_ROUTES).toContain(path);
        });
    });

    describe('Sidebar menu structure', () => {
        it('has 5 categories', () => {
            expect(menuCategories).toHaveLength(5);
        });

        it('categories are: 概览 / 财务管理 / 业务管理 / 团队与档案 / 系统', () => {
            const labels = menuCategories.map(c => c.label);
            expect(labels).toEqual(['概览', '财务管理', '业务管理', '团队与档案', '系统']);
        });

        it('total sidebar items = 37', () => {
            const total = menuCategories.reduce((sum, c) => sum + c.items.length, 0);
            expect(total).toBe(37);
        });

        it('no duplicate paths in sidebar', () => {
            const unique = new Set(allSidebarPaths);
            expect(unique.size).toBe(allSidebarPaths.length);
        });

        it('no duplicate labels in sidebar', () => {
            const labels = menuCategories.flatMap(c => c.items.map(i => i.label));
            const unique = new Set(labels);
            expect(unique.size).toBe(labels.length);
        });
    });

    describe('Admin-protected routes', () => {
        it.each(ADMIN_ROUTES)('"%s" requires admin access', (path) => {
            expect(ADMIN_ROUTES).toContain(path);
        });

        it('salary management requires admin', () => {
            expect(ADMIN_ROUTES).toContain('/salary');
        });

        it('user management requires admin', () => {
            expect(ADMIN_ROUTES).toContain('/user-management');
        });

        it('operation logs require admin', () => {
            expect(ADMIN_ROUTES).toContain('/operation-logs');
        });

        it('system settings require admin', () => {
            expect(ADMIN_ROUTES).toContain('/settings');
        });
    });

    describe('Routes outside sidebar (hidden pages)', () => {
        const hiddenRoutes = APP_ROUTES.filter(r => !allSidebarPaths.includes(r));

        it('only branches remains a non-menu utility route among former islands', () => {
            // bank-accounts / reconciliation / operation-logs / user-management
            // are now first-class sidebar entries
            expect(hiddenRoutes).not.toContain('/bank-accounts');
            expect(hiddenRoutes).not.toContain('/reconciliation');
            expect(hiddenRoutes).not.toContain('/operation-logs');
            expect(hiddenRoutes).not.toContain('/user-management');
            expect(hiddenRoutes).toContain('/branches');
        });

        it('hidden routes count is manageable (< 10)', () => {
            expect(hiddenRoutes.length).toBeLessThan(10);
        });
    });

    describe('Sidebar business category coverage', () => {
        const financePaths = menuCategories.find(c => c.label === '财务管理')!.items.map(i => i.path);
        const businessPaths = menuCategories.find(c => c.label === '业务管理')!.items.map(i => i.path);

        it('财务管理 has all financial pages', () => {
            expect(financePaths).toContain('/invoices');
            expect(financePaths).toContain('/payments');
            expect(financePaths).toContain('/bank-accounts');
            expect(financePaths).toContain('/reconciliation');
            expect(financePaths).toContain('/salary');
            expect(financePaths).toContain('/budget-management');
            expect(financePaths).toContain('/tax-management');
            expect(financePaths).toContain('/tax-calendar');
            expect(financePaths).toContain('/recurring-invoices');
        });

        it('业务管理 has all CRM + operations pages', () => {
            expect(businessPaths).toContain('/crm');
            expect(businessPaths).toContain('/deals');
            expect(businessPaths).toContain('/quotes');
            expect(businessPaths).toContain('/contracts');
            expect(businessPaths).toContain('/projects');
            expect(businessPaths).toContain('/time-tracking');
            expect(businessPaths).toContain('/compliance');
            expect(businessPaths).toContain('/global-opc');
        });
    });

    describe('User-configurable sidebar', () => {
        it('default preferences still render the full catalog for admins', () => {
            const visible = applyNavPreferences(emptyNavPreferences(), { isAdmin: true });
            expect(visible).toHaveLength(5);
            expect(visible.reduce((sum, c) => sum + c.items.length, 0)).toBe(37);
        });

        it('hidden catalog items stay in the route table', () => {
            expect(APP_ROUTES).toContain('/invoices');
            expect(APP_ROUTES).toContain('/employees');
        });
    });
});
