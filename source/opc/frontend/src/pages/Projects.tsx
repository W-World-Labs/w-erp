import { Fragment, useState } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
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
import { Progress } from "@/components/ui/progress";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Rocket,
  Plus,
  Search,
  Target,
  TrendingUp,
  Calendar,
  CheckCircle2,
  Clock,
  AlertTriangle,
  Edit,
  Trash2,
  Brain,
  DollarSign,
  PieChart,
  ChevronDown,
  ChevronRight,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction, type DeleteAction } from "@/integrations/api/client";
import { API_ENDPOINTS } from "@/lib/api";
import { ProjectDialog } from "@/components/projects/ProjectDialog";
import { RdCostDialog } from "@/components/projects/RdCostDialog";
import { AIAuditDialog } from "@/components/applications/AIAuditDialog";
import {
  PieChart as RechartsPieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  Tooltip as RechartsTooltip,
  Legend,
} from "recharts";

type BadgeVariant = "default" | "secondary" | "destructive" | "outline";

interface ProjectBadge {
  label: string;
  variant: BadgeVariant;
  icon?: string;
  class_name?: string;
}

interface OverviewOption {
  value: string;
  label: string;
  description?: string;
}

interface ProjectsOverview {
  stats: {
    total_projects: number;
    ongoing_count: number;
    total_budget: number;
    total_budget_label: string;
    total_members: number;
    total_rd_cost: number;
    total_rd_cost_label: string;
  };
  projects: any[];
  status_options: OverviewOption[];
  rd_cost_type_options: OverviewOption[];
  employee_options: OverviewOption[];
  asset_options: OverviewOption[];
  create_action: BackendAction | null;
  permissions: {
    can_create: boolean;
    can_edit: boolean;
    can_delete: boolean;
    can_manage_rd_costs: boolean;
  };
  empty_message: string;
}

const emptyProjectsOverview: ProjectsOverview = {
  stats: {
    total_projects: 0,
    ongoing_count: 0,
    total_budget: 0,
    total_budget_label: "¥0.0万",
    total_members: 0,
    total_rd_cost: 0,
    total_rd_cost_label: "¥0.0万",
  },
  projects: [],
  status_options: [],
  rd_cost_type_options: [],
  employee_options: [],
  asset_options: [],
  create_action: null,
  permissions: {
    can_create: false,
    can_edit: false,
    can_delete: false,
    can_manage_rd_costs: false,
  },
  empty_message: "暂无项目数据",
};

const COST_COLORS = [
  "#6366f1",
  "#22c55e",
  "#f59e0b",
  "#ef4444",
  "#8b5cf6",
  "#64748b",
];

const badgeIcons: Record<string, LucideIcon> = {
  AlertTriangle,
  Calendar,
  CheckCircle2,
  Clock,
  TrendingUp,
};

function ProjectBadgeView({ badge }: { badge?: ProjectBadge }) {
  if (!badge) return null;
  const Icon = badge.icon ? badgeIcons[badge.icon] : null;
  return (
    <Badge variant={badge.variant} className={badge.class_name}>
      {Icon && <Icon className="h-3 w-3" />}
      {badge.label}
    </Badge>
  );
}

