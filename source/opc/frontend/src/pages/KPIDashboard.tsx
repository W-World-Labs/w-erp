import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api } from "@/integrations/api/client";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { TrendingUp, Plus, Settings } from "lucide-react";
import { Skeleton } from "@/components/ui/skeleton";
import { KPIMetricCard } from "@/components/kpi/KPIMetricCard";
import { KPIChart } from "@/components/kpi/KPIChart";
import { KPIConfigDialog } from "@/components/kpi/KPIConfigDialog";
import { KPIAlerts, type KpiAlert } from "@/components/kpi/KPIAlerts";

const KPI_DASHBOARD_ENDPOINT = "/api/v1/kpi/dashboard";

interface KpiDisplayBadge {
  label: string;
  class_name: string;
  icon?: string | null;
}

interface KpiChartPoint {
  value: number;
  value_label: string;
  period: string;
  period_label: string;
  recorded_at?: string | null;
}

interface KpiRuntimeItem {
  id: string;
  name: string;
  category?: string | null;
  unit?: string | null;
  target_value?: number | null;
  current_value: number;
  previous_value: number;
  chart_points: KpiChartPoint[];
  current_value_label: string;
  previous_value_label: string;
  change_label: string;
  change_icon: string;
  change_class: string;
  has_previous_value: boolean;
  target_value_label?: string | null;
  alert_type: "normal" | "warning" | "danger";
  alert_badge: KpiDisplayBadge;
  chart_empty_message: string;
}

interface KpiCategoryOption {
  value: string;
  label: string;
  items: KpiRuntimeItem[];
  chart_items: KpiRuntimeItem[];
  empty_message: string;
}

interface KpiDashboardData {
  period: string;
  kpis: KpiRuntimeItem[];
  alerts: KpiAlert[];
  category_options: KpiCategoryOption[];
  empty_message: string;
}

const EMPTY_CATEGORY: KpiCategoryOption = {
  value: "all",
  label: "全部指标",
  items: [],
  chart_items: [],
  empty_message: '暂无KPI指标，请点击"新增指标"创建',
};

const KPIDashboard = () => {
  const [configDialogOpen, setConfigDialogOpen] = useState(false);
  const [selectedCategory, setSelectedCategory] = useState<string>("all");

  const { data: dashboardData, isLoading, refetch } = useQuery({
    queryKey: ['kpi-dashboard'],
    queryFn: async () => {
      return api.get<KpiDashboardData>(KPI_DASHBOARD_ENDPOINT);
    },
  });

  const alerts = dashboardData?.alerts || [];
  const categories = dashboardData?.category_options || [EMPTY_CATEGORY];
  const selectedCategoryData =
    categories.find((category) => category.value === selectedCategory) || categories[0] || EMPTY_CATEGORY;
  const categoryItems = selectedCategoryData.items;
  const categoryChartItems = selectedCategoryData.chart_items;

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex justify-between items-center">
          <div>
            <Skeleton className="h-8 w-48 mb-2" />
            <Skeleton className="h-4 w-64" />
          </div>
          <Skeleton className="h-10 w-32" />
        </div>
        <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
          {[1, 2, 3, 4].map((i) => (
            <Skeleton key={i} className="h-32" />
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* 页头 */}
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-foreground flex items-center gap-2">
            <TrendingUp className="h-8 w-8 text-primary" />
            经营数据看板
          </h1>
          <p className="text-muted-foreground mt-2">实时监控企业关键经营指标</p>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => setConfigDialogOpen(true)}
            className="flex items-center gap-2"
          >
            <Settings className="h-4 w-4" />
            KPI配置
          </Button>
          <Button
            onClick={() => setConfigDialogOpen(true)}
            className="flex items-center gap-2"
          >
            <Plus className="h-4 w-4" />
            新增指标
          </Button>
        </div>
      </div>

      {/* KPI预警 */}
      <KPIAlerts alerts={alerts} />

      {/* 分类选择 */}
      <Tabs value={selectedCategory} onValueChange={setSelectedCategory}>
        <TabsList>
          {categories.map(category => (
            <TabsTrigger
              key={category.value}
              value={category.value}
              data-count={category.items.length}
              data-chart-count={category.chart_items.length}
            >
              {category.label}
            </TabsTrigger>
          ))}
        </TabsList>

        <TabsContent value={selectedCategory} className="space-y-6">
          {/* KPI指标卡片 */}
          <div className="grid gap-4 md:grid-cols-2 lg:grid-cols-4">
            {categoryItems.length === 0 ? (
              <Card className="col-span-full">
                <CardContent className="py-8 text-center text-muted-foreground">
                  {selectedCategoryData.empty_message}
                </CardContent>
              </Card>
            ) : (
              categoryItems.map(kpi => (
                <KPIMetricCard
                  key={kpi.id}
                  config={kpi}
                />
              ))
            )}
          </div>

          {/* KPI趋势图表 */}
          {categoryChartItems.length > 0 && (
            <div className="grid gap-4 md:grid-cols-2">
              {categoryChartItems.map(kpi => (
                <KPIChart
                  key={kpi.id}
                  config={kpi}
                />
              ))}
            </div>
          )}
        </TabsContent>
      </Tabs>

      {/* KPI配置对话框 */}
      <KPIConfigDialog
        open={configDialogOpen}
        onOpenChange={setConfigDialogOpen}
        onSuccess={refetch}
      />
    </div>
  );
};

export default KPIDashboard;
