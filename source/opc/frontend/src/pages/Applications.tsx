import { useState, useRef } from "react";
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query";
import {
  Card,
  CardContent,
  CardHeader,
  CardTitle,
  CardDescription,
} from "@/components/ui/card";
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
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
  DialogFooter,
} from "@/components/ui/dialog";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Progress } from "@/components/ui/progress";
import { ScrollArea } from "@/components/ui/scroll-area";
import {
  FileCheck,
  Plus,
  Search,
  Download,
  Upload,
  CheckCircle,
  Clock,
  FileText,
  Award,
  FolderOpen,
  ArrowLeft,
  Copy,
  Trash2,
  Eye,
  Library,
  Sparkles,
  AlertCircle,
  History,
  Brain,
  type LucideIcon,
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { API_ENDPOINTS } from "@/lib/api";
import { api, files, type BackendAction } from "@/integrations/api/client";
import { MaterialVersionHistory } from "@/components/applications/MaterialVersionHistory";
import { MaterialUploadDialog } from "@/components/applications/MaterialUploadDialog";
import { AIAuditDialog } from "@/components/applications/AIAuditDialog";

interface DisplayBadge {
  label: string;
  variant?: "default" | "secondary" | "destructive" | "outline";
  icon?: string;
  class_name?: string;
  visible?: boolean;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface Application {
  id: string;
  application_no: string;
  title: string;
  type: string;
  category: string;
  deadline: string | null;
  status: string;
  progress: number;
  created_at: string;
  total_materials: number;
  completed_materials: number;
  required_materials: number;
  completed_required_materials: number;
  reusable_materials: number;
  pending_materials: number;
  status_badge: DisplayBadge;
  deadline_label: string;
  deadline_badge: DisplayBadge | null;
  days_left_label: string;
  progress_label: string;
  row_class_name: string;
  category_badge: DisplayBadge;
  can_delete: boolean;
  delete_action: DeleteAction | null;
}

interface Material {
  id: string;
  application_id: string;
  name: string;
  required: boolean;
  status: string;
  description: string | null;
  upload_date: string | null;
  file_size: string | null;
  file_path: string | null;
  can_reuse: boolean;
  reuse_source: string | null;
  status_badge: DisplayBadge;
  required_badge: DisplayBadge;
  upload_date_label: string;
  file_size_label: string;
  uploaded_summary: string;
  description_label: string;
  reusable_badge: DisplayBadge | null;
  is_completed: boolean;
  can_upload: boolean;
}

interface MaterialLibraryItem {
  id: string;
  name: string;
  category: string;
  upload_date: string;
  file_size: string;
  file_path: string;
  used_count: number;
  upload_date_label: string;
  file_size_label: string;
  used_count_label: string;
  metadata_label: string;
  file_type: string;
  preview_supported: boolean;
  can_delete: boolean;
  delete_action: DeleteAction | null;
}

interface ApplicationStats {
  total_applications: number;
  active_applications: number;
  approved_applications: number;
  material_library_items: number;
  urgent_applications: number;
  approaching_applications: number;
  overdue_applications: number;
}

interface TemplateOption {
  type: string;
  name: string;
  category: string;
  items: string[];
  recommended: boolean;
  visual_icon: string;
  visual_color: string;
  card_class_name: string;
  recommended_badge: DisplayBadge | null;
}

interface RecommendationContext {
  has_company_info: boolean;
  registered_capital: number | null;
  years_in_business: number | null;
  recommended_count: number;
  registered_capital_label: string;
  years_in_business_label: string;
  recommended_count_label: string;
}

interface ApplicationsOverview {
  stats: ApplicationStats;
  applications: Application[];
  material_library_items: MaterialLibraryItem[];
  create_from_template_action: BackendAction | null;
  material_library_create_action: BackendAction | null;
  template_catalog: TemplateOption[];
  recommended_templates: TemplateOption[];
  type_options: Array<{ value: string; label: string }>;
  recommendation_context: RecommendationContext;
  permissions: {
    can_delete: boolean;
  };
}

interface MaterialStats {
  total_materials: number;
  completed_materials: number;
  required_materials: number;
  completed_required_materials: number;
  reusable_materials: number;
  pending_materials: number;
  completion_label: string;
  required_completion_label: string;
  completion_progress: number;
}

interface MaterialsOverview {
  stats: MaterialStats;
  materials: Material[];
  required_materials: Material[];
  pending_materials: Material[];
  reuse_action: BackendAction | null;
}

const emptyApplicationStats: ApplicationStats = {
  total_applications: 0,
  active_applications: 0,
  approved_applications: 0,
  material_library_items: 0,
  urgent_applications: 0,
  approaching_applications: 0,
  overdue_applications: 0,
};

const emptyMaterialStats: MaterialStats = {
  total_materials: 0,
  completed_materials: 0,
  required_materials: 0,
  completed_required_materials: 0,
  reusable_materials: 0,
  pending_materials: 0,
  completion_label: "0/0",
  required_completion_label: "0/0",
  completion_progress: 0,
};

const iconMap: Record<string, LucideIcon> = {
  AlertCircle,
  Award,
  CheckCircle,
  Clock,
  FileCheck,
  FileText,
  Sparkles,
  Upload,
};

const DisplayIcon = ({
  name,
  className,
}: {
  name?: string;
  className?: string;
}) => {
  const Icon = iconMap[name || ""] || FileText;
  return <Icon className={className || "h-3 w-3"} />;
};

const BackendBadge = ({ badge }: { badge?: DisplayBadge | null }) => {
  if (!badge) return null;
  return (
    <Badge variant={badge.variant || "outline"} className={`gap-1 ${badge.class_name || ""}`}>
      {badge.icon && <DisplayIcon name={badge.icon} className="h-3 w-3" />}
      {badge.label}
    </Badge>
  );
};

const Applications = () => {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [searchTerm, setSearchTerm] = useState("");
  const [typeFilter, setTypeFilter] = useState("all");
  const [selectedApp, setSelectedApp] = useState<Application | null>(null);
  const [showMaterialLibrary, setShowMaterialLibrary] = useState(false);
  const [showNewAppDialog, setShowNewAppDialog] = useState(false);
  const [showUploadDialog, setShowUploadDialog] = useState(false);
  const [showReuseDialog, setShowReuseDialog] = useState(false);
  const [selectedMaterial, setSelectedMaterial] = useState<Material | null>(
    null,
  );
  const [showPreviewDialog, setShowPreviewDialog] = useState(false);
  const [previewFile, setPreviewFile] = useState<{
    url: string;
    name: string;
    type: string;
  } | null>(null);
  const [showVersionHistory, setShowVersionHistory] = useState(false);
  const [versionHistoryMaterial, setVersionHistoryMaterial] =
    useState<Material | null>(null);
  const [showUploadMaterialDialog, setShowUploadMaterialDialog] =
    useState(false);
  const [uploadMaterial, setUploadMaterial] = useState<Material | null>(null);
  const [showAIAudit, setShowAIAudit] = useState(false);
  const [aiAuditType, setAiAuditType] = useState<"application" | "material">(
    "material",
  );
  const [aiAuditTarget, setAiAuditTarget] = useState<{
    id: string;
    title: string;
  } | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);

  const { data: overview, isLoading: appsLoading } = useQuery({
    queryKey: ["applications-overview", searchTerm, typeFilter],
    queryFn: async () => {
      const params = new URLSearchParams();
      if (searchTerm.trim()) params.set("search", searchTerm.trim());
      if (typeFilter !== "all") params.set("type", typeFilter);
      const query = params.toString();
      return api.get<ApplicationsOverview>(
        `${API_ENDPOINTS.applicationsOverview}${query ? `?${query}` : ""}`,
      );
    },
  });

  const applications = overview?.applications ?? [];
  const appStats = overview?.stats ?? emptyApplicationStats;
  const templates = overview?.template_catalog ?? [];
  const recommendedTemplates = overview?.recommended_templates ?? [];
  const typeOptions = overview?.type_options ?? [];
  const recommendationContext = overview?.recommendation_context;
  const materialLibrary = overview?.material_library_items ?? [];
  const createFromTemplateAction = overview?.create_from_template_action ?? null;
  const materialLibraryCreateAction = overview?.material_library_create_action ?? null;

  const { data: materialsOverview } = useQuery({
    queryKey: ["application-materials-overview", selectedApp?.id],
    queryFn: async () => {
      if (!selectedApp) {
        return {
          stats: emptyMaterialStats,
          materials: [],
          required_materials: [],
          pending_materials: [],
          reuse_action: null,
        };
      }

      return api.get<MaterialsOverview>(
        `${API_ENDPOINTS.applicationMaterialsOverview}?application_id=${encodeURIComponent(selectedApp.id)}`,
      );
    },
    enabled: !!selectedApp,
  });

  const materials = materialsOverview?.materials ?? [];
  const materialStats = materialsOverview?.stats ?? emptyMaterialStats;
  const requiredMaterialItems = materialsOverview?.required_materials ?? [];
  const pendingMaterialItems = materialsOverview?.pending_materials ?? [];
  const reuseAction = materialsOverview?.reuse_action ?? null;

  // 创建申报项目
  const createApplicationMutation = useMutation({
    mutationFn: async (params: {
      templateType: string;
      templateName: string;
      category: string;
      action: BackendAction;
    }) => {
      const action = params.action;
      return api.runAction(action, {
        template_type: params.templateType,
        template_name: params.templateName,
        category: params.category,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["applications-overview"] });
      setShowNewAppDialog(false);
      toast({
        title: "创建成功",
        description: "申报项目已创建",
      });
    },
    onError: (error) => {
      toast({
        title: "创建失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  // 删除申报项目
  const deleteApplicationMutation = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["applications-overview"] });
      if (selectedApp) {
        setSelectedApp(null);
      }
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

  // 上传材料到材料库
  const uploadToLibraryMutation = useMutation({
    mutationFn: async (file: File) => {
      const uploaded = await files.upload(file);

      // 保存到材料库表
      const action = materialLibraryCreateAction;
      if (!action) {
        throw new Error("后端未返回材料库上传动作");
      }
      await api.runAction(action, {
        name: file.name,
        category: "通用材料",
        file_path: uploaded.id,
        file_size_bytes: uploaded.size || file.size,
        used_count: 0,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ["applications-overview"] });
      setShowUploadDialog(false);
      toast({
        title: "上传成功",
        description: "材料已添加到材料库",
      });
    },
    onError: (error) => {
      toast({
        title: "上传失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  // 删除材料库项目
  const deleteLibraryItemMutation = useMutation({
    mutationFn: async (action: DeleteAction) => {
      await api.runAction(action);
      return action;
    },
    onSuccess: (action) => {
      queryClient.invalidateQueries({ queryKey: ["applications-overview"] });
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

  // 从材料库复用材料到申报项目
  const reuseMaterialMutation = useMutation({
    mutationFn: async (params: {
      materialId: string;
      libraryItemId: string;
      action: BackendAction;
    }) => {
      const action = params.action;
      return api.runAction(action, {
        material_id: params.materialId,
        library_item_id: params.libraryItemId,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({
        queryKey: ["application-materials-overview"],
      });
      queryClient.invalidateQueries({ queryKey: ["applications-overview"] });
      setShowReuseDialog(false);
      setSelectedMaterial(null);
      toast({
        title: "复用成功",
        description: "已从材料库复用材料",
      });
    },
    onError: (error) => {
      toast({
        title: "复用失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  // 下载材料库文件
  const downloadLibraryFile = async (item: MaterialLibraryItem) => {
    try {
      const data = await files.download(item.file_path);

      // 创建下载链接
      const url = URL.createObjectURL(data);
      const a = document.createElement("a");
      a.href = url;
      a.download = item.name;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      toast({
        title: "下载成功",
        description: `已下载 ${item.name}`,
      });
    } catch (error: any) {
      toast({
        title: "下载失败",
        description: error.message,
        variant: "destructive",
      });
    }
  };

  // 预览材料库文件
  const previewLibraryFile = async (item: MaterialLibraryItem) => {
    try {
      if (!item.preview_supported) {
        toast({
          title: "不支持预览",
          description: "该文件类型不支持在线预览，请下载后查看",
          variant: "destructive",
        });
        return;
      }

      const data = await files.download(item.file_path);

      const url = URL.createObjectURL(data);
      setPreviewFile({
        url,
        name: item.name,
        type: item.file_type,
      });
      setShowPreviewDialog(true);
    } catch (error: any) {
      toast({
        title: "预览失败",
        description: error.message,
        variant: "destructive",
      });
    }
  };

  // 关闭预览时清理URL
  const closePreview = () => {
    if (previewFile?.url) {
      URL.revokeObjectURL(previewFile.url);
    }
    setPreviewFile(null);
    setShowPreviewDialog(false);
  };

  const handleFileUpload = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (file) {
      if (!materialLibraryCreateAction) {
        toast({
          title: "上传失败",
          description: "后端未返回材料库上传动作",
          variant: "destructive",
        });
        return;
      }
      uploadToLibraryMutation.mutate(file);
    }
  };

  const handleCreateApplication = (
    templateType: string,
    templateName: string,
    category: string,
  ) => {
    const action = createFromTemplateAction;
    if (!action) {
      toast({
        title: "创建失败",
        description: "后端未返回申报创建动作",
        variant: "destructive",
      });
      return;
    }
    createApplicationMutation.mutate({ templateType, templateName, category, action });
  };

  const handleDeleteApplication = (action: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteApplicationMutation.mutate(action);
  };

  const handleDeleteLibraryItem = (action: DeleteAction | null) => {
    if (!action || !confirm(action.confirm_message)) return;
    deleteLibraryItemMutation.mutate(action);
  };

  // 列表视图
  if (!selectedApp) {
    return (
      <div className="space-y-6">
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold flex items-center gap-2">
              <FileCheck className="h-8 w-8" />
              项目申报材料准备
            </h1>
            <p className="text-muted-foreground mt-2">
              智能管理申报材料，快速复用已有资源
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              onClick={() => setShowMaterialLibrary(true)}
            >
              <Library className="mr-2 h-4 w-4" />
              材料库
            </Button>
            <Button
              onClick={() => setShowNewAppDialog(true)}
              disabled={!createFromTemplateAction}
            >
              <Plus className="mr-2 h-4 w-4" />
              新建申报
            </Button>
          </div>
        </div>

        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">申报项目</CardTitle>
              <FileCheck className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {appStats.total_applications}
              </div>
              <p className="text-xs text-muted-foreground">总项目数</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">准备中</CardTitle>
              <Clock className="h-4 w-4 text-orange-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-orange-500">
                {appStats.active_applications}
              </div>
              <p className="text-xs text-muted-foreground">正在准备</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">已通过</CardTitle>
              <Award className="h-4 w-4 text-primary" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-primary">
                {appStats.approved_applications}
              </div>
              <p className="text-xs text-muted-foreground">成功获批</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">材料库</CardTitle>
              <Library className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {appStats.material_library_items}
              </div>
              <p className="text-xs text-muted-foreground">可复用材料</p>
            </CardContent>
          </Card>
        </div>

        <Card>
          <CardHeader>
            <div className="flex items-center justify-between">
              <div>
                <CardTitle>申报项目列表</CardTitle>
                <CardDescription>点击项目进入材料准备页面</CardDescription>
              </div>
              <div className="flex gap-4">
                <div className="relative w-64">
                  <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                  <Input
                    placeholder="搜索项目..."
                    value={searchTerm}
                    onChange={(e) => setSearchTerm(e.target.value)}
                    className="pl-9"
                  />
                </div>
                <Select value={typeFilter} onValueChange={setTypeFilter}>
                  <SelectTrigger className="w-[150px]">
                    <SelectValue placeholder="筛选类型" />
                  </SelectTrigger>
                  <SelectContent>
                    {typeOptions.map((option) => (
                      <SelectItem key={option.value} value={option.value}>
                        {option.label}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            {appsLoading ? (
              <div className="text-center py-8 text-muted-foreground">
                加载中...
              </div>
            ) : applications.length === 0 ? (
              <div className="text-center py-8 text-muted-foreground">
                暂无申报项目，点击"新建申报"开始
              </div>
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>编号</TableHead>
                    <TableHead>项目名称</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>级别</TableHead>
                    <TableHead>截止日期</TableHead>
                    <TableHead>进度</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead>操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {applications.map((app) => (
                      <TableRow
                        key={app.id}
                        className={`cursor-pointer hover:bg-muted/50 ${app.row_class_name || ""}`}
                        onClick={() => setSelectedApp(app)}
                      >
                        <TableCell className="font-medium">
                          {app.application_no}
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <span className="font-medium">{app.title}</span>
                            <BackendBadge badge={app.deadline_badge} />
                          </div>
                        </TableCell>
                        <TableCell>{app.type}</TableCell>
                        <TableCell>
                          <BackendBadge badge={app.category_badge} />
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <span>{app.deadline_label}</span>
                            {app.deadline_badge && (
                              <span className="text-xs text-muted-foreground">
                                ({app.days_left_label})
                              </span>
                            )}
                          </div>
                        </TableCell>
                        <TableCell>
                          <div className="flex items-center gap-2">
                            <Progress value={app.progress} className="w-20" />
                            <span className="text-sm text-muted-foreground">
                              {app.progress_label}
                            </span>
                          </div>
                        </TableCell>
                        <TableCell>
                          <BackendBadge badge={app.status_badge} />
                        </TableCell>
                        <TableCell>
                          <div className="flex gap-1">
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                setSelectedApp(app);
                              }}
                            >
                              <FolderOpen className="h-4 w-4" />
                            </Button>
                            {app.delete_action && (
                              <Button
                                variant="ghost"
                                size="sm"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleDeleteApplication(app.delete_action);
                                }}
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
            )}
          </CardContent>
        </Card>

        {/* 新建申报对话框 */}
        <Dialog open={showNewAppDialog} onOpenChange={setShowNewAppDialog}>
          <DialogContent className="max-w-5xl max-h-[90vh]">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Sparkles className="h-5 w-5" />
                选择申报模板
              </DialogTitle>
              <DialogDescription>
                根据企业条件智能推荐，或浏览全部项目
              </DialogDescription>
            </DialogHeader>
            <Tabs defaultValue="recommended" className="mt-4">
              <TabsList className="grid w-full grid-cols-2">
                <TabsTrigger
                  value="recommended"
                  className="flex items-center gap-2"
                >
                  <Sparkles className="h-4 w-4" />
                  智能推荐
                </TabsTrigger>
                <TabsTrigger value="all">全部项目</TabsTrigger>
              </TabsList>

              <TabsContent value="recommended">
                <ScrollArea className="h-[500px] pr-4">
                  {!recommendationContext?.has_company_info ? (
                    <div className="flex flex-col items-center justify-center py-12 text-center">
                      <Award className="h-16 w-16 text-muted-foreground mb-4" />
                      <h3 className="text-lg font-medium mb-2">
                        完善企业信息以获取个性化推荐
                      </h3>
                      <p className="text-sm text-muted-foreground mb-4">
                        系统将根据您的注册资本、成立年限等条件推荐适合的申报项目
                      </p>
                      <Button
                        onClick={() => (window.location.href = "/company-info")}
                      >
                        前往企业信息
                      </Button>
                    </div>
                  ) : recommendedTemplates.length === 0 ? (
                    <div className="text-center py-12 text-muted-foreground">
                      <p>暂无推荐项目</p>
                    </div>
                  ) : (
                    <div className="space-y-4">
                      <div className="bg-muted/50 rounded-lg p-4 mb-4">
                        <h4 className="font-medium mb-2 flex items-center gap-2">
                          <Sparkles className="h-4 w-4 text-primary" />
                          根据您的企业条件推荐
                        </h4>
                        <div className="text-sm text-muted-foreground space-y-1">
                          <p>
                            • 注册资本:{" "}
                            {recommendationContext.registered_capital_label}
                          </p>
                          <p>
                            • 成立年限:{" "}
                            {recommendationContext.years_in_business_label}
                          </p>
                          <p>• {recommendationContext.recommended_count_label}</p>
                        </div>
                      </div>
                      <div className="grid grid-cols-2 gap-4">
                        {recommendedTemplates.map((template) => (
                            <Card
                              key={template.type}
                              className={template.card_class_name}
                              onClick={() =>
                                handleCreateApplication(
                                  template.type,
                                  template.name,
                                  template.category,
                                )
                              }
                            >
                              <div className="absolute top-3 right-3">
                                <BackendBadge badge={template.recommended_badge} />
                              </div>
                              <CardHeader>
                                <CardTitle className="flex items-center gap-2 text-base pr-16">
                                  <DisplayIcon
                                    name={template.visual_icon}
                                    className={`h-5 w-5 ${template.visual_color}`}
                                  />
                                  {template.name}
                                </CardTitle>
                                <CardDescription>
                                  {template.category}
                                </CardDescription>
                              </CardHeader>
                              <CardContent className="space-y-3">
                                <div className="text-sm text-muted-foreground">
                                  {template.items.map((item, idx) => (
                                    <p key={idx}>✓ {item}</p>
                                  ))}
                                </div>
                                <Button
                                  className="w-full"
                                  disabled={
                                    createApplicationMutation.isPending ||
                                    !createFromTemplateAction
                                  }
                                >
                                  {createApplicationMutation.isPending
                                    ? "创建中..."
                                    : "使用此模板"}
                                </Button>
                              </CardContent>
                            </Card>
                        ))}
                      </div>
                    </div>
                  )}
                </ScrollArea>
              </TabsContent>

              <TabsContent value="all">
                <ScrollArea className="h-[500px] pr-4">
                  <div className="grid grid-cols-2 gap-4">
                    {templates.map((template) => (
                        <Card
                          key={template.type}
                          className={template.card_class_name}
                          onClick={() =>
                            handleCreateApplication(
                              template.type,
                              template.name,
                              template.category,
                            )
                          }
                        >
                          <div className="absolute top-3 right-3">
                            <BackendBadge badge={template.recommended_badge} />
                          </div>
                          <CardHeader>
                            <CardTitle className="flex items-center gap-2 text-base pr-16">
                              <DisplayIcon
                                name={template.visual_icon}
                                className={`h-5 w-5 ${template.visual_color}`}
                              />
                              {template.name}
                            </CardTitle>
                            <CardDescription>
                              {template.category}
                            </CardDescription>
                          </CardHeader>
                          <CardContent className="space-y-3">
                            <div className="text-sm text-muted-foreground">
                              {template.items.map((item, idx) => (
                                <p key={idx}>✓ {item}</p>
                              ))}
                            </div>
                            <Button
                              className="w-full"
                              disabled={
                                createApplicationMutation.isPending ||
                                !createFromTemplateAction
                              }
                            >
                              {createApplicationMutation.isPending
                                ? "创建中..."
                                : "使用此模板"}
                            </Button>
                          </CardContent>
                        </Card>
                    ))}
                  </div>
                </ScrollArea>
              </TabsContent>
            </Tabs>
          </DialogContent>
        </Dialog>

        {/* 材料库对话框 */}
        <Dialog
          open={showMaterialLibrary}
          onOpenChange={setShowMaterialLibrary}
        >
          <DialogContent className="max-w-4xl max-h-[80vh]">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Library className="h-5 w-5" />
                材料库
              </DialogTitle>
              <DialogDescription>管理所有可复用的申报材料</DialogDescription>
            </DialogHeader>
            <div className="flex justify-end mb-4">
              <Button
                onClick={() => setShowUploadDialog(true)}
                disabled={!materialLibraryCreateAction}
              >
                <Upload className="mr-2 h-4 w-4" />
                上传材料
              </Button>
            </div>
            <ScrollArea className="h-[500px] pr-4">
              {materialLibrary.length === 0 ? (
                <div className="text-center py-12">
                  <Library className="h-16 w-16 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground mb-4">
                    材料库为空，上传常用材料以便快速复用
                  </p>
                  <Button
                    onClick={() => setShowUploadDialog(true)}
                    disabled={!materialLibraryCreateAction}
                  >
                    <Upload className="mr-2 h-4 w-4" />
                    上传第一个材料
                  </Button>
                </div>
              ) : (
                <div className="space-y-2">
                  {materialLibrary.map((item) => (
                    <Card key={item.id}>
                      <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-3">
                            <FileText className="h-8 w-8 text-primary" />
                            <div>
                              <div className="font-medium">{item.name}</div>
                              <div className="text-sm text-muted-foreground">
                                {item.category} · {item.upload_date_label} · {item.file_size_label}
                              </div>
                            </div>
                          </div>
                          <div className="flex items-center gap-2">
                            <Badge variant="secondary">
                              {item.used_count_label}
                            </Badge>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => previewLibraryFile(item)}
                              title="预览"
                            >
                              <Eye className="h-4 w-4" />
                            </Button>
                            <Button
                              variant="ghost"
                              size="icon"
                              onClick={() => downloadLibraryFile(item)}
                              title="下载"
                            >
                              <Download className="h-4 w-4" />
                            </Button>
                            {item.delete_action && (
                              <Button
                                variant="ghost"
                                size="icon"
                                onClick={() =>
                                  handleDeleteLibraryItem(item.delete_action)
                                }
                                title="删除"
                              >
                                <Trash2 className="h-4 w-4 text-destructive" />
                              </Button>
                            )}
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </ScrollArea>
          </DialogContent>
        </Dialog>

        {/* 上传材料对话框 */}
        <Dialog open={showUploadDialog} onOpenChange={setShowUploadDialog}>
          <DialogContent>
            <DialogHeader>
              <DialogTitle>上传材料到材料库</DialogTitle>
              <DialogDescription>
                上传常用材料，以便在多个申报项目中快速复用
              </DialogDescription>
            </DialogHeader>
            <div className="space-y-4 py-4">
              <div className="border-2 border-dashed rounded-lg p-8 text-center">
                <input
                  ref={fileInputRef}
                  type="file"
                  onChange={handleFileUpload}
                  className="hidden"
                  accept=".pdf,.doc,.docx,.xls,.xlsx,.jpg,.jpeg,.png"
                />
                <Upload className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                <p className="text-sm text-muted-foreground mb-4">
                  支持 PDF、Word、Excel、图片等格式
                </p>
                <Button
                  onClick={() => fileInputRef.current?.click()}
                  disabled={
                    uploadToLibraryMutation.isPending ||
                    !materialLibraryCreateAction
                  }
                >
                  {uploadToLibraryMutation.isPending ? "上传中..." : "选择文件"}
                </Button>
              </div>
            </div>
          </DialogContent>
        </Dialog>

        {/* 从材料库复用对话框 */}
        <Dialog open={showReuseDialog} onOpenChange={setShowReuseDialog}>
          <DialogContent className="max-w-3xl max-h-[80vh]">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Copy className="h-5 w-5" />
                从材料库复用
              </DialogTitle>
              <DialogDescription>
                选择材料库中的文件复用到当前申报材料
              </DialogDescription>
            </DialogHeader>
            <ScrollArea className="h-[400px] pr-4">
              {materialLibrary.length === 0 ? (
                <div className="text-center py-8">
                  <Library className="h-12 w-12 text-muted-foreground mx-auto mb-4" />
                  <p className="text-muted-foreground">
                    材料库为空，请先上传材料
                  </p>
                </div>
              ) : (
                <div className="space-y-2">
                  {materialLibrary.map((item) => (
                    <Card
                      key={item.id}
                      className="cursor-pointer hover:border-primary transition-colors"
                      onClick={() => {
                        if (selectedMaterial && reuseAction) {
                          reuseMaterialMutation.mutate({
                            materialId: selectedMaterial.id,
                            libraryItemId: item.id,
                            action: reuseAction,
                          });
                        }
                      }}
                    >
                      <CardContent className="p-4">
                        <div className="flex items-center justify-between">
                          <div className="flex items-center gap-3">
                            <FileText className="h-8 w-8 text-primary" />
                            <div>
                              <div className="font-medium">{item.name}</div>
                              <div className="text-sm text-muted-foreground">
                                {item.category} · {item.upload_date_label} · {item.file_size_label}
                              </div>
                            </div>
                          </div>
                          <Badge variant="secondary">
                            {item.used_count_label}
                          </Badge>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                </div>
              )}
            </ScrollArea>
          </DialogContent>
        </Dialog>
      </div>
    );
  }

  // 详细材料准备视图
  const totalMaterials = materialStats.total_materials;
  const requiredMaterials = materialStats.required_materials;
  const reusableMaterials = materialStats.reusable_materials;
  const pendingMaterials = materialStats.pending_materials;

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-4">
          <Button
            variant="ghost"
            size="icon"
            onClick={() => setSelectedApp(null)}
          >
            <ArrowLeft className="h-5 w-5" />
          </Button>
          <div>
            <h1 className="text-2xl font-bold">{selectedApp.title}</h1>
            <p className="text-muted-foreground">
              {selectedApp.application_no} · {selectedApp.category_badge?.label || selectedApp.category} · 截止日期:{" "}
              {selectedApp.deadline_label}
            </p>
          </div>
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            onClick={() => {
              setAiAuditType("application");
              setAiAuditTarget({
                id: selectedApp.id,
                title: selectedApp.title,
              });
              setShowAIAudit(true);
            }}
          >
            <Brain className="mr-2 h-4 w-4" />
            AI 综合评估
          </Button>
          <Button
            variant="outline"
            onClick={() => setShowMaterialLibrary(true)}
          >
            <Library className="mr-2 h-4 w-4" />
            材料库
          </Button>
          <Button>
            <Download className="mr-2 h-4 w-4" />
            导出材料包
          </Button>
        </div>
      </div>

      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">材料完成度</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {materialStats.completion_label}
            </div>
            <Progress
              value={materialStats.completion_progress}
              className="mt-2"
            />
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">必需材料</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {materialStats.required_completion_label}
            </div>
            <p className="text-xs text-muted-foreground mt-1">已完成必需材料</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">可复用材料</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {reusableMaterials}
            </div>
            <p className="text-xs text-muted-foreground mt-1">可从材料库添加</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-3">
            <CardTitle className="text-sm font-medium">项目状态</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="mt-1">
              <BackendBadge badge={selectedApp.status_badge} />
            </div>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>材料清单</CardTitle>
          <CardDescription>根据申报要求准备以下材料</CardDescription>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="all" className="space-y-4">
            <TabsList>
              <TabsTrigger value="all">全部材料 ({totalMaterials})</TabsTrigger>
              <TabsTrigger value="required">
                必需材料 ({requiredMaterials})
              </TabsTrigger>
              <TabsTrigger value="pending">
                待准备 ({pendingMaterials})
              </TabsTrigger>
            </TabsList>

            <TabsContent value="all" className="space-y-3">
              {materials.length === 0 ? (
                <div className="text-center py-8 text-muted-foreground">
                  暂无材料清单
                </div>
              ) : (
                materials.map((material) => (
                  <Card key={material.id}>
                    <CardContent className="p-4">
                      <div className="flex items-start justify-between">
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <h4 className="font-medium">{material.name}</h4>
                            <BackendBadge badge={material.required_badge} />
                            <BackendBadge badge={material.status_badge} />
                            <BackendBadge badge={material.reusable_badge} />
                          </div>
                          <p className="text-sm text-muted-foreground mb-2">
                            {material.description_label}
                          </p>
                          {material.uploaded_summary !== "-" && (
                            <div className="text-xs text-muted-foreground">
                              {material.uploaded_summary}
                            </div>
                          )}
                        </div>
                        <div className="flex gap-2">
                          {material.is_completed ? (
                            <>
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => {
                                  setAiAuditType("material");
                                  setAiAuditTarget({
                                    id: material.id,
                                    title: material.name,
                                  });
                                  setShowAIAudit(true);
                                }}
                              >
                                <Brain className="h-4 w-4 mr-1" />
                                AI 审核
                              </Button>
                              <Button
                                variant="outline"
                                size="sm"
                                onClick={() => {
                                  setVersionHistoryMaterial(material);
                                  setShowVersionHistory(true);
                                }}
                              >
                                <History className="h-4 w-4 mr-1" />
                                版本历史
                              </Button>
                              <Button variant="outline" size="sm">
                                <Eye className="h-4 w-4 mr-1" />
                                查看
                              </Button>
                              <Button variant="outline" size="sm">
                                <Download className="h-4 w-4 mr-1" />
                                下载
                              </Button>
                              <Button variant="ghost" size="sm">
                                <Trash2 className="h-4 w-4 text-destructive" />
                              </Button>
                            </>
                          ) : (
                            <>
                              <Button
                                variant="outline"
                                size="sm"
                                disabled={!reuseAction}
                                onClick={() => {
                                  setSelectedMaterial(material);
                                  setShowReuseDialog(true);
                                }}
                              >
                                <Copy className="h-4 w-4 mr-1" />
                                从材料库复用
                              </Button>
                              {material.can_upload && (
                                <Button
                                  size="sm"
                                  onClick={() => {
                                    setUploadMaterial(material);
                                    setShowUploadMaterialDialog(true);
                                  }}
                                >
                                  <Upload className="h-4 w-4 mr-1" />
                                  上传文件
                                </Button>
                              )}
                            </>
                          )}
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))
              )}
            </TabsContent>

            <TabsContent value="required">
              <div className="space-y-3">
                {requiredMaterialItems.length === 0 ? (
                  <div className="text-center py-8 text-muted-foreground">
                    无必需材料
                  </div>
                ) : (
                  requiredMaterialItems.map((material) => (
                      <Card key={material.id}>
                        <CardContent className="p-4">
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-2">
                                <h4 className="font-medium">{material.name}</h4>
                                <BackendBadge badge={material.status_badge} />
                              </div>
                              <p className="text-sm text-muted-foreground">
                                {material.description_label}
                              </p>
                            </div>
                            <div className="flex gap-2">
                              {material.can_upload && (
                                <Button
                                  size="sm"
                                  onClick={() => {
                                    setUploadMaterial(material);
                                    setShowUploadMaterialDialog(true);
                                  }}
                                >
                                  <Upload className="h-4 w-4 mr-1" />
                                  上传
                                </Button>
                              )}
                            </div>
                          </div>
                        </CardContent>
                      </Card>
                    ))
                )}
              </div>
            </TabsContent>

            <TabsContent value="pending">
              <div className="space-y-3">
                {pendingMaterialItems.length === 0 ? (
                  <div className="text-center py-8 text-muted-foreground">
                    无待准备材料
                  </div>
                ) : (
                  pendingMaterialItems.map((material) => (
                      <Card key={material.id}>
                        <CardContent className="p-4">
                          <div className="flex items-start justify-between">
                            <div className="flex-1">
                              <div className="flex items-center gap-2 mb-2">
                                <h4 className="font-medium">{material.name}</h4>
                                <BackendBadge badge={material.required_badge} />
                              </div>
                              <p className="text-sm text-muted-foreground">
                                {material.description_label}
                              </p>
                            </div>
                            {material.can_upload && (
                              <Button
                                size="sm"
                                onClick={() => {
                                  setUploadMaterial(material);
                                  setShowUploadMaterialDialog(true);
                                }}
                              >
                                <Upload className="h-4 w-4 mr-1" />
                                上传文件
                              </Button>
                            )}
                          </div>
                        </CardContent>
                      </Card>
                    ))
                )}
              </div>
            </TabsContent>
          </Tabs>
        </CardContent>
      </Card>

      {/* 文件预览对话框 */}
      <Dialog
        open={showPreviewDialog}
        onOpenChange={(open) => !open && closePreview()}
      >
        <DialogContent className="max-w-5xl max-h-[90vh]">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Eye className="h-5 w-5" />
              文件预览
            </DialogTitle>
            <DialogDescription>{previewFile?.name}</DialogDescription>
          </DialogHeader>
          <ScrollArea className="h-[70vh] w-full">
            {previewFile && (
              <div className="flex items-center justify-center p-4">
                {previewFile.type === "pdf" ? (
                  <iframe
                    src={previewFile.url}
                    className="w-full h-[70vh] border rounded"
                    title={previewFile.name}
                  />
                ) : ["jpg", "jpeg", "png", "gif", "bmp", "webp"].includes(
                    previewFile.type,
                  ) ? (
                  <img
                    src={previewFile.url}
                    alt={previewFile.name}
                    className="max-w-full h-auto rounded shadow-lg"
                  />
                ) : (
                  <div className="text-center py-12">
                    <FileText className="h-16 w-16 text-muted-foreground mx-auto mb-4" />
                    <p className="text-muted-foreground">
                      不支持预览此文件类型
                    </p>
                  </div>
                )}
              </div>
            )}
          </ScrollArea>
          <DialogFooter>
            <Button variant="outline" onClick={closePreview}>
              关闭
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>

      {/* 版本历史对话框 */}
      {versionHistoryMaterial && (
        <MaterialVersionHistory
          materialId={versionHistoryMaterial.id}
          materialName={versionHistoryMaterial.name}
          open={showVersionHistory}
          onOpenChange={setShowVersionHistory}
        />
      )}

      {/* 材料上传对话框 */}
      {uploadMaterial && (
        <MaterialUploadDialog
          materialId={uploadMaterial.id}
          materialName={uploadMaterial.name}
          open={showUploadMaterialDialog}
          onOpenChange={setShowUploadMaterialDialog}
        />
      )}

      {/* AI 审核对话框 */}
      {aiAuditTarget && (
        <AIAuditDialog
          open={showAIAudit}
          onOpenChange={setShowAIAudit}
          auditType={aiAuditType}
          targetId={aiAuditTarget.id}
          title={aiAuditTarget.title}
        />
      )}
    </div>
  );
};

export default Applications;
