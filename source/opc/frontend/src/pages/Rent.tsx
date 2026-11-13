import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  AlertCircle,
  Building2,
  Download,
  Droplet,
  Edit,
  Flame,
  MapPin,
  Plus,
  Search,
  Trash2,
  TrendingUp,
  Zap,
  type LucideIcon,
} from "lucide-react";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction, type DeleteAction } from "@/integrations/api/client";
import { toast } from "@/components/ui/use-toast";
import { RentDialog } from "@/components/rent/RentDialog";
import { UtilityDialog } from "@/components/utilities/UtilityDialog";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { DependencyAlert } from "@/components/common/DependencyAlert";

type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

interface DisplayBadge {
  value: string;
  label: string;
  variant: BadgeVariant;
  class_name?: string;
}

interface RentRow {
  id: string;
  location_name: string;
  address?: string | null;
  area_label: string;
  monthly_rent_label: string;
  contract_period_label: string;
  next_payment_label: string;
  status_badge: DisplayBadge;
  update_action: BackendAction | null;
  can_delete: boolean;
  delete_action: DeleteAction;
}

interface UtilityRow {
  id: string;
  location_name: string;
  utility_type_icon: string;
  utility_type_label: string;
  utility_type_class: string;
  period?: string | null;
  usage_label: string;
  unit_price_label: string;
  amount_label: string;
  due_date_label: string;
  status_badge: DisplayBadge;
  update_action: BackendAction | null;
  can_mark_paid: boolean;
  mark_paid_action: BackendAction | null;
  can_delete: boolean;
  delete_action: DeleteAction;
}

interface RentOverview {
  rent_stats: {
    total: number;
    total_label: string;
    locations: number;
    area: number;
    area_label: string;
    expiring: number;
  };
  utility_stats: {
    total: number;
    total_label: string;
    electricity: number;
    electricity_label: string;
    water: number;
    water_label: string;
    gas: number;
    gas_label: string;
  };
  rent_trend: Array<{
    period: string;
    month: string;
    total: number;
    total_label: string;
  }>;
  utility_trend: Array<{
    period: string;
    month: string;
    electricity: number;
    electricity_label: string;
    water: number;
    water_label: string;
    gas: number;
    gas_label: string;
  }>;
  rents: RentRow[];
  utilities: UtilityRow[];
  rent_create_action: BackendAction | null;
  utility_create_action: BackendAction | null;
  permissions: {
    can_create: boolean;
    can_update: boolean;
    can_delete: boolean;
    can_mark_paid: boolean;
  };
  empty_messages: {
    rents: string;
    utilities: string;
  };
}

const RENT_OVERVIEW_ENDPOINT = "/api/v1/rent/overview";

const emptyRentOverview: RentOverview = {
  rent_stats: {
    total: 0,
    total_label: "¥0",
    locations: 0,
    area: 0,
    area_label: "0㎡",
    expiring: 0,
  },
  utility_stats: {
    total: 0,
    total_label: "¥0",
    electricity: 0,
    electricity_label: "¥0",
    water: 0,
    water_label: "¥0",
    gas: 0,
    gas_label: "¥0",
  },
  rent_trend: [],
  utility_trend: [],
  rents: [],
  utilities: [],
  rent_create_action: null,
  utility_create_action: null,
  permissions: {
    can_create: false,
    can_update: false,
    can_delete: false,
    can_mark_paid: false,
  },
  empty_messages: {
    rents: "暂无场地记录",
    utilities: "暂无账单记录",
  },
};

const iconMap: Record<string, LucideIcon> = {
  Droplet,
  Flame,
  Zap,
};

function StatusBadge({ badge }: { badge: DisplayBadge }) {
  return (
    <Badge variant={badge.variant} className={badge.class_name}>
      {badge.label}
    </Badge>
  );
}

function UtilityIcon({
  icon,
  className,
}: {
  icon: string;
  className?: string;
}) {
  const Icon = iconMap[icon] || Zap;
  return <Icon className={`h-4 w-4 ${className || ""}`} />;
}

function rentOverviewQuery(rentSearch: string, utilitySearch: string) {
  const query: string[] = [];
  if (rentSearch.trim()) {
    query.push(`rent_search=${encodeURIComponent(rentSearch.trim())}`);
  }
  if (utilitySearch.trim()) {
    query.push(`utility_search=${encodeURIComponent(utilitySearch.trim())}`);
  }
  return query.length ? `?${query.join("&")}` : "";
}

