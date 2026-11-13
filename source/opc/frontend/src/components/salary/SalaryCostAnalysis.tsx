import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Legend,
  Line,
  LineChart,
  Pie,
  PieChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import {
  DollarSign,
  PieChart as PieChartIcon,
  TrendingUp,
  Users,
} from "lucide-react";

const COLORS = [
  "hsl(var(--primary))",
  "hsl(var(--accent))",
  "hsl(var(--secondary))",
  "hsl(var(--chart-1))",
  "hsl(var(--chart-2))",
  "hsl(var(--chart-3))",
  "hsl(var(--chart-4))",
  "hsl(var(--chart-5))",
];

interface SalaryCostAnalysisProps {
  analysis: {
    stats: {
      total_cost: number;
      total_cost_label: string;
      employee_cost: number;
      employee_cost_label: string;
      company_cost: number;
      company_cost_label: string;
      departments: number;
      departments_label: string;
      month_over_month_label: string;
      employee_cost_share_label: string;
    };
    monthly_trend: Array<{
      month: string;
      employee_cost: number;
      employee_cost_label: string;
      social_security_cost: number;
      social_security_cost_label: string;
      housing_fund_cost: number;
      housing_fund_cost_label: string;
      total_cost: number;
      total_cost_label: string;
    }>;
    department_costs: Array<{
      name: string;
      value: number;
      value_label: string;
      percent_label: string;
    }>;
  };
}

const trendLabel = (payload: any, key: string) => {
  const labelKey = `${key}_label`;
  return payload?.[labelKey] || "";
};

export const SalaryCostAnalysis = ({ analysis }: SalaryCostAnalysisProps) => {
  const stats = analysis.stats;
  const monthlyTrend = analysis.monthly_trend;
  const departmentCosts = analysis.department_costs;

  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">人力总成本</CardTitle>
            <DollarSign className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {stats.total_cost_label}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              {stats.month_over_month_label} 环比上月
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">员工实发</CardTitle>
            <Users className="h-4 w-4 text-accent" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.employee_cost_label}
            </div>
            <p className="text-xs text-muted-foreground mt-1">
              占总成本 {stats.employee_cost_share_label}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">公司承担</CardTitle>
            <TrendingUp className="h-4 w-4 text-secondary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.company_cost_label}</div>
            <p className="text-xs text-muted-foreground mt-1">
              社保公积金企业部分
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">部门数量</CardTitle>
            <PieChartIcon className="h-4 w-4 text-chart-1" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.departments_label}</div>
            <p className="text-xs text-muted-foreground mt-1">活跃部门统计</p>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>月度成本趋势</CardTitle>
            <CardDescription>人力成本构成及变化趋势</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={monthlyTrend}>
                <CartesianGrid
                  strokeDasharray="3 3"
                  stroke="hsl(var(--border))"
                />
                <XAxis dataKey="month" stroke="hsl(var(--muted-foreground))" />
                <YAxis stroke="hsl(var(--muted-foreground))" />
                <Tooltip
                  contentStyle={{
                    backgroundColor: "hsl(var(--card))",
                    border: "1px solid hsl(var(--border))",
                    borderRadius: "var(--radius)",
                  }}
                  formatter={(_value, key, item: any) =>
                    trendLabel(item?.payload, String(key))
                  }
                />
                <Legend />
                <Bar
                  dataKey="employee_cost"
                  fill="hsl(var(--primary))"
                  name="员工实发"
                  stackId="a"
                />
                <Bar
                  dataKey="social_security_cost"
                  fill="hsl(var(--accent))"
                  name="公司社保"
                  stackId="a"
                />
                <Bar
                  dataKey="housing_fund_cost"
                  fill="hsl(var(--secondary))"
                  name="公司公积金"
                  stackId="a"
                  radius={[4, 4, 0, 0]}
                />
              </BarChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>部门成本分布</CardTitle>
            <CardDescription>各部门人力成本占比</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <PieChart>
                <Pie
                  data={departmentCosts}
                  cx="50%"
                  cy="50%"
                  labelLine={false}
                  label={(entry: any) => `${entry.name} ${entry.percent_label}`}
                  outerRadius={100}
                  fill="hsl(var(--primary))"
                  dataKey="value"
                >
                  {departmentCosts.map((entry, index) => (
                    <Cell
                      key={entry.name}
                      fill={COLORS[index % COLORS.length]}
                    />
                  ))}
                </Pie>
                <Tooltip
                  contentStyle={{
                    backgroundColor: "hsl(var(--card))",
                    border: "1px solid hsl(var(--border))",
                    borderRadius: "var(--radius)",
                  }}
                  formatter={(_value, _name, item: any) =>
                    item?.payload?.value_label || ""
                  }
                />
              </PieChart>
            </ResponsiveContainer>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>人力总成本趋势</CardTitle>
          <CardDescription>
            含员工工资、公司社保及公积金的总人力成本
          </CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={250}>
            <LineChart data={monthlyTrend}>
              <CartesianGrid
                strokeDasharray="3 3"
                stroke="hsl(var(--border))"
              />
              <XAxis dataKey="month" stroke="hsl(var(--muted-foreground))" />
              <YAxis stroke="hsl(var(--muted-foreground))" />
              <Tooltip
                contentStyle={{
                  backgroundColor: "hsl(var(--card))",
                  border: "1px solid hsl(var(--border))",
                  borderRadius: "var(--radius)",
                }}
                formatter={(_value, _name, item: any) =>
                  item?.payload?.total_cost_label || ""
                }
              />
              <Legend />
              <Line
                type="monotone"
                dataKey="total_cost"
                stroke="hsl(var(--primary))"
                strokeWidth={3}
                name="总成本"
                dot={{ fill: "hsl(var(--primary))", r: 5 }}
              />
            </LineChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>
    </div>
  );
};
