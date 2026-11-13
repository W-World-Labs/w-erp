import { useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { api, type BackendAction } from "@/integrations/api/client";
import { Button } from "@/components/ui/button";
import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Separator } from "@/components/ui/separator";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Building2,
  Calendar,
  CheckCircle2,
  CircleOff,
  Clock,
  Edit,
  FileText,
  Globe,
  GraduationCap,
  Mail,
  MapPin,
  Network,
  PauseCircle,
  Phone,
  Plane,
  Plus,
  Star,
  Trash2,
} from "lucide-react";
import { CompanyInfoDialog } from "@/components/company/CompanyInfoDialog";
import { BranchDialog } from "@/components/branches/BranchDialog";
import { MilestoneDialog } from "@/components/timeline/MilestoneDialog";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
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
import { ScrollArea } from "@/components/ui/scroll-area";
import { toast } from "sonner";

const COMPANY_INFO_OVERVIEW_ENDPOINT = "/api/v1/company_info/overview";

interface BadgeDisplay {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon?: string | null;
}

interface DisplayOption {
  value: string;
  label: string;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

interface CompanyOverview {
  id: string;
  company_name: string;
  legal_representative?: string;
  unified_social_credit_code?: string;
  registration_date?: string;
  registration_date_label: string;
  registered_capital?: number;
  registered_capital_label: string;
  contact_phone?: string;
  contact_email?: string;
  company_address?: string;
  business_scope?: string;
  notes?: string;
  founder_badges: BadgeDisplay[];
  record_badge: BadgeDisplay;
  update_action: BackendAction | null;
}

interface BranchOverview {
  id: string;
  branch_name: string;
  branch_type?: string;
  branch_type_badge: BadgeDisplay;
  legal_representative?: string;
  contact_person?: string;
  branch_address?: string;
  status?: string;
  status_badge: BadgeDisplay;
  parent_branch_options: DisplayOption[];
  can_delete: boolean;
  can_activate: boolean;
  can_mark_preparing: boolean;
  can_suspend: boolean;
  can_close: boolean;
  activate_action: BackendAction | null;
  mark_preparing_action: BackendAction | null;
  suspend_action: BackendAction | null;
  close_action: BackendAction | null;
  update_action: BackendAction | null;
  delete_action: DeleteAction | null;
}

interface TimelineFilter {
  value: string;
  label: string;
  icon?: string | null;
}

interface TimelineEvent {
  id: string;
  date_label: string;
  title: string;
  description?: string;
  type: string;
  icon: string;
  color: string;
  record_id?: string;
  milestone_date?: string;
  milestone_type?: string;
  importance?: string;
  notes?: string;
  update_action?: BackendAction | null;
}

interface CompanyInfoOverview {
  company: CompanyOverview | null;
  branches: BranchOverview[];
  parent_branch_options: DisplayOption[];
  company_create_action: BackendAction | null;
  branch_create_action: BackendAction | null;
  milestone_create_action: BackendAction | null;
  timeline: {
    selected_type: string;
    filters: TimelineFilter[];
    events: TimelineEvent[];
    empty_message: string;
  };
  permissions: {
    can_create_company_info: boolean;
    can_edit_company_info: boolean;
    can_create_branch: boolean;
    can_delete_branch: boolean;
    can_create_milestone: boolean;
  };
  empty_messages: {
    company: string;
    branches: string;
    timeline: string;
  };
}

type CompanyInfoTab = "info" | "branches" | "timeline";

interface CompanyInfoProps {
  initialTab?: CompanyInfoTab;
}

const emptyOverview: CompanyInfoOverview = {
  company: null,
  branches: [],
  parent_branch_options: [],
  company_create_action: null,
  branch_create_action: null,
  milestone_create_action: null,
  timeline: {
    selected_type: "all",
    filters: [
      { value: "all", label: "全部事件" },
      { value: "milestone", label: "里程碑", icon: "Star" },
      { value: "company", label: "公司", icon: "Building2" },
      { value: "branch", label: "分支机构", icon: "Network" },
    ],
    events: [],
    empty_message: "暂无时间线事件",
  },
  permissions: {
    can_create_company_info: false,
    can_edit_company_info: false,
    can_create_branch: false,
    can_delete_branch: false,
    can_create_milestone: false,
  },
  empty_messages: {
    company: "还未添加公司信息",
    branches: "还未添加分支机构",
    timeline: "暂无时间线事件",
  },
};

const iconMap = {
  Building2,
  Network,
  Star,
  GraduationCap,
  Globe,
  Plane,
};

function IconByName({
  name,
  className = "h-4 w-4",
}: {
  name?: string | null;
  className?: string;
}) {
  if (!name) return null;
  const Icon = iconMap[name as keyof typeof iconMap];
  return Icon ? <Icon className={className} /> : null;
}

function DisplayBadge({ badge }: { badge: BadgeDisplay }) {
  return (
    <Badge
      variant={badge.variant || "secondary"}
      className="flex items-center gap-1"
    >
      <IconByName name={badge.icon} className="h-3 w-3" />
      {badge.label}
    </Badge>
  );
}

const CompanyInfo = ({ initialTab = "info" }: CompanyInfoProps) => {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedCompany, setSelectedCompany] =
    useState<CompanyOverview | null>(null);
  const [branchDialogOpen, setBranchDialogOpen] = useState(false);
  const [selectedBranch, setSelectedBranch] = useState<BranchOverview | null>(
    null,
  );
  const [deleteBranchDialogOpen, setDeleteBranchDialogOpen] = useState(false);
  const [branchToDelete, setBranchToDelete] = useState<DeleteAction | null>(
    null,
  );
  const [milestoneDialogOpen, setMilestoneDialogOpen] = useState(false);
  const [timelineFilter, setTimelineFilter] = useState("all");

