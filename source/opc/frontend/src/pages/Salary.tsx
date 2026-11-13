import { useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Calendar,
  DollarSign,
  Download,
  Edit,
  Plus,
  Search,
  Settings,
  Trash2,
  TrendingUp,
  Users,
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
import { SalaryDialog } from "@/components/salary/SalaryDialog";
import { SocialInsuranceConfigDialog } from "@/components/salary/SocialInsuranceConfigDialog";
import { SalaryCostAnalysis } from "@/components/salary/SalaryCostAnalysis";
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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DependencyAlert } from "@/components/common/DependencyAlert";

const SALARY_OVERVIEW_ENDPOINT = "/api/v1/salary/overview";

interface SalaryBadge {
  label: string;
  class_name?: string;
}

interface SalaryRow {
  id: string;
  employee_name: string;
  department?: string;
  position?: string;
  period: string;
  base_salary_label: string;
  bonus_label: string;
  deduction_label: string;
  social_security_label: string;
  social_security_company_label: string;
  housing_fund_label: string;
  housing_fund_company_label: string;
  actual_salary_label: string;
  status_badge: SalaryBadge;
  update_action: BackendAction | null;
  can_mark_paid: boolean;
  mark_paid_action: BackendAction | null;
  can_delete: boolean;
  delete_action: DeleteAction | null;
}

interface SalaryOverview {
  stats: {
    total_label: string;
    employees_label: string;
    average_label: string;
    pending_label: string;
  };
  monthly_trend: Array<{
    period: string;
    month: string;
    total: number;
    total_label: string;
  }>;
  salaries: SalaryRow[];
  employees: any[];
  current_insurance_config: any | null;
  cost_analysis: any;
  create_action: BackendAction | null;
  insurance_config_create_action: BackendAction | null;
  insurance_preview_action: BackendAction | null;
  permissions: {
    can_create: boolean;
    can_update: boolean;
    can_delete: boolean;
    can_mark_paid: boolean;
  };
  empty_message: string;
}

const emptySalaryOverview: SalaryOverview = {
  stats: {
    total_label: "¥0",
    employees_label: "0",
    average_label: "¥0",
    pending_label: "0",
  },
  monthly_trend: [],
  salaries: [],
  employees: [],
  current_insurance_config: null,
  create_action: null,
  insurance_config_create_action: null,
  insurance_preview_action: null,
  cost_analysis: {
    stats: {
      total_cost: 0,
      total_cost_label: "¥0",
      employee_cost: 0,
      employee_cost_label: "¥0",
      company_cost: 0,
      company_cost_label: "¥0",
      departments: 0,
      departments_label: "0",
      month_over_month: 0,
      month_over_month_label: "+0.0%",
      employee_cost_share_label: "0.0%",
    },
    monthly_trend: [],
    department_costs: [],
  },
  permissions: {
    can_create: false,
    can_update: false,
    can_delete: false,
    can_mark_paid: false,
  },
  empty_message: "暂无工资记录",
};

