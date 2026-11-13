import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { toast } from "sonner";
import { api } from "@/integrations/api/client";
import { useMonthlyClose } from "@/hooks/useMonthlyClose";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { Skeleton } from "@/components/ui/skeleton";
import { QueryStateGate } from "@/components/common/QueryStateGate";
import {
  RadarChart,
  PolarGrid,
  PolarAngleAxis,
  PolarRadiusAxis,
  Radar,
  Legend,
  ResponsiveContainer,
  BarChart,
  Bar,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
} from "recharts";
import {
  TrendingUp,
  TrendingDown,
  AlertCircle,
  CheckCircle2,
  FileText,
  Loader2,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

interface FinancialBadge {
  label: string;
  class_name: string;
}

interface FinancialTrend {
  icon: string;
  class_name: string;
  tone: "positive" | "negative";
}

const FINANCIAL_HEALTH_ENDPOINT = "/api/v1/reports/financial-health";

function currentPeriod() {
  const now = new Date();
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, "0")}`;
}

interface FinancialMetrics {
  asset_liability_ratio: number;
  asset_liability_ratio_label: string;
  current_ratio: number;
  current_ratio_label: string;
  profit_margin: number;
  profit_margin_label: string;
  profit_margin_trend: FinancialTrend;
  operating_cash_flow: number;
  operating_cash_flow_label: string;
  operating_cash_flow_trend: FinancialTrend;
  debt_to_equity_ratio: number;
  debt_to_equity_ratio_label: string;
}

interface HealthScore {
  total: number;
  total_label: string;
  scale_label: string;
  level: "excellent" | "good" | "fair" | "poor";
  badge: FinancialBadge;
  breakdown: {
    asset_liability: number;
    liquidity: number;
    profitability: number;
    cash_flow: number;
  };
}

interface Recommendation {
  type: "warning" | "success";
  message: string;
  variant: "default" | "destructive";
  icon: string;
}

interface RadarPoint {
  metric: string;
  score: number;
  full_mark?: number;
  fullMark: number;
  score_label: string;
}

interface BarPoint {
  name: string;
  value: number;
  value_label: string;
  ideal: number;
  ideal_label: string;
}

interface FinancialHealthReport {
  metrics: FinancialMetrics;
  health_score: HealthScore;
  recommendations: Recommendation[];
  charts: {
    radar: RadarPoint[];
    bar: BarPoint[];
  };
}

export default function FinancialReports() {
  const [period, setPeriod] = useState(currentPeriod());
  const {
    data: report,
    isLoading,
    isError,
    refetch,
  } = useQuery({
    queryKey: ["financial-health-report"],
    queryFn: async () =>
      api.get<FinancialHealthReport>(FINANCIAL_HEALTH_ENDPOINT),
  });
  const { reports, isLoadingReports, generateClose, isGenerating } =
    useMonthlyClose();

  const handleGenerateClose = async () => {
    try {
      const result = await generateClose(period);
      toast.success(
        result.empty
          ? `${result.period} 占位关账已归档`
          : `${result.period} 关账完成：净额 ${result.net_label}`,
      );
    } catch (error) {
      toast.error(error instanceof Error ? error.message : "生成月度关账失败");
    }
  };

  const metrics = report?.metrics;
  const healthScore = report?.health_score;
  const recommendations = report?.recommendations || [];
  const radarData = report?.charts.radar || [];
  const barData = report?.charts.bar || [];

  const trendIcons: Record<string, LucideIcon> = {
    TrendingDown,
    TrendingUp,
  };
  const recommendationIcons: Record<string, LucideIcon> = {
    AlertCircle,
    CheckCircle2,
  };

  const TrendIcon = ({ trend }: { trend: FinancialTrend }) => {
    const Icon = trendIcons[trend.icon] || TrendingUp;
    return <Icon className={trend.class_name} />;
  };

  const loadingFallback = (
    <div className="container mx-auto p-6 space-y-6">
      <Skeleton className="h-10 w-64" />
      <div className="grid gap-6 md:grid-cols-2">
        <Skeleton className="h-72" />
        <Skeleton className="h-72" />
      </div>
      <Skeleton className="h-96" />
    </div>
  );

  return (
    <QueryStateGate
      isLoading={isLoading}
      isError={isError}
      refetch={refetch}
      loadingFallback={loadingFallback}
    >
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold">财务健康评分</h1>
          <p className="text-muted-foreground">
            综合分析企业财务状况，给出健康度评分和改善建议
          </p>
        </div>
      </div>

      <Card>
        <CardHeader>
          <CardTitle className="flex items-center gap-2">
            <FileText className="h-5 w-5" />
            月度关账包
          </CardTitle>
          <CardDescription>
            一键汇总当月收支、应收账龄、税务与银行核对，生成可归档 PDF
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="flex flex-wrap items-end gap-3">
            <div className="space-y-1">
              <div className="text-sm text-muted-foreground">期间</div>
              <Input
                type="month"
                value={period}
                onChange={(event) => setPeriod(event.target.value)}
                className="w-44"
              />
            </div>
            <Button
              onClick={() => void handleGenerateClose()}
              disabled={isGenerating || !period}
            >
              {isGenerating ? (
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
              ) : (
                <FileText className="mr-2 h-4 w-4" />
              )}
              生成月度报告
            </Button>
          </div>
          <div className="space-y-2">
            {reports.map((item) => (
              <div key={item.id} className="flex items-center justify-between rounded-lg bg-muted px-3 py-2 text-sm">
                <span>{item.period} · {item.original_name}</span>
                <a className="text-primary underline" href={item.url}>
                  下载
                </a>
              </div>
            ))}
            {!isLoadingReports && reports.length === 0 && (
              <p className="text-sm text-muted-foreground">还没有关账报告。</p>
            )}
          </div>
        </CardContent>
      </Card>

      {healthScore && (
        <div className="grid gap-6 md:grid-cols-2">
          <Card>
            <CardHeader>
              <CardTitle className="flex items-center justify-between">
                综合健康评分
                <Badge className={healthScore.badge.class_name}>
                  {healthScore.badge.label}
                </Badge>
              </CardTitle>
              <CardDescription>
                基于资产负债率、流动比率、利润率和现金流综合评估
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="space-y-4">
                <div className="text-center">
                  <div className="text-6xl font-bold text-primary">
                    {healthScore.total_label}
                  </div>
                  <div className="text-muted-foreground">
                    {healthScore.scale_label}
                  </div>
                </div>
                <Progress value={healthScore.total} className="h-3" />
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>指标评分雷达图</CardTitle>
              <CardDescription>各项财务指标的健康度评分</CardDescription>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={250}>
                <RadarChart data={radarData}>
                  <PolarGrid />
                  <PolarAngleAxis dataKey="metric" />
                  <PolarRadiusAxis angle={90} domain={[0, 100]} />
                  <Radar
                    name="评分"
                    dataKey="score"
                    stroke="hsl(var(--primary))"
                    fill="hsl(var(--primary))"
                    fillOpacity={0.6}
                  />
                  <Legend />
                </RadarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>
        </div>
      )}

      {metrics && (
        <Card>
          <CardHeader>
            <CardTitle>财务指标详情</CardTitle>
            <CardDescription>关键财务指标的具体数值和对比</CardDescription>
          </CardHeader>
          <CardContent>
            <ResponsiveContainer width="100%" height={300}>
              <BarChart data={barData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="name" />
                <YAxis />
                <Tooltip />
                <Legend />
                <Bar dataKey="value" fill="hsl(var(--primary))" name="实际值" />
                <Bar dataKey="ideal" fill="hsl(var(--muted))" name="理想值" />
              </BarChart>
            </ResponsiveContainer>
            <div className="mt-6 grid gap-4 md:grid-cols-2 lg:grid-cols-4">
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium text-muted-foreground">
                    资产负债率
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">
                    {metrics.asset_liability_ratio_label}
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    理想值: &lt; 60%
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium text-muted-foreground">
                    流动比率
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">
                    {metrics.current_ratio_label}
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    理想值: &gt; 2.0
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium text-muted-foreground">
                    利润率
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold flex items-center">
                    {metrics.profit_margin_label}
                    <TrendIcon trend={metrics.profit_margin_trend} />
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    理想值: &gt; 20%
                  </p>
                </CardContent>
              </Card>
              <Card>
                <CardHeader className="pb-2">
                  <CardTitle className="text-sm font-medium text-muted-foreground">
                    经营现金流
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold flex items-center">
                    {metrics.operating_cash_flow_label}
                    <TrendIcon trend={metrics.operating_cash_flow_trend} />
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    理想值: &gt; 0
                  </p>
                </CardContent>
              </Card>
            </div>
          </CardContent>
        </Card>
      )}

      <Card>
        <CardHeader>
          <CardTitle>改善建议</CardTitle>
          <CardDescription>基于当前财务状况的针对性建议</CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {recommendations.map((rec, index) => {
            const RecommendationIcon = recommendationIcons[rec.icon] || CheckCircle2;
            return (
              <Alert key={`${rec.type}:${index}`} variant={rec.variant}>
                <RecommendationIcon className="h-4 w-4" />
                <AlertDescription>{rec.message}</AlertDescription>
              </Alert>
            );
          })}
        </CardContent>
      </Card>
    </div>
    </QueryStateGate>
  );
}
