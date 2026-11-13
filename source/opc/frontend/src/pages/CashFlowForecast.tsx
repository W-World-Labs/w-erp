import { Card } from "@/components/ui/card";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { Skeleton } from "@/components/ui/skeleton";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { useState } from "react";
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, Legend, ResponsiveContainer } from "recharts";
import { TrendingUp, TrendingDown, DollarSign, Calendar } from "lucide-react";

const CASHFLOW_FORECAST_ENDPOINT = "/api/v1/reports/cashflow-forecast";

interface CashflowBreakdown {
  receivables: number;
  contract_income: number;
  salary: number;
  rent: number;
  subscription: number;
  payables: number;
}

interface CashflowForecastItem {
  month: string;
  month_label: string;
  month_short_label: string;
  cash_inflow: number;
  cash_outflow: number;
  net_cash_flow: number;
  cash_inflow_label: string;
  cash_outflow_label: string;
  net_cash_flow_label: string;
  net_cash_flow_class: string;
  receivables_label: string;
  contract_income_label: string;
  salary_label: string;
  rent_label: string;
  subscription_label: string;
  payables_label: string;
  breakdown: CashflowBreakdown;
}

interface CashflowChartPoint {
  month: string;
  monthShort: string;
  cashInflow: number;
  cashOutflow: number;
  netCashFlow: number;
  receivables: number;
  contractIncome: number;
  payables: number;
  salary: number;
  rent: number;
  subscription: number;
}

interface CashflowMonthOption {
  value: string;
  label: string;
}

interface CashflowForecastResponse {
  forecast_months: number;
  month_options: CashflowMonthOption[];
  forecast: CashflowForecastItem[];
  chart_points: CashflowChartPoint[];
  summary: {
    total_inflow: number;
    total_outflow: number;
    net_cash_flow: number;
    avg_monthly_net: number;
    total_inflow_label: string;
    total_outflow_label: string;
    net_cash_flow_label: string;
    avg_monthly_net_label: string;
    net_cash_flow_class: string;
    net_cash_flow_icon_class: string;
    avg_monthly_net_class: string;
  };
}