const Projects = () => {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedProject, setSelectedProject] = useState<any>(null);
  const [projectToDelete, setProjectToDelete] = useState<DeleteAction | null>(
    null,
  );
  const [aiAuditOpen, setAiAuditOpen] = useState(false);
  const [aiAuditProject, setAiAuditProject] = useState<any>(null);
  // R&D cost states
  const [expandedProjectId, setExpandedProjectId] = useState<string | null>(
    null,
  );
  const [rdCostDialogOpen, setRdCostDialogOpen] = useState(false);
  const [rdCostProjectId, setRdCostProjectId] = useState<string>("");
  const [selectedRdCost, setSelectedRdCost] = useState<any>(null);
  const [rdCostToDelete, setRdCostToDelete] = useState<DeleteAction | null>(
    null,
  );

  const { data: overview = emptyProjectsOverview, isLoading } = useQuery({
    queryKey: ["projects-overview", searchTerm, statusFilter],
    queryFn: async () => {
      const params = new URLSearchParams();
      if (searchTerm.trim()) params.set("search", searchTerm.trim());
      if (statusFilter !== "all") params.set("status", statusFilter);
      const suffix = params.toString() ? `?${params.toString()}` : "";
      return api.get<ProjectsOverview>(
        `${API_ENDPOINTS.projectsOverview}${suffix}`,
      );
    },
  });
  const projects = overview.projects;
  const stats = overview.stats;
  const permissions = overview.permissions;

  const saveMutation = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => {
      return await api.runAction(action, values);
    },
    onSuccess: (_data, variables) => {
      queryClient.invalidateQueries({ queryKey: ["projects-overview"] });
      toast({
        title: variables.action.success_title || "保存成功",
        description: variables.action.success_message || "项目已保存",
      });
      setDialogOpen(false);
      setSelectedProject(null);
    },
    onError: (error) => {
      toast({
        title: "保存失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  // 删除项目
  const deleteMutation = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["projects-overview"] });
      toast({ title: action.success_title, description: action.success_message });
      setProjectToDelete(null);
    },
    onError: (error) => {
      toast({
        title: "删除失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  // 删除研发费用
  const deleteRdCostMutation = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["projects-overview"] });
      toast({ title: action.success_title, description: action.success_message });
      setRdCostToDelete(null);
    },
    onError: (error) => {
      toast({
        title: "删除失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const handleEdit = (project: any) => {
    setSelectedProject(project);
    setDialogOpen(true);
  };

  const handleDelete = (action: DeleteAction) => {
    setProjectToDelete(action);
  };

  const handleSubmit = (values: any) => {
    const action = selectedProject
      ? selectedProject.update_action
      : overview.create_action;
    if (!action) {
      toast({
        title: "无可用操作",
        description: "当前账号没有保存项目的权限",
        variant: "destructive",
      });
      return;
    }
    saveMutation.mutate({ action, values });
  };

  const toggleExpand = (projectId: string) => {
    setExpandedProjectId(expandedProjectId === projectId ? null : projectId);
  };

  const openAddRdCost = (projectId: string) => {
    setRdCostProjectId(projectId);
    setSelectedRdCost(null);
    setRdCostDialogOpen(true);
  };

  const openEditRdCost = (projectId: string, cost: any) => {
    setRdCostProjectId(projectId);
    setSelectedRdCost(cost);
    setRdCostDialogOpen(true);
  };

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-foreground">研究项目管理</h1>
          <p className="text-muted-foreground">
            项目全生命周期管理与研发费用归集
          </p>
        </div>
        <Button
          className="gap-2"
          disabled={!overview.create_action}
          onClick={() => {
            setSelectedProject(null);
            setDialogOpen(true);
          }}
        >
          <Plus className="h-4 w-4" />
          新建项目
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-4 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">项目总数</CardTitle>
            <Rocket className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_projects}</div>
            <p className="text-xs text-muted-foreground">累计立项项目</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">进行中</CardTitle>
            <TrendingUp className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {stats.ongoing_count}
            </div>
            <p className="text-xs text-muted-foreground">正在执行的项目</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">项目预算</CardTitle>
            <Target className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_budget_label}</div>
            <p className="text-xs text-muted-foreground">总项目预算</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">研发投入</CardTitle>
            <DollarSign className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {stats.total_rd_cost_label}
            </div>
            <p className="text-xs text-muted-foreground">累计研发费用</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <Tabs defaultValue="list" className="w-full">
            <TabsList>
              <TabsTrigger value="list">项目列表</TabsTrigger>
            </TabsList>
            <div className="flex gap-4 mt-4">
              <div className="relative flex-1">
                <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder="搜索项目编号或名称..."
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-10"
                />
              </div>
              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-[180px]">
                  <SelectValue placeholder="筛选状态" />
                </SelectTrigger>
                <SelectContent>
                  {overview.status_options.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>
            <TabsContent value="list" className="mt-6">
              {isLoading ? (
                <div className="flex justify-center py-8">加载中...</div>
              ) : projects.length === 0 ? (
                <div className="flex justify-center py-8 text-muted-foreground">
                  {overview.empty_message}
                </div>
              ) : (
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead className="w-8"></TableHead>
                      <TableHead>项目编号</TableHead>
                      <TableHead>项目名称</TableHead>
                      <TableHead>类型</TableHead>
                      <TableHead>负责人</TableHead>
                      <TableHead>预算</TableHead>
                      <TableHead>研发投入</TableHead>
                      <TableHead>进度</TableHead>
                      <TableHead>状态</TableHead>
                      <TableHead>操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {projects.map((project) => {
                      const projectCosts = project.rd_cost_summary;
                      const isExpanded = expandedProjectId === project.id;
                      return (
                        <Fragment key={project.id}>
                          <TableRow
                            className="cursor-pointer"
                            onClick={() => toggleExpand(project.id)}
                          >
                            <TableCell>
                              {isExpanded ? (
                                <ChevronDown className="h-4 w-4 text-muted-foreground" />
                              ) : (
                                <ChevronRight className="h-4 w-4 text-muted-foreground" />
                              )}
                            </TableCell>
                            <TableCell className="font-medium">
                              {project.project_no}
                            </TableCell>
                            <TableCell>{project.name}</TableCell>
                            <TableCell>{project.project_type}</TableCell>
                            <TableCell>{project.leader_name}</TableCell>
                            <TableCell>{project.budget_label}</TableCell>
                            <TableCell>
                              {project.rd_cost_summary.items.length > 0 ? (
                                <Badge
                                  variant="outline"
                                  className="gap-1 text-green-600 border-green-200"
                                >
                                  <DollarSign className="h-3 w-3" />
                                  {project.rd_cost_summary.total_wan_label}
                                </Badge>
                              ) : (
                                <span className="text-muted-foreground text-sm">
                                  -
                                </span>
                              )}
                            </TableCell>
                            <TableCell>
                              <div className="space-y-1 min-w-[100px]">
                                <div className="flex justify-between text-sm">
                                  <span>{project.progress_label}</span>
                                </div>
                                <Progress
                                  value={project.progress}
                                  className="h-2"
                                />
                              </div>
                            </TableCell>
                            <TableCell>
                              <ProjectBadgeView badge={project.status_badge} />
                            </TableCell>
                            <TableCell>
                              <div
                                className="flex gap-2"
                                onClick={(e) => e.stopPropagation()}
                              >
                                {project.can_add_rd_cost && project.rd_cost_create_action && (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    title="添加研发费用"
                                    onClick={() => openAddRdCost(project.id)}
                                  >
                                    <DollarSign className="h-4 w-4" />
                                  </Button>
                                )}
                                <Button
                                  variant="outline"
                                  size="sm"
                                  onClick={() => {
                                    setAiAuditProject(project);
                                    setAiAuditOpen(true);
                                  }}
                                >
                                  <Brain className="h-4 w-4" />
                                </Button>
                                {project.can_edit && project.update_action && (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() => handleEdit(project)}
                                  >
                                    <Edit className="h-4 w-4" />
                                  </Button>
                                )}
                                {project.can_delete && project.delete_action && (
                                  <Button
                                    variant="outline"
                                    size="sm"
                                    onClick={() =>
                                      handleDelete(project.delete_action)
                                    }
                                  >
                                    <Trash2 className="h-4 w-4" />
                                  </Button>
                                )}
                              </div>
                            </TableCell>
                          </TableRow>

                          {/* 展开的研发费用详情 */}
                          {isExpanded && (
                            <TableRow key={`${project.id}-expand`}>
                              <TableCell
                                colSpan={10}
                                className="bg-muted/30 p-0"
                              >
                                <div className="p-4 space-y-4">
                                  <div className="flex items-center justify-between">
                                    <h4 className="font-semibold text-sm flex items-center gap-2">
                                      <PieChart className="h-4 w-4 text-primary" />
                                      研发费用明细 — {project.name}
                                    </h4>
                                    {project.can_add_rd_cost && project.rd_cost_create_action && (
                                      <Button
                                        size="sm"
                                        variant="outline"
                                        onClick={() =>
                                          openAddRdCost(project.id)
                                        }
                                      >
                                        <Plus className="h-3 w-3 mr-1" />{" "}
                                        添加费用
                                      </Button>
                                    )}
                                  </div>

                                  {!projectCosts ||
                                  projectCosts.items.length === 0 ? (
                                    <div className="text-center py-6 text-muted-foreground text-sm">
                                      {project.rd_cost_summary.empty_message}
                                    </div>
                                  ) : (
                                    <div className="grid grid-cols-1 lg:grid-cols-3 gap-4">
                                      {/* 饼图 */}
                                      <Card className="lg:col-span-1">
                                        <CardContent className="pt-4">
                                          <div className="text-center mb-2">
                                            <div className="text-lg font-bold text-primary">
                                              {
                                                project.rd_cost_summary
                                                  .total_label
                                              }
                                            </div>
                                            <div className="text-xs text-muted-foreground">
                                              研发费用合计
                                            </div>
                                          </div>
                                          <ResponsiveContainer
                                            width="100%"
                                            height={200}
                                          >
                                            <RechartsPieChart>
                                              <Pie
                                                data={projectCosts.pie_data}
                                                cx="50%"
                                                cy="50%"
                                                innerRadius={40}
                                                outerRadius={70}
                                                paddingAngle={2}
                                                dataKey="value"
                                              >
                                                {projectCosts.pie_data.map(
                                                  (_: any, index: number) => (
                                                    <Cell
                                                      key={`cell-${index}`}
                                                      fill={
                                                        COST_COLORS[
                                                          index %
                                                            COST_COLORS.length
                                                        ]
                                                      }
                                                    />
                                                  ),
                                                )}
                                              </Pie>
                                              <RechartsTooltip
                                                formatter={(
                                                  _value: number,
                                                  _name: string,
                                                  item: any,
                                                ) =>
                                                  item?.payload?.amount_label ||
                                                  ""
                                                }
                                              />
                                              <Legend />
                                            </RechartsPieChart>
                                          </ResponsiveContainer>
                                        </CardContent>
                                      </Card>

                                      {/* 费用明细表 */}
                                      <Card className="lg:col-span-2">
                                        <CardContent className="pt-4">
                                          <Table>
                                            <TableHeader>
                                              <TableRow>
                                                <TableHead>费用类型</TableHead>
                                                <TableHead>日期</TableHead>
                                                <TableHead>说明</TableHead>
                                                <TableHead className="text-right">
                                                  金额
                                                </TableHead>
                                                <TableHead className="text-right">
                                                  操作
                                                </TableHead>
                                              </TableRow>
                                            </TableHeader>
                                            <TableBody>
                                              {projectCosts.items.map(
                                                (cost: any) => (
                                                  <TableRow key={cost.id}>
                                                    <TableCell>
                                                      <ProjectBadgeView
                                                        badge={
                                                          cost.cost_type_badge
                                                        }
                                                      />
                                                    </TableCell>
                                                    <TableCell className="text-sm">
                                                      {cost.cost_date || "-"}
                                                    </TableCell>
                                                    <TableCell className="text-sm text-muted-foreground max-w-[200px] truncate">
                                                      {cost.description || "-"}
                                                    </TableCell>
                                                    <TableCell className="text-right font-medium">
                                                      {cost.amount_label}
                                                    </TableCell>
                                                    <TableCell className="text-right">
                                                      <div className="flex justify-end gap-1">
                                                        {cost.can_edit && cost.update_action && (
                                                          <Button
                                                            variant="ghost"
                                                            size="sm"
                                                            onClick={() =>
                                                              openEditRdCost(
                                                                project.id,
                                                                cost,
                                                              )
                                                            }
                                                          >
                                                            <Edit className="h-3 w-3" />
                                                          </Button>
                                                        )}
                                                        {cost.can_delete &&
                                                          cost.delete_action && (
                                                          <Button
                                                            variant="ghost"
                                                            size="sm"
                                                            onClick={() =>
                                                              setRdCostToDelete(
                                                                cost.delete_action,
                                                              )
                                                            }
                                                          >
                                                            <Trash2 className="h-3 w-3" />
                                                          </Button>
                                                        )}
                                                      </div>
                                                    </TableCell>
                                                  </TableRow>
                                                ),
                                              )}
                                            </TableBody>
                                          </Table>
                                        </CardContent>
                                      </Card>
                                    </div>
                                  )}
                                </div>
                              </TableCell>
                            </TableRow>
                          )}
                        </Fragment>
                      );
                    })}
                  </TableBody>
                </Table>
              )}
            </TabsContent>
          </Tabs>
        </CardHeader>
      </Card>

      <ProjectDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSubmit={handleSubmit}
        initialData={selectedProject}
      />

      <RdCostDialog
        open={rdCostDialogOpen}
        onOpenChange={setRdCostDialogOpen}
        projectId={rdCostProjectId}
        cost={selectedRdCost}
        action={
          selectedRdCost
            ? selectedRdCost.update_action
            : overview.projects.find((project) => project.id === rdCostProjectId)
                ?.rd_cost_create_action
        }
        rdCostTypeOptions={overview.rd_cost_type_options}
        employeeOptions={overview.employee_options}
        assetOptions={overview.asset_options}
        onSuccess={() => {
          queryClient.invalidateQueries({ queryKey: ["projects-overview"] });
        }}
      />

      <AlertDialog
        open={!!projectToDelete}
        onOpenChange={() => setProjectToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {projectToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = projectToDelete;
                if (action) deleteMutation.mutate(action);
              }}
            >
              确认删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AlertDialog
        open={!!rdCostToDelete}
        onOpenChange={() => setRdCostToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {rdCostToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = rdCostToDelete;
                if (action) deleteRdCostMutation.mutate(action);
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
        auditType="project"
        targetId={aiAuditProject?.id}
        title={`项目评估：${aiAuditProject?.name || ""}`}
      />
    </div>
  );
};

export default Projects;
