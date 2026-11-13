import { useEffect, useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Progress } from "@/components/ui/progress";
import {
  Activity,
  Database,
  Globe,
  RefreshCw,
  XCircle,
  Server,
  HardDrive,
  Layers,
  Timer,
  ChevronDown,
  ChevronUp,
} from "lucide-react";
import { api } from "@/integrations/api/client";

interface DisplayBadge {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  class_name?: string;
}

interface TableStat {
  table: string;
  label: string;
  count_label: string;
  status_badge: DisplayBadge;
  card_class?: string;
  error?: string;
}

interface ApiEndpoint {
  method: string;
  path: string;
  name: string;
  status_badge: DisplayBadge;
}

interface SystemStatusData {
  health: {
    version: string;
    timestamp_label: string;
    status_badge: DisplayBadge;
    database_badge: DisplayBadge;
    database_label: string;
    database_text_class?: string;
    status_background_class?: string;
    status_text_class?: string;
    status_border_class?: string;
  };
  database: {
    file_size_label: string;
    total_rows_label: string;
    table_count_label: string;
    summary_label: string;
    health_percent: number;
    health_percent_label: string;
    error_table_count: number;
    tables: TableStat[];
  };
  storage: {
    data_files_label: string;
  };
  api: {
    total_endpoints_label: string;
    checked_count: number;
    ok_count: number;
    error_count: number;
    health_percent: number;
    health_percent_label: string;
    summary_label: string;
    endpoints: ApiEndpoint[];
  };
}

function SystemBadge({ badge }: { badge?: DisplayBadge | null }) {
  if (!badge) return null;
  return (
    <Badge
      variant={badge.variant || "outline"}
      className={badge.class_name || ""}
    >
      {badge.label}
    </Badge>
  );
}