export default function CashFlowForecast() {
  const [forecastMonths, setForecastMonths] = useState<string>("");

  const { data: forecast, isLoading } = useQuery({
    queryKey: ["cashflow-forecast", forecastMonths],
    queryFn: async () => {
      const suffix = forecastMonths ? `?months=${forecastMonths}` : "";
      return api.get<CashflowForecastResponse>(`${CASHFLOW_FORECAST_ENDPOINT}${suffix}`);
    },
  });

  const forecastData = forecast?.chart_points || [];
  const monthOptions = forecast?.month_options ?? [];
  const rows = forecast?.forecast || [];
  const summary = forecast?.summary || {
    total_inflow: 0,
    total_outflow: 0,
    net_cash_flow: 0,
    avg_monthly_net: 0,
    total_inflow_label: "¥0",
    total_outflow_label: "¥0",
    net_cash_flow_label: "¥0",
    avg_monthly_net_label: "¥0",
    net_cash_flow_class: "text-green-600",
    net_cash_flow_icon_class: "bg-green-500/10 text-green-600",
    avg_monthly_net_class: "text-green-600",
  };

  if (isLoading) {
    return (
      <div className="container mx-auto p-6 space-y-6">
        <Skeleton className="h-12 w-64" />
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
          {[1, 2, 3, 4].map((i) => (
            <Skeleton key={i} className="h-32" />
          ))}
        </div>
        <Skeleton className="h-96" />
      </div>
    );
  }

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">现金流预测</h1>
          <p className="text-muted-foreground mt-2">
            基于应收应付、工资、租金等数据预测未来现金流情况
          </p>
        </div>
        <Select value={forecastMonths || String(forecast?.forecast_months ?? "")} onValueChange={setForecastMonths}>
          <SelectTrigger className="w-40">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {monthOptions.map((option) => (
              <SelectItem key={option.value} value={option.value}>
                {option.label}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
      </div>

      {/* 汇总统计卡片 */}
      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">预计总流入</p>
              <p className="text-2xl font-bold text-foreground mt-1">
                {summary.total_inflow_label}
              </p>
            </div>
            <div className="p-3 bg-green-500/10 rounded-full">
              <TrendingUp className="h-6 w-6 text-green-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">预计总流出</p>
              <p className="text-2xl font-bold text-foreground mt-1">
                {summary.total_outflow_label}
              </p>
            </div>
            <div className="p-3 bg-red-500/10 rounded-full">
              <TrendingDown className="h-6 w-6 text-red-600" />
            </div>
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">净现金流</p>
              <p className={`text-2xl font-bold mt-1 ${summary.net_cash_flow_class}`}>
                {summary.net_cash_flow_label}
              </p>
            </div>
            <div className={`p-3 rounded-full ${summary.net_cash_flow_icon_class}`}>
              <DollarSign className="h-6 w-6" />
            </div>
          </div>
        </Card>

        <Card className="p-6">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm text-muted-foreground">月均净流量</p>
              <p className={`text-2xl font-bold mt-1 ${summary.avg_monthly_net_class}`}>
                {summary.avg_monthly_net_label}
              </p>
            </div>
            <div className="p-3 bg-primary/10 rounded-full">
              <Calendar className="h-6 w-6 text-primary" />
            </div>
          </div>
        </Card>
      </div>

      {/* 现金流趋势图 */}
      <Card className="p-6">
        <h2 className="text-xl font-semibold text-foreground mb-4">现金流趋势</h2>
        <ResponsiveContainer width="100%" height={400}>
          <LineChart data={forecastData}>
            <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
            <XAxis dataKey="monthShort" stroke="hsl(var(--muted-foreground))" />
            <YAxis stroke="hsl(var(--muted-foreground))" />
            <Tooltip
              contentStyle={{
                backgroundColor: "hsl(var(--background))",
                border: "1px solid hsl(var(--border))",
                borderRadius: "8px",
              }}
            />
            <Legend />
            <Line
              type="monotone"
              dataKey="cashInflow"
              stroke="hsl(142 76% 36%)"
              strokeWidth={2}
              name="现金流入"
              dot={{ fill: "hsl(142 76% 36%)", r: 4 }}
            />
            <Line
              type="monotone"
              dataKey="cashOutflow"
              stroke="hsl(0 84% 60%)"
              strokeWidth={2}
              name="现金流出"
              dot={{ fill: "hsl(0 84% 60%)", r: 4 }}
            />
            <Line
              type="monotone"
              dataKey="netCashFlow"
              stroke="hsl(var(--primary))"
              strokeWidth={3}
              name="净现金流"
              dot={{ fill: "hsl(var(--primary))", r: 5 }}
            />
          </LineChart>
        </ResponsiveContainer>
      </Card>

      {/* 现金流出结构 */}
      <Card className="p-6">
        <h2 className="text-xl font-semibold text-foreground mb-4">现金流出结构预测</h2>
        <ResponsiveContainer width="100%" height={400}>
          <BarChart data={forecastData}>
            <CartesianGrid strokeDasharray="3 3" stroke="hsl(var(--border))" />
            <XAxis dataKey="monthShort" stroke="hsl(var(--muted-foreground))" />
            <YAxis stroke="hsl(var(--muted-foreground))" />
            <Tooltip
              contentStyle={{
                backgroundColor: "hsl(var(--background))",
                border: "1px solid hsl(var(--border))",
                borderRadius: "8px",
              }}
            />
            <Legend />
            <Bar dataKey="salary" stackId="a" fill="hsl(221 83% 53%)" name="工资成本" />
            <Bar dataKey="rent" stackId="a" fill="hsl(280 100% 70%)" name="租金" />
            <Bar dataKey="subscription" stackId="a" fill="hsl(48 96% 53%)" name="订阅服务" />
            <Bar dataKey="payables" stackId="a" fill="hsl(0 84% 60%)" name="应付账款" />
          </BarChart>
        </ResponsiveContainer>
      </Card>

      {/* 详细数据表格 */}
      <Card className="p-6">
        <h2 className="text-xl font-semibold text-foreground mb-4">详细预测数据</h2>
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left py-3 px-4 text-sm font-semibold text-foreground">月份</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">应收账款</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">合同收入</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">工资</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">租金</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">订阅服务</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">应付账款</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">总流入</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">总流出</th>
                <th className="text-right py-3 px-4 text-sm font-semibold text-foreground">净现金流</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row, index) => (
                <tr key={index} className="border-b border-border hover:bg-muted/50 transition-colors">
                  <td className="py-3 px-4 text-sm text-foreground">{row.month_label}</td>
                  <td className="text-right py-3 px-4 text-sm text-foreground">
                    {row.receivables_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm text-foreground">
                    {row.contract_income_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm text-foreground">
                    {row.salary_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm text-foreground">
                    {row.rent_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm text-foreground">
                    {row.subscription_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm text-foreground">
                    {row.payables_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm font-medium text-green-600">
                    {row.cash_inflow_label}
                  </td>
                  <td className="text-right py-3 px-4 text-sm font-medium text-red-600">
                    {row.cash_outflow_label}
                  </td>
                  <td className={`text-right py-3 px-4 text-sm font-semibold ${row.net_cash_flow_class}`}>
                    {row.net_cash_flow_label}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </Card>
    </div>
  );
}
