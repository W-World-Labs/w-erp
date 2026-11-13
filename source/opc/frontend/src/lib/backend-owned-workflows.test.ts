import { describe, expect, it } from 'vitest';
import {
  normalizeWorkbenchSummary,
  WORKBENCH_SUMMARY_ENDPOINT,
} from '@/hooks/useSmartWorkbench';
import {
  AI_INSIGHTS_ENDPOINT,
  normalizeAIInsightsData,
} from '@/hooks/useAIInsights';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '..');

describe('backend-owned workflow clients', () => {
  it('uses backend aggregate endpoints for workbench and insights', () => {
    expect(WORKBENCH_SUMMARY_ENDPOINT).toBe('/api/v1/workbench/summary');
    expect(AI_INSIGHTS_ENDPOINT).toBe('/api/v1/ai/insights');
  });

  it('normalizes workbench summary without recalculating backend values', () => {
    const result = normalizeWorkbenchSummary({
      greeting: '下午好',
      generated_date_label: '2026年5月9日 星期六',
      operating_control: {
        id: 'operating_control',
        title: '运营指挥台',
        status: 'blocked',
        status_label: '需要处理',
        tone: 'danger',
        backbone_rate: 44,
        backbone_rate_label: '44%',
        completed: false,
        blocked_step_id: 'contracts',
        blocked_step_label: '合同',
        risk_count: 1,
        critical_count: 1,
        risk_count_label: '1 个风险',
        summary_label: '运营骨架卡在合同，优先补齐关键证据。',
        next_action: {
          route: '/contracts',
          label: '去归档合同',
          description: '补齐有效合同',
          action: null,
        },
        control_items: [
          {
            id: 'backbone_gap',
            type: 'backbone_gap',
            title: '运营骨架缺口：合同',
            description: '尚未归档当前有效合同',
            priority: 'critical',
            route: '/contracts',
            action_label: '去归档合同',
            badge: {
              label: '逾期',
              class_name: 'text-destructive',
              color_class: 'bg-destructive',
            },
            metadata: {
              blocked_step_id: 'contracts',
            },
          },
        ],
      },
      stats: {
        pending_invoices: 2,
        pending_invoices_label: '2 张待审发票',
        active_contracts: 3,
        active_contracts_label: '3',
        expiring_assets: 4,
        expiring_assets_label: '4 项即将到期',
        ongoing_projects: 5,
        ongoing_projects_label: '5',
        employees: 6,
        employees_label: '6',
        monthly_expense: 1234.5,
        monthly_expense_label: '¥1,234.50',
      },
      header_badges: [
        {
          label: '2 张待审发票',
          variant: 'secondary',
          class_name: 'py-1.5 px-3',
        },
        {
          label: '4 项即将到期',
          variant: 'outline',
          class_name: 'py-1.5 px-3 border-warning text-warning',
        },
      ],
      urgent_items: [
        {
          id: 'invoice:1',
          title: '发票待处理',
          type: 'invoice',
          priority: 'critical',
          due_date: '2026-05-01',
          days_left: -8,
          description: 'backend description',
          action: '去审批',
          path: '/invoices',
          priority_badge: {
            label: '逾期',
            class_name: 'text-destructive',
            color_class: 'bg-destructive',
          },
          days_left_label: '8 天前',
          type_icon: 'FileText',
        },
      ],
      attention_items: [
        {
          id: 'invoice:1',
          title: '发票待处理',
          type: 'invoice',
          priority: 'critical',
          due_date: '2026-05-01',
          days_left: -8,
          description: 'backend description',
          action: '去审批',
          path: '/invoices',
          priority_badge: {
            label: '逾期',
            class_name: 'text-destructive',
            color_class: 'bg-destructive',
          },
          days_left_label: '8 天前',
          type_icon: 'FileText',
        },
      ],
      application_reminders: [
        {
          id: 'application:1',
          title: '申报截止',
          type: 'application',
          priority: 'high',
          due_date: '2026-05-11',
          days_left: 2,
          description: 'backend application',
          action: '查看申报',
          path: '/applications',
          priority_badge: {
            label: '紧急',
            class_name: 'text-orange-600',
            color_class: 'bg-orange-500',
          },
          days_left_label: '2 天后截止',
          type_icon: 'FileCheck',
        },
      ],
      asset_expiry_reminders: [
        {
          id: 'asset:1',
          title: '资产到期',
          type: 'asset',
          priority: 'medium',
          due_date: '2026-05-12',
          days_left: 3,
          description: 'backend asset',
          action: '查看资产',
          path: '/assets',
          priority_badge: {
            label: '即将到期',
            class_name: 'text-yellow-600',
            color_class: 'bg-yellow-500',
          },
          days_left_label: '3 天后到期',
          type_icon: 'Archive',
        },
      ],
      today_tasks: [{ id: 'task:1' }],
      today_tasks_count: 1,
      todos: [
        {
          id: 'todo:1',
          title: '处理事项',
          category: '财务',
          priority: 'high',
          completed: false,
          path: '/invoices',
        },
      ],
      ai_suggestions: ['backend suggestion'],
      completion_rate: 55,
      stage_stats: {
        total_assets: 7,
        total_assets_label: '7',
        total_value: 8000,
        total_value_label: '¥8,000.00',
        pending_invoices: 2,
        pending_invoices_label: '2',
        total_salary: 900,
        total_salary_label: '¥900.00',
        ongoing_projects: 5,
        ongoing_projects_label: '5',
        active_contracts: 3,
        active_contracts_label: '3',
        employee_count: 6,
        employee_count_label: '6',
        total_projects: 9,
        total_projects_label: '9',
        total_contracts: 10,
        total_contracts_label: '10',
        project_execution_rate_label: '56%',
        contract_activity_rate_label: '30%',
      },
      monthly_expense_trend: [
        {
          month: '5月',
          salary: 900,
          salary_label: '¥900.00',
          invoice: 1234.5,
          invoice_label: '¥1,234.50',
          total: 2134.5,
          total_label: '¥2,134.50',
        },
      ],
      overview_cards: [
        {
          key: 'monthly_expense',
          icon: 'TrendingUp',
          title: '本月支出',
          value_label: '¥1,234.50',
          subtitle: null,
          trend_label: '+12%',
          trend_up: false,
          trend_variant: 'secondary',
          color: 'blue',
          class_name: 'from-blue-500/10 to-blue-500/5 text-blue-500',
        },
      ],
      quick_actions: [],
    });

    expect(result.stats.pendingInvoices).toBe(2);
    expect(result.stats.monthlyExpenseLabel).toBe('¥1,234.50');
    expect(result.headerBadges[0].label).toBe('2 张待审发票');
    expect(result.headerBadges[1].variant).toBe('outline');
    expect(result.urgentItems[0]).toMatchObject({
      dueDate: '2026-05-01',
      daysLeftLabel: '8 天前',
      priorityBadge: {
        label: '逾期',
      },
      path: '/invoices',
    });
    expect(result.generatedDateLabel).toBe('2026年5月9日 星期六');
    expect(result.operatingControl.statusLabel).toBe('需要处理');
    expect(result.operatingControl.summaryLabel).toContain('运营骨架');
    expect(result.operatingControl.backboneRateLabel).toBe('44%');
    expect(result.operatingControl.riskCountLabel).toBe('1 个风险');
    expect(result.operatingControl.nextAction.route).toBe('/contracts');
    expect(result.operatingControl.controlItems[0].actionLabel).toBe('去归档合同');
    expect(result.attentionItems[0].typeIcon).toBe('FileText');
    expect(result.applicationReminders[0].daysLeftLabel).toBe('2 天后截止');
    expect(result.assetExpiryReminders[0].priorityBadge.label).toBe('即将到期');
    expect(result.todayTasks).toBe(1);
    expect(result.aiSuggestions).toEqual(['backend suggestion']);
    expect(result.completionRate).toBe(55);
    expect(result.stageStats.totalAssets).toBe(7);
    expect(result.stageStats.totalValueLabel).toBe('¥8,000.00');
    expect(result.stageStats.projectExecutionRateLabel).toBe('56%');
    expect(result.monthlyExpenseTrend[0].totalLabel).toBe('¥2,134.50');
    expect(result.overviewCards[0].valueLabel).toBe('¥1,234.50');
    expect(result.overviewCards[0].className).toBe('from-blue-500/10 to-blue-500/5 text-blue-500');
    expect(result.overviewCards[0].trendVariant).toBe('secondary');
  });

  it('renders dashboard workbench display metadata without local business formatting', () => {
    const workbenchSource = readFileSync(resolve(root, 'components/dashboard/SmartWorkbench.tsx'), 'utf8');
    const startupSource = readFileSync(resolve(root, 'components/dashboard/StartupStage.tsx'), 'utf8');
    const matureSource = readFileSync(resolve(root, 'components/dashboard/MatureStage.tsx'), 'utf8');
    const hookSource = readFileSync(resolve(root, 'hooks/useSmartWorkbench.ts'), 'utf8');
    const applicationRemindersSource = readFileSync(resolve(root, 'components/dashboard/ApplicationReminders.tsx'), 'utf8');
    const assetExpiryRemindersSource = readFileSync(resolve(root, 'components/dashboard/AssetExpiryReminders.tsx'), 'utf8');

    expect(workbenchSource).toContain('generatedDateLabel');
    expect(workbenchSource).toContain('priorityBadge');
    expect(workbenchSource).toContain('daysLeftLabel');
    expect(workbenchSource).toContain('overviewCards');
    expect(workbenchSource).toContain('card.className');
    expect(workbenchSource).toContain('card.trendVariant');
    expect(workbenchSource).toContain('headerBadges');
    expect(workbenchSource).toContain('operatingControl');
    expect(workbenchSource).toContain('survivalItems');
    expect(workbenchSource).toContain('survivalTitle');
    expect(workbenchSource).toContain('survivalCountLabel');
    expect(workbenchSource).toContain('guidance');
    expect(workbenchSource).toContain('emptyUrgentTitle');
    expect(workbenchSource).toContain('emptyUrgentDescription');
    expect(workbenchSource).not.toContain('太棒了');
    expect(workbenchSource).not.toContain('startsWith');
    expect(workbenchSource).toContain('item.badge');
    expect(workbenchSource).toContain('item.actionLabel');
    expect(workbenchSource).not.toContain('priorityConfig');
    expect(workbenchSource).not.toContain('colorMap');
    expect(workbenchSource).not.toContain('card.trendUp ?');
    expect(workbenchSource).not.toContain('stats.pendingInvoices > 0');
    expect(workbenchSource).not.toContain('stats.expiringAssets > 0');
    expect(workbenchSource).not.toContain('operatingControl.backboneRate <');
    expect(workbenchSource).not.toContain('operatingControl.criticalCount >');
    expect(workbenchSource).not.toContain('daysLeft <');
    expect(workbenchSource).not.toContain('.slice(0, 5)');
    expect(workbenchSource).not.toContain('toLocaleString');
    expect(workbenchSource).not.toContain('format(new Date()');

    expect(startupSource).toContain('totalSalaryLabel');
    expect(startupSource).not.toContain('toFixed');
    expect(matureSource).toContain('totalValueLabel');
    expect(matureSource).toContain('projectExecutionRateLabel');
    expect(matureSource).toContain('payload.salaryLabel');
    expect(matureSource).not.toContain('toFixed');
    expect(matureSource).not.toContain('Number(value).toLocaleString');

    expect(hookSource).toContain('today_tasks_count');
    expect(hookSource).not.toContain('today_tasks?.length');

    expect(hookSource).toContain('operating_control');
    expect(hookSource).toContain('application_reminders');
    expect(hookSource).toContain('asset_expiry_reminders');
    expect(hookSource).toContain('survival_items');
    expect(hookSource).toContain('survival_title');
    expect(hookSource).toContain('empty_urgent_title');
    expect(hookSource).toContain('guidance');
    expect(applicationRemindersSource).toContain('applicationReminders');
    expect(applicationRemindersSource).toContain('priorityBadge');
    expect(applicationRemindersSource).toContain('daysLeftLabel');
    expect(applicationRemindersSource).not.toContain('urgentItems.filter');
    expect(applicationRemindersSource).not.toContain('item.type ===');
    expect(applicationRemindersSource).not.toContain('priorityLabel');
    expect(applicationRemindersSource).not.toContain('daysLeft <');
    expect(assetExpiryRemindersSource).toContain('assetExpiryReminders');
    expect(assetExpiryRemindersSource).toContain('priorityBadge');
    expect(assetExpiryRemindersSource).toContain('daysLeftLabel');
    expect(assetExpiryRemindersSource).not.toContain('urgentItems.filter');
    expect(assetExpiryRemindersSource).not.toContain('item.type ===');
    expect(assetExpiryRemindersSource).not.toContain('priorityLabel');
    expect(assetExpiryRemindersSource).not.toContain('daysLeft <');
  });

  it('normalizes backend AI insights and completeness scores', () => {
    const result = normalizeAIInsightsData({
      insights: [
        {
          id: 'urgent:1',
          type: 'urgent',
          priority: 'high',
          title: '合同即将到期',
          description: 'backend insight',
          category: 'contract',
          action: '/contracts',
          action_label: '去处理',
          icon: '📝',
          display_type: 'warning',
          display_icon: 'AlertTriangle',
          icon_class_name: 'text-red-500',
          icon_bg_class_name: 'bg-red-500/10',
          card_border_class_name: 'border-red-500/30',
          card_ring_class_name: '',
          action_class_name: 'text-red-500',
          priority_badge: {
            label: '重要',
            class_name: 'bg-orange-500 text-white',
            color_class: 'bg-orange-500',
          },
          confidence: 0.9,
          data: { days_left: 3 },
        },
      ],
      critical_insights: [
        {
          id: 'urgent:critical',
          type: 'urgent',
          priority: 'critical',
          title: '发票已逾期',
          description: 'backend critical insight',
          category: 'invoice',
          action: '/invoices',
          action_label: '去处理',
          icon: '📄',
          display_type: 'warning',
          display_icon: 'AlertTriangle',
          icon_class_name: 'text-red-500',
          icon_bg_class_name: 'bg-red-500/10',
          card_border_class_name: 'border-red-500/30',
          card_ring_class_name: 'ring-1 ring-red-500/20',
          action_class_name: 'text-red-500',
          priority_badge: {
            label: '紧急',
            class_name: 'bg-red-500 text-white animate-pulse',
            color_class: 'bg-red-500',
          },
          confidence: 0.95,
          data: { days_left: -1 },
        },
      ],
      other_insights: [
        {
          id: 'urgent:1',
          type: 'urgent',
          priority: 'high',
          title: '合同即将到期',
          description: 'backend insight',
          category: 'contract',
          action: '/contracts',
          action_label: '去处理',
          icon: '📝',
          display_type: 'warning',
          display_icon: 'AlertTriangle',
          icon_class_name: 'text-red-500',
          icon_bg_class_name: 'bg-red-500/10',
          card_border_class_name: 'border-red-500/30',
          card_ring_class_name: '',
          action_class_name: 'text-red-500',
          priority_badge: {
            label: '重要',
            class_name: 'bg-orange-500 text-white',
            color_class: 'bg-orange-500',
          },
          confidence: 0.9,
          data: { days_left: 3 },
        },
      ],
      completeness: {
        overall_score: 70,
        overall_score_label: '70%',
        overall_gradient_class: 'from-yellow-500 to-orange-500',
        unlock_title: '完善数据可解锁',
        unlock_description: '智能分析报告 · 成本优化建议 · 风险预警提醒',
        completed_count: 1,
        incomplete_count: 1,
        total_count: 2,
        completed_label: '已完成 1/2 个模块',
        incomplete_label: '还有 1 个模块待完善',
        is_complete: false,
        status_title: '快速完善系统数据',
        status_description: '还有 1 个模块待完善，后端会据此解锁更多自动化能力',
        completion_badge_label: '70%',
        remaining_overflow_count: 0,
        remaining_overflow_label: null,
        items: [
          {
            key: 'contracts',
            label: '合同台账',
            icon: 'FileText',
            path: '/contracts',
            score: 1,
            max_score: 1,
            percentage: 100,
            percentage_label: '100%',
            missing_items: [],
            description: 'backend completeness',
          },
          {
            key: 'assets',
            label: '资产台账',
            icon: 'Archive',
            path: '/assets',
            score: 0,
            max_score: 1,
            percentage: 0,
            percentage_label: '0%',
            missing_items: ['缺少资产台账'],
            description: 'backend missing item',
          },
        ],
        completed_items: [
          {
            key: 'contracts',
            label: '合同台账',
            icon: 'FileText',
            path: '/contracts',
            score: 1,
            max_score: 1,
            percentage: 100,
            percentage_label: '100%',
            missing_items: [],
            description: 'backend completeness',
          },
        ],
        incomplete_items: [
          {
            key: 'assets',
            label: '资产台账',
            icon: 'Archive',
            path: '/assets',
            score: 0,
            max_score: 1,
            percentage: 0,
            percentage_label: '0%',
            missing_items: ['缺少资产台账'],
            description: 'backend missing item',
          },
        ],
        top_incomplete_items: [
          {
            key: 'assets',
            label: '资产台账',
            icon: 'Archive',
            path: '/assets',
            score: 0,
            max_score: 1,
            percentage: 0,
            percentage_label: '0%',
            missing_items: ['缺少资产台账'],
            description: 'backend missing item',
          },
        ],
      },
      summary: {
        total: 1,
        critical: 0,
        high: 1,
        medium: 0,
        low: 0,
      },
    });

    expect(result.insights[0]).toMatchObject({
      type: 'warning',
      priority: 'high',
      icon: '📝',
      displayIcon: 'AlertTriangle',
      priorityBadge: { label: '重要' },
      action: { label: '去处理', path: '/contracts' },
    });
    expect(result.criticalInsights[0]).toMatchObject({
      priority: 'critical',
      action: { label: '去处理', path: '/invoices' },
    });
    expect(result.otherInsights[0]).toMatchObject({
      priority: 'high',
      action: { label: '去处理', path: '/contracts' },
    });
    expect(result.completeness.overall).toBe(70);
    expect(result.completeness.overallLabel).toBe('70%');
    expect(result.completeness.overallGradientClass).toBe('from-yellow-500 to-orange-500');
    expect(result.completeness.unlockTitle).toBe('完善数据可解锁');
    expect(result.completeness.unlockDescription).toContain('智能分析报告');
    expect(result.completeness.completedLabel).toBe('已完成 1/2 个模块');
    expect(result.completeness.incompleteLabel).toBe('还有 1 个模块待完善');
    expect(result.completeness.statusDescription).toContain('后端会据此');
    expect(result.completeness.completedItems).toHaveLength(1);
    expect(result.completeness.incompleteItems).toHaveLength(1);
    expect(result.completeness.topIncompleteItems[0].key).toBe('assets');
    expect(result.completeness.items[0]).toMatchObject({
      maxScore: 1,
      missingItems: [],
      percentageLabel: '100%',
    });
    expect(result.summary.totalInsights).toBe(1);
    expect(result.summary.highCount).toBe(1);
  });

  it('renders completion overview from backend-owned display collections', () => {
    const source = readFileSync(
      resolve(root, 'components/dashboard/CompletionOverview.tsx'),
      'utf8',
    );

    expect(source).toContain('completeness.completedItems');
    expect(source).toContain('completeness.incompleteItems');
    expect(source).toContain('completeness.topIncompleteItems');
    expect(source).toContain('completeness.overallLabel');
    expect(source).toContain('completeness.completedLabel');
    expect(source).toContain('completeness.incompleteLabel');
    expect(source).toContain('completeness.statusTitle');
    expect(source).toContain('completeness.statusDescription');
    expect(source).toContain('module.percentageLabel');
    expect(source).not.toContain('modules.filter');
    expect(source).not.toContain('incompleteModules.slice');
    expect(source).not.toContain('completedModules.length');
    expect(source).not.toContain('incompleteModules.length');
    expect(source).not.toContain('${module.percentage}%');
  });

  it('renders AI insights and data completeness from backend-owned display collections', () => {
    const insightsSource = readFileSync(
      resolve(root, 'components/dashboard/AIInsights.tsx'),
      'utf8',
    );
    const completenessSource = readFileSync(
      resolve(root, 'components/dashboard/DataCompleteness.tsx'),
      'utf8',
    );

    expect(insightsSource).toContain('criticalInsights');
    expect(insightsSource).toContain('otherInsights');
    expect(insightsSource).toContain('insight.priorityBadge');
    expect(insightsSource).toContain('insight.cardBorderClassName');
    expect(insightsSource).toContain('insight.displayIcon');
    expect(insightsSource).toContain('completeness={completeness}');
    expect(insightsSource).not.toContain('typeConfig');
    expect(insightsSource).not.toContain('priorityConfig');
    expect(insightsSource).not.toContain('insights.filter');
    expect(insightsSource).not.toContain('activeInsights');
    expect(insightsSource).not.toContain('insight.priority ===');
    expect(insightsSource).not.toContain("i.priority === 'critical'");
    expect(insightsSource).not.toContain('.slice(0, 6)');

    expect(completenessSource).toContain('completeness.completedItems');
    expect(completenessSource).toContain('completeness.incompleteItems');
    expect(completenessSource).toContain('completeness.topIncompleteItems');
    expect(completenessSource).toContain('completeness.overallGradientClass');
    expect(completenessSource).toContain('completeness.unlockDescription');
    expect(completenessSource).not.toContain('items.filter');
    expect(completenessSource).not.toContain('incompleteItems.slice');
    expect(completenessSource).not.toContain('overall >=');
    expect(completenessSource).not.toContain('percentage >=');
  });
});
