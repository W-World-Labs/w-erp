import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { toast } from "sonner";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  CalendarDays,
  Globe,
  MapPin,
  AlertCircle,
  ExternalLink,
  Loader2,
} from "lucide-react";

type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

interface DisplayBadge {
  label: string;
  variant: BadgeVariant;
}

interface TaxStats {
  country_count: number;
  state_count: number;
  deadline_count: number;
  upcoming_deadline_count: number;
}

interface TaxCalendarMonth {
  month: number;
  label: string;
  events: any[];
  visible_events: any[];
  count_badge: DisplayBadge | null;
  empty_message: string;
  overflow_label: string;
  card_class_name: string;
}

interface TaxFilterOption {
  value: string;
  label: string;
}

interface TaxCalendarOverview {
  stats: TaxStats;
  filters: {
    filtered_count: number;
    current_month: number;
  };
  jurisdictions: any[];
  country_options: TaxFilterOption[];
  month_options: TaxFilterOption[];
  filtered_deadlines: any[];
  calendar_by_month: TaxCalendarMonth[];
  is_empty: boolean;
  seed_action: BackendAction | null;
}

const emptyOverview: TaxCalendarOverview = {
  stats: {
    country_count: 0,
    state_count: 0,
    deadline_count: 0,
    upcoming_deadline_count: 0,
  },
  filters: {
    filtered_count: 0,
    current_month: 0,
  },
  jurisdictions: [],
  country_options: [],
  month_options: [],
  filtered_deadlines: [],
  calendar_by_month: [],
  is_empty: true,
  seed_action: null,
};