const Salary = () => {
  const queryClient = useQueryClient();
  const [searchTerm, setSearchTerm] = useState("");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedSalary, setSelectedSalary] = useState<any>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [salaryToDelete, setSalaryToDelete] = useState<DeleteAction | null>(
    null,
  );
  const [configDialogOpen, setConfigDialogOpen] = useState(false);

  const { data: salaryOverview = emptySalaryOverview, isLoading } = useQuery({
    queryKey: ["salary-overview", searchTerm],
    queryFn: () =>
      api.get<SalaryOverview>(
        `${SALARY_OVERVIEW_ENDPOINT}?search=${encodeURIComponent(searchTerm)}`,
      ),
  });
  const salaries = salaryOverview.salaries;
  const employees = salaryOverview.employees;
  const monthlyTrend = salaryOverview.monthly_trend;
  const stats = salaryOverview.stats;
  const costAnalysis = salaryOverview.cost_analysis;
  const permissions = salaryOverview.permissions;
  const currentInsuranceConfig = salaryOverview.current_insurance_config;

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
      queryClient.invalidateQueries({ queryKey: ["salary-overview"] });
      setDialogOpen(false);
      setSelectedSalary(null);
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "工资记录已保存",
      });
    },
    onError: (error) => {
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
      queryClient.invalidateQueries({ queryKey: ["salary-overview"] });
      setDeleteDialogOpen(false);
      setSalaryToDelete(null);
      toast({
        title: action.success_title,
        description: action.success_message,
      });
    },
    onError: (error) => {
      toast({
        title: "删除失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const markAsPaidMutation = useMutation({
    mutationFn: (action: BackendAction) => api.runAction(action),
    onSuccess: (_result, action) => {
      queryClient.invalidateQueries({ queryKey: ["salary-overview"] });
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "已标记为已发放",
      });
    },
    onError: (error) => {
      toast({
        title: "操作失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const saveConfigMutation = useMutation({
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
      queryClient.invalidateQueries({ queryKey: ["salary-overview"] });
      setConfigDialogOpen(false);
      toast({
        title: action.success_title || "成功",
        description: action.success_message || "社保公积金配置已保存",
      });
    },
    onError: (error) => {
      toast({
        title: "保存失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const handleEdit = (salary: SalaryRow) => {
    setSelectedSalary(salary);
    setDialogOpen(true);
  };

  const handleDelete = (action: DeleteAction) => {
    setSalaryToDelete(action);
    setDeleteDialogOpen(true);
  };

  return (
    <div className="space-y-6">
      <Tabs defaultValue="list" className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold text-foreground">工资管理</h1>
            <p className="text-muted-foreground mt-2">管理员工薪资和绩效</p>
          </div>
          <div className="flex items-center gap-4">
            <TabsList>
              <TabsTrigger value="list">工资列表</TabsTrigger>
              <TabsTrigger value="analysis">成本分析</TabsTrigger>
            </TabsList>
            <div className="flex gap-2">
              <Button
                variant="outline"
                className="gap-2"
                disabled={
                  !currentInsuranceConfig?.update_action &&
                  !salaryOverview.insurance_config_create_action
                }
                onClick={() => setConfigDialogOpen(true)}
              >
                <Settings className="h-4 w-4" />
                社保配置
              </Button>
              <Button variant="outline" className="gap-2">
                <Download className="h-4 w-4" />
                导出工资表
              </Button>
              {salaryOverview.create_action && (
                <Button
                  className="gap-2"
                  onClick={() => {
                    setSelectedSalary(null);
                    setDialogOpen(true);
                  }}
                >
                  <Plus className="h-4 w-4" />
                  录入工资
                </Button>
              )}
            </div>
          </div>
        </div>

        <TabsContent value="list" className="space-y-6 mt-0">
          <DependencyAlert dependency="employees" />

          <div className="grid gap-4 md:grid-cols-4">
            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">
                  累计工资总额
                </CardTitle>
                <DollarSign className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats.total_label}</div>
                <p className="text-xs text-muted-foreground">所有记录</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">员工人数</CardTitle>
                <Users className="h-4 w-4 text-accent" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">
                  {stats.employees_label}
                </div>
                <p className="text-xs text-muted-foreground">已录入工资</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">平均工资</CardTitle>
                <TrendingUp className="h-4 w-4 text-primary" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats.average_label}</div>
                <p className="text-xs text-muted-foreground">月平均薪资</p>
              </CardContent>
            </Card>

            <Card>
              <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                <CardTitle className="text-sm font-medium">待发放</CardTitle>
                <Calendar className="h-4 w-4 text-warning" />
              </CardHeader>
              <CardContent>
                <div className="text-2xl font-bold">{stats.pending_label}</div>
                <p className="text-xs text-muted-foreground">待发放记录</p>
              </CardContent>
            </Card>
          </div>

          <Card>
            <CardHeader>
              <CardTitle>工资月度趋势</CardTitle>
            </CardHeader>
            <CardContent>
              <ResponsiveContainer width="100%" height={300}>
                <BarChart data={monthlyTrend}>
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
                    formatter={(_value, _name, item: any) =>
                      item?.payload?.total_label || ""
                    }
                  />
                  <Legend />
                  <Bar
                    dataKey="total"
                    fill="hsl(var(--primary))"
                    name="工资总额"
                    radius={[4, 4, 0, 0]}
                  />
                </BarChart>
              </ResponsiveContainer>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle>员工工资列表</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="mb-4">
                <div className="relative">
                  <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
                  <Input
                    placeholder="搜索员工姓名、部门、职位..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-10"
                  />
                </div>
              </div>

              {isLoading ? (
                <div className="text-center py-8">加载中...</div>
              ) : salaries.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  {salaryOverview.empty_message}
                </div>
              ) : (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>姓名</TableHead>
                        <TableHead>部门</TableHead>
                        <TableHead>职位</TableHead>
                        <TableHead>工资期</TableHead>
                        <TableHead>基本工资</TableHead>
                        <TableHead>奖金</TableHead>
                        <TableHead>扣款</TableHead>
                        <TableHead>社保（个人）</TableHead>
                        <TableHead>社保（公司）</TableHead>
                        <TableHead>公积金（个人）</TableHead>
                        <TableHead>公积金（公司）</TableHead>
                        <TableHead>实发工资</TableHead>
                        <TableHead>状态</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {salaries.map((salary) => (
                        <TableRow key={salary.id}>
                          <TableCell className="font-medium">
                            {salary.employee_name}
                          </TableCell>
                          <TableCell>{salary.department || "-"}</TableCell>
                          <TableCell>{salary.position || "-"}</TableCell>
                          <TableCell>{salary.period}</TableCell>
                          <TableCell>{salary.base_salary_label}</TableCell>
                          <TableCell className="text-accent">
                            {salary.bonus_label}
                          </TableCell>
                          <TableCell className="text-destructive">
                            {salary.deduction_label}
                          </TableCell>
                          <TableCell className="text-muted-foreground">
                            {salary.social_security_label}
                          </TableCell>
                          <TableCell className="text-muted-foreground">
                            {salary.social_security_company_label}
                          </TableCell>
                          <TableCell className="text-muted-foreground">
                            {salary.housing_fund_label}
                          </TableCell>
                          <TableCell className="text-muted-foreground">
                            {salary.housing_fund_company_label}
                          </TableCell>
                          <TableCell className="font-semibold">
                            {salary.actual_salary_label}
                          </TableCell>
                          <TableCell>
                            <Badge className={salary.status_badge.class_name}>
                              {salary.status_badge.label}
                            </Badge>
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-2">
                              {salary.can_mark_paid &&
                                salary.mark_paid_action && (
                                 <Button
                                   variant="outline"
                                   size="sm"
                                   onClick={() =>
                                     markAsPaidMutation.mutate(
                                       salary.mark_paid_action,
                                     )
                                   }
                                 >
                                   {salary.mark_paid_action.label || "标记已发"}
                                 </Button>
                               )}
                              {permissions.can_update &&
                                salary.update_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleEdit(salary)}
                                >
                                  <Edit className="h-4 w-4" />
                                </Button>
                              )}
                              {salary.can_delete && salary.delete_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() =>
                                    handleDelete(salary.delete_action)
                                  }
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

        <TabsContent value="analysis" className="space-y-6 mt-0">
          <SalaryCostAnalysis analysis={costAnalysis} />
        </TabsContent>
      </Tabs>

      <SalaryDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSubmit={(values) => {
          const action = selectedSalary
            ? selectedSalary.update_action
            : salaryOverview.create_action;
          if (!action) {
            toast({
              title: "操作失败",
              description: "当前账号没有保存工资记录的权限",
              variant: "destructive",
            });
            return;
          }
          saveMutation.mutate({ action, values });
        }}
        initialData={selectedSalary}
        employees={employees || []}
        insurancePreviewAction={salaryOverview.insurance_preview_action}
      />

      <SocialInsuranceConfigDialog
        open={configDialogOpen}
        onOpenChange={setConfigDialogOpen}
        onSubmit={(values) => {
          const action =
            currentInsuranceConfig?.update_action ??
            salaryOverview.insurance_config_create_action;
          if (!action) {
            toast({
              title: "保存失败",
              description: "当前账号没有保存社保配置的权限",
              variant: "destructive",
            });
            return;
          }
          saveConfigMutation.mutate({ action, values });
        }}
        initialData={currentInsuranceConfig}
      />

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {salaryToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = salaryToDelete;
                if (action) deleteMutation.mutate(action);
              }}
            >
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default Salary;
