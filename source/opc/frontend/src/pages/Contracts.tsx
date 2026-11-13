import { useState } from "react";
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
import {
  AlertCircle,
  Archive,
  Brain,
  CheckCircle,
  Clock,
  Download,
  Edit,
  Eye,
  FileText,
  Loader2,
  Plus,
  Search,
  ShieldAlert,
  Trash2,
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  api,
  files,
  type BackendAction,
  type DeleteAction,
} from "@/integrations/api/client";
import { ContractDialog } from "@/components/contracts/ContractDialog";
import { ContractReviewDialog } from "@/components/contracts/ContractReviewDialog";
import { DependencyAlert } from "@/components/common/DependencyAlert";
import { AIAuditDialog } from "@/components/applications/AIAuditDialog";
import { FilePreviewDialog } from "@/components/common/FilePreviewDialog";
import { API_ENDPOINTS } from "@/lib/api/contracts";
import { cn } from "@/lib/utils";

const badgeIcons = {
  AlertCircle,
  Archive,
  CheckCircle,
  Clock,
  FileText,
  Loader2,
  ShieldAlert,
};

type BadgeData = {
  label?: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon?: string;
  spin?: boolean;
  class_name?: string;
};

const defaultOverview = {
  stats: {
    total: 0,
    active: 0,
    expired: 0,
    total_amount_label: "¥0.0万",
    ai_guard_label: "安全",
    ai_guard_tone: "safe",
    ai_guard_icon: "CheckCircle",
    ai_guard_display: {
      label: "安全",
      icon: "CheckCircle",
      card_class_name: "",
      icon_class_name: "text-green-500",
      value_class_name: "text-green-500",
    },
  },
  contracts: [] as any[],
  status_options: [] as any[],
  create_action: null as BackendAction | null,
  permissions: {
    can_manage: false,
    can_create: false,
    can_edit: false,
    can_delete: false,
  },
};

const renderBadgeIcon = (badge?: BadgeData, className = "h-3 w-3") => {
  const Icon =
    badgeIcons[badge?.icon as keyof typeof badgeIcons] ?? badgeIcons.FileText;
  return <Icon className={cn(className, badge?.spin && "animate-spin")} />;
};