function TaxCalendar() {
  const queryClient = useQueryClient();
  const [selectedCountry, setSelectedCountry] = useState<string>("all");
  const [selectedMonth, setSelectedMonth] = useState<string>("all");

  const { data: overview = emptyOverview, isLoading } = useQuery({
    queryKey: ["tax-calendar-overview", selectedCountry, selectedMonth],
    queryFn: () => {
      const params = new URLSearchParams();
      if (selectedCountry !== "all") params.set("country", selectedCountry);
      if (selectedMonth !== "all") params.set("month", selectedMonth);
      const suffix = params.toString() ? `?${params.toString()}` : "";
      return api.get<TaxCalendarOverview>(
        `${API_ENDPOINTS.taxCalendarOverview}${suffix}`,
      );
    },
  });

  const seedMutation = useMutation({
    mutationFn: () => {
      const action = overview.seed_action;
      if (!action) {
        throw new Error("后端未返回税务数据导入动作");
      }
      return api.runAction(action, {});
    },
    onSuccess: (data: any) => {
      queryClient.invalidateQueries({ queryKey: ["tax-calendar-overview"] });
      toast.success(
        `已导入 ${data.jurisdictions_seeded} 个管辖区和 ${data.deadlines_seeded} 个税务截止日`,
      );
    },
    onError: () => {
      toast.error("导入失败");
    },
  });

  const jurisdictions = overview.jurisdictions;
  const filteredDeadlines = overview.filtered_deadlines;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold">全球税务日历</h1>
          <p className="text-muted-foreground">
            跟踪全球征税截止日期 — 中国 / 韩国 / 美国 / 日本
          </p>
        </div>
        {overview.is_empty && (
          <Button
            onClick={() => seedMutation.mutate()}
            disabled={seedMutation.isPending || !overview.seed_action}
          >
            {seedMutation.isPending ? (
              <>
                <Loader2 className="h-4 w-4 mr-2 animate-spin" /> 导入中...
              </>
            ) : (
              <>
                <Globe className="h-4 w-4 mr-2" /> 一键导入全球税务数据
              </>
            )}
          </Button>
        )}
      </div>

      <div className="grid grid-cols-4 gap-4">
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <Globe className="h-5 w-5 text-blue-500" />
              <div>
                <div className="text-2xl font-bold">
                  {overview.stats.country_count}
                </div>
                <p className="text-xs text-muted-foreground">覆盖国家</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <MapPin className="h-5 w-5 text-green-500" />
              <div>
                <div className="text-2xl font-bold">
                  {overview.stats.state_count}
                </div>
                <p className="text-xs text-muted-foreground">美国州</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <CalendarDays className="h-5 w-5 text-purple-500" />
              <div>
                <div className="text-2xl font-bold">
                  {overview.stats.deadline_count}
                </div>
                <p className="text-xs text-muted-foreground">税务截止日</p>
              </div>
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardContent className="pt-6">
            <div className="flex items-center gap-2">
              <AlertCircle className="h-5 w-5 text-red-500" />
              <div>
                <div className="text-2xl font-bold">
                  {overview.stats.upcoming_deadline_count}
                </div>
                <p className="text-xs text-muted-foreground">近期截止</p>
              </div>
            </div>
          </CardContent>
        </Card>
      </div>

      <Tabs defaultValue="calendar">
        <TabsList>
          <TabsTrigger value="calendar">📅 年度日历总览</TabsTrigger>
          <TabsTrigger value="deadlines">📋 截止日期列表</TabsTrigger>
          <TabsTrigger value="jurisdictions">🌍 管辖区信息</TabsTrigger>
        </TabsList>

        <TabsContent value="calendar" className="space-y-4">
          {isLoading ? (
            <p className="text-center py-8">加载中...</p>
          ) : (
            <div className="grid grid-cols-3 gap-4">
              {overview.calendar_by_month.map((monthBucket) => (
                  <Card
                    key={monthBucket.month}
                    className={monthBucket.card_class_name}
                  >
                    <CardHeader className="pb-2">
                      <CardTitle className="text-sm flex items-center justify-between">
                        <span>{monthBucket.label}</span>
                        {monthBucket.count_badge && (
                          <Badge variant={monthBucket.count_badge.variant}>
                            {monthBucket.count_badge.label}
                          </Badge>
                        )}
                      </CardTitle>
                    </CardHeader>
                    <CardContent className="space-y-1">
                      {monthBucket.visible_events.length === 0 ? (
                        <p className="text-xs text-muted-foreground">
                          {monthBucket.empty_message}
                        </p>
                      ) : (
                        monthBucket.visible_events.map((event: any) => {
                          const flag = event.country_flag || "🏳️";
                          return (
                            <div
                              key={event.id}
                              className="flex items-center gap-1 text-xs"
                            >
                              <span>{flag}</span>
                              <span className="font-mono text-muted-foreground w-5">
                                {event.deadline_day}日
                              </span>
                              <span
                                className={`w-1.5 h-1.5 rounded-full ${
                                  event.tax_type_color || "bg-gray-400"
                                }`}
                              />
                              <span className="truncate flex-1">
                                {event.title_local || event.title}
                              </span>
                            </div>
                          );
                        })
                      )}
                      {monthBucket.overflow_label && (
                        <p className="text-xs text-muted-foreground">
                          {monthBucket.overflow_label}
                        </p>
                      )}
                    </CardContent>
                  </Card>
                ))}
            </div>
          )}
        </TabsContent>

        <TabsContent value="deadlines" className="space-y-4">
          <div className="flex gap-4">
            <Select value={selectedCountry} onValueChange={setSelectedCountry}>
              <SelectTrigger className="w-48">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {overview.country_options.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={selectedMonth} onValueChange={setSelectedMonth}>
              <SelectTrigger className="w-32">
                <SelectValue />
              </SelectTrigger>
              <SelectContent>
                {overview.month_options.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <div className="flex-1" />
            <Badge variant="outline">
              {overview.filters.filtered_count} 条记录
            </Badge>
          </div>
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>管辖区</TableHead>
                    <TableHead>税种</TableHead>
                    <TableHead>截止日期</TableHead>
                    <TableHead>事件</TableHead>
                    <TableHead>频率</TableHead>
                    <TableHead>适用对象</TableHead>
                    <TableHead className="text-right">申报链接</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {filteredDeadlines.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={7}
                        className="text-center py-8 text-muted-foreground"
                      >
                        {overview.is_empty
                          ? "请点击「一键导入全球税务数据」开始使用"
                          : "无匹配记录"}
                      </TableCell>
                    </TableRow>
                  ) : (
                    filteredDeadlines.map((deadline: any) => {
                      const flag = deadline.country_flag || "🏳️";
                      return (
                        <TableRow key={deadline.id}>
                          <TableCell>
                            <span className="font-medium">
                              {flag} {deadline.jurisdiction_code}
                            </span>
                          </TableCell>
                          <TableCell>
                            <Badge variant="outline" className="text-xs">
                              {deadline.tax_type_label || deadline.tax_type}
                            </Badge>
                          </TableCell>
                          <TableCell className="font-mono">
                            {deadline.deadline_month}月{deadline.deadline_day}日
                          </TableCell>
                          <TableCell className="max-w-[250px]">
                            <div className="font-medium text-sm">
                              {deadline.title_local || deadline.title}
                            </div>
                            {deadline.description && (
                              <div className="text-xs text-muted-foreground truncate">
                                {deadline.description}
                              </div>
                            )}
                          </TableCell>
                          <TableCell>
                            <Badge variant="secondary" className="text-xs">
                              {deadline.recurrence_label ||
                                deadline.recurrence}
                            </Badge>
                          </TableCell>
                          <TableCell className="text-xs">
                            {deadline.applies_to_label || deadline.applies_to}
                          </TableCell>
                          <TableCell className="text-right">
                            {deadline.filing_url ? (
                              <a
                                href={deadline.filing_url}
                                target="_blank"
                                rel="noopener noreferrer"
                                className="text-primary hover:underline inline-flex items-center gap-1"
                              >
                                <ExternalLink className="h-3 w-3" /> 申报
                              </a>
                            ) : (
                              "-"
                            )}
                          </TableCell>
                        </TableRow>
                      );
                    })
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="jurisdictions" className="space-y-4">
          <Card>
            <CardContent className="p-0">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>代码</TableHead>
                    <TableHead>名称</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>企业税率</TableHead>
                    <TableHead>个人税率</TableHead>
                    <TableHead>增值/消费税</TableHead>
                    <TableHead>销售税</TableHead>
                    <TableHead>州所得税</TableHead>
                    <TableHead>备注</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {isLoading ? (
                    <TableRow>
                      <TableCell colSpan={9} className="text-center py-8">
                        加载中...
                      </TableCell>
                    </TableRow>
                  ) : jurisdictions.length === 0 ? (
                    <TableRow>
                      <TableCell
                        colSpan={9}
                        className="text-center py-8 text-muted-foreground"
                      >
                        {overview.is_empty ? "请先导入数据" : "暂无数据"}
                      </TableCell>
                    </TableRow>
                  ) : (
                    jurisdictions.map((jurisdiction: any) => {
                      const flag = jurisdiction.country_flag || "";
                      return (
                        <TableRow key={jurisdiction.id}>
                          <TableCell className="font-mono font-bold">
                            {flag} {jurisdiction.code}
                          </TableCell>
                          <TableCell>
                            <div>{jurisdiction.name}</div>
                            {jurisdiction.name_local &&
                              jurisdiction.name_local !== jurisdiction.name && (
                                <div className="text-xs text-muted-foreground">
                                  {jurisdiction.name_local}
                                </div>
                              )}
                          </TableCell>
                          <TableCell>
                            <Badge
                              variant={
                                jurisdiction.jurisdiction_type_badge
                                  ?.variant || "outline"
                              }
                            >
                              {jurisdiction.jurisdiction_type_badge?.label ||
                                jurisdiction.jurisdiction_type}
                            </Badge>
                          </TableCell>
                          <TableCell>
                            {jurisdiction.corporate_tax_rate_label}
                          </TableCell>
                          <TableCell>
                            {jurisdiction.personal_tax_rate_label}
                          </TableCell>
                          <TableCell>
                            {jurisdiction.vat_rate_label}
                          </TableCell>
                          <TableCell>
                            {jurisdiction.sales_tax_rate_label}
                          </TableCell>
                          <TableCell>
                            {jurisdiction.state_income_tax_badge?.label ? (
                              <Badge
                                variant={
                                  jurisdiction.state_income_tax_badge
                                    .variant || "outline"
                                }
                                className="text-xs"
                              >
                                {jurisdiction.state_income_tax_badge.label}
                              </Badge>
                            ) : (
                              jurisdiction.state_income_tax_label || "-"
                            )}
                          </TableCell>
                          <TableCell className="text-xs text-muted-foreground max-w-[200px] truncate">
                            {jurisdiction.notes || "-"}
                          </TableCell>
                        </TableRow>
                      );
                    })
                  )}
                </TableBody>
              </Table>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}

export default TaxCalendar;
