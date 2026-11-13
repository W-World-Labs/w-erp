import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '..');

describe('backend-owned KPI dashboard', () => {
  it('loads KPI runtime values from the backend aggregate endpoint', () => {
    const source = readFileSync(resolve(root, 'pages/KPIDashboard.tsx'), 'utf8');

    expect(source).toContain('/api/v1/kpi/dashboard');
    expect(source).toContain('category_options');
    expect(source).toContain('category.items');
    expect(source).toContain('category.chart_items');
    expect(source).not.toContain('kpis.filter');
    expect(source).not.toContain('alerts.find');
    expect(source).not.toContain('getAlertType');
    expect(source).not.toContain('.slice(0, 4)');
    expect(source).not.toContain('business-data-for-kpi');
    expect(source).not.toContain('calculateKPIValue');
    expect(source).not.toContain('getMonthRevenue');
    expect(source).not.toContain('getMonthCost');
    expect(source).not.toContain('API_ENDPOINTS.invoices');
    expect(source).not.toContain('API_ENDPOINTS.salary');
    expect(source).not.toContain('API_ENDPOINTS.projects');
    expect(source).not.toContain('API_ENDPOINTS.assets');
  });

  it('renders backend KPI alerts without recomputing thresholds in UI widgets', () => {
    const alertsSource = readFileSync(resolve(root, 'components/kpi/KPIAlerts.tsx'), 'utf8');
    const cardSource = readFileSync(resolve(root, 'components/kpi/KPIMetricCard.tsx'), 'utf8');
    const chartSource = readFileSync(resolve(root, 'components/kpi/KPIChart.tsx'), 'utf8');

    expect(alertsSource).toContain('alerts');
    expect(alertsSource).toContain('alert.variant');
    expect(alertsSource).toContain('alert.badge.icon');
    expect(alertsSource).not.toContain('calculateValue');
    expect(alertsSource).not.toContain('danger_threshold');
    expect(alertsSource).not.toContain('warning_threshold');
    expect(alertsSource).not.toContain('alert.type ===');
    expect(cardSource).toContain('current_value_label');
    expect(cardSource).toContain('change_label');
    expect(cardSource).toContain('change_icon');
    expect(cardSource).toContain('target_value_label');
    expect(cardSource).toContain('alert_badge');
    expect(cardSource).not.toContain('alertType');
    expect(cardSource).not.toContain('getAlertStatus');
    expect(cardSource).not.toContain('danger_threshold');
    expect(cardSource).not.toContain('warning_threshold');
    expect(cardSource).not.toContain('toFixed');
    expect(cardSource).not.toContain('formatValue');
    expect(cardSource).not.toContain('((currentValue - previousValue)');

    expect(chartSource).toContain('chart_points');
    expect(chartSource).toContain('point.period_label');
    expect(chartSource).toContain('point.value_label');
    expect(chartSource).not.toContain('safeFormat');
    expect(chartSource).not.toContain('.sort((a, b)');
    expect(chartSource).not.toContain('.slice(-6)');
    expect(chartSource).not.toContain('toFixed');
    expect(chartSource).not.toContain('formatValue');
  });

  it('keeps KPI config threshold normalization on the backend', () => {
    const source = readFileSync(resolve(root, 'components/kpi/KPIConfigDialog.tsx'), 'utf8');

    expect(source).toContain('/api/v1/kpi_configs/form_options');
    expect(source).toContain('formOptions?.category_options');
    expect(source).toContain('formOptions?.unit_options');
    expect(source).toContain('formOptions?.default_values');
    expect(source).toContain('formOptions?.create_action');
    expect(source).toContain('type BackendAction');
    expect(source).toContain('api.runAction(action');
    expect(source).not.toContain('parseFloat');
    expect(source).not.toContain('target_value: values.target_value ? parse');
    expect(source).not.toContain('warning_threshold: values.warning_threshold ? parse');
    expect(source).not.toContain('danger_threshold: values.danger_threshold ? parse');
    expect(source).not.toContain("api.post('/api/kpi_configs'");
    expect(source).not.toContain('api.post("/api/kpi_configs"');
    expect(source).not.toContain('user_id: user.id');
    expect(source).not.toContain('<SelectItem value="financial">');
    expect(source).not.toContain('<SelectItem value="元">');
    expect(source).not.toContain('category: "financial"');
    expect(source).not.toContain('unit: "元"');
  });
});
