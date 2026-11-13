import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { TrendingUp, BarChart3, PieChart, Activity, ArrowRight } from "lucide-react";
import { Bar, BarChart, ResponsiveContainer, XAxis, YAxis, Tooltip, Legend } from "recharts";
import { useNavigate } from "react-router-dom";

interface MatureStageProps {
  stats: {
    totalAssets: number;
    totalAssetsLabel: string;
    totalValue: number;
    totalValueLabel: string;
    employeeCount: number;
    employeeCountLabel: string;
    totalSalary: number;
    totalSalaryLabel: string;
    ongoingProjects: number;
    ongoingProjectsLabel: string;
    activeContracts: number;
    activeContractsLabel: string;
    totalProjects: number;
    totalProjectsLabel: string;
    totalContracts: number;
    totalContractsLabel: string;
    projectExecutionRateLabel: string;
    contractActivityRateLabel: string;
  };
  monthlyExpenseTrend: Array<{
    month: string;
    salary: number;
    salaryLabel?: string;
    invoice: number;
    invoiceLabel?: string;
    total: number;
    totalLabel?: string;
  }>;
}

export const MatureStage = ({ stats, monthlyExpenseTrend }: MatureStageProps) => {
  const navigate = useNavigate();

  return (
    <div className="space-y-6">
      {/* 全面数据概览 */}
      <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">资产总值</CardTitle>
            <Activity className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalValueLabel}</div>
            <p className="text-xs text-muted-foreground">
              资产数量: {stats.totalAssetsLabel}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">人力成本</CardTitle>
            <TrendingUp className="h-4 w-4 text-accent" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.totalSalaryLabel}</div>
            <p className="text-xs text-muted-foreground">
              团队人数: {stats.employeeCountLabel}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">项目执行率</CardTitle>
            <PieChart className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.projectExecutionRateLabel}
            </div>
            <p className="text-xs text-muted-foreground">
              进行中: {stats.ongoingProjectsLabel}/{stats.totalProjectsLabel}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">合同活跃度</CardTitle>
            <BarChart3 className="h-4 w-4 text-accent" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {stats.contractActivityRateLabel}
            </div>
            <p className="text-xs text-muted-foreground">
              执行中: {stats.activeContractsLabel}/{stats.totalContractsLabel}
            </p>
          </CardContent>
        </Card>
      </div>

      {/* 月度支出趋势 - 重点展示 */}
      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <TrendingUp className="h-5 w-5 text-primary" />
            月度支出趋势分析
          </CardTitle>
          <CardDescription>工资、发票等支出统计（近6个月），用于战略规划</CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={350}>
            <BarChart data={monthlyExpenseTrend}>
              <XAxis dataKey="month" stroke="hsl(var(--muted-foreground))" />
              <YAxis stroke="hsl(var(--muted-foreground))" />
              <Tooltip 
                contentStyle={{ 
                  backgroundColor: "hsl(var(--card))",
                  border: "1px solid hsl(var(--border))",
                  borderRadius: "var(--radius)"
                }}
                formatter={(_, name, item: any) => {
                  const payload = item.payload;
                  if (name === "工资支出") return [payload.salaryLabel, name];
                  if (name === "发票支出") return [payload.invoiceLabel, name];
                  return [payload.totalLabel, name];
                }}
              />
              <Legend />
              <Bar dataKey="salary" fill="hsl(var(--primary))" name="工资支出" radius={[4, 4, 0, 0]} stackId="a" />
              <Bar dataKey="invoice" fill="hsl(var(--accent))" name="发票支出" radius={[4, 4, 0, 0]} stackId="a" />
            </BarChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      {/* 数据分析入口 */}
      <div className="grid gap-4 md:grid-cols-3">
        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/financial-reports')}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <BarChart3 className="h-5 w-5 text-primary" />
              财务报表
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-3">
              查看详细的财务报表和分析
            </p>
            <Button variant="outline" className="w-full">
              进入报表 <ArrowRight className="h-4 w-4 ml-2" />
            </Button>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/budget')}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <Activity className="h-5 w-5 text-accent" />
              预算分析
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-3">
              预算执行情况和优化建议
            </p>
            <Button variant="outline" className="w-full">
              查看预算 <ArrowRight className="h-4 w-4 ml-2" />
            </Button>
          </CardContent>
        </Card>

        <Card className="cursor-pointer hover:shadow-lg transition-shadow" onClick={() => navigate('/projects')}>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-base">
              <PieChart className="h-5 w-5 text-primary" />
              项目统计
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm text-muted-foreground mb-3">
              项目数据分析和绩效评估
            </p>
            <Button variant="outline" className="w-full">
              项目分析 <ArrowRight className="h-4 w-4 ml-2" />
            </Button>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};