  const {
    data: overview = emptyOverview,
    isLoading,
    refetch,
  } = useQuery({
    queryKey: ["company-info-overview", timelineFilter],
    queryFn: () =>
      api.get<CompanyInfoOverview>(
        `${COMPANY_INFO_OVERVIEW_ENDPOINT}?timeline_type=${timelineFilter}`,
      ),
  });

  const companyInfo = overview.company;
  const branches = overview.branches;
  const timeline = overview.timeline;
  const permissions = overview.permissions;

  const handleAdd = () => {
    setSelectedCompany(null);
    setDialogOpen(true);
  };

  const handleEdit = () => {
    setSelectedCompany(companyInfo);
    setDialogOpen(true);
  };

  const handleSuccess = () => {
    refetch();
  };

  const handleAddBranch = () => {
    setSelectedBranch(null);
    setBranchDialogOpen(true);
  };

  const handleEditBranch = (branch: BranchOverview) => {
    setSelectedBranch(branch);
    setBranchDialogOpen(true);
  };

  const handleDeleteBranchClick = (branch: BranchOverview) => {
    setBranchToDelete(branch.delete_action);
    setDeleteBranchDialogOpen(true);
  };

  const handleBranchStatusAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "分支机构状态已更新");
      refetch();
    } catch (error) {
      console.error("Error updating branch status:", error);
      toast.error("状态更新失败");
    }
  };

  const handleDeleteBranch = async () => {
    if (!branchToDelete) return;
    try {
      await api.runAction(branchToDelete);
      toast.success(branchToDelete.success_message);
      refetch();
    } catch (error) {
      console.error("Error deleting branch:", error);
      toast.error("删除失败");
    } finally {
      setDeleteBranchDialogOpen(false);
      setBranchToDelete(null);
    }
  };

  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-bold">公司信息</h1>
            <p className="text-muted-foreground mt-2">
              管理公司基本信息和注册资料
            </p>
          </div>
        </div>
        <Card>
          <CardContent className="p-6">
            <div className="animate-pulse space-y-4">
              <div className="h-4 bg-muted rounded w-1/4"></div>
              <div className="h-4 bg-muted rounded w-1/2"></div>
              <div className="h-4 bg-muted rounded w-3/4"></div>
            </div>
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold flex items-center gap-2">
            <Building2 className="h-8 w-8 text-primary" />
            公司信息
          </h1>
          <p className="text-muted-foreground mt-2">
            管理公司基本信息、分支机构和企业时间线
          </p>
        </div>
      </div>

      <Tabs defaultValue={initialTab} className="space-y-6">
        <TabsList>
          <TabsTrigger value="info">基本信息</TabsTrigger>
          <TabsTrigger value="branches">分支机构</TabsTrigger>
          <TabsTrigger value="timeline">企业时间线</TabsTrigger>
        </TabsList>

        <TabsContent value="info" className="space-y-6">
          <div className="flex justify-end">
            {companyInfo
              ? companyInfo.update_action && (
                  <Button onClick={handleEdit}>
                    <Edit className="mr-2 h-4 w-4" />
                    编辑信息
                  </Button>
                )
              : overview.company_create_action && (
                  <Button onClick={handleAdd}>
                    <Plus className="mr-2 h-4 w-4" />
                    添加公司信息
                  </Button>
                )}
          </div>

          {companyInfo ? (
            <Card>
              <CardHeader>
                <div className="flex items-center justify-between">
                  <div>
                    <CardTitle className="text-2xl">
                      {companyInfo.company_name}
                    </CardTitle>
                    {companyInfo.legal_representative && (
                      <CardDescription className="mt-2">
                        法定代表人: {companyInfo.legal_representative}
                      </CardDescription>
                    )}
                  </div>
                  <DisplayBadge badge={companyInfo.record_badge} />
                </div>
              </CardHeader>
              <CardContent className="space-y-6">
                <div className="space-y-4">
                  <h3 className="font-semibold flex items-center gap-2">
                    <FileText className="h-4 w-4 text-primary" />
                    注册信息
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pl-6">
                    {companyInfo.unified_social_credit_code && (
                      <div>
                        <div className="text-sm text-muted-foreground">
                          统一社会信用代码
                        </div>
                        <div className="font-medium">
                          {companyInfo.unified_social_credit_code}
                        </div>
                      </div>
                    )}
                    {companyInfo.registration_date && (
                      <div>
                        <div className="text-sm text-muted-foreground">
                          注册日期
                        </div>
                        <div className="font-medium flex items-center gap-1">
                          <Calendar className="h-3 w-3" />
                          {companyInfo.registration_date_label}
                        </div>
                      </div>
                    )}
                    {companyInfo.registered_capital !== undefined && (
                      <div>
                        <div className="text-sm text-muted-foreground">
                          注册资本
                        </div>
                        <div className="font-medium">
                          {companyInfo.registered_capital_label}
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                <Separator />

                <div className="space-y-4">
                  <h3 className="font-semibold flex items-center gap-2">
                    <Phone className="h-4 w-4 text-primary" />
                    联系方式
                  </h3>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pl-6">
                    {companyInfo.contact_phone && (
                      <div>
                        <div className="text-sm text-muted-foreground">
                          联系电话
                        </div>
                        <div className="font-medium flex items-center gap-1">
                          <Phone className="h-3 w-3" />
                          {companyInfo.contact_phone}
                        </div>
                      </div>
                    )}
                    {companyInfo.contact_email && (
                      <div>
                        <div className="text-sm text-muted-foreground">
                          联系邮箱
                        </div>
                        <div className="font-medium flex items-center gap-1">
                          <Mail className="h-3 w-3" />
                          {companyInfo.contact_email}
                        </div>
                      </div>
                    )}
                    {companyInfo.company_address && (
                      <div className="md:col-span-2">
                        <div className="text-sm text-muted-foreground">
                          公司地址
                        </div>
                        <div className="font-medium flex items-center gap-1">
                          <MapPin className="h-3 w-3" />
                          {companyInfo.company_address}
                        </div>
                      </div>
                    )}
                  </div>
                </div>

                {companyInfo.founder_badges.length > 0 && (
                  <>
                    <Separator />
                    <div className="space-y-4">
                      <h3 className="font-semibold flex items-center gap-2">
                        <GraduationCap className="h-4 w-4 text-primary" />
                        创始人画像 (OPC)
                      </h3>
                      <div className="flex flex-wrap gap-2 pl-6">
                        {companyInfo.founder_badges.map((badge) => (
                          <DisplayBadge key={badge.label} badge={badge} />
                        ))}
                      </div>
                    </div>
                  </>
                )}

                {companyInfo.business_scope && (
                  <>
                    <Separator />
                    <div className="space-y-2">
                      <h3 className="font-semibold">经营范围</h3>
                      <p className="text-sm text-muted-foreground pl-6">
                        {companyInfo.business_scope}
                      </p>
                    </div>
                  </>
                )}

                {companyInfo.notes && (
                  <>
                    <Separator />
                    <div className="space-y-2">
                      <h3 className="font-semibold">备注</h3>
                      <p className="text-sm text-muted-foreground pl-6">
                        {companyInfo.notes}
                      </p>
                    </div>
                  </>
                )}
              </CardContent>
            </Card>
          ) : (
            <Card>
              <CardContent className="flex flex-col items-center justify-center py-12">
                <Building2 className="h-16 w-16 text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">
                  {overview.empty_messages.company}
                </h3>
                <p className="text-muted-foreground text-center mb-6 max-w-md">
                  添加公司基本信息，包括注册资料、联系方式等，便于系统管理和后续使用
                </p>
                {overview.company_create_action && (
                  <Button onClick={handleAdd} size="lg">
                    <Plus className="mr-2 h-4 w-4" />
                    立即添加
                  </Button>
                )}
              </CardContent>
            </Card>
          )}
        </TabsContent>

        <TabsContent value="branches" className="space-y-6">
          <div className="flex justify-end">
            {overview.branch_create_action && (
              <Button onClick={handleAddBranch}>
                <Plus className="mr-2 h-4 w-4" />
                添加分支机构
              </Button>
            )}
          </div>

          <Card>
            <CardHeader>
              <CardTitle>分支机构列表</CardTitle>
              <CardDescription>管理所有分支机构信息</CardDescription>
            </CardHeader>
            <CardContent>
              {branches.length > 0 ? (
                <div className="rounded-md border">
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>机构名称</TableHead>
                        <TableHead>类型</TableHead>
                        <TableHead>负责人</TableHead>
                        <TableHead>地址</TableHead>
                        <TableHead>状态</TableHead>
                        <TableHead className="text-right">操作</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {branches.map((branch) => (
                        <TableRow key={branch.id}>
                          <TableCell className="font-medium">
                            {branch.branch_name}
                          </TableCell>
                          <TableCell>
                            <DisplayBadge badge={branch.branch_type_badge} />
                          </TableCell>
                          <TableCell>
                            {branch.legal_representative ||
                              branch.contact_person ||
                              "-"}
                          </TableCell>
                          <TableCell className="max-w-[200px] truncate">
                            {branch.branch_address || "-"}
                          </TableCell>
                          <TableCell>
                            <DisplayBadge badge={branch.status_badge} />
                          </TableCell>
                          <TableCell className="text-right">
                            <div className="flex justify-end gap-2">
                              {branch.can_activate && branch.activate_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  title={branch.activate_action.label || "恢复运营"}
                                  onClick={() =>
                                    handleBranchStatusAction(
                                      branch.activate_action,
                                    )
                                  }
                                >
                                  <CheckCircle2 className="h-4 w-4" />
                                </Button>
                              )}
                              {branch.can_mark_preparing &&
                                branch.mark_preparing_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  title={
                                    branch.mark_preparing_action.label ||
                                    "标记筹建中"
                                  }
                                  onClick={() =>
                                    handleBranchStatusAction(
                                      branch.mark_preparing_action,
                                    )
                                  }
                                >
                                  <Clock className="h-4 w-4" />
                                </Button>
                              )}
                              {branch.can_suspend && branch.suspend_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  title={branch.suspend_action.label || "暂停营业"}
                                  onClick={() =>
                                    handleBranchStatusAction(
                                      branch.suspend_action,
                                    )
                                  }
                                >
                                  <PauseCircle className="h-4 w-4" />
                                </Button>
                              )}
                              {branch.can_close && branch.close_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  title={branch.close_action.label || "关闭机构"}
                                  onClick={() =>
                                    handleBranchStatusAction(branch.close_action)
                                  }
                                >
                                  <CircleOff className="h-4 w-4" />
                                </Button>
                              )}
                              {branch.update_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() => handleEditBranch(branch)}
                                >
                                  <Edit className="h-4 w-4" />
                                </Button>
                              )}
                              {branch.can_delete && branch.delete_action && (
                                <Button
                                  variant="ghost"
                                  size="sm"
                                  onClick={() =>
                                    handleDeleteBranchClick(branch)
                                  }
                                >
                                  <Trash2 className="h-4 w-4 text-destructive" />
                                </Button>
                              )}
                            </div>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </div>
              ) : (
                <div className="text-center py-12">
                  <Network className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                  <h3 className="text-lg font-semibold mb-2">
                    {overview.empty_messages.branches}
                  </h3>
                  <p className="text-muted-foreground mb-6">
                    添加第一个分支机构，开始管理多机构业务
                  </p>
                  {overview.branch_create_action && (
                    <Button onClick={handleAddBranch}>
                      <Plus className="mr-2 h-4 w-4" />
                      添加分支机构
                    </Button>
                  )}
                </div>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        <TabsContent value="timeline" className="space-y-6">
          <div className="flex justify-end gap-2">
            {overview.milestone_create_action && (
              <Button onClick={() => setMilestoneDialogOpen(true)}>
                <Plus className="mr-2 h-4 w-4" />
                添加里程碑
              </Button>
            )}
          </div>

          <Card>
            <CardHeader>
              <CardTitle>企业时间线</CardTitle>
              <CardDescription>
                展示企业发展历程中的重要时间节点
              </CardDescription>
            </CardHeader>
            <CardContent>
              <div className="flex flex-wrap gap-2 mb-6">
                {timeline.filters.map((filter) => (
                  <Button
                    key={filter.value}
                    variant={
                      timelineFilter === filter.value ? "default" : "outline"
                    }
                    size="sm"
                    onClick={() => setTimelineFilter(filter.value)}
                  >
                    <IconByName name={filter.icon} className="mr-1 h-4 w-4" />
                    {filter.label}
                  </Button>
                ))}
              </div>

              <ScrollArea className="h-[600px] pr-4">
                {timeline.events.length > 0 ? (
                  <div className="space-y-4">
                    {timeline.events.map((event, index) => (
                      <div key={event.id} className="flex gap-4">
                        <div className="flex flex-col items-center">
                          <div
                            className={`w-10 h-10 rounded-full ${event.color} flex items-center justify-center text-white`}
                          >
                            <IconByName name={event.icon} className="h-5 w-5" />
                          </div>
                          {index < timeline.events.length - 1 && (
                            <div className="w-0.5 h-full bg-border mt-2"></div>
                          )}
                        </div>
                        <div className="flex-1 pb-8">
                          <div className="text-sm text-muted-foreground mb-1">
                            {event.date_label}
                          </div>
                          <h4 className="font-semibold mb-1">{event.title}</h4>
                          {event.description && (
                            <p className="text-sm text-muted-foreground">
                              {event.description}
                            </p>
                          )}
                        </div>
                      </div>
                    ))}
                  </div>
                ) : (
                  <div className="text-center py-12">
                    <Clock className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                    <h3 className="text-lg font-semibold mb-2">
                      {timeline.empty_message ||
                        overview.empty_messages.timeline}
                    </h3>
                    <p className="text-muted-foreground">
                      开始添加公司信息和里程碑，构建企业发展历程
                    </p>
                  </div>
                )}
              </ScrollArea>
            </CardContent>
          </Card>
        </TabsContent>
      </Tabs>

      <CompanyInfoDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        companyInfo={selectedCompany}
        action={
          selectedCompany
            ? selectedCompany.update_action
            : overview.company_create_action
        }
        onSuccess={handleSuccess}
      />

      <BranchDialog
        open={branchDialogOpen}
        onOpenChange={setBranchDialogOpen}
        branch={selectedBranch}
        action={
          selectedBranch
            ? selectedBranch.update_action
            : overview.branch_create_action
        }
        parentBranchOptions={
          selectedBranch?.parent_branch_options || overview.parent_branch_options
        }
        onSuccess={handleSuccess}
      />

      <MilestoneDialog
        open={milestoneDialogOpen}
        onOpenChange={setMilestoneDialogOpen}
        milestone={null}
        action={overview.milestone_create_action}
      />

      <AlertDialog
        open={deleteBranchDialogOpen}
        onOpenChange={setDeleteBranchDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {branchToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteBranch}>
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default CompanyInfo;
