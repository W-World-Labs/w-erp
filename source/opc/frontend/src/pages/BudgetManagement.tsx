import { useState } from "react";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Progress } from "@/components/ui/progress";
import { Badge } from "@/components/ui/badge";
import {
  AlertTriangle,
  Brain,
  Building2,
  CheckCircle2,
  DollarSign,
  Edit,
  FileText,
  Plus,
  Target,
  Trash2,
  TrendingDown,
  Users,
  Zap,
  type LucideIcon,
} from "lucide-react";
import {
  Bar,
  BarChart,
  Legend,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from "recharts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction, type DeleteAction } from "@/integrations/api/client";
import { toast } from "@/components/ui/use-toast";
import { BudgetDialog } from "@/components/budget/BudgetDialog";
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
import { AIAuditDialog } from "@/components/applications/AIAuditDialog";

type BudgetType = "monthly" | "quarterly";
type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

interface BudgetBadge {
  value: string;
  label: string;
  variant: BadgeVariant;
  icon?: string;
  class_name?: string;
}

interface BudgetExecution {
  id: string;
  category: string;
  category_label: string;
  category_icon: string;
  category_class: string;
  budget_type: BudgetType;
  period: string;
  budget_amount: number;
  budget_amount_label: string;
  actual_amount: number;
  actual_amount_label: string;
  status: string;
  status_badge: BudgetBadge;
  notes?: string | null;
  percentage: number;
  is_exceeded: boolean;
  usage_label: string;
  remaining_label: string;
  progress_value: number;
  progress_class: string;
  update_action: BackendAction | null;
  can_delete: boolean;
  delete_action: DeleteAction | null;
}

interface BudgetOverview {
  budget_type: BudgetType;
  period: string;
  budget_type_options: Array<{ value: BudgetType; label: string }>;
  budgets: BudgetExecution[];
  stats: {
    total_budget: number;
    total_budget_label: string;
    total_actual: number;
    total_actual_label: string;
    exceeded_count: number;
    saved_amount: number;
    saved_amount_label: string;
    utilization: number;
    utilization_label: string;
  };
  trend: Array<{
    period: string;
    period_label: string;
    budget: number;
    budget_label: string;
    actual: number;
    actual_label: string;
  }>;
  create_action: BackendAction | null;
  permissions: {
    can_create: boolean;
    can_update: boolean;
    can_delete: boolean;
  };
  empty_message: string;
}

const BUDGET_OVERVIEW_ENDPOINT = "/api/budgets/overview";

const emptyOverview: BudgetOverview = {
  budget_type: "monthly",
  period: "",
  budget_type_options: [
    { value: "monthly", label: "月度预算" },
    { value: "quarterly", label: "季度预算" },
  ],
  budgets: [],
  stats: {
    total_budget: 0,
    total_budget_label: "¥0",
    total_actual: 0,
    total_actual_label: "¥0",
    exceeded_count: 0,
    saved_amount: 0,
    saved_amount_label: "¥0",
    utilization: 0,
    utilization_label: "0%",
  },
  trend: [],
  create_action: null,
  permissions: {
    can_create: false,
    can_update: false,
    can_delete: false,
  },
  empty_message: '暂无预算设置，点击"新建预算"开始设置',
};

const iconMap: Record<string, LucideIcon> = {
  AlertTriangle,
  Building2,
  CheckCircle2,
  DollarSign,
  FileText,
  Users,
  Zap,
};

function BudgetIcon({ icon, className }: { icon: string; className?: string }) {
  const Icon = iconMap[icon] || DollarSign;
  return <Icon className={`h-4 w-4 ${className || ""}`} />;
}

function BudgetBadgeView({ badge }: { badge: BudgetBadge }) {
  const Icon = badge.icon ? iconMap[badge.icon] : null;
  return (
    <Badge variant={badge.variant} className={badge.class_name}>
      {Icon && <Icon className="h-3 w-3" />}
      {badge.label}
    </Badge>
  );
}

function budgetTooltipFormatter(_: unknown, __: unknown, item: any) {
  const payload = item?.payload;
  if (item?.dataKey === "budget") {
    return payload.budget_label;
  }
  if (item?.dataKey === "actual") {
    return payload.actual_label;
  }
  return "";
}

function trendPeriodLabel(point: BudgetOverview["trend"][number]) {
  return point.period_label;
}

const BudgetManagement = () => {
  const queryClient = useQueryClient();
  const [budgetType, setBudgetType] = useState<BudgetType>("monthly");
  const [selectedPeriod, setSelectedPeriod] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedBudget, setSelectedBudget] = useState<any>(null);
  const [budgetToDelete, setBudgetToDelete] = useState<DeleteAction | null>(
    null,
  );
  const [aiAuditOpen, setAiAuditOpen] = useState(false);
  const [aiAuditBudget, setAiAuditBudget] = useState<BudgetExecution | null>(
    null,
  );

  const { data: overview = emptyOverview } = useQuery({
    queryKey: ["budget-overview", budgetType, selectedPeriod],
    queryFn: async () => {
      const params = new URLSearchParams({ budget_type: budgetType });
      const period = selectedPeriod.trim();
      if (period) {
        params.set("period", period);
      }
      return api.get<BudgetOverview>(`${BUDGET_OVERVIEW_ENDPOINT}?${params}`);
    },
  });

  const saveMutation = useMutation({
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
      queryClient.invalidateQueries({ queryKey: ["budget-overview"] });
      setDialogOpen(false);
      setSelectedBudget(null);
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "预算已保存",
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
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["budget-overview"] });
      setBudgetToDelete(null);
      toast({
        title: action.success_title,
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

  const budgetsWithActual = overview.budgets;
  const stats = overview.stats;
  const trendData = overview.trend;
  const displayedPeriod = selectedPeriod || overview.period;

  const handleEdit = (budget: BudgetExecution) => {
    setSelectedBudget(budget);
    setDialogOpen(true);
  };

  const handleDelete = (action: DeleteAction) => {
    setBudgetToDelete(action);
  };

  const handleSubmit = (values: any) => {
    const action = selectedBudget
      ? selectedBudget.update_action
      : overview.create_action;
    if (!action) {
      toast({
        title: "操作失败",
        description: "当前账号没有保存预算的权限",
        variant: "destructive",
      });
      return;
    }
    saveMutation.mutate({ action, values });
  };

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="flex items-center gap-2 text-3xl font-bold text-foreground">
            <Target className="h-8 w-8 text-primary" />
            预算管理
          </h1>
          <p className="mt-2 text-muted-foreground">
            设置和监控月度、季度预算执行情况
          </p>
        </div>
        <div className="flex gap-2">
          <Select
            value={budgetType}
            onValueChange={(value: BudgetType) => {
              setBudgetType(value);
              setSelectedPeriod("");
            }}
          >
            <SelectTrigger className="w-[150px]">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {overview.budget_type_options.map((option) => (
                <SelectItem key={option.value} value={option.value}>
                  {option.label}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Input
            type={budgetType === "monthly" ? "month" : "text"}
            value={displayedPeriod}
            onChange={(event) => setSelectedPeriod(event.target.value)}
            className="w-[180px]"
            placeholder={budgetType === "monthly" ? "2024-01" : "2024-Q1"}
          />
          <Button
            disabled={!overview.create_action}
            onClick={() => {
              setSelectedBudget(null);
              setDialogOpen(true);
            }}
          >
            <Plus className="mr-2 h-4 w-4" />
            新建预算
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">预算总额</CardTitle>
            <Target className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_budget_label}</div>
            <p className="mt-1 text-xs text-muted-foreground">当期预算设置</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">实际支出</CardTitle>
            <DollarSign className="h-4 w-4 text-accent" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_actual_label}</div>
            <p className="mt-1 text-xs text-muted-foreground">
              利用率: {stats.utilization_label}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">节余金额</CardTitle>
            <TrendingDown className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {stats.saved_amount_label}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">预算结余</p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">超支项目</CardTitle>
            <AlertTriangle className="h-4 w-4 text-destructive" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-destructive">
              {stats.exceeded_count}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">需要关注</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>预算执行情况</CardTitle>
          <CardDescription>当期各项预算执行明细</CardDescription>
        </CardHeader>
        <CardContent className="space-y-4">
          {budgetsWithActual.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              {overview.empty_message}
            </div>
          ) : (
            budgetsWithActual.map((budget) => (
              <div key={budget.id} className="space-y-2">
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <BudgetIcon
                      icon={budget.category_icon}
                      className={budget.category_class}
                    />
                    <span className="font-medium">{budget.category_label}</span>
                    <BudgetBadgeView badge={budget.status_badge} />
                  </div>
                  <div className="flex items-center gap-4">
                    <span className="text-sm text-muted-foreground">
                      {budget.actual_amount_label} /{" "}
                      {budget.budget_amount_label}
                    </span>
                    <div className="flex gap-1">
                      <Button
                        variant="ghost"
                        size="sm"
                        onClick={() => {
                          setAiAuditBudget(budget);
                          setAiAuditOpen(true);
                        }}
                      >
                        <Brain className="h-4 w-4" />
                      </Button>
                      {budget.update_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleEdit(budget)}
                        >
                          <Edit className="h-4 w-4" />
                        </Button>
                      )}
                      {budget.can_delete &&
                        overview.permissions.can_delete &&
                        budget.delete_action && (
                        <Button
                          variant="ghost"
                          size="sm"
                          onClick={() => handleDelete(budget.delete_action)}
                        >
                          <Trash2 className="h-4 w-4" />
                        </Button>
                      )}
                    </div>
                  </div>
                </div>
                <Progress
                  value={budget.progress_value}
                  className={budget.progress_class}
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>{budget.usage_label}</span>
                  <span>{budget.remaining_label}</span>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>预算与实际支出趋势</CardTitle>
          <CardDescription>近6期预算执行对比</CardDescription>
        </CardHeader>
        <CardContent>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={trendData}>
              <XAxis
                dataKey={trendPeriodLabel}
                stroke="hsl(var(--muted-foreground))"
              />
              <YAxis stroke="hsl(var(--muted-foreground))" />
              <Tooltip
                contentStyle={{
                  backgroundColor: "hsl(var(--card))",
                  border: "1px solid hsl(var(--border))",
                  borderRadius: "var(--radius)",
                }}
                formatter={budgetTooltipFormatter}
              />
              <Legend />
              <Bar
                dataKey="budget"
                fill="hsl(var(--primary))"
                name="预算金额"
                radius={[4, 4, 0, 0]}
              />
              <Bar
                dataKey="actual"
                fill="hsl(var(--accent))"
                name="实际支出"
                radius={[4, 4, 0, 0]}
              />
            </BarChart>
          </ResponsiveContainer>
        </CardContent>
      </Card>

      <BudgetDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSubmit={handleSubmit}
        initialData={selectedBudget}
        budgetType={budgetType}
        defaultPeriod={displayedPeriod}
      />

      <AlertDialog
        open={!!budgetToDelete}
        onOpenChange={() => setBudgetToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {budgetToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = budgetToDelete;
                if (action) deleteMutation.mutate(action);
              }}
            >
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AIAuditDialog
        open={aiAuditOpen}
        onOpenChange={setAiAuditOpen}
        auditType="budget"
        targetId={aiAuditBudget?.id}
        title={`预算分析：${aiAuditBudget?.category_label || ""} (${aiAuditBudget?.period || ""})`}
      />
    </div>
  );
};

export default BudgetManagement;
