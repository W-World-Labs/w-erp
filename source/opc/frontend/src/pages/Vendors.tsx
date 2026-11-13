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
  Building2,
  CheckCircle,
  Edit,
  Phone,
  Plus,
  Search,
  Trash2,
  Users,
  XCircle,
  type LucideIcon,
} from "lucide-react";
import { useToast } from "@/hooks/use-toast";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { api, type BackendAction, type DeleteAction } from "@/integrations/api/client";
import { VendorDialog } from "@/components/vendors/VendorDialog";
import { DependencyAlert } from "@/components/common/DependencyAlert";

const VENDORS_OVERVIEW_ENDPOINT = "/api/v1/vendors/overview";

interface VendorBadge {
  value: string;
  label: string;
  variant: "default" | "secondary" | "outline" | "destructive";
  icon?: string;
}

interface VendorOverviewItem {
  id: string;
  name?: string;
  short_name?: string;
  unified_code?: string;
  contact_person?: string;
  contact_phone?: string;
  vendor_type?: string;
  status?: string;
  type_badge: VendorBadge;
  status_badge: VendorBadge;
  can_delete?: boolean;
  update_action: BackendAction | null;
  delete_action: DeleteAction | null;
}

interface VendorsOverview {
  stats: {
    total: number;
    suppliers: number;
    customers: number;
    active: number;
  };
  vendors: VendorOverviewItem[];
  type_options: VendorBadge[];
  status_options: VendorBadge[];
  create_action: BackendAction | null;
  empty_message: string;
}

const emptyVendorsOverview: VendorsOverview = {
  stats: {
    total: 0,
    suppliers: 0,
    customers: 0,
    active: 0,
  },
  vendors: [],
  type_options: [],
  status_options: [],
  create_action: null,
  empty_message: "暂无友商资料，点击「新增友商」添加",
};

const vendorBadgeIcons: Record<string, LucideIcon> = {
  CheckCircle,
  XCircle,
};

function VendorBadgeView({ badge }: { badge: VendorBadge }) {
  const Icon = badge.icon ? vendorBadgeIcons[badge.icon] : null;

  return (
    <Badge variant={badge.variant} className={Icon ? "gap-1" : undefined}>
      {Icon && <Icon className="h-3 w-3" />}
      {badge.label}
    </Badge>
  );
}

function appendOverviewParam(
  params: URLSearchParams,
  key: string,
  value: string,
) {
  if (value && value !== "all") {
    params.set(key, value);
  }
}