export function SystemStatusCard() {
  const [data, setData] = useState<SystemStatusData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showAllTables, setShowAllTables] = useState(false);
  const [showAllEndpoints, setShowAllEndpoints] = useState(false);

  const fetchStatus = async () => {
    setLoading(true);
    setError(null);
    try {
      const result = await api.get<SystemStatusData>("/api/system/status");
      setData(result);
    } catch (e: any) {
      setError(e.message || "无法连接到后端服务");
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    fetchStatus();
  }, []);

  const displayedTables = showAllTables
    ? data?.database.tables || []
    : (data?.database.tables || []).slice(0, 12);
  const displayedEndpoints = showAllEndpoints
    ? data?.api.endpoints || []
    : (data?.api.endpoints || []).slice(0, 12);

  return (
    <Card className={`border-2 ${data?.health.status_border_class || "border-border"}`}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`p-2 rounded-lg ${data?.health.status_background_class || "bg-muted"}`}>
              <Activity className={`h-5 w-5 ${data?.health.status_text_class || "text-muted-foreground"}`} />
            </div>
            <div>
              <CardTitle className="text-lg">系统状态监控</CardTitle>
              <CardDescription>
                {data ? `上次检查: ${data.health.timestamp_label}` : "点击刷新开始检查"}
              </CardDescription>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <SystemBadge badge={data?.health.status_badge} />
            <Button
              variant="outline"
              size="sm"
              onClick={fetchStatus}
              disabled={loading}
              className="gap-1"
            >
              <RefreshCw className={`h-4 w-4 ${loading ? "animate-spin" : ""}`} />
              刷新
            </Button>
          </div>
        </div>
      </CardHeader>

      <CardContent className="space-y-6">
        {error && (
          <div className="flex items-center gap-2 p-4 rounded-lg bg-red-500/10 text-red-600 border border-red-500/20">
            <XCircle className="h-5 w-5 flex-shrink-0" />
            <div>
              <p className="font-medium">后端连接失败</p>
              <p className="text-sm opacity-80">{error}</p>
            </div>
          </div>
        )}

        {data && (
          <>
            <div className="grid grid-cols-2 md:grid-cols-4 gap-3">
              <div className="p-3 rounded-lg bg-muted/50 border">
                <div className="flex items-center gap-2 mb-1">
                  <Server className="h-4 w-4 text-muted-foreground" />
                  <span className="text-xs text-muted-foreground">后端版本</span>
                </div>
                <p className="font-mono font-semibold text-sm">
                  v{data.health.version}
                </p>
              </div>
              <div className="p-3 rounded-lg bg-muted/50 border">
                <div className="flex items-center gap-2 mb-1">
                  <Database className={`h-4 w-4 ${data.health.database_text_class || ""}`} />
                  <span className="text-xs text-muted-foreground">数据库</span>
                </div>
                <p className={`font-semibold text-sm ${data.health.database_text_class || ""}`}>
                  {data.health.database_label}
                </p>
              </div>
              <div className="p-3 rounded-lg bg-muted/50 border">
                <div className="flex items-center gap-2 mb-1">
                  <HardDrive className="h-4 w-4 text-muted-foreground" />
                  <span className="text-xs text-muted-foreground">数据库大小</span>
                </div>
                <p className="font-mono font-semibold text-sm">
                  {data.database.file_size_label}
                </p>
              </div>
              <div className="p-3 rounded-lg bg-muted/50 border">
                <div className="flex items-center gap-2 mb-1">
                  <Layers className="h-4 w-4 text-muted-foreground" />
                  <span className="text-xs text-muted-foreground">数据总量</span>
                </div>
                <p className="font-mono font-semibold text-sm">
                  {data.database.total_rows_label}
                </p>
              </div>
            </div>

            <Tabs defaultValue="database" className="w-full">
              <TabsList className="grid w-full grid-cols-3">
                <TabsTrigger value="health" className="gap-1">
                  <Activity className="h-3.5 w-3.5" />
                  健康状态
                </TabsTrigger>
                <TabsTrigger value="api" className="gap-1">
                  <Globe className="h-3.5 w-3.5" />
                  接口状态
                  <Badge variant="secondary" className="ml-1 h-5 text-xs px-1.5">
                    {data.api.ok_count}/{data.api.checked_count}
                  </Badge>
                </TabsTrigger>
                <TabsTrigger value="database" className="gap-1">
                  <Database className="h-3.5 w-3.5" />
                  数据库状态
                </TabsTrigger>
              </TabsList>

              <TabsContent value="health" className="mt-4">
                <div className="grid grid-cols-2 lg:grid-cols-3 gap-3">
                  <div className="p-4 rounded-lg border bg-card flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <Server className="h-4 w-4 text-emerald-500" />
                      <span className="font-medium text-sm">后端服务</span>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      Rust Axum · v{data.health.version}
                    </p>
                    <SystemBadge badge={data.health.status_badge} />
                  </div>

                  <div className="p-4 rounded-lg border bg-card flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <Database className={`h-4 w-4 ${data.health.database_text_class || ""}`} />
                      <span className="font-medium text-sm">SQLite 数据库</span>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {data.database.file_size_label} · {data.database.table_count_label}
                    </p>
                    <SystemBadge badge={data.health.database_badge} />
                  </div>

                  <div className="p-4 rounded-lg border bg-card flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <Globe className="h-4 w-4 text-blue-500" />
                      <span className="font-medium text-sm">API 接口</span>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {data.api.summary_label}
                    </p>
                    <Badge className="bg-blue-500/10 text-blue-600 border-0 w-fit">
                      {data.api.total_endpoints_label}
                    </Badge>
                  </div>

                  <div className="p-4 rounded-lg border bg-card flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <HardDrive className="h-4 w-4 text-purple-500" />
                      <span className="font-medium text-sm">文件存储</span>
                    </div>
                    <p className="text-xs text-muted-foreground">附件及上传文件</p>
                    <Badge className="bg-purple-500/10 text-purple-600 border-0 w-fit">
                      {data.storage.data_files_label}
                    </Badge>
                  </div>

                  <div className="p-4 rounded-lg border bg-card flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <Layers className="h-4 w-4 text-cyan-500" />
                      <span className="font-medium text-sm">数据总量</span>
                    </div>
                    <p className="text-xs text-muted-foreground">
                      {data.database.summary_label}
                    </p>
                    <span className="font-mono font-bold text-lg text-foreground mt-auto">
                      {data.database.total_rows_label}
                    </span>
                  </div>

                  <div className="p-4 rounded-lg border bg-card flex flex-col gap-2">
                    <div className="flex items-center gap-2">
                      <Timer className="h-4 w-4 text-amber-500" />
                      <span className="font-medium text-sm">检查时间</span>
                    </div>
                    <p className="text-xs text-muted-foreground">服务器时间</p>
                    <span className="text-xs font-mono text-muted-foreground mt-auto">
                      {data.health.timestamp_label}
                    </span>
                  </div>
                </div>
              </TabsContent>

              <TabsContent value="api" className="space-y-4 mt-4">
                <div className="flex items-center justify-between">
                  <p className="text-sm font-medium">
                    {data.api.summary_label}
                    <span className="ml-2 text-muted-foreground">
                      · <span className="text-emerald-500">{data.api.ok_count} 正常</span>
                      {data.api.error_count > 0 && (
                        <span className="text-red-500"> · {data.api.error_count} 异常</span>
                      )}
                    </span>
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={fetchStatus}
                    disabled={loading}
                    className="gap-1"
                  >
                    <RefreshCw className={`h-3.5 w-3.5 ${loading ? "animate-spin" : ""}`} />
                    刷新接口状态
                  </Button>
                </div>

                <div className="space-y-1.5">
                  <Progress value={data.api.health_percent} className="h-2" />
                  <p className="text-xs text-muted-foreground text-right">
                    {data.api.health_percent_label}
                  </p>
                </div>

                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                  {displayedEndpoints.map((ep, i) => (
                    <div key={i} className="p-3 rounded-lg border bg-card flex flex-col gap-1.5">
                      <div className="flex items-center justify-between">
                        <Badge variant="outline" className="font-mono text-[10px] px-1.5 py-0 bg-blue-500/5 text-blue-600 border-blue-500/20">
                          {ep.method}
                        </Badge>
                        <SystemBadge badge={ep.status_badge} />
                      </div>
                      <p className="font-mono text-[11px] text-muted-foreground truncate" title={ep.path}>
                        {ep.path}
                      </p>
                      <p className="text-xs font-medium truncate">{ep.name}</p>
                    </div>
                  ))}
                </div>

                {data.api.endpoints.length > 12 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setShowAllEndpoints(!showAllEndpoints)}
                    className="w-full text-muted-foreground"
                  >
                    {showAllEndpoints ? (
                      <><ChevronUp className="h-4 w-4 mr-1" /> 收起</>
                    ) : (
                      <><ChevronDown className="h-4 w-4 mr-1" /> 展示全部 {data.api.total_endpoints_label}</>
                    )}
                  </Button>
                )}
              </TabsContent>

              <TabsContent value="database" className="space-y-4 mt-4">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium">{data.database.summary_label}</p>
                    {data.database.error_table_count > 0 && (
                      <p className="text-xs text-red-500">
                        {data.database.error_table_count} 张表状态异常
                      </p>
                    )}
                  </div>
                  <Badge className="border-0 bg-emerald-500/10 text-emerald-600">
                    {data.database.health_percent_label}
                  </Badge>
                </div>

                <Progress value={data.database.health_percent} className="h-2" />

                <div className="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                  {displayedTables.map((t, i) => (
                    <div
                      key={i}
                      className={`p-3 rounded-lg border flex flex-col gap-1.5 ${t.card_class || ""}`}
                    >
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-mono text-[11px] text-muted-foreground truncate" title={t.table}>
                          {t.table}
                        </span>
                        <SystemBadge badge={t.status_badge} />
                      </div>
                      <p className="text-xs font-medium truncate">{t.label}</p>
                      <div className="flex items-baseline gap-1 mt-auto">
                        <span className="font-mono text-lg font-bold text-foreground">
                          {t.count_label}
                        </span>
                      </div>
                    </div>
                  ))}
                </div>

                {data.database.tables.length > 12 && (
                  <Button
                    variant="ghost"
                    size="sm"
                    onClick={() => setShowAllTables(!showAllTables)}
                    className="w-full text-muted-foreground"
                  >
                    {showAllTables ? (
                      <><ChevronUp className="h-4 w-4 mr-1" /> 收起</>
                    ) : (
                      <><ChevronDown className="h-4 w-4 mr-1" /> 展示全部 {data.database.table_count_label}</>
                    )}
                  </Button>
                )}
              </TabsContent>
            </Tabs>
          </>
        )}
      </CardContent>
    </Card>
  );
}
