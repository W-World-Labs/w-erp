import { useState, useEffect } from "react";
import { api, pdf, type BackendAction } from "@/integrations/api/client";
import { toast } from "sonner";
import {
  Loader2,
  Plus,
  Search,
  Edit,
  Trash2,
  FileText,
  Clock,
  CheckCircle,
  XCircle,
  Eye,
  Filter,
  Upload,
  Send,
  Wallet,
  ArrowUpRight,
  ArrowDownLeft,
  Download,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";

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
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { DependencyAlert } from "@/components/common/DependencyAlert";
import { QueryStateGate } from "@/components/common/QueryStateGate";
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
import { InvoiceDialog } from "@/components/invoices/InvoiceDialog";
import { InvoiceDetailDialog } from "@/components/invoices/InvoiceDetailDialog";
import { InvoiceImportDialog } from "@/components/invoices/InvoiceImportDialog";

type BadgeVariant = "secondary" | "default" | "destructive" | "outline";

interface InvoiceBadge {
  label: string;
  variant: BadgeVariant;
  icon?: string;
}

interface DeleteAction {
  method: "DELETE";
  endpoint: string;
  confirm_message: string;
  success_title: string;
  success_message: string;
}

interface RecordPaymentAction extends BackendAction {
  amount?: number;
  amount_label?: string;
  paid_total?: number;
  paid_total_label?: string;
  remaining?: number;
  remaining_label?: string;
  default_amount?: number;
}

interface Invoice {
  id: string;
  invoice_number: string;
  invoice_type: string | null;
  amount: number;
  amount_label: string;
  tax_amount: number | null;
  tax_amount_label: string;
  total_with_tax_label: string;
  paid_total?: number;
  paid_total_label?: string;
  remaining?: number;
  remaining_label?: string;
  invoice_date: string;
  invoice_date_label: string;
  due_date: string | null;
  due_date_label: string;
  status: string;
  status_badge: InvoiceBadge;
  supplier: string | null;
  client_id?: string | null;
  currency?: string | null;
  description: string | null;
  file_path: string | null;
  file_name: string | null;
  file_label: string | null;
  has_file: boolean;
  created_by: string | null;
  created_at: string;
  created_at_label: string;
  updated_at: string;
  can_edit: boolean;
  update_action: BackendAction | null;
  can_delete: boolean;
  delete_action: DeleteAction | null;
  can_mark_sent: boolean;
  can_mark_paid: boolean;
  can_record_payment: boolean;
  mark_sent_action: BackendAction | null;
  mark_paid_action: BackendAction | null;
  record_payment_action: RecordPaymentAction | null;
}

interface InvoiceDirectionStats {
  total: number;
  pending: number;
  sent: number;
  paid: number;
  total_amount: number;
  total_amount_label: string;
  total_tax: number;
  total_tax_label: string;
  pending_message: string;
}

interface InvoicesOverview {
  invoices: Invoice[];
  output_invoices: Invoice[];
  input_invoices: Invoice[];
  output_stats: InvoiceDirectionStats;
  input_stats: InvoiceDirectionStats;
  status_options: Array<{ value: string; label: string }>;
  create_action: BackendAction | null;
  import_preview_action: BackendAction | null;
  import_commit_action: BackendAction | null;
  permissions: {
    can_create: boolean;
    can_import: boolean;
    can_manage: boolean;
  };
  empty_messages: {
    filtered: string;
    output: string;
    input: string;
  };
}

const INVOICES_OVERVIEW_ENDPOINT = "/api/invoices/overview";

const emptyInvoiceStats: InvoiceDirectionStats = {
  total: 0,
  pending: 0,
  sent: 0,
  paid: 0,
  total_amount: 0,
  total_amount_label: "¥0",
  total_tax: 0,
  total_tax_label: "¥0",
  pending_message: "共 0 张待审批",
};

const emptyInvoicesOverview: InvoicesOverview = {
  invoices: [],
  output_invoices: [],
  input_invoices: [],
  output_stats: emptyInvoiceStats,
  input_stats: emptyInvoiceStats,
  status_options: [],
  create_action: null,
  import_preview_action: null,
  import_commit_action: null,
  permissions: {
    can_create: true,
    can_import: true,
    can_manage: false,
  },
  empty_messages: {
    filtered: "没有找到匹配的发票",
    output: "暂无销项发票，点击上方按钮添加",
    input: "暂无进项发票，点击上方按钮添加",
  },
};

const badgeIcons: Record<string, LucideIcon> = {
  CheckCircle,
  Clock,
  XCircle,
};

function InvoiceBadgeView({ badge }: { badge: InvoiceBadge }) {
  const Icon = badge.icon ? badgeIcons[badge.icon] : null;
  return (
    <Badge variant={badge.variant}>
      {Icon && <Icon className="h-3 w-3 mr-1" />}
      {badge.label}
    </Badge>
  );
}

export default function Invoices() {
  const [invoiceOverview, setInvoiceOverview] = useState<InvoicesOverview>(
    emptyInvoicesOverview,
  );
  const [loading, setLoading] = useState(true);
  const [loadError, setLoadError] = useState(false);
  const [searchTerm, setSearchTerm] = useState("");
  const [statusFilter, setStatusFilter] = useState<string>("all");
  const [activeTab, setActiveTab] = useState<"output" | "input">("output");
  const [invoiceDialogOpen, setInvoiceDialogOpen] = useState(false);
  const [importDialogOpen, setImportDialogOpen] = useState(false);
  const [detailDialogOpen, setDetailDialogOpen] = useState(false);
  const [selectedInvoice, setSelectedInvoice] = useState<Invoice | null>(null);
  const [deleteDialogOpen, setDeleteDialogOpen] = useState(false);
  const [invoiceToDelete, setInvoiceToDelete] = useState<DeleteAction | null>(
    null,
  );
  const [paymentDialogOpen, setPaymentDialogOpen] = useState(false);
  const [paymentAction, setPaymentAction] = useState<RecordPaymentAction | null>(
    null,
  );
  const [paymentAmount, setPaymentAmount] = useState("");
  const [paymentSubmitting, setPaymentSubmitting] = useState(false);

  useEffect(() => {
    fetchInvoices();
  }, [searchTerm, statusFilter]);

  const fetchInvoices = async () => {
    try {
      setLoading(true);
      setLoadError(false);
      const params = new URLSearchParams();
      if (searchTerm.trim()) params.set("search", searchTerm.trim());
      if (statusFilter !== "all") params.set("status", statusFilter);
      const suffix = params.toString() ? `?${params.toString()}` : "";
      const result = await api.get<InvoicesOverview>(
        `${INVOICES_OVERVIEW_ENDPOINT}${suffix}`,
      );
      setInvoiceOverview(result || emptyInvoicesOverview);
    } catch (error) {
      console.error("获取发票列表失败:", error);
      setLoadError(true);
      toast.error("获取发票列表失败");
    } finally {
      setLoading(false);
    }
  };

  const handleDeleteInvoice = async () => {
    const action = invoiceToDelete;
    if (!action) return;

    try {
      await api.runAction(action);
      toast.success(action.success_message);
      fetchInvoices();
    } catch (error) {
      console.error("删除发票失败:", error);
      toast.error("删除失败，请重试");
    } finally {
      setDeleteDialogOpen(false);
      setInvoiceToDelete(null);
    }
  };

  const handleInvoiceAction = async (action: BackendAction) => {
    try {
      await api.runAction(action);
      toast.success(action.success_message || "发票状态已更新");
      fetchInvoices();
    } catch (error) {
      console.error("标记发票失败:", error);
      toast.error("标记失败，请重试");
    }
  };

  const openPaymentDialog = (action: RecordPaymentAction) => {
    setPaymentAction(action);
    const defaultAmt =
      action.default_amount ?? action.remaining ?? action.amount ?? 0;
    setPaymentAmount(String(defaultAmt));
    setPaymentDialogOpen(true);
  };

  const handleRecordPayment = async () => {
    if (!paymentAction) return;
    const amount = Number(paymentAmount);
    if (!Number.isFinite(amount) || amount <= 0) {
      toast.error("请输入有效的收款金额");
      return;
    }
    try {
      setPaymentSubmitting(true);
      const result = await api.runAction<{
        toast_message?: string;
        invoice_status?: string;
        remaining?: number;
      }>(paymentAction, { amount });
      toast.success(
        result?.toast_message ||
          paymentAction.success_message ||
          "收款已记录",
      );
      setPaymentDialogOpen(false);
      setPaymentAction(null);
      fetchInvoices();
    } catch (error) {
      console.error("记录收款失败:", error);
      toast.error(
        error instanceof Error ? error.message : "记录收款失败，请重试",
      );
    } finally {
      setPaymentSubmitting(false);
    }
  };

  const outputInvoices = invoiceOverview.output_invoices;
  const inputInvoices = invoiceOverview.input_invoices;
  const invoiceOutputStats = invoiceOverview.output_stats;
  const invoiceInputStats = invoiceOverview.input_stats;

  const currentInvoices =
    activeTab === "output" ? outputInvoices : inputInvoices;

  const renderStatsCards = () => {
    if (activeTab === "output") {
      return (
        <div className="grid gap-4 md:grid-cols-4">
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">销项发票</CardTitle>
              <ArrowUpRight className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {invoiceOutputStats.total}
              </div>
              <p className="text-xs text-muted-foreground mt-1">
                {invoiceOutputStats.pending_message}
              </p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">已开具</CardTitle>
              <CheckCircle className="h-4 w-4 text-green-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {invoiceOutputStats.sent}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">销项总额</CardTitle>
              <FileText className="h-4 w-4 text-muted-foreground" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold">
                {invoiceOutputStats.total_amount_label}
              </div>
            </CardContent>
          </Card>
          <Card>
            <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
              <CardTitle className="text-sm font-medium">销项税额</CardTitle>
              <FileText className="h-4 w-4 text-orange-500" />
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-orange-600">
                {invoiceOutputStats.total_tax_label}
              </div>
            </CardContent>
          </Card>
        </div>
      );
    }

    return (
      <div className="grid gap-4 md:grid-cols-4">
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">进项发票</CardTitle>
            <ArrowDownLeft className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{invoiceInputStats.total}</div>
            <p className="text-xs text-muted-foreground mt-1">
              {invoiceInputStats.pending_message}
            </p>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">已付款</CardTitle>
            <CheckCircle className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">{invoiceInputStats.paid}</div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">进项总额</CardTitle>
            <FileText className="h-4 w-4 text-muted-foreground" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold">
              {invoiceInputStats.total_amount_label}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
            <CardTitle className="text-sm font-medium">可抵扣税额</CardTitle>
            <FileText className="h-4 w-4 text-green-500" />
          </CardHeader>
          <CardContent>
            <div className="text-2xl font-bold text-green-600">
              {invoiceInputStats.total_tax_label}
            </div>
          </CardContent>
        </Card>
      </div>
    );
  };

  const renderInvoiceTable = () => {
    if (loading) {
      return (
        <div className="flex items-center justify-center py-8">
          <Loader2 className="h-8 w-8 animate-spin text-primary" />
        </div>
      );
    }

    if (currentInvoices.length === 0) {
      const isFiltered = Boolean(searchTerm) || statusFilter !== "all";
      const emptyMessage = isFiltered
        ? invoiceOverview.empty_messages.filtered
        : activeTab === "output"
          ? invoiceOverview.empty_messages.output
          : invoiceOverview.empty_messages.input;
      return (
        <div className="flex flex-col items-center justify-center text-center py-12">
          <FileText className="h-16 w-16 text-muted-foreground mb-4" />
          <h3 className="text-lg font-semibold mb-2">{emptyMessage}</h3>
          {!isFiltered && invoiceOverview.create_action && (
            <Button
              className="mt-2"
              onClick={() => {
                setSelectedInvoice(null);
                setInvoiceDialogOpen(true);
              }}
            >
              <Plus className="mr-2 h-4 w-4" />
              {activeTab === "output" ? "开具第一张发票" : "录入第一张发票"}
            </Button>
          )}
        </div>
      );
    }

    return (
      <div className="rounded-md border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>发票号</TableHead>
              <TableHead>
                {activeTab === "output" ? "客户" : "供应商"}
              </TableHead>
              <TableHead>摘要</TableHead>
              <TableHead>金额</TableHead>
              <TableHead>税额</TableHead>
              <TableHead>开票日期</TableHead>
              <TableHead>状态</TableHead>
              <TableHead className="text-right">操作</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {currentInvoices.map((invoice) => (
              <TableRow key={invoice.id}>
                <TableCell className="font-mono text-sm">
                  {invoice.invoice_number}
                </TableCell>
                <TableCell className="font-medium">
                  {invoice.supplier || "-"}
                </TableCell>
                <TableCell className="max-w-[200px] truncate">
                  {invoice.description || "-"}
                </TableCell>
                <TableCell>{invoice.amount_label}</TableCell>
                <TableCell className="text-muted-foreground">
                  {invoice.tax_amount_label}
                </TableCell>
                <TableCell>{invoice.invoice_date_label}</TableCell>
                <TableCell>
                  <InvoiceBadgeView badge={invoice.status_badge} />
                </TableCell>
                <TableCell className="text-right">
                  <div className="flex justify-end gap-2">
                    <Button
                      size="sm"
                      variant="ghost"
                      onClick={() => {
                        setSelectedInvoice(invoice);
                        setDetailDialogOpen(true);
                      }}
                    >
                      <Eye className="h-4 w-4" />
                    </Button>
                    <Button
                      size="sm"
                      variant="ghost"
                      title="下载 PDF"
                      onClick={() =>
                        pdf
                          .downloadInvoice(
                            invoice.id,
                            `${invoice.invoice_number}.pdf`,
                          )
                          .catch(() => toast.error("PDF 下载失败"))
                      }
                    >
                      <Download className="h-4 w-4" />
                    </Button>
                    {invoice.can_mark_sent && invoice.mark_sent_action && (
                      <Button
                        size="sm"
                        variant="ghost"
                        title={invoice.mark_sent_action.label || "标记已开具"}
                        onClick={() =>
                          handleInvoiceAction(invoice.mark_sent_action)
                        }
                      >
                        <Send className="h-4 w-4" />
                      </Button>
                    )}
                    {invoice.can_mark_paid && invoice.mark_paid_action && (
                      <Button
                        size="sm"
                        variant="ghost"
                        title={invoice.mark_paid_action.label || "标记已支付"}
                        onClick={() =>
                          handleInvoiceAction(invoice.mark_paid_action)
                        }
                      >
                        <CheckCircle className="h-4 w-4" />
                      </Button>
                    )}
                    {invoice.can_record_payment &&
                      invoice.record_payment_action && (
                        <Button
                          size="sm"
                          variant="ghost"
                          title={
                            invoice.record_payment_action.label || "记录收款"
                          }
                          onClick={() =>
                            openPaymentDialog(invoice.record_payment_action!)
                          }
                        >
                          <Wallet className="h-4 w-4 text-green-600" />
                        </Button>
                      )}
                    {invoice.can_edit && invoice.update_action && (
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => {
                          setSelectedInvoice(invoice);
                          setInvoiceDialogOpen(true);
                        }}
                      >
                        <Edit className="h-4 w-4" />
                      </Button>
                    )}
                    {invoice.delete_action && (
                      <Button
                        size="sm"
                        variant="ghost"
                        onClick={() => {
                          setInvoiceToDelete(invoice.delete_action);
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
    );
  };

  return (
    <QueryStateGate isError={loadError} refetch={fetchInvoices}>
    <div className="space-y-6">
      {/* 依赖检查 */}
      <DependencyAlert dependency="company-info" />

      {/* Tab 切换：销项 / 进项 */}
      <Tabs
        value={activeTab}
        onValueChange={(v) => {
          setActiveTab(v as "output" | "input");
          setSearchTerm("");
          setStatusFilter("all");
        }}
      >
        <div className="flex items-center justify-between">
          <TabsList>
            <TabsTrigger value="output" className="gap-2">
              <ArrowUpRight className="h-4 w-4" />
              销项发票
              <Badge variant="secondary" className="ml-1 text-xs">
                {invoiceOutputStats.total}
              </Badge>
            </TabsTrigger>
            <TabsTrigger value="input" className="gap-2">
              <ArrowDownLeft className="h-4 w-4" />
              进项发票
              <Badge variant="secondary" className="ml-1 text-xs">
                {invoiceInputStats.total}
              </Badge>
            </TabsTrigger>
          </TabsList>

          <div className="flex gap-2">
            <Button
              variant="outline"
              disabled={!invoiceOverview.permissions.can_import}
              onClick={() => setImportDialogOpen(true)}
            >
              <Upload className="mr-2 h-4 w-4" />
              导入发票
            </Button>
            <Button
              disabled={!invoiceOverview.create_action}
              onClick={() => {
                setSelectedInvoice(null);
                setInvoiceDialogOpen(true);
              }}
            >
              <Plus className="mr-2 h-4 w-4" />
              {activeTab === "output" ? "开具发票" : "录入发票"}
            </Button>
          </div>
        </div>

        {/* 统计卡片 — 随 Tab 切换 */}
        <div className="mt-4">{renderStatsCards()}</div>

        {/* 主内容区 */}
        <Card className="mt-4">
          <CardHeader>
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
              <div>
                <CardTitle>
                  {activeTab === "output" ? "销项发票管理" : "进项发票管理"}
                </CardTitle>
                <CardDescription>
                  {activeTab === "output"
                    ? "管理我方开具给客户的发票"
                    : "管理供应商开具给我方的发票"}
                </CardDescription>
              </div>
            </div>
          </CardHeader>

          <CardContent>
            {/* 搜索和过滤 */}
            <div className="flex flex-col sm:flex-row gap-4 mb-6">
              <div className="flex-1 relative">
                <Search className="absolute left-3 top-1/2 -translate-y-1/2 h-4 w-4 text-muted-foreground" />
                <Input
                  placeholder={
                    activeTab === "output"
                      ? "搜索发票号、客户、摘要..."
                      : "搜索发票号、供应商、摘要..."
                  }
                  value={searchTerm}
                  onChange={(e) => setSearchTerm(e.target.value)}
                  className="pl-9"
                />
              </div>

              <Select value={statusFilter} onValueChange={setStatusFilter}>
                <SelectTrigger className="w-full sm:w-40">
                  <Filter className="mr-2 h-4 w-4" />
                  <SelectValue placeholder="状态" />
                </SelectTrigger>
                <SelectContent>
                  {invoiceOverview.status_options.map((option) => (
                    <SelectItem key={option.value} value={option.value}>
                      {option.label}
                    </SelectItem>
                  ))}
                </SelectContent>
              </Select>
            </div>

            {/* 发票列表 */}
            {renderInvoiceTable()}
          </CardContent>
        </Card>

        {/* TabsContent 不实际渲染内容（由上方统一渲染），但需要存在才不报错 */}
        <TabsContent value="output" className="mt-0" />
        <TabsContent value="input" className="mt-0" />
      </Tabs>

      {/* 对话框 */}
      <InvoiceDialog
        open={invoiceDialogOpen}
        onOpenChange={setInvoiceDialogOpen}
        invoice={selectedInvoice}
        action={
          selectedInvoice
            ? selectedInvoice.update_action
            : invoiceOverview.create_action
        }
        onSuccess={fetchInvoices}
        defaultInvoiceType={activeTab}
      />

      <InvoiceImportDialog
        open={importDialogOpen}
        onOpenChange={setImportDialogOpen}
        onSuccess={fetchInvoices}
        previewAction={invoiceOverview.import_preview_action}
        commitAction={invoiceOverview.import_commit_action}
      />

      <InvoiceDetailDialog
        open={detailDialogOpen}
        onOpenChange={setDetailDialogOpen}
        invoice={selectedInvoice}
      />

      {/* 删除确认 */}
      <AlertDialog open={deleteDialogOpen} onOpenChange={setDeleteDialogOpen}>
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>确认删除</AlertDialogTitle>
            <AlertDialogDescription>
              {invoiceToDelete?.confirm_message}
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel>取消</AlertDialogCancel>
            <AlertDialogAction onClick={handleDeleteInvoice}>
              删除
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>

      {/* 部分收款 / 分期 */}
      <AlertDialog
        open={paymentDialogOpen}
        onOpenChange={(open) => {
          setPaymentDialogOpen(open);
          if (!open) setPaymentAction(null);
        }}
      >
        <AlertDialogContent>
          <AlertDialogHeader>
            <AlertDialogTitle>
              {paymentAction?.label || "记录收款"}
            </AlertDialogTitle>
            <AlertDialogDescription asChild>
              <div className="space-y-3 text-sm text-muted-foreground">
                <p>
                  发票金额 {paymentAction?.amount_label ?? "-"} · 已收{" "}
                  {paymentAction?.paid_total_label ?? "¥0"} · 剩余{" "}
                  {paymentAction?.remaining_label ?? "-"}
                </p>
                <div className="space-y-1.5">
                  <label className="text-sm font-medium text-foreground">
                    本次收款金额
                  </label>
                  <Input
                    type="number"
                    inputMode="decimal"
                    min={0}
                    step="0.01"
                    value={paymentAmount}
                    onChange={(e) => setPaymentAmount(e.target.value)}
                    placeholder="默认剩余应收"
                  />
                </div>
              </div>
            </AlertDialogDescription>
          </AlertDialogHeader>
          <AlertDialogFooter>
            <AlertDialogCancel disabled={paymentSubmitting}>
              取消
            </AlertDialogCancel>
            <AlertDialogAction
              onClick={(e) => {
                e.preventDefault();
                handleRecordPayment();
              }}
              disabled={paymentSubmitting}
            >
              {paymentSubmitting ? "提交中…" : "确认收款"}
            </AlertDialogAction>
          </AlertDialogFooter>
        </AlertDialogContent>
      </AlertDialog>
    </div>
    </QueryStateGate>
  );
}
