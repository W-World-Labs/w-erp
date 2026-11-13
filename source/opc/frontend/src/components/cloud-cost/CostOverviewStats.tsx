import { useState, useEffect } from "react";
import { api } from "@/integrations/api/client";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  Cloud,
  Server,
  Globe,
  Package,
  DollarSign,
  TrendingUp,
} from "lucide-react";

const CLOUD_COST_OVERVIEW_ENDPOINT = "/api/v1/cloud_cost/overview";

interface CloudCostOverview {
  total_providers: number;
  total_resources: number;
  total_domains: number;
  total_software: number;
  monthly_cloud_cost_label: string;
  monthly_domain_cost_label: string;
  monthly_software_cost_label: string;
  total_monthly_cost_label: string;
}

const emptyCloudCostOverview: CloudCostOverview = {
  total_providers: 0,
  total_resources: 0,
  total_domains: 0,
  total_software: 0,
  monthly_cloud_cost_label: "¥0",
  monthly_domain_cost_label: "¥0",
  monthly_software_cost_label: "¥0",
  total_monthly_cost_label: "¥0",
};

const CostOverviewStats = () => {
  const [stats, setStats] = useState<CloudCostOverview>(emptyCloudCostOverview);

  useEffect(() => {
    const fetchStats = async () => {
      try {
        const overview = await api.get<CloudCostOverview>(
          CLOUD_COST_OVERVIEW_ENDPOINT,
        );
        setStats(overview || emptyCloudCostOverview);
      } catch (error: any) {
        console.error("Failed to fetch stats:", error);
      }
    };

    fetchStats();
  }, []);

  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">云服务商</CardTitle>
          <Cloud className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats.total_providers}</div>
          <p className="text-xs text-muted-foreground">活跃账户数</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">云资源</CardTitle>
          <Server className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats.total_resources}</div>
          <p className="text-xs text-muted-foreground">运行中实例</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">域名</CardTitle>
          <Globe className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats.total_domains}</div>
          <p className="text-xs text-muted-foreground">活跃域名数</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">软件订阅</CardTitle>
          <Package className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">{stats.total_software}</div>
          <p className="text-xs text-muted-foreground">使用中订阅</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">云服务月成本</CardTitle>
          <DollarSign className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {stats.monthly_cloud_cost_label}
          </div>
          <p className="text-xs text-muted-foreground">每月云资源费用</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">域名月成本</CardTitle>
          <DollarSign className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {stats.monthly_domain_cost_label}
          </div>
          <p className="text-xs text-muted-foreground">每月域名费用</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">软件月成本</CardTitle>
          <DollarSign className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {stats.monthly_software_cost_label}
          </div>
          <p className="text-xs text-muted-foreground">每月软件订阅费用</p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">总月成本</CardTitle>
          <TrendingUp className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold text-primary">
            {stats.total_monthly_cost_label}
          </div>
          <p className="text-xs text-muted-foreground">所有订阅月均成本</p>
        </CardContent>
      </Card>
    </div>
  );
};

export default CostOverviewStats;