const Contracts = () => {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState("all");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedContract, setSelectedContract] = useState<any>(null);
  const [contractToDelete, setContractToDelete] =
    useState<DeleteAction | null>(null);
  const [aiAuditOpen, setAiAuditOpen] = useState(false);
  const [aiAuditContract, setAiAuditContract] = useState<any>(null);
  const [previewContract, setPreviewContract] = useState<any>(null);
  const [reviewDialogOpen, setReviewDialogOpen] = useState(false);
  const [reviewContract, setReviewContract] = useState<any>(null);

  const { data: overview = defaultOverview, isLoading } = useQuery({
    queryKey: ["contracts-overview", searchTerm, statusFilter],
    queryFn: async () => {
      const params = new URLSearchParams();
      const trimmedSearch = searchTerm.trim();
      if (trimmedSearch) {
        params.set("search", trimmedSearch);
      }
      if (statusFilter !== "all") {
        params.set("status", statusFilter);
      }
      const query = params.toString();
      return api.get<typeof defaultOverview>(
        `${API_ENDPOINTS.contractsOverview}${query ? `?${query}` : ""}`,
      );
    },
  });

  const invalidateContracts = () => {
    queryClient.invalidateQueries({ queryKey: ["contracts-overview"] });
  };

  const saveMutation = useMutation({
    mutationFn: async ({
      action,
      values,
    }: {
      action: BackendAction;
      values: Record<string, any>;
    }) => {
      let file_path = values.file_path;
      let file_name = values.file_name;
      if (values.file) {
        const uploadResult = await files.upload(values.file);
        file_path = uploadResult.id;
        file_name = uploadResult.original_name;
      }

      return api.runAction(action, {
        ...values,
        file_path,
        file_name,
        file: undefined,
      });
    },
    onSuccess: (_data, variables) => {
      invalidateContracts();
      toast({
        title: variables.action.success_title || "保存成功",
        description: variables.action.success_message || "合同已保存",
      });
      setDialogOpen(false);
      setSelectedContract(null);
    },
    onError: (error) => {
      toast({
        title: "保存失败",
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
      invalidateContracts();
      toast({ title: action.success_title, description: action.success_message });
      setContractToDelete(null);
    },
    onError: (error) => {
      toast({
        title: "删除失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const handleEdit = (contract: any) => {
    setSelectedContract(contract);
    setDialogOpen(true);
  };

  const handleDelete = (action: DeleteAction) => {
    setContractToDelete(action);
  };

  const handleSubmit = (values: any) => {
    const action = selectedContract
      ? selectedContract.update_action
      : overview.create_action;
    if (!action) {
      toast({
        title: "无可用操作",
        description: "当前账号没有保存合同的权限",
        variant: "destructive",
      });
      return;
    }
    saveMutation.mutate({ action, values });
  };

  const handleDownload = async (filePath: string, fileName: string) => {
    try {
      const data = await files.download(filePath);

      const url = URL.createObjectURL(data);
      const a = document.createElement("a");
      a.href = url;
      a.download = fileName;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      toast({ title: "下载成功", description: `已下载文件: ${fileName}` });
    } catch (error: any) {
      toast({
        title: "下载失败",
        description: error.message,
        variant: "destructive",
      });
    }
  };

  const renderStatusBadge = (badge: BadgeData) => (
    <Badge variant={badge.variant || "outline"} className="gap-1">
      {renderBadgeIcon(badge)}
      {badge.label}
    </Badge>
  );

  const renderAiRiskBadge = (contract: any) => {
    const badge = contract.ai_risk_badge as BadgeData | null;
    if (!badge?.label) {
      return null;
    }
    return (
      <Badge
        variant={badge.variant || "outline"}
        className={cn("gap-1 text-xs cursor-pointer", badge.class_name)}
        onClick={() => {
          setReviewContract(contract);
          setReviewDialogOpen(true);
        }}
      >
        {renderBadgeIcon(badge)}
        {badge.label}
      </Badge>
    );
  };

  const stats = overview.stats;
  const contracts = overview.contracts;
  const permissions = overview.permissions;
  const guardDisplay = stats.ai_guard_display;
  const GuardIcon =
    badgeIcons[guardDisplay.icon as keyof typeof badgeIcons] ??
    badgeIcons.CheckCircle;

  return (
    <div className="container mx-auto p-6 space-y-6">
      <DependencyAlert dependency="company-info" />

      <div className="flex justify-between items-center">
        <div>
          <h1 className="text-3xl font-bold text-foreground">合同管理</h1>
          <p className="text-muted-foreground">合同管理与归档</p>
        </div>
        <Button
          className="gap-2"
          disabled={!overview.create_action}
          onClick={() => {
            setSelectedContract(null);
            setDialogOpen(true);
          }}
        >
          <Plus className="h-4 w-4" />
          新建合同
        </Button>
      </div>

      <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">合同总数</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="text-xs text-muted-foreground">累计签订合同</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">执行中</CardTitle>
            <CheckCircle className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {stats.active}
            </div>
            <p className="text-xs text-muted-foreground">正在履行的合同</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">已到期</CardTitle>
            <AlertCircle className="h-4 w-4 text-destructive" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-destructive">
              {stats.expired}
            </div>
            <p className="text-xs text-muted-foreground">需要续签或归档</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">合同总额</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_amount_label}</div>
            <p className="text-xs text-muted-foreground">总合同金额</p>
          </CardContent>
        </Card>
        <Card className={guardDisplay.card_class_name}>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">AI 防坑卫士</CardTitle>
            <GuardIcon
              className={cn(
                "h-4 w-4",
                guardDisplay.icon_class_name,
              )}
            />
          </CardHeader>
          <CardContent>
            <div
              className={cn(
                "text-2xl font-bold",
                guardDisplay.value_class_name,
              )}
            >
              {guardDisplay.label}
            </div>
            <p className="text-xs text-muted-foreground">AI 自动审查所有合同</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>合同列表</CardTitle>
          <div className="flex gap-4 mt-4">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-muted-foreground" />
              <Input
                placeholder="搜索合同编号或标题..."
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
                {overview.status_options.map((status) => (
                  <SelectItem key={status.value} value={status.value}>
                    {status.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex justify-center py-8">加载中...</div>
          ) : contracts.length === 0 ? (
            <div className="flex justify-center py-8 text-muted-foreground">
              暂无合同数据
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>合同编号</TableHead>
                  <TableHead>合同标题</TableHead>
                  <TableHead>类型</TableHead>
                  <TableHead>甲方</TableHead>
                  <TableHead>乙方</TableHead>
                  <TableHead>金额</TableHead>
                  <TableHead>有效期</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead>AI 风险</TableHead>
                  <TableHead>操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {contracts.map((contract) => (
                  <TableRow key={contract.id}>
                    <TableCell className="font-medium">
                      {contract.contract_number}
                    </TableCell>
                    <TableCell>{contract.contract_name}</TableCell>
                    <TableCell>{contract.contract_type}</TableCell>
                    <TableCell>{contract.party_a || "-"}</TableCell>
                    <TableCell>
                      {contract.party_b || contract.counterparty || "-"}
                    </TableCell>
                    <TableCell>{contract.amount_label}</TableCell>
                    <TableCell>
                      <div className="text-sm">
                        <div>{contract.start_date}</div>
                        <div className="text-muted-foreground">
                          至 {contract.end_date}
                        </div>
                      </div>
                    </TableCell>
                    <TableCell>
                      {renderStatusBadge(contract.status_badge)}
                    </TableCell>
                    <TableCell>{renderAiRiskBadge(contract)}</TableCell>
                    <TableCell>
                      <div className="flex gap-2">
                        {contract.file_path && (
                          <>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() => setPreviewContract(contract)}
                              title="预览文件"
                            >
                              <Eye className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={() =>
                                handleDownload(
                                  contract.file_path,
                                  contract.file_name,
                                )
                              }
                              title="下载文件"
                            >
                              <Download className="h-4 w-4" />
                            </Button>
                          </>
                        )}
                        <Button
                          variant="outline"
                          size="sm"
                          onClick={() => {
                            setAiAuditContract(contract);
                            setAiAuditOpen(true);
                          }}
                          title="AI审核"
                        >
                          <Brain className="h-4 w-4" />
                        </Button>
                        {contract.update_action && (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleEdit(contract)}
                            title="编辑"
                          >
                            <Edit className="h-4 w-4" />
                          </Button>
                        )}
                        {(contract.can_delete ?? permissions.can_delete) &&
                          contract.delete_action && (
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => handleDelete(contract.delete_action)}
                            title="删除"
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
          )}
        </CardContent>
      </Card>

      <ContractDialog
        open={dialogOpen}
        onOpenChange={setDialogOpen}
        onSubmit={handleSubmit}
        initialData={selectedContract}
      />

      <AlertDialog
        open={!!contractToDelete}
        onOpenChange={() => setContractToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {contractToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = contractToDelete;
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
        auditType="contract"
        targetId={aiAuditContract?.id}
        title={`合同审核：${aiAuditContract?.contract_name || ""}`}
      />

      <FilePreviewDialog
        open={!!previewContract}
        onOpenChange={(open) => !open && setPreviewContract(null)}
        filePath={previewContract?.file_path || null}
        fileName={previewContract?.file_name || null}
        bucketName="contracts"
      />
      <ContractReviewDialog
        open={reviewDialogOpen}
        onOpenChange={setReviewDialogOpen}
        contract={reviewContract}
      />
    </div>
  );
};

export default Contracts;
