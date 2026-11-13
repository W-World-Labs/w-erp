import { describe, expect, it } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';

const root = resolve(__dirname, '..');

describe('backend-owned cash flow forecast', () => {
  it('loads forecast values from the backend report endpoint', () => {
    const source = readFileSync(resolve(root, 'pages/CashFlowForecast.tsx'), 'utf8');

    expect(source).toContain('/api/v1/reports/cashflow-forecast');
    expect(source).toContain('forecast?.chart_points');
    expect(source).toContain('forecast?.month_options');
    expect(source).toContain('summary.total_inflow_label');
    expect(source).toContain('summary.total_outflow_label');
    expect(source).toContain('summary.net_cash_flow_label');
    expect(source).toContain('summary.avg_monthly_net_label');
    expect(source).toContain('summary.net_cash_flow_class');
    expect(source).toContain('row.month_label');
    expect(source).toContain('row.receivables_label');
    expect(source).toContain('row.contract_income_label');
    expect(source).toContain('row.salary_label');
    expect(source).toContain('row.rent_label');
    expect(source).toContain('row.subscription_label');
    expect(source).toContain('row.payables_label');
    expect(source).toContain('row.cash_inflow_label');
    expect(source).toContain('row.cash_outflow_label');
    expect(source).toContain('row.net_cash_flow_label');
    expect(source).not.toContain('calculateForecast');
    expect(source).not.toContain('API_ENDPOINTS.invoices');
    expect(source).not.toContain('API_ENDPOINTS.salary');
    expect(source).not.toContain('API_ENDPOINTS.rent');
    expect(source).not.toContain('API_ENDPOINTS.contracts');
    expect(source).not.toContain('API_ENDPOINTS.assets');
    expect(source).not.toContain('monthlySalaryBaseline');
    expect(source).not.toContain('monthlyContractIncome');
    expect(source).not.toContain('monthReceivables');
    expect(source).not.toContain('monthPayables');
    expect(source).not.toContain('date-fns');
    expect(source).not.toContain('safeFormat');
    expect(source).not.toContain('toLocaleString');
    expect(source).not.toContain('Math.round');
    expect(source).not.toContain('totalNet >= 0');
    expect(source).not.toContain('avgMonthlyNet >= 0');
    expect(source).not.toContain('<SelectItem value="3">');
    expect(source).not.toContain('<SelectItem value="6">');
  });
});
