import { useCallback, useEffect, useRef, useState } from "react";
import { api, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import {
  Camera,
  Archive,
  CheckCircle2,
  Edit,
  Filter,
  FolderOpen,
  Globe,
  Loader2,
  Package,
  Plus,
  QrCode,
  Search,
  Tags,
  Trash2,
  Wrench,
} from "lucide-react";
import { Html5Qrcode } from "html5-qrcode";
import { AssetCodeDialog } from "@/components/assets/AssetCodeDialog";

import {
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Badge } from "@/components/ui/badge";
import { DependencyAlert } from "@/components/common/DependencyAlert";
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
import { AssetDialog } from "@/components/assets/AssetDialog";
import { CategoryDialog } from "@/components/assets/CategoryDialog";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";

type BadgeVariant = "default" | "secondary" | "outline" | "destructive";

interface AssetBadge {
  value: string;
  label: string;
  variant: BadgeVariant;
  class_name?: string;
  visible?: boolean;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title?: string;
  success_message: string;
}

interface Asset {
  id: string;
  name: string;
  category_id: string | null;
  serial_number: string | null;
  purchase_date: string | null;
  purchase_price: number | null;
  current_value: number | null;
  current_value_label: string;
  status: string;
  status_badge: AssetBadge;
  location: string | null;
  description: string | null;
  created_at: string | null;
  created_by?: string | null;
  asset_type?: string;
  asset_type_badge: AssetBadge;
  expiry_date?: string | null;
  expiry_badge: AssetBadge;
  registration_number?: string | null;
  holder_name?: string | null;
  useful_life?: number | null;
  salvage_rate?: number | null;
  depreciation_method?: string | null;
  monthly_depreciation?: number | null;
  monthly_depreciation_label: string;
  accumulated_depreciation?: number | null;
  depreciation_visible: boolean;
  net_value?: number;
  net_value_label: string;
  can_delete: boolean;
  delete_action: DeleteAction | null;
  can_mark_maintenance: boolean;
  can_mark_scrapped: boolean;
  can_mark_in_use: boolean;
  mark_maintenance_action: BackendAction | null;
  mark_scrapped_action: BackendAction | null;
  mark_in_use_action: BackendAction | null;
  update_action: BackendAction | null;
  asset_categories: {
    name: string;
  } | null;
}

interface Category {
  id: string;
  value: string;
  name: string;
  label: string;
  description: string | null;
  category_type: string;
  category_type_label: string;
  update_action: BackendAction | null;
  delete_action: DeleteAction | null;
}

interface AssetOverview {
  total: number;
  in_use: number;
  total_value: number;
  total_depreciation: number;
  total_value_label: string;
  total_depreciation_label: string;
  expiring_soon: number;
  expired: number;
  type_counts: {
    physical: number;
    virtual: number;
    data: number;
    intangible: number;
  };
  assets: Asset[];
  categories: Category[];
  status_options: Array<{ value: string; label: string }>;
  asset_type_options: Array<{ value: string; label: string }>;
  category_options: Array<{ value: string; label: string }>;
  permissions: {
    can_manage_categories: boolean;
  };
  empty_message: string;
  create_action: BackendAction | null;
  category_create_action: BackendAction | null;
}

const ASSETS_OVERVIEW_ENDPOINT = "/api/assets/overview";

const emptyAssetOverview: AssetOverview = {
  total: 0,
  in_use: 0,
  total_value: 0,
  total_depreciation: 0,
  total_value_label: "¥0",
  total_depreciation_label: "¥0",
  expiring_soon: 0,
  expired: 0,
  type_counts: {
    physical: 0,
    virtual: 0,
    data: 0,
    intangible: 0,
  },
  assets: [],
  categories: [],
  status_options: [],
  asset_type_options: [],
  category_options: [],
  permissions: {
    can_manage_categories: false,
  },
  empty_message: "暂无资产数据，点击上方按钮添加",
  create_action: null,
  category_create_action: null,
};

function AssetBadgeView({ badge }: { badge: AssetBadge }) {
  if (badge.visible === false) {
    return null;
  }

  return (
    <Badge variant={badge.variant} className={badge.class_name}>
      {badge.label}
    </Badge>
  );
}

function overviewQuery(
  searchTerm: string,
  statusFilter: string,
  categoryFilter: string,
  assetTypeFilter: string,
) {
  const query: string[] = [];
  const search = searchTerm.trim();
  if (search) {
    query.push(`search=${encodeURIComponent(search)}`);
  }
  if (statusFilter !== "all") {
    query.push(`status=${encodeURIComponent(statusFilter)}`);
  }
  if (categoryFilter !== "all") {
    query.push(`category_id=${encodeURIComponent(categoryFilter)}`);
  }
  if (assetTypeFilter !== "all") {
    query.push(`asset_type=${encodeURIComponent(assetTypeFilter)}`);
  }
  return query.length ? `?${query.join("&")}` : "";
}

export default function Assets() {
  const [assetOverview, setAssetOverview] =
    useState<AssetOverview>(emptyAssetOverview);
  const [loading, setLoading] = useState(true);
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [categoryFilter, setCategoryFilter] = useState<string>("all");
  const [assetTypeFilter, setAssetTypeFilter] = useState<string>("all");
  const [assetDialogOpen, setAssetDialogOpen] = useState(false);
  const [categoryDialogOpen, setCategoryDialogOpen] = useState(false);
  const [selectedAsset, setSelectedAsset] = useState<Asset | null>(null);
  const [selectedCategory, setSelectedCategory] = useState<Category | null>(
    null,
  );
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [assetToDelete, setAssetToDelete] = useState<DeleteAction | null>(null);
  const [deleteCategoryDialogOpen, setDeleteCategoryDialogOpen] =
    useState(false);
  const [categoryToDelete, setCategoryToDelete] =
    useState<DeleteAction | null>(null);
  const [scanDialogOpen, setScanDialogOpen] = useState(false);
  const [isScanning, setIsScanning] = useState(false);
  const [codeDialogOpen, setCodeDialogOpen] = useState(false);
  const [codeDialogAsset, setCodeDialogAsset] = useState<Asset | null>(null);
  const [scannedSerialNumber, setScannedSerialNumber] = useState("");
  const scannerRef = useRef<Html5Qrcode | null>(null);
  const videoRef = useRef<HTMLDivElement>(null);

  const assets = assetOverview.assets;
  const categories = assetOverview.categories;
  const canManageCategories = assetOverview.permissions.can_manage_categories;

  const fetchAssetOverview = useCallback(async () => {
    try {
      setLoading(true);
      const query = overviewQuery(
        searchTerm,
        statusFilter,
        categoryFilter,
        assetTypeFilter,
      );
      const result = await api.get<AssetOverview>(
        `${ASSETS_OVERVIEW_ENDPOINT}${query}`,
      );
      setAssetOverview(result || emptyAssetOverview);
    } catch (error) {
      console.error("获取资产总览失败:", error);
      toast.error("获取资产总览失败");
      setAssetOverview(emptyAssetOverview);
    } finally {
      setLoading(false);
    }
  }, [assetTypeFilter, categoryFilter, searchTerm, statusFilter]);

  useEffect(() => {
    fetchAssetOverview();
  }, [fetchAssetOverview]);

  const refreshAssetData = () => {
    fetchAssetOverview();
  };

  const handleDeleteAsset = async () => {
    if (!assetToDelete) return;

    try {
      await api.runAction(assetToDelete);
      toast.success(assetToDelete.success_message);
      refreshAssetData();
    } catch (error) {
      console.error("删除资产失败:", error);
      toast.error("删除失败，请重试");
    } finally {
      setDeleteDialogOpen(false);
      setAssetToDelete(null);
    }
  };

  const handleAssetStatusAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      refreshAssetData();
      toast.success(action.success_message || "资产状态已更新");
    } catch (error) {
      console.error("更新资产状态失败:", error);
      toast.error("更新资产状态失败");
    }
  };

  const handleDeleteCategory = async () => {
    if (!categoryToDelete) return;

    try {
      await api.runAction(categoryToDelete);
      toast.success(categoryToDelete.success_message);
      refreshAssetData();
    } catch (error) {
      console.error("删除分类失败:", error);
      toast.error("删除失败，请重试");
    } finally {
      setDeleteCategoryDialogOpen(false);
      setCategoryToDelete(null);
    }
  };

  const stopScanning = async () => {
    if (scannerRef.current) {
      try {
        await scannerRef.current.stop();
        scannerRef.current = null;
        setIsScanning(false);
      } catch (error) {
        console.error("停止扫描失败:", error);
      }
    }
  };

  const openCodeDialog = (asset: Asset) => {
    setCodeDialogAsset({
      ...asset,
      category_name: asset.asset_categories?.name || null,
    } as any);
    setCodeDialogOpen(true);
  };

  const startScanning = async () => {
    try {
      if (!videoRef.current) return;

      const html5QrCode = new Html5Qrcode("qr-reader");
      scannerRef.current = html5QrCode;

      await html5QrCode.start(
        { facingMode: "environment" },
        { fps: 10, qrbox: { width: 250, height: 250 } },
        async (decodedText) => {
          const barcodePrefix = "OM:";
          let handled = false;

          try {
            const qrData = JSON.parse(decodedText);
            if (qrData.type === "asset" && qrData.id) {
              handled = true;
              stopScanning();
              setScanDialogOpen(false);
              const result = await api.get<{ data: Asset }>(
                `/api/assets/${qrData.id}`,
              );
              const foundAsset = result?.data;
              if (foundAsset) {
                toast.success(`扫描成功：${foundAsset.name}`);
                openCodeDialog(foundAsset);
              } else {
                toast.error("系统码已过期或资产已删除");
              }
            }
          } catch {
            // Ignore non-OPC QR codes and continue with barcode matching.
          }

          if (!handled && decodedText.startsWith(barcodePrefix)) {
            handled = true;
            const assetIdPrefix = decodedText.slice(barcodePrefix.length);
            stopScanning();
            setScanDialogOpen(false);
            const foundAsset = assets.find((asset) =>
              asset.id.startsWith(assetIdPrefix),
            );
            if (foundAsset) {
              toast.success(`扫描成功：${foundAsset.name}`);
              openCodeDialog(foundAsset);
            } else {
              toast.error("系统条形码对应的资产未找到");
            }
          }

          if (!handled) {
            stopScanning();
            setScanDialogOpen(false);
            const existing = assets.find(
              (asset) => asset.serial_number === decodedText,
            );
            if (existing) {
              toast.success(`已找到匹配资产：${existing.name}`);
              openCodeDialog(existing);
            } else {
              toast(`扫到外部码：${decodedText.slice(0, 30)}`, {
                description: "正在打开新增资产表单...",
              });
              setScannedSerialNumber(decodedText);
              setSelectedAsset(null);
              setAssetDialogOpen(true);
            }
          }
        },
        () => {},
      );
      setIsScanning(true);
    } catch (error) {
      console.error("启动扫描失败:", error);
      toast.error("无法访问摄像头，请检查权限设置");
    }
  };

  useEffect(() => {
    return () => {
      stopScanning();
    };
  }, []);

  const stats = assetOverview;

  return (
    <div className="space-y-6">
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">资产总数</CardTitle>
            <Package className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total}</div>
            <p className="mt-1 text-xs text-muted-foreground">
              固定 {stats.type_counts.physical} | 虚拟{" "}
              {stats.type_counts.virtual} | 数据 {stats.type_counts.data} | 无形{" "}
              {stats.type_counts.intangible}
            </p>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">在用资产</CardTitle>
            <FolderOpen className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.in_use}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">资产总值</CardTitle>
            <Tags className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{stats.total_value_label}</div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">即将到期</CardTitle>
            <Globe className="h-4 w-4 text-destructive" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-destructive">
              {stats.expiring_soon}
            </div>
            <p className="mt-1 text-xs text-muted-foreground">30天内到期</p>
          </CardContent>
        </Card>
      </div>

      <DependencyAlert
        dependency="asset-categories"
        onAction={
          canManageCategories && assetOverview.category_create_action
            ? () => {
                setSelectedCategory(null);
                setCategoryDialogOpen(true);
              }
            : undefined
        }
      />

      <Card>
        <CardHeader>
          <div className="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
            <div>
              <CardTitle>资产管理</CardTitle>
              <CardDescription>管理企业所有资产信息</CardDescription>
            </div>
            <div className="flex gap-2">
              {canManageCategories && (
                <Button
                  variant="outline"
                  disabled={!assetOverview.category_create_action}
                  onClick={() => {
                    setSelectedCategory(null);
                    setCategoryDialogOpen(true);
                  }}
                >
                  <Tags className="mr-2 h-4 w-4" />
                  管理分类
                </Button>
              )}
              <Button variant="outline" onClick={() => setScanDialogOpen(true)}>
                <QrCode className="mr-2 h-4 w-4" />
                扫码录入
              </Button>
              <Button
                disabled={!assetOverview.create_action}
                onClick={() => {
                  setSelectedAsset(null);
                  setAssetDialogOpen(true);
                }}
              >
                <Plus className="mr-2 h-4 w-4" />
                添加资产
              </Button>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          <div className="mb-6 flex flex-col gap-4 sm:flex-row">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-muted-foreground" />
              <Input
                placeholder="搜索资产名称、序列号、位置..."
                value={searchTerm}
                onChange={(event) => setSearchTerm(event.target.value)}
                className="pl-9"
              />
            </div>

            <Select value={statusFilter} onValueChange={setStatusFilter}>
              <SelectTrigger className="w-full sm:w-40">
                <Filter className="mr-2 h-4 w-4" />
                <SelectValue placeholder="状态" />
              </SelectTrigger>
              <SelectContent>
                {assetOverview.status_options.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={categoryFilter} onValueChange={setCategoryFilter}>
              <SelectTrigger className="w-full sm:w-40">
                <Tags className="mr-2 h-4 w-4" />
                <SelectValue placeholder="分类" />
              </SelectTrigger>
              <SelectContent>
                {assetOverview.category_options.map((category) => (
                  <SelectItem key={category.value} value={category.value}>
                    {category.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>

            <Select value={assetTypeFilter} onValueChange={setAssetTypeFilter}>
              <SelectTrigger className="w-full sm:w-40">
                <Globe className="mr-2 h-4 w-4" />
                <SelectValue placeholder="资产类型" />
              </SelectTrigger>
              <SelectContent>
                {assetOverview.asset_type_options.map((option) => (
                  <SelectItem key={option.value} value={option.value}>
                    {option.label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </div>

          {loading ? (
            <div className="flex items-center justify-center py-8">
              <Loader2 className="h-8 w-8 animate-spin text-primary" />
            </div>
          ) : assets.length === 0 ? (
            <div className="py-8 text-center text-muted-foreground">
              {assetOverview.empty_message}
            </div>
          ) : (
            <div className="rounded-md border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>资产名称</TableHead>
                    <TableHead>类型</TableHead>
                    <TableHead>分类</TableHead>
                    <TableHead>序列号</TableHead>
                    <TableHead>状态</TableHead>
                    <TableHead>到期/折旧</TableHead>
                    <TableHead className="text-right">当前价值</TableHead>
                    <TableHead className="text-right">净值</TableHead>
                    <TableHead className="text-right">操作</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {assets.map((asset) => (
                    <TableRow key={asset.id}>
                      <TableCell className="font-medium">
                        {asset.name}
                      </TableCell>
                      <TableCell>
                        <AssetBadgeView badge={asset.asset_type_badge} />
                      </TableCell>
                      <TableCell>
                        {asset.asset_categories?.name || "-"}
                      </TableCell>
                      <TableCell className="font-mono text-sm">
                        {asset.serial_number || "-"}
                      </TableCell>
                      <TableCell>
                        <AssetBadgeView badge={asset.status_badge} />
                      </TableCell>
                      <TableCell>
                        {asset.expiry_badge.visible === false ? (
                          "-"
                        ) : (
                          <AssetBadgeView badge={asset.expiry_badge} />
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        {asset.current_value_label}
                      </TableCell>
                      <TableCell className="text-right">
                        {asset.depreciation_visible ? (
                          <div className="text-xs">
                            <div>{asset.net_value_label}</div>
                            <div className="text-muted-foreground">
                              {asset.monthly_depreciation_label}
                            </div>
                          </div>
                        ) : (
                          "-"
                        )}
                      </TableCell>
                      <TableCell className="text-right">
                        <div className="flex justify-end gap-2">
                          <Button
                            size="sm"
                            variant="ghost"
                            onClick={() => openCodeDialog(asset)}
                            title="二维码 / 条形码"
                          >
                            <QrCode className="h-4 w-4" />
                          </Button>
                          {asset.can_mark_in_use && asset.mark_in_use_action && (
                            <Button
                              size="sm"
                              variant="ghost"
                              title={asset.mark_in_use_action.label || "标记为使用中"}
                              onClick={() =>
                                handleAssetStatusAction(asset.mark_in_use_action)
                              }
                            >
                              <CheckCircle2 className="h-4 w-4" />
                            </Button>
                          )}
                          {asset.can_mark_maintenance &&
                            asset.mark_maintenance_action && (
                            <Button
                              size="sm"
                              variant="ghost"
                              title={
                                asset.mark_maintenance_action.label ||
                                "标记为维修中"
                              }
                              onClick={() =>
                                handleAssetStatusAction(
                                  asset.mark_maintenance_action,
                                )
                              }
                            >
                              <Wrench className="h-4 w-4" />
                            </Button>
                          )}
                          {asset.can_mark_scrapped &&
                            asset.mark_scrapped_action && (
                            <Button
                              size="sm"
                              variant="ghost"
                              title={
                                asset.mark_scrapped_action.label ||
                                "标记为已报废"
                              }
                              onClick={() =>
                                handleAssetStatusAction(
                                  asset.mark_scrapped_action,
                                )
                              }
                            >
                              <Archive className="h-4 w-4" />
                            </Button>
                          )}
                          {asset.update_action && (
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => {
                                setSelectedAsset(asset);
                                setAssetDialogOpen(true);
                              }}
                            >
                              <Edit className="h-4 w-4" />
                            </Button>
                          )}
                          {asset.can_delete && asset.delete_action && (
                            <Button
                              size="sm"
                              variant="ghost"
                              onClick={() => {
                                setAssetToDelete(asset.delete_action);
                                setDeleteDialogOpen(true);
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
            </div>
          )}

          {canManageCategories && categories.length > 0 && (
            <div className="mt-8">
              <h3 className="mb-4 text-lg font-semibold">分类列表</h3>
              <div className="rounded-md border">
                <Table>
                  <TableHeader>
                    <TableRow>
                      <TableHead>分类名称</TableHead>
                      <TableHead>描述</TableHead>
                      <TableHead className="text-right">操作</TableHead>
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {categories.map((category) => (
                      <TableRow key={category.id}>
                        <TableCell className="font-medium">
                          {category.name}
                          <Badge variant="outline" className="ml-2">
                            {category.category_type_label}
                          </Badge>
                        </TableCell>
                        <TableCell>{category.description || "-"}</TableCell>
                        <TableCell className="text-right">
                          <div className="flex justify-end gap-2">
                            {category.update_action && (
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => {
                                  setSelectedCategory(category);
                                  setCategoryDialogOpen(true);
                                }}
                              >
                                <Edit className="h-4 w-4" />
                              </Button>
                            )}
                            {category.delete_action && (
                              <Button
                                size="sm"
                                variant="ghost"
                                onClick={() => {
                                  setCategoryToDelete(category.delete_action);
                                  setDeleteCategoryDialogOpen(true);
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
              </div>
            </div>
          )}
        </CardContent>
      </Card>

      <AssetDialog
        open={assetDialogOpen}
        onOpenChange={(open) => {
          setAssetDialogOpen(open);
          if (!open) setScannedSerialNumber("");
        }}
        asset={selectedAsset}
        action={
          selectedAsset ? selectedAsset.update_action : assetOverview.create_action
        }
        onSuccess={refreshAssetData}
        defaultSerialNumber={scannedSerialNumber}
      />

      <CategoryDialog
        open={categoryDialogOpen}
        onOpenChange={setCategoryDialogOpen}
        category={selectedCategory}
        action={
          selectedCategory
            ? selectedCategory.update_action
            : assetOverview.category_create_action
        }
        onSuccess={refreshAssetData}
      />

      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {assetToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteAsset}>
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <Dialog
        open={scanDialogOpen}
        onOpenChange={(open) => {
          setScanDialogOpen(open);
          if (!open) stopScanning();
        }}
      >
        <DialogContent className="max-w-2xl">
          <DialogHeader>
            <DialogTitle>扫码录入资产</DialogTitle>
            <DialogDescription>扫描资产二维码快速查询和管理</DialogDescription>
          </DialogHeader>
          <div className="space-y-4">
            <div
              id="qr-reader"
              ref={videoRef}
              className="w-full overflow-hidden rounded-lg"
              style={{ minHeight: "300px" }}
            />
            {!isScanning && (
              <div className="flex h-80 items-center justify-center rounded-lg border-2 border-dashed bg-muted">
                <div className="space-y-4 text-center">
                  <QrCode className="mx-auto h-24 w-24 text-muted-foreground" />
                  <div>
                    <p className="font-medium">请将二维码对准摄像头</p>
                    <p className="text-sm text-muted-foreground">
                      自动识别并查询资产信息
                    </p>
                  </div>
                  <Button onClick={startScanning}>
                    <Camera className="mr-2 h-4 w-4" />
                    开启摄像头
                  </Button>
                </div>
              </div>
            )}
            {isScanning && (
              <Button
                onClick={stopScanning}
                variant="destructive"
                className="w-full"
              >
                停止扫描
              </Button>
            )}
          </div>
        </DialogContent>
      </Dialog>

      <AlertDialog
        open={deleteCategoryDialogOpen}
        onOpenChange={setDeleteCategoryDialogOpen}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {categoryToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteCategory}>
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      <AssetCodeDialog
        open={codeDialogOpen}
        onOpenChange={setCodeDialogOpen}
        asset={codeDialogAsset}
        onEdit={(asset) => {
          const fullAsset = assets.find((item) => item.id === asset.id);
          if (fullAsset) {
            setSelectedAsset(fullAsset);
            setAssetDialogOpen(true);
          }
        }}
      />
    </div>
  );
}