const Vendors = () => {
  const { toast } = useToast();
  const queryClient = useQueryClient();
  const [searchTerm, setSearchTerm] = useState("");
  const [typeFilter, setTypeFilter] = useState("all");
  const [statusFilter, setStatusFilter] = useState("all");
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedVendor, setSelectedVendor] = useState<any>(null);
  const [vendorToDelete, setVendorToDelete] = useState<DeleteAction | null>(
    null,
  );

  const { data: overview = emptyVendorsOverview, isLoading } = useQuery({
    queryKey: ["vendors-overview", searchTerm, typeFilter, statusFilter],
    queryFn: async () => {
      const params = new URLSearchParams();
      const search = searchTerm.trim();
      if (search) {
        params.set("search", search);
      }
      appendOverviewParam(params, "vendor_type", typeFilter);
      appendOverviewParam(params, "status", statusFilter);
      const query = params.toString();
      return await api.get<VendorsOverview>(
        `${VENDORS_OVERVIEW_ENDPOINT}${query ? `?${query}` : ""}`,
      );
    },
  });
  const vendors = overview.vendors;
  const vendorStats = overview.stats;

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
      queryClient.invalidateQueries({ queryKey: ["vendors-overview"] });
      toast({
        title: variables.action.success_title || "保存成功",
        description: variables.action.success_message || "友商资料已保存",
      });
      setDialogOpen(false);
      setSelectedVendor(null);
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
      queryClient.invalidateQueries({ queryKey: ["vendors-overview"] });
      toast({ title: action.success_title, description: action.success_message });
      setVendorToDelete(null);
    },
    onError: (error) => {
      toast({
        title: "删除失败",
        description: error.message,
        variant: "destructive",
      });
    },
  });

  const handleEdit = (vendor: any) => {
    setSelectedVendor(vendor);
    setDialogOpen(true);
  };

  const handleSubmit = (values: any) => {
    const action = selectedVendor
      ? selectedVendor.update_action
      : overview.create_action;
    if (!action) {
      toast({
        title: "无可用操作",
        description: "当前账号没有保存友商资料的权限",
        variant: "destructive",
      });
      return;
    }
    saveMutation.mutate({ action, values });
  };

  return (
    <div className="container mx-auto space-y-6 p-6">
      <DependencyAlert dependency="company-info" />

      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-foreground">友商管理</h1>
          <p className="text-muted-foreground">供应商与客户资料管理</p>
        </div>
        <Button
          className="gap-2"
          disabled={!overview.create_action}
          onClick={() => {
            setSelectedVendor(null);
            setDialogOpen(true);
          }}
        >
          <Plus className="h-4 w-4" />
          新增友商
        </Button>
      </div>

      <div className="grid grid-cols-1 gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">友商总数</CardTitle>
            <Building2 className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{vendorStats.total}</div>
            <p className="text-xs text-muted-foreground">已登记友商</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">供应商</CardTitle>
            <Users className="h-4 w-4 text-primary" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-primary">
              {vendorStats.suppliers}
            </div>
            <p className="text-xs text-muted-foreground">上游供应商</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">客户</CardTitle>
            <Users className="h-4 w-4 text-blue-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-blue-500">
              {vendorStats.customers}
            </div>
            <p className="text-xs text-muted-foreground">下游客户</p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">合作中</CardTitle>
            <CheckCircle className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-500">
              {vendorStats.active}
            </div>
            <p className="text-xs text-muted-foreground">正在合作的友商</p>
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>友商列表</CardTitle>
          <div className="mt-4 flex gap-4">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 transform text-muted-foreground" />
              <Input
                placeholder="搜索友商名称、简称、联系人、信用代码..."
                className="pl-10"
                value={searchTerm}
                onChange={(event) => setSearchTerm(event.target.value)}
              />
            </div>
            <Select value={typeFilter} onValueChange={setTypeFilter}>
              <SelectTrigger className="w-[150px]">
                <SelectValue placeholder="友商类型" />
              </SelectTrigger>
              <SelectContent>
                {overview.type_options.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-[150px]">
                <SelectValue placeholder="合作状态" />
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
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="py-8 text-center text-muted-foreground">
              加载中...
            </div>
          ) : vendors.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              {overview.empty_message}
            </div>
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>友商名称</TableHead>
                  <TableHead>简称</TableHead>
                  <TableHead>类型</TableHead>
                  <TableHead>联系人</TableHead>
                  <TableHead>联系电话</TableHead>
                  <TableHead>状态</TableHead>
                  <TableHead className="text-right">操作</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {vendors.map((vendor) => (
                  <TableRow key={vendor.id}>
                    <TableCell className="font-medium">
                      <div>
                        <div>{vendor.name}</div>
                        {vendor.unified_code && (
                          <div className="text-xs text-muted-foreground">
                            {vendor.unified_code}
                          </div>
                        )}
                      </div>
                    </TableCell>
                    <TableCell>{vendor.short_name || "-"}</TableCell>
                    <TableCell>
                      <VendorBadgeView badge={vendor.type_badge} />
                    </TableCell>
                    <TableCell>
                      {vendor.contact_person ? (
                        <div className="flex items-center gap-1">
                          <Users className="h-3 w-3 text-muted-foreground" />
                          {vendor.contact_person}
                        </div>
                      ) : (
                        "-"
                      )}
                    </TableCell>
                    <TableCell>
                      {vendor.contact_phone ? (
                        <div className="flex items-center gap-1">
                          <Phone className="h-3 w-3 text-muted-foreground" />
                          {vendor.contact_phone}
                        </div>
                      ) : (
                        "-"
                      )}
                    </TableCell>
                    <TableCell>
                      <VendorBadgeView badge={vendor.status_badge} />
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-2">
                        <Button
                          variant="ghost"
                          size="icon"
                          disabled={!vendor.update_action}
                          onClick={() => handleEdit(vendor)}
                        >
                          <Edit className="h-4 w-4" />
                        </Button>
                        {vendor.delete_action && (
                          <Button
                            variant="ghost"
                            size="icon"
                            onClick={() => setVendorToDelete(vendor.delete_action)}
                            className="text-destructive hover:text-destructive"
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

      <VendorDialog
        open={dialogOpen}
        onOpenChange={(open) => {
          setDialogOpen(open);
          if (!open) setSelectedVendor(null);
        }}
        vendor={selectedVendor}
        onSubmit={handleSubmit}
      />

      <AlertDialog
        open={!!vendorToDelete}
        onOpenChange={() => setVendorToDelete(null)}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {vendorToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction
              onClick={() => {
                const action = vendorToDelete;
                if (action) deleteMutation.mutate(action);
              }}
              className="bg-destructive text-destructive-foreground hover:bg-destructive/90"
            >
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
  );
};

export default Vendors;