function rentTooltipFormatter(_: unknown, __: unknown, item: any) {
  const payload = item?.payload;
  return payload.total_label;
}

function utilityTooltipFormatter(_: unknown, __: unknown, item: any) {
  const payload = item?.payload;
  if (item?.dataKey === "electricity") return payload.electricity_label;
  if (item?.dataKey === "water") return payload.water_label;
  if (item?.dataKey === "gas") return payload.gas_label;
  return "";
}

const Rent = () => {
  const queryClient = useQueryClient();
  const [rentSearchTerm, setRentSearchTerm] = useState("");
  const [utilitySearchTerm, setUtilitySearchTerm] = useState("");
  const [rentDialogOpen, setRentDialogOpen] = useState(false);
  const [utilityDialogOpen, setUtilityDialogOpen] = useState(false);
  const [selectedRent, setSelectedRent] = useState<any>(null);
  const [selectedUtility, setSelectedUtility] = useState<any>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [itemToDelete, setItemToDelete] = useState<DeleteAction | null>(null);

  const invalidateRentOverview = () => {
    queryClient.invalidateQueries({ queryKey: ["rent-overview"] });
  };

  const { data: overview = emptyRentOverview, isLoading } = useQuery({
    queryKey: ["rent-overview", rentSearchTerm, utilitySearchTerm],
    queryFn: async () => {
      return await api.get<RentOverview>(
        `${RENT_OVERVIEW_ENDPOINT}${rentOverviewQuery(rentSearchTerm, utilitySearchTerm)}`,
      );
    },
  });

  const rentStats = overview.rent_stats;
  const utilityStats = overview.utility_stats;
  const rentTrend = overview.rent_trend;
  const utilityTrend = overview.utility_trend;

  const saveRentMutation = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => {
      await api.runAction(action, values);
      return action;
    },
    onSuccess: (action) => {
      invalidateRentOverview();
      setRentDialogOpen(false);
      setSelectedRent(null);
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "场地已保存",
      });
    },
    onError: (error: any) => {
      toast({
        title: "操作失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const saveUtilityMutation = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => {
      await api.runAction(action, values);
      return action;
    },
    onSuccess: (action) => {
      invalidateRentOverview();
      setUtilityDialogOpen(false);
      setSelectedUtility(null);
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "账单已保存",
      });
    },
    onError: (error: any) => {
      toast({
        title: "操作失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const deleteMutation = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
    },
    onSuccess: (_, action) => {
      invalidateRentOverview();
      setDeleteDialogOpen(false);
      setItemToDelete(null);
      toast({
        title: "删除成功",
        description: action.success_message,
      });
    },
    onError: (error: any) => {
      toast({
        title: "删除失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const markAsPaidMutation = useMutation({
    mutationFn: async (action: BackendAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      invalidateRentOverview();
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "已标记为已支付",
      });
    },
    onError: (error: any) => {
      toast({
        title: "操作失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">场地水电管理</h1>
          <p className="mt-2 text-muted-foreground">
            管理企业场地租赁和水电费用
          </p>
        </div>
      </div>

      <DependencyAlert dependency="company-info" />

      <Tabs defaultValue="rent" className="w-full">
        <TabsList className="grid w-full grid-cols-2">
          <TabsTrigger value="rent">场地租金</TabsTrigger>
          <TabsTrigger value="utilities">水电费用</TabsTrigger>
        </TabsList>

        <TabsContent value="rent" className="space-y-6">
          <div className="flex justify-end gap-2">
            <Button variant="outline" className="gap-2">
              <Download className="h-4 w-4" />
              导出租金记录
            </Button>
            <Button
              className="gap-2"
              disabled={!overview.rent_create_action}
              onClick={() => {
                setSelectedRent(null);
                setRentDialogOpen(true);
              }}
            >
              <Plus className="h-4 w-4" />
              添加场地
            </Button>
          </div>

          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  月租金总额
                </CardTitle>
                <Building2 className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {rentStats.total_label}
                </div>
                <p className="text-xs text-muted-foreground">活跃场地</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">租赁场地</CardTitle>
                <MapPin className="h-4 w-4 text-accent" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{rentStats.locations}</div>
                <p className="text-xs text-muted-foreground">活跃租赁点</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">总面积</CardTitle>
                <Building2 className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{rentStats.area_label}</div>
                <p className="text-xs text-muted-foreground">办公面积</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">即将到期</CardTitle>
                <AlertCircle className="h-4 w-4 text-warning" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{rentStats.expiring}</div>
                <p className="text-xs text-muted-foreground">3个月内到期</p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>租金月度趋势</CardTitle>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={rentTrend}>
                  <XAxis
                    dataKey="month"
                    stroke="hsl(var(--muted-foreground))"
                  />
                  <YAxis stroke="hsl(var(--muted-foreground))" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "hsl(var(--card))",
                      border: "1px solid hsl(var(--border))",
                      borderRadius: "var(--radius)",
                    }}
                    formatter={rentTooltipFormatter}
                  />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="total"
                    stroke="hsl(var(--primary))"
                    strokeWidth={2}
                    name="租金总额"
                    dot={{ fill: "hsl(var(--primary))", r: 4 }}
                  />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>场地租赁列表</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="mb-4">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    placeholder="搜索场地名称、地址..."
                    value={rentSearchTerm}
                    onChange={(event) => setRentSearchTerm(event.target.value)}
                    className="pl-10"
                  />
                </div>
              </div>

              {isLoading ? (
                <div className="py-8 text-center">加载中...</div>
              ) : overview.rents.length === 0 ? (
                <div className="py-8 text-center text-muted-foreground">
                  {overview.empty_messages.rents}
                </div>
              ) : (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>场地名称</TableHead>
                        <TableHead>地址</TableHead>
                        <TableHead>面积</TableHead>
                        <TableHead>月租金</TableHead>
                        <TableHead>合同期限</TableHead>
                        <TableHead>下次缴费</TableHead>
                        <TableHead>状态</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {overview.rents.map((rent) => (
                        <TableRow key={rent.id}>
                          <TableCell className="font-medium">
                            {rent.location_name}
                          </TableCell>
                          <TableCell>{rent.address || "-"}</TableCell>
                          <TableCell>{rent.area_label}</TableCell>
                          <TableCell className="font-semibold">
                            {rent.monthly_rent_label}
                          </TableCell>
                          <TableCell className="text-sm">
                            {rent.contract_period_label}
                          </TableCell>
                          <TableCell>{rent.next_payment_label}</TableCell>
                          <TableCell>
                            <StatusBadge badge={rent.status_badge} />
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-2">
                              {rent.update_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => {
                                    setSelectedRent(rent);
                                    setRentDialogOpen(true);
                                  }}
                                >
                                  <Edit className="h-4 w-4" />
                                </Button>
                              )}
                              {rent.can_delete &&
                                overview.permissions.can_delete && (
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => {
                                      setItemToDelete(rent.delete_action);
                                      setDeleteDialogOpen(true);
                                    }}
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                )}
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="utilities" className="space-y-6">
          <div className="flex justify-end gap-2">
            <Button variant="outline" className="gap-2">
              <Download className="h-4 w-4" />
              导出账单
            </Button>
            <Button
              className="gap-2"
              disabled={!overview.utility_create_action}
              onClick={() => {
                setSelectedUtility(null);
                setUtilityDialogOpen(true);
              }}
            >
              <Plus className="h-4 w-4" />
              录入账单
            </Button>
          </div>

          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  待支付总额
                </CardTitle>
                <TrendingUp className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {utilityStats.total_label}
                </div>
                <p className="text-xs text-muted-foreground">待支付账单</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">电费</CardTitle>
                <Zap className="h-4 w-4 text-warning" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {utilityStats.electricity_label}
                </div>
                <p className="text-xs text-muted-foreground">待支付</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">水费</CardTitle>
                <Droplet className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {utilityStats.water_label}
                </div>
                <p className="text-xs text-muted-foreground">待支付</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">燃气费</CardTitle>
                <Flame className="h-4 w-4 text-destructive" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {utilityStats.gas_label}
                </div>
                <p className="text-xs text-muted-foreground">待支付</p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>费用月度趋势</CardTitle>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <LineChart data={utilityTrend}>
                  <XAxis
                    dataKey="month"
                    stroke="hsl(var(--muted-foreground))"
                  />
                  <YAxis stroke="hsl(var(--muted-foreground))" />
                  <Tooltip
                    contentStyle={{
                      backgroundColor: "hsl(var(--card))",
                      border: "1px solid hsl(var(--border))",
                      borderRadius: "var(--radius)",
                    }}
                    formatter={utilityTooltipFormatter}
                  />
                  <Legend />
                  <Line
                    type="monotone"
                    dataKey="electricity"
                    stroke="hsl(var(--warning))"
                    strokeWidth={2}
                    name="电费"
                  />
                  <Line
                    type="monotone"
                    dataKey="water"
                    stroke="hsl(var(--primary))"
                    strokeWidth={2}
                    name="水费"
                  />
                  <Line
                    type="monotone"
                    dataKey="gas"
                    stroke="hsl(var(--destructive))"
                    strokeWidth={2}
                    name="燃气费"
                  />
                </LineChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>账单列表</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="mb-4">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    placeholder="搜索场地、账单类型..."
                    value={utilitySearchTerm}
                    onChange={(event) =>
                      setUtilitySearchTerm(event.target.value)
                    }
                    className="pl-10"
                  />
                </div>
              </div>

              {isLoading ? (
                <div className="py-8 text-center">加载中...</div>
              ) : overview.utilities.length === 0 ? (
                <div className="py-8 text-center text-muted-foreground">
                  {overview.empty_messages.utilities}
                </div>
              ) : (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>场地</TableHead>
                        <TableHead>类型</TableHead>
                        <TableHead>账期</TableHead>
                        <TableHead>用量</TableHead>
                        <TableHead>单价</TableHead>
                        <TableHead>金额</TableHead>
                        <TableHead>缴费截止</TableHead>
                        <TableHead>状态</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {overview.utilities.map((utility) => (
                        <TableRow key={utility.id}>
                          <TableCell>{utility.location_name}</TableCell>
                          <TableCell>
                            <div className="flex items-center gap-2">
                              <UtilityIcon
                                icon={utility.utility_type_icon}
                                className={utility.utility_type_class}
                              />
                              {utility.utility_type_label}
                            </div>
                          </TableCell>
                          <TableCell>{utility.period || "-"}</TableCell>
                          <TableCell>{utility.usage_label}</TableCell>
                          <TableCell>{utility.unit_price_label}</TableCell>
                          <TableCell className="font-semibold">
                            {utility.amount_label}
                          </TableCell>
                          <TableCell>{utility.due_date_label}</TableCell>
                          <TableCell>
                            <StatusBadge badge={utility.status_badge} />
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-2">
                              {utility.can_mark_paid &&
                                overview.permissions.can_mark_paid &&
                                utility.mark_paid_action && (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() =>
                                      markAsPaidMutation.mutate(
                                        utility.mark_paid_action,
                                      )
                                    }
                                  >
                                    {utility.mark_paid_action.label || "标记已付"}
                                  </Button>
                                )}
                              {utility.update_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => {
                                    setSelectedUtility(utility);
                                    setUtilityDialogOpen(true);
                                  }}
                                >
                                  <Edit className="h-4 w-4" />
                                </Button>
                              )}
                              {utility.can_delete &&
                                overview.permissions.can_delete && (
                                  <Button
                                    variant="ghost"
                                    size="sm"
                                    onClick={() => {
                                      setItemToDelete(utility.delete_action);
                                      setDeleteDialogOpen(true);
                                    }}
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                )}
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <RentDialog
        open={rentDialogOpen}
        onOpenChange={setRentDialogOpen}
        onSubmit={(values) => {
          const action = selectedRent
            ? selectedRent.update_action
            : overview.rent_create_action;
          if (!action) {
            toast({
              title: "操作失败",
              description: "当前账号没有保存场地的权限",
              variant: "destructive",
            });
            return;
          }
          saveRentMutation.mutate({ action, values });
        }}
        initialData={selectedRent}
      />

      <UtilityDialog
        open={utilityDialogOpen}
        onOpenChange={setUtilityDialogOpen}
        onSubmit={(values) => {
          const action = selectedUtility
            ? selectedUtility.update_action
            : overview.utility_create_action;
          if (!action) {
            toast({
              title: "操作失败",
              description: "当前账号没有保存账单的权限",
              variant: "destructive",
            });
            return;
          }
          saveUtilityMutation.mutate({ action, values });
        }}
        initialData={selectedUtility}
      />

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {itemToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() =>
                itemToDelete && deleteMutation.mutate(itemToDelete)
              }
            >
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default Rent;
